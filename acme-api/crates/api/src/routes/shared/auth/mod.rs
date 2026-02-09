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
use underlay_http::{clear_auth_cookies, extract_refresh_token, set_auth_cookies, ApiError};
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

/// Extract session fingerprint from request headers with proxy trust validation.
///
/// Returns IP address (from X-Forwarded-For or X-Real-IP if trusted) and User-Agent.
/// Only trusts proxy headers if properly configured via TRUSTED_PROXIES env var.
pub(super) fn extract_session_fingerprint(
    headers: &HeaderMap,
    config: &acme_infra::TrustedProxyConfig,
) -> SessionFingerprint {
    // Use secure IP extraction that validates proxy headers
    let ip_address = acme_infra::extract_client_ip(headers, config);

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
pub(super) fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
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

/// Convert validation errors to an ApiError response.
pub(super) fn validation_error_response(validation_err: validator::ValidationErrors) -> impl IntoResponse {
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
pub(super) fn map_auth_error_to_response(err: underlay_auth::AuthError) -> axum::response::Response {
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

pub(super) fn login_client_fingerprint(headers: &HeaderMap) -> String {
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
