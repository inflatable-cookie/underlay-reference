//! Authentication route handlers.
//!
//! These endpoints handle user registration, login, logout, token refresh, password changes,
//! and session management.
//!
//! ## Hybrid Token Auth
//!
//! Auth endpoints use cookie-first refresh token handling:
//! - Access token: Returned in response body, sent as `Authorization: Bearer` header
//! - Refresh token: Set as httpOnly cookie by default
//! - Optional body refresh token: only returned when `X-Auth-Token-Mode: body` is sent
//! - `logged_in` cookie: UI flag for CSS switching (not httpOnly)
//!
//! The refresh endpoint accepts the refresh token from EITHER request body or cookie.

mod basic;
mod email_totp;
mod passkeys;
mod password_reset;
mod sessions;
mod totp;

pub use basic::*;
pub use email_totp::*;
pub use passkeys::*;
pub use password_reset::*;
pub use sessions::*;
pub use totp::*;

use acme_core::Uuid;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use underlay_core::{ListResponse, SingleResponse};
use underlay_http::{
    clear_auth_cookies, context::RequestContext, extract_refresh_token, set_auth_cookies, ApiError,
    SameSite,
};
use validator::Validate;

use crate::dto::auth::{
    auth_session_dto_from_session, roles_for_user, AuthUserDto, ChangePasswordRequest,
    ChangePasswordWithVerificationRequest, EmailTotpRequestRequest, EmailTotpRequestResponse,
    EmailTotpVerifyRequest, EmailTotpVerifyResponse, LoginFinishRequest, LoginRequest,
    LoginStartRequest, LoginStartResponse, LogoutRequest, PasswordResetCompleteRequest,
    PasswordResetRequestRequest, PasswordResetVerifyRequest, PasswordResetVerifyResponse,
    RefreshRequest, RegisterRequest, SessionDto, TotpEnableRequest, TotpStatusResponse,
    TotpVerifyRequest, TwoFactorStatusResponse,
};
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

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CsrfTokenResponse {
    pub csrf_token: String,
}

/// Ensure a minimum amount of time has elapsed since `start`.
///
/// Used to prevent timing attacks by making all code paths take similar time.
pub(super) async fn ensure_min_response_time(start: Instant) {
    let elapsed = start.elapsed();
    let min_duration = Duration::from_millis(MIN_RESPONSE_TIME_MS);
    if elapsed < min_duration {
        tokio::time::sleep(min_duration - elapsed).await;
    }
}

/// The client IP this app is willing to use as policy input.
///
/// Resolved by Underlay's peer-aware [`RequestContext`], which consults the
/// [`TrustedProxyConfig`](underlay_http::TrustedProxyConfig) extension
/// installed at bootstrap and falls back to the socket peer. A forwarding
/// header is honoured only because the declared proxy topology says a trusted
/// proxy set it — never merely because the header is present.
///
/// This is the single client-IP entry point for auth. Rate limiting, lockout,
/// session fingerprints, and auth audit all key on it, so a second
/// handler-local parse would be a way to bypass the trust boundary.
pub(super) fn client_ip(ctx: &RequestContext) -> Option<String> {
    ctx.ip_address().map(|ip| ip.to_string())
}

/// Session fingerprint (client IP + User-Agent) used to detect a refresh
/// arriving from a different client than the one that created the session.
pub(super) fn session_fingerprint(ctx: &RequestContext) -> SessionFingerprint {
    SessionFingerprint::new(client_ip(ctx), ctx.user_agent().map(|ua| ua.to_string()))
}

pub(crate) fn csrf_cookie_name(config: &underlay_http::AuthCookieConfig) -> String {
    format!("{}csrf_token", config.cookie_prefix())
}

pub(crate) fn extract_csrf_token(
    headers: &HeaderMap,
    config: &underlay_http::AuthCookieConfig,
) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    let cookie_name = csrf_cookie_name(config);
    let prefix = format!("{}=", cookie_name);

    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix(&prefix) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

/// Token to put in the CSRF cookie and JSON body for this request.
///
/// Reuses a non-empty CSRF cookie so a second tab cannot rotate the
/// browser-wide cookie out from under the first tab's cached header.
pub(crate) fn csrf_token_to_issue(
    headers: &HeaderMap,
    config: &underlay_http::AuthCookieConfig,
) -> String {
    extract_csrf_token(headers, config).unwrap_or_else(|| Uuid::new_v7().to_string())
}

pub(crate) fn set_csrf_cookie(
    headers: &mut HeaderMap,
    csrf_token: &str,
    config: &underlay_http::AuthCookieConfig,
) -> Result<(), String> {
    let same_site = match config.same_site() {
        SameSite::Lax => "Lax",
        SameSite::Strict => "Strict",
        SameSite::None => "None",
    };

    let mut cookie = format!(
        "{}={}; SameSite={}; Path=/; Max-Age={}",
        csrf_cookie_name(config),
        csrf_token,
        same_site,
        config.refresh_token_max_age()
    );

    if config.secure() {
        cookie.push_str("; Secure");
    }

    if let Some(domain) = config.domain() {
        cookie.push_str(&format!("; Domain={}", domain));
    }

    let value =
        HeaderValue::from_str(&cookie).map_err(|e| format!("invalid CSRF cookie header: {}", e))?;
    headers.append(header::SET_COOKIE, value);
    Ok(())
}

pub async fn csrf_token(
    headers: HeaderMap,
    State(cookie_config): State<underlay_http::AuthCookieConfig>,
) -> impl IntoResponse {
    let csrf_token = csrf_token_to_issue(&headers, &cookie_config);

    let mut response_headers = HeaderMap::new();
    if let Err(e) = set_csrf_cookie(&mut response_headers, &csrf_token, &cookie_config) {
        tracing::warn!("Failed to set CSRF cookie: {}", e);
    }

    (
        StatusCode::OK,
        response_headers,
        Json(SingleResponse {
            data: CsrfTokenResponse { csrf_token },
        }),
    )
}

/// Convert validation errors to an ApiError response.
pub(super) fn validation_error_response(
    validation_err: validator::ValidationErrors,
) -> impl IntoResponse {
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
    ApiError::bad_request(
        "auth.validation_failed",
        "There is a problem with one or more fields.",
    )
    .with_field_errors(field_errors)
    .into_response()
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
pub(super) fn map_auth_error_to_response(
    err: underlay_auth::AuthError,
) -> axum::response::Response {
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
        let mut response = ApiError::new(status, err.code(), err.message()).into_response();
        if let Ok(value) = HeaderValue::try_from(retry_after_seconds.to_string()) {
            response.headers_mut().insert("Retry-After", value);
        }
        return response;
    }

    ApiError::new(status, err.code(), err.message()).into_response()
}

/// Hash of the client's resolved IP + User-Agent, used to bind multi-step
/// login state to one client. The IP comes from the peer-aware trusted-proxy
/// path only: a spoofed `X-Forwarded-For` from an untrusted peer cannot
/// influence it.
pub(super) fn login_client_fingerprint(ctx: &RequestContext) -> String {
    let ua = ctx.user_agent().unwrap_or("");
    let ip = client_ip(ctx).unwrap_or_default();

    let raw = format!("{ip}|{ua}");
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    hex::encode(h.finalize())
}

pub(super) fn include_refresh_token_in_body(headers: &HeaderMap) -> bool {
    headers
        .get("x-auth-token-mode")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("body"))
}

#[cfg(test)]
#[path = "../../../tests/routes/client_ip_tests.rs"]
mod client_ip_tests;

#[cfg(test)]
#[path = "../../../tests/routes/csrf_tests.rs"]
mod csrf_tests;
