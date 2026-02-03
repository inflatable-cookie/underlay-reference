//! Authentication route handlers.
//!
//! These endpoints handle user registration, login, logout, token refresh, password changes,
//! and session management.
//!
//! ## Hybrid Token Auth
//!
//! Auth endpoints support a hybrid token approach for browser and mobile clients:
//! - Access token: Returned in response body, sent as `Authorization: Bearer` header
//! - Refresh token: Returned in response body AND set as httpOnly cookie (browser)
//! - `logged_in` cookie: UI flag for CSS switching (not httpOnly)
//!
//! The refresh endpoint accepts the refresh token from EITHER the request body OR cookie.

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use acme_core::{AppError, Uuid};
use serde_json::json;
use sha2::{Digest, Sha256};
use underlay_core::{ListResponse, SingleResponse};
use underlay_http::{clear_auth_cookies, extract_refresh_token, set_auth_cookies};
use validator::Validate;

use crate::dto::auth::{
    auth_session_dto_from_session, roles_for_user, AuthUserDto,
    ChangePasswordRequest, ChangePasswordWithVerificationRequest,
    EmailTotpRequestRequest, EmailTotpRequestResponse, EmailTotpVerifyRequest, EmailTotpVerifyResponse,
    LoginFinishRequest, LoginRequest, LoginStartRequest, LoginStartResponse,
    LogoutRequest, PasswordResetCompleteRequest, PasswordResetRequestRequest,
    PasswordResetVerifyRequest, PasswordResetVerifyResponse,
    RefreshRequest, RegisterRequest, SessionDto,
    TotpEnableRequest, TotpStatusResponse, TotpVerifyRequest, TwoFactorStatusResponse,
};
use crate::error::error_response;
use crate::state::{AppState, AuthenticatedUser};

use acme_auth::SessionFingerprint;
use std::time::{Duration, Instant};

// ============================================================================
// Helpers
// ============================================================================

/// Minimum response time for timing-attack-resistant endpoints (in milliseconds).
///
/// This ensures all code paths take at least this long, preventing attackers
/// from inferring information based on response timing.
const MIN_RESPONSE_TIME_MS: u64 = 200;

/// Ensure a minimum amount of time has elapsed since `start`.
///
/// Used to prevent timing attacks by making all code paths take similar time.
async fn ensure_min_response_time(start: Instant) {
    let elapsed = start.elapsed();
    let min_duration = Duration::from_millis(MIN_RESPONSE_TIME_MS);
    if elapsed < min_duration {
        tokio::time::sleep(min_duration - elapsed).await;
    }
}

/// Extract session fingerprint from request headers.
///
/// Returns IP address (from X-Forwarded-For or X-Real-IP) and User-Agent.
fn extract_session_fingerprint(headers: &HeaderMap) -> SessionFingerprint {
    // Try to get client IP from proxy headers
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    SessionFingerprint::new(ip_address, user_agent)
}

/// Extract client IP from request headers.
///
/// Checks `X-Forwarded-For` first (for proxied requests), then `X-Real-IP`.
/// Returns `None` if no IP header is present.
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // X-Forwarded-For can contain multiple IPs: "client, proxy1, proxy2"
    // The first one is the original client IP
    if let Some(forwarded) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        return Some(forwarded.to_string());
    }

    // X-Real-IP is a single IP set by some proxies (e.g., nginx)
    if let Some(real_ip) = headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        return Some(real_ip.to_string());
    }

    None
}

/// Convert validation errors to an AppError response.
fn validation_error_response(validation_err: validator::ValidationErrors) -> impl IntoResponse {
    let mut field_errors = std::collections::HashMap::new();
    for (field, errors) in validation_err.field_errors() {
        if let Some(err) = errors.first() {
            let msg = err
                .message
                .clone()
                .unwrap_or_else(|| "Invalid value".into());
            field_errors.insert(field.to_string(), msg.to_string());
        }
    }
    let err = AppError {
        code: "auth.validation_failed",
        message: "There is a problem with one or more fields.".to_string(),
        field_errors: Some(field_errors),
    };
    error_response(StatusCode::BAD_REQUEST, err)
}

