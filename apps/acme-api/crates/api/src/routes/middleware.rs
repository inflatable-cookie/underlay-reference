//! Cross-cutting HTTP policy applied above the route families.
//!
//! Both layers here are app-owned policy, not Underlay mechanics:
//!
//! - [`api_version_middleware`] enforces this app's declared `X-Api-Version`
//!   header across the business families.
//! - [`csrf_protection_middleware`] protects cookie-backed browser mutations.
//!
//! Neither reads the environment per request. Startup resolves the policy once
//! (see `acme_infra::resolve_csrf_protection`) and hands the decision here, so
//! a later environment change cannot weaken a running deployment.

use axum::extract::State;
use axum::http::{header::HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::sync::OnceLock;
use underlay_http::{ApiError, AuthCookieConfig};

use crate::routes::shared::auth::extract_csrf_token;

/// The resolved CSRF decision plus the cookie names needed to check it.
///
/// Built once in `main.rs`. Deliberately narrower than `AppState`: the
/// middleware needs exactly these two things, and a small state makes the
/// policy directly testable against a router without standing up auth, email,
/// and blob services.
#[derive(Clone)]
pub struct CsrfState {
    /// Resolved under the environment-aware policy. A deployed environment can
    /// never reach `false` here — startup rejects the attempt.
    pub enabled: bool,
    pub cookie_config: AuthCookieConfig,
}

impl CsrfState {
    pub fn new(enabled: bool, cookie_config: AuthCookieConfig) -> Self {
        Self {
            enabled,
            cookie_config,
        }
    }
}

fn supported_api_versions() -> &'static Vec<String> {
    static SUPPORTED: OnceLock<Vec<String>> = OnceLock::new();
    SUPPORTED.get_or_init(|| {
        let configured = std::env::var("SUPPORTED_API_VERSIONS").unwrap_or_default();
        let parsed: Vec<String> = configured
            .split(',')
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToString::to_string)
            .collect();

        if parsed.is_empty() {
            vec!["2025-01-01".to_string()]
        } else {
            parsed
        }
    })
}

fn default_api_version() -> &'static String {
    static DEFAULT: OnceLock<String> = OnceLock::new();
    DEFAULT.get_or_init(|| {
        let supported = supported_api_versions();
        let fallback = supported
            .first()
            .cloned()
            .unwrap_or_else(|| "2025-01-01".to_string());

        match std::env::var("DEFAULT_API_VERSION") {
            Ok(version) if supported.contains(&version) => version,
            Ok(version) => {
                tracing::warn!(
                    default_api_version = %version,
                    supported_versions = ?supported,
                    "DEFAULT_API_VERSION is not in SUPPORTED_API_VERSIONS; falling back"
                );
                fallback
            }
            Err(_) => fallback,
        }
    })
}

pub async fn api_version_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
    if !req.uri().path().starts_with("/v1/") {
        return next.run(req).await;
    }

    let supported = supported_api_versions();
    let requested = req
        .headers()
        .get("x-api-version")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(default_api_version().as_str())
        .to_string();

    if !supported.iter().any(|v| v == &requested) {
        return ApiError::bad_request(
            "api.unsupported_version",
            "Unsupported API version. Set X-Api-Version to a supported version.",
        )
        .with_context(serde_json::json!({
            "requested_version": requested,
            "supported_versions": supported,
            "default_version": default_api_version(),
        }))
        .into_response();
    }

    let mut response = next.run(req).await;
    if let Ok(version_header) = HeaderValue::from_str(&requested) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-api-version"), version_header);
    }
    response
}

/// Unauthenticated bootstrap endpoints, exempt from CSRF under the route-family
/// contract's bounded exemption.
///
/// A route belongs here only when the caller cannot yet hold a session cookie,
/// so there is no cookie for an attacker to ride. Authenticated cookie-backed
/// mutations — including passkey **registration**, which requires a logged-in
/// user — must never appear in this list. Passkey *login* start/finish do
/// belong here: they are the bootstrap that creates the session.
pub fn is_csrf_exempt_bootstrap_path(path: &str) -> bool {
    matches!(
        path,
        "/v1/auth/register"
            | "/v1/auth/login"
            | "/v1/auth/login/start"
            | "/v1/auth/login/finish"
            | "/v1/auth/csrf-token"
            | "/v1/auth/password/reset/request"
            | "/v1/auth/password/reset/verify"
            | "/v1/auth/password/reset/complete"
            | "/v1/auth/passkeys/login/start"
            | "/v1/auth/passkeys/login/finish"
    )
}

pub async fn csrf_protection_middleware(
    State(csrf): State<CsrfState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Resolved once at bootstrap under the environment-aware policy. Deployed
    // environments cannot reach `false` here: startup rejects the attempt.
    if !csrf.enabled {
        return next.run(req).await;
    }

    let method = req.method();
    if !matches!(
        method,
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
    ) {
        return next.run(req).await;
    }

    if is_csrf_exempt_bootstrap_path(req.uri().path()) {
        return next.run(req).await;
    }

    // Only enforce for browser-style cookie flows.
    // Mobile/native clients can keep using bearer tokens without CSRF.
    let headers = req.headers();
    let has_refresh_cookie =
        underlay_http::extract_refresh_token(headers, &csrf.cookie_config).is_some();
    if !has_refresh_cookie {
        return next.run(req).await;
    }

    let cookie_token = extract_csrf_token(headers, &csrf.cookie_config);
    let header_token = headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if cookie_token.is_none() || header_token.is_none() || cookie_token != header_token {
        return underlay_http::ApiError::new(
            StatusCode::FORBIDDEN,
            "auth.csrf.invalid",
            "CSRF validation failed",
        )
        .into_response();
    }

    next.run(req).await
}

#[cfg(test)]
#[path = "../tests/routes/middleware_tests.rs"]
mod tests;
