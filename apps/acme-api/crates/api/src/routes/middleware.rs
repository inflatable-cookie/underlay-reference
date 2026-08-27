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
use std::sync::Arc;
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

/// The declared API-version vocabulary, resolved once at bootstrap from typed
/// config.
///
/// Previously two `OnceLock`s read `SUPPORTED_API_VERSIONS` and
/// `DEFAULT_API_VERSION` lazily from process env on first request. That put a
/// policy input outside the config/bootstrap boundary and made the effective
/// version set depend on whichever request arrived first.
#[derive(Clone)]
pub struct ApiVersionState {
    supported: Arc<Vec<String>>,
    default: Arc<String>,
}

impl ApiVersionState {
    /// Build from the loaded typed config. `AppBehaviorConfig::load` has
    /// already guaranteed the default is one of the supported versions.
    pub fn from_behavior(api: &acme_infra::ApiBehaviorDefaults) -> Self {
        Self {
            supported: Arc::new(api.supported_versions.clone()),
            default: Arc::new(api.default_version.clone()),
        }
    }

    pub fn supports(&self, version: &str) -> bool {
        self.supported.iter().any(|v| v == version)
    }

    pub fn default_version(&self) -> &str {
        &self.default
    }

    pub fn supported_versions(&self) -> &[String] {
        &self.supported
    }
}

/// True when a request is a business-family surface subject to the declared
/// `X-Api-Version` header.
///
/// Path versioning is the baseline; the header is optional until an app
/// declares one. This app has declared it — the TypeScript client sends it on
/// every request — so the server applies it consistently across shared, front,
/// and admin. Runtime endpoints are exempt by contract even when they sit
/// under `/v1`: readiness, metrics, and OpenAPI must stay callable by platform
/// infrastructure that knows nothing about the app's version vocabulary.
pub fn is_versioned_business_path(path: &str) -> bool {
    path.starts_with("/v1/") && !crate::routes::runtime::is_runtime_path(path)
}

pub async fn api_version_middleware(
    State(versions): State<ApiVersionState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if !is_versioned_business_path(req.uri().path()) {
        return next.run(req).await;
    }

    let requested = req
        .headers()
        .get("x-api-version")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| versions.default_version())
        .to_string();

    if !versions.supports(&requested) {
        return ApiError::bad_request(
            "api.unsupported_version",
            "Unsupported API version. Set X-Api-Version to a supported version.",
        )
        .with_context(serde_json::json!({
            "requested_version": requested,
            "supported_versions": versions.supported_versions(),
            "default_version": versions.default_version(),
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