/// Map an AuthError to an HTTP response with appropriate status code.
///
/// This centralizes error handling for all auth endpoints to ensure consistent
/// error responses across the API. Status code mapping:
/// - RateLimited -> 429 TOO_MANY_REQUESTS (with Retry-After header)
/// - Session/token errors -> 401 UNAUTHORIZED
/// - Forbidden -> 403 FORBIDDEN
/// - Internal -> 500 INTERNAL_SERVER_ERROR
/// - All others -> 400 BAD_REQUEST
fn map_auth_error_to_response(err: underlay_auth::AuthError) -> axum::response::Response {
    use underlay_auth::AuthError;

    let status = match &err {
        AuthError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        AuthError::SessionExpired
        | AuthError::SessionRevoked
        | AuthError::TokenInvalid
        | AuthError::TokenMalformed
        | AuthError::TokenNotYetValid
        | AuthError::TokenFingerprintMismatch
        | AuthError::InvalidToken
        | AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
        AuthError::Forbidden => StatusCode::FORBIDDEN,
        AuthError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    };

    // For rate limiting, add Retry-After header
    if let AuthError::RateLimited {
        retry_after_seconds,
    } = &err
    {
        let mut response =
            error_response(status, AppError::new(err.code(), err.message())).into_response();
        if let Ok(value) = HeaderValue::try_from(retry_after_seconds.to_string()) {
            response.headers_mut().insert("Retry-After", value);
        }
        return response;
    }

    error_response(status, AppError::new(err.code(), err.message())).into_response()
}

fn login_client_fingerprint(headers: &HeaderMap) -> String {
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .unwrap_or("");

    let raw = format!("{ip}|{ua}");
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    hex::encode(h.finalize())
}

// ============================================================================
// Route Handlers
// ============================================================================

pub async fn register(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let password = payload.password.trim();
    let display_name = payload.display_name.trim();

    let client_ip = extract_client_ip(&headers);

    match state
        .local_auth
        .register_with_ip(email, password, display_name, client_ip.as_deref())
        .await
    {
        Ok(session) => {
            // Enqueue welcome email job
            if let Some(ref job_repo) = state.job_repository {
                let job_payload = serde_json::json!({
                    "user_id": session.user.id,
                    "email": email,
                    "display_name": display_name,
                });
                if let Err(e) = job_repo
                    .create("email.welcome", job_payload, &Default::default())
                    .await
                {
                    tracing::warn!("Failed to enqueue welcome email job: {}", e);
                }
            }

            let refresh_token = session.refresh_token.clone();
            let dto = auth_session_dto_from_session(session, "user".to_string());

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            (StatusCode::OK, response_headers, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn login(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let password = payload.password.trim();

    let client_ip = extract_client_ip(&headers);

    match state
        .local_auth
        .login_with_password_and_ip(
            email,
            password,
            payload.code.as_deref(),
            client_ip.as_deref(),
        )
        .await
    {
        Ok(session) => {
            let role = state
                .local_auth
                .me(session.user.id)
                .await
                .map(|(_, r)| r)
                .unwrap_or_else(|_| "user".to_string());

            let refresh_token = session.refresh_token.clone();
            let dto = auth_session_dto_from_session(session, role);

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            (StatusCode::OK, response_headers, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn login_start(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginStartRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let password = payload.password.trim();

    let fingerprint = login_client_fingerprint(&headers);
    let client_ip = extract_client_ip(&headers);

    // Use the method that supports email fallback
    match state
        .local_auth
        .login_start_with_email_fallback(
            email,
            password,
            &fingerprint,
            client_ip.as_deref(),
            payload.enforce_email_fallback,
        )
        .await
    {
        Ok(acme_auth::LoginStartOutcome::Complete { session, role }) => {
            let refresh_token = session.refresh_token.clone();
            let dto = auth_session_dto_from_session(*session, role);
            let response = LoginStartResponse {
                requires_two_factor: false,
                is_email_verification: None,
                login_state_id: None,
                session: Some(dto),
            };

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            (StatusCode::OK, response_headers, Json(SingleResponse { data: response }))
                .into_response()
        }
        Ok(acme_auth::LoginStartOutcome::TwoFactorRequired { login_state_id }) => {
            let response = LoginStartResponse {
                requires_two_factor: true,
                is_email_verification: Some(false),
                login_state_id: Some(login_state_id.to_string()),
                session: None,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Ok(acme_auth::LoginStartOutcome::EmailVerificationRequired {
            login_state_id,
            user_id,
            email,
        }) => {
            // Send the email verification code
            if let Err(e) = state
                .email_totp
                .request_code(user_id, &email, acme_db::auth::EmailTotpPurpose::Login)
                .await
            {
                tracing::error!("Failed to send email code: {}", e);
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::new("auth.email_send_failed", "Failed to send verification email"),
                )
                .into_response();
            }

            let response = LoginStartResponse {
                requires_two_factor: true,
                is_email_verification: Some(true),
                login_state_id: Some(login_state_id.to_string()),
                session: None,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn login_finish(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let code = payload.code.trim();

    let state_id = match Uuid::parse_str(payload.login_state_id.trim()) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_login_state", "Invalid login state"),
            )
            .into_response();
        }
    };

    let fingerprint = login_client_fingerprint(&headers);
    let session_fp = extract_session_fingerprint(&headers);

    // Try TOTP first
    match state
        .local_auth
        .login_finish_with_totp(state_id, code, &fingerprint, Some(session_fp.clone()))
        .await
    {
        Ok((session, role)) => {
            let refresh_token = session.refresh_token.clone();
            let dto = auth_session_dto_from_session(session, role);

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            return (StatusCode::OK, response_headers, Json(SingleResponse { data: dto })).into_response();
        }
        Err(underlay_auth::AuthError::TwoFactorNotSetUp) => {
            // Not a TOTP state - try email verification
        }
        Err(err) => {
            return map_auth_error_to_response(err);
        }
    }

    // Try email verification
    // First validate the state and code
    let user_id = match state.local_auth.get_email_login_state(state_id, &fingerprint).await {
        Ok(uid) => uid,
        Err(err) => return map_auth_error_to_response(err),
    };

    // Verify the email code
    match state
        .email_totp
        .verify_code(user_id, acme_db::auth::EmailTotpPurpose::Login, code)
        .await
    {
        Ok(_session) => {
            // Email verified - complete the login
            match state
                .local_auth
                .login_finish_email_verified(state_id, Some(session_fp))
                .await
            {
                Ok((session, role)) => {
                    let refresh_token = session.refresh_token.clone();
                    let dto = auth_session_dto_from_session(session, role);

                    let mut response_headers = HeaderMap::new();
                    if let Err(e) =
                        set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
                    {
                        tracing::warn!("Failed to set auth cookies: {}", e);
                    }

                    (StatusCode::OK, response_headers, Json(SingleResponse { data: dto })).into_response()
                }
                Err(err) => map_auth_error_to_response(err),
            }
        }
        Err(acme_auth::EmailTotpError::InvalidCode) => {
            let _ = state.local_auth.increment_email_login_attempts(state_id).await;
            error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("auth.invalid_code", "Invalid verification code"),
            )
            .into_response()
        }
        Err(acme_auth::EmailTotpError::CodeExpired) => {
            error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("auth.code_expired", "Verification code has expired"),
            )
            .into_response()
        }
        Err(acme_auth::EmailTotpError::TooManyAttempts) => {
            error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("auth.too_many_attempts", "Too many invalid attempts"),
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Email TOTP verification error: {}", e);
            error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("auth.verification_failed", "Verification failed"),
            )
            .into_response()
        }
    }
}

pub async fn refresh(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    // Accept refresh token from body (mobile) or cookie (browser)
    let refresh_token = if !payload.refresh_token.is_empty() {
        payload.refresh_token.clone()
    } else if let Some(cookie_token) = extract_refresh_token(&headers) {
        cookie_token
    } else {
        return error_response(
            StatusCode::UNAUTHORIZED,
            AppError::new("auth.missing_refresh_token", "No refresh token provided"),
        )
        .into_response();
    };

    // Extract fingerprint for session validation
    let fingerprint = extract_session_fingerprint(&headers);

    match state.local_auth.refresh_with_fingerprint(&refresh_token, Some(fingerprint)).await {
        Ok(session) => {
            let role = state
                .local_auth
                .me(session.user.id)
                .await
                .map(|(_, r)| r)
                .unwrap_or_else(|_| "user".to_string());

            let new_refresh_token = session.refresh_token.clone();
            let dto = auth_session_dto_from_session(session, role);

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &new_refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            (StatusCode::OK, response_headers, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn logout(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    // Accept refresh token from body (mobile) or cookie (browser)
    let refresh_token = if !payload.refresh_token.is_empty() {
        payload.refresh_token.clone()
    } else if let Some(cookie_token) = extract_refresh_token(&headers) {
        cookie_token
    } else {
        // No token provided - just clear cookies and return success
        // This handles the case where cookies were already cleared
        let mut response_headers = HeaderMap::new();
        let _ = clear_auth_cookies(&mut response_headers, &state.cookie_config);
        return (StatusCode::NO_CONTENT, response_headers).into_response();
    };

    match state.local_auth.logout(&refresh_token).await {
        Ok(()) => {
            let mut response_headers = HeaderMap::new();
            if let Err(e) = clear_auth_cookies(&mut response_headers, &state.cookie_config) {
                tracing::warn!("Failed to clear auth cookies: {}", e);
            }
            (StatusCode::NO_CONTENT, response_headers).into_response()
        }
        Err(err) => {
            // Even on error, try to clear cookies
            let mut response_headers = HeaderMap::new();
            let _ = clear_auth_cookies(&mut response_headers, &state.cookie_config);
            (
                StatusCode::BAD_REQUEST,
                response_headers,
                error_response(
                    StatusCode::BAD_REQUEST,
                    AppError::new(err.code(), err.message()),
                ),
            )
                .into_response()
        }
    }
}

pub async fn me(
    AuthenticatedUser(principal): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.me(principal.user_id.0).await {
        Ok((user, role)) => {
            // Use display_name if present, otherwise fall back to email username
            let display_name = user
                .display_name
                .unwrap_or_else(|| user.email.split('@').next().unwrap_or("User").to_string());

            let dto = AuthUserDto {
                user_id: user.id.to_string(),
                email: user.email,
                display_name,
                roles: roles_for_user(&role),
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn change_password(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    match state
        .local_auth
        .change_password(
            user.user_id.0,
            &payload.current_password,
            &payload.new_password,
        )
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

// ============================================================================
// Session handlers
// ============================================================================

pub async fn list_sessions(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.list_sessions(user.user_id.0).await {
        Ok(sessions) => {
            let data: Vec<SessionDto> = sessions.into_iter().map(SessionDto::from).collect();
            (StatusCode::OK, Json(ListResponse { data })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn revoke_session(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session_uuid = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_session_id", "Invalid session id"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .revoke_session_for_user(user.user_id.0, session_uuid, "user_revoked")
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            let status = match err.code() {
                "auth.forbidden" => StatusCode::FORBIDDEN,
                "auth.bad_request" => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            error_response(status, AppError::new(err.code(), err.message())).into_response()
        }
    }
}

// ============================================================================
// Password Requirements
// ============================================================================

/// Get password requirements.
///
/// Returns the password requirements configuration so UIs can display
/// accurate feedback without hardcoding values.
pub async fn password_requirements(State(state): State<AppState>) -> impl IntoResponse {
    let requirements = state.local_auth.password_requirements();
    // Return as JSON directly - the PasswordRequirements type is already Serialize
    (StatusCode::OK, Json(json!({ "data": requirements })))
}

// ============================================================================
// TOTP handlers
// ============================================================================

pub async fn totp_setup(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.totp_setup(user.user_id.0).await {
        Ok(setup) => (StatusCode::OK, Json(SingleResponse { data: setup })).into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn totp_enable(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<TotpEnableRequest>,
) -> impl IntoResponse {
    let setup_id = match Uuid::parse_str(&payload.setup_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_setup_id", "Invalid setup id"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .totp_enable(user.user_id.0, setup_id, &payload.code)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn totp_disable(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.totp_disable(user.user_id.0).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn totp_status(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.totp_is_enabled(user.user_id.0).await {
        Ok(enabled) => (
            StatusCode::OK,
            Json(SingleResponse {
                data: TotpStatusResponse { enabled },
            }),
        )
            .into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Get comprehensive 2FA status for the current user.
pub async fn two_factor_status(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.get_2fa_status(user.user_id.0).await {
        Ok(status) => (
            StatusCode::OK,
            Json(SingleResponse {
                data: TwoFactorStatusResponse {
                    has_totp_configured: status.has_totp_configured,
                    has_passkey_configured: status.has_passkey_configured,
                    totp_enabled_at: status.totp_enabled_at,
                    passkey_count: status.passkey_count,
                },
            }),
        )
            .into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

// ============================================================================
// Email TOTP handlers
// ============================================================================

pub async fn email_totp_request(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<EmailTotpRequestRequest>,
) -> impl IntoResponse {
    // Get the user's email
    let email = match state.local_auth.me(user.user_id.0).await {
        Ok((user_info, _)) => user_info.email,
        Err(err) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new(err.code(), err.message()),
            )
            .into_response();
        }
    };

    let purpose = payload.purpose.to_db_purpose();

    match state
        .email_totp
        .request_code(user.user_id.0, &email, purpose)
        .await
    {
        Ok(expires_at) => {
            let response = EmailTotpRequestResponse { expires_at };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(acme_auth::EmailTotpError::RateLimited) => {
            let mut res = error_response(
                StatusCode::TOO_MANY_REQUESTS,
                AppError::new(
                    "auth.email_totp.rate_limited",
                    "Too many code requests. Please wait before requesting another code.",
                ),
            );
            if let Ok(v) = HeaderValue::from_str("3600") {
                res.headers_mut().insert(header::RETRY_AFTER, v);
            }
            res
        }
        Err(err) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new("auth.email_totp.request_failed", err.to_string()),
        )
        .into_response(),
    }
}

pub async fn email_totp_verify(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<EmailTotpVerifyRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let purpose = payload.purpose.to_db_purpose();

    match state
        .email_totp
        .verify_code(user.user_id.0, purpose, &payload.code)
        .await
    {
        Ok(session) => {
            let response = EmailTotpVerifyResponse {
                verification_session_id: session.session_id.to_string(),
                expires_at: session.expires_at,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(acme_auth::EmailTotpError::InvalidCode) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new("auth.email_totp.invalid_code", "Invalid verification code"),
        )
        .into_response(),
        Err(acme_auth::EmailTotpError::CodeExpired) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new(
                "auth.email_totp.code_expired",
                "Verification code has expired. Please request a new one.",
            ),
        )
        .into_response(),
        Err(acme_auth::EmailTotpError::TooManyAttempts) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new(
                "auth.email_totp.too_many_attempts",
                "Too many invalid attempts. Please request a new code.",
            ),
        )
        .into_response(),
        Err(acme_auth::EmailTotpError::NoActiveCode) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new(
                "auth.email_totp.no_active_code",
                "No active verification code found. Please request a new one.",
            ),
        )
        .into_response(),
        Err(err) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new("auth.email_totp.verify_failed", err.to_string()),
        )
        .into_response(),
    }
}

// ============================================================================
// TOTP Verification handler (for 2FA gates)
// ============================================================================

pub async fn totp_verify(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<TotpVerifyRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let purpose = payload.purpose.to_db_purpose();

    match state
        .local_auth
        .verify_totp_for_verification(user.user_id.0, &payload.code, purpose)
        .await
    {
        Ok(session) => {
            let response = EmailTotpVerifyResponse {
                verification_session_id: session.id.to_string(),
                expires_at: session.expires_at,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(underlay_auth::AuthError::TwoFactorNotSetUp) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new("auth.totp.not_configured", "TOTP is not configured for this account"),
        )
        .into_response(),
        Err(underlay_auth::AuthError::TwoFactorInvalid) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new("auth.totp.invalid_code", "Invalid TOTP code"),
        )
        .into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

// ============================================================================
// Password change with verification
// ============================================================================

pub async fn change_password_with_verification(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordWithVerificationRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let verification_session_id = match Uuid::parse_str(&payload.verification_session_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_session_id", "Invalid verification session ID"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .change_password_with_verification(
            user.user_id.0,
            verification_session_id,
            &payload.new_password,
        )
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(underlay_auth::AuthError::BadRequest(msg)) if msg.contains("verification session") => {
            error_response(
                StatusCode::FORBIDDEN,
                AppError::new(
                    "auth.password.invalid_verification_session",
                    "Invalid, expired, or already used verification session",
                ),
            )
            .into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

// ============================================================================
// Password Reset (Forgot Password) handlers
// ============================================================================

/// Request a password reset email.
///
/// Sends a verification code to the provided email if the account exists.
/// Always returns success to prevent email enumeration attacks.
pub async fn password_reset_request(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetRequestRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    // Validate input
    if let Err(validation_err) = payload.validate() {
        ensure_min_response_time(start).await;
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();

    // Check if user exists (without revealing to the client)
    match state.local_auth.get_user_for_password_reset(email).await {
        Ok(Some((user_id, user_email))) => {
            // User exists - send the reset code
            if let Err(e) = state
                .email_totp
                .request_code(user_id, &user_email, acme_db::auth::EmailTotpPurpose::PasswordReset)
                .await
            {
                // Log error but don't reveal to client
                tracing::error!("Failed to send password reset email: {}", e);
            }
        }
        Ok(None) => {
            // User doesn't exist - don't reveal this to the client
            tracing::debug!("Password reset requested for non-existent email: {}", email);
        }
        Err(e) => {
            // Error looking up user - don't reveal to client
            tracing::error!("Error looking up user for password reset: {:?}", e);
        }
    }

    // Always return success to prevent email enumeration
    ensure_min_response_time(start).await;
    StatusCode::NO_CONTENT.into_response()
}

/// Verify a password reset code.
///
/// Returns a reset token that can be used to complete the password reset.
pub async fn password_reset_verify(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetVerifyRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let code = payload.code.trim();

    match state
        .local_auth
        .verify_password_reset_code(email, code, &state.email_totp)
        .await
    {
        Ok(session) => {
            let response = PasswordResetVerifyResponse {
                reset_token: session.session_id.to_string(),
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(underlay_auth::AuthError::BadRequest(msg)) => {
            error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("auth.password_reset.invalid_code", msg),
            )
            .into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Complete a password reset using a reset token.
///
/// Sets the new password and revokes all existing sessions.
pub async fn password_reset_complete(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetCompleteRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let reset_token = match Uuid::parse_str(payload.reset_token.trim()) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_reset_token", "Invalid reset token"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .complete_password_reset(reset_token, &payload.new_password)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(underlay_auth::AuthError::BadRequest(msg)) if msg.contains("reset token") => {
            error_response(
                StatusCode::BAD_REQUEST,
                AppError::new(
                    "auth.password_reset.invalid_token",
                    "Invalid or expired reset token",
                ),
            )
            .into_response()
        }
        Err(underlay_auth::AuthError::PasswordTooWeak) => error_response(
            StatusCode::BAD_REQUEST,
            AppError::new(
                "auth.password.too_weak",
                "Password does not meet strength requirements",
            ),
        )
        .into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

// ============================================================================
// Passkey handlers
// ============================================================================

use crate::dto::auth::{
    PasskeyCredentialDto, PasskeyLoginFinishRequest, PasskeyLoginStartDto, PasskeyLoginStartRequest,
    PasskeyRegisterFinishRequest, PasskeyRenameRequest, PasskeyStartRegistrationDto,
    PasskeyVerifyFinishRequest, PasskeyVerifyStartRequest,
};

/// List all passkeys for the current user.
pub async fn list_passkeys(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.list_passkeys(user.user_id.0).await {
        Ok(passkeys) => {
            let data: Vec<PasskeyCredentialDto> = passkeys
                .into_iter()
                .map(|pk| PasskeyCredentialDto {
                    id: pk.credential.id.to_string(),
                    display_name: pk.display_name,
                    metadata: serde_json::to_value(&pk.credential.metadata).unwrap_or_default(),
                    created_at: pk.credential.created_at,
                    last_used_at: pk.credential.last_used_at,
                })
                .collect();
            (StatusCode::OK, Json(ListResponse { data })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Rename a passkey.
pub async fn rename_passkey(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(credential_id): Path<String>,
    Json(payload): Json<PasskeyRenameRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let credential_uuid = match Uuid::parse_str(&credential_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_credential_id", "Invalid credential ID"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .rename_passkey(user.user_id.0, credential_uuid, &payload.display_name)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Delete a passkey.
pub async fn delete_passkey(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(credential_id): Path<String>,
) -> impl IntoResponse {
    let credential_uuid = match Uuid::parse_str(&credential_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_credential_id", "Invalid credential ID"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .delete_passkey(user.user_id.0, credential_uuid)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Start passkey registration for the current user.
pub async fn passkey_register_start(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.passkey_register_start(user.user_id.0).await {
        Ok(response) => {
            let dto = PasskeyStartRegistrationDto {
                options: response.options,
                state_id: response.state_id,
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Finish passkey registration.
pub async fn passkey_register_finish(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegisterFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let state_id = match Uuid::parse_str(&payload.state_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_state_id", "Invalid state ID"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .passkey_register_finish(
            user.user_id.0,
            state_id,
            payload.credential,
            payload.display_name.as_deref(),
        )
        .await
    {
        Ok(_credential) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Start passkey login (unauthenticated).
pub async fn passkey_login_start(
    State(state): State<AppState>,
    Json(payload): Json<PasskeyLoginStartRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    match state.local_auth.passkey_login_start(payload.email.as_deref()).await {
        Ok(response) => {
            let dto = PasskeyLoginStartDto {
                options: response.options,
                state_id: response.state_id,
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Finish passkey login (unauthenticated).
pub async fn passkey_login_finish(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyLoginFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let state_id = match Uuid::parse_str(&payload.state_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_state_id", "Invalid state ID"),
            )
            .into_response();
        }
    };

    let session_fp = extract_session_fingerprint(&headers);

    match state
        .local_auth
        .passkey_login_finish(state_id, payload.credential, Some(session_fp))
        .await
    {
        Ok(session) => {
            let role = state
                .local_auth
                .me(session.user.id)
                .await
                .map(|(_, r)| r)
                .unwrap_or_else(|_| "user".to_string());

            let refresh_token = session.refresh_token.clone();
            let dto = auth_session_dto_from_session(session, role);

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            (StatusCode::OK, response_headers, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Start passkey verification (for 2FA gates).
pub async fn passkey_verify_start(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyVerifyStartRequest>,
) -> impl IntoResponse {
    let purpose = payload.purpose.to_db_purpose();

    match state
        .local_auth
        .passkey_verify_start(user.user_id.0, purpose)
        .await
    {
        Ok(response) => {
            let dto = PasskeyLoginStartDto {
                options: response.options,
                state_id: response.state_id,
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Finish passkey verification (for 2FA gates).
pub async fn passkey_verify_finish(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyVerifyFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let state_id = match Uuid::parse_str(&payload.state_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("validation.invalid_state_id", "Invalid state ID"),
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .passkey_verify_finish(user.user_id.0, state_id, payload.credential)
        .await
    {
        Ok(session) => {
            let response = crate::dto::auth::PasskeyVerifyResponse {
                verification_session_id: session.id.to_string(),
                expires_at: session.expires_at,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}
