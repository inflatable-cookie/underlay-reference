//! Runtime family: operational surfaces for the service itself.
//!
//! These are not product resources. They carry no authentication, no role
//! gate, and no CSRF, and they are exempt from the business `X-Api-Version`
//! header — platform infrastructure and browsers must be able to call them
//! without knowing the app's version vocabulary.
//!
//! Paths are historical and deliberately unchanged: `/v1/health` sits under
//! `/v1` even though it is not a versioned business surface. Family ownership
//! is expressed by this module, not by the path string.

use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::Serialize;
use underlay_http::ApiError;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HealthData {
    pub status: &'static str,
}

/// Health check endpoint.
///
/// Returns the service health status. Used by load balancers and monitoring.
pub async fn health() -> axum::response::Response {
    underlay_http::ok(HealthData { status: "ok" })
}

/// Dev-only smoke endpoint for validating error logging capture.
///
/// This intentionally returns a 500 with structured context so developers can
/// validate that `platform.error_log` captures `error_code`, `message`, and
/// `context.handler_context`.
#[cfg(debug_assertions)]
pub async fn error_smoke() -> axum::response::Response {
    ApiError::internal(
        "smoke.forced_db_failure",
        "Forced failure for error-log smoke testing",
    )
    .with_context(serde_json::json!({
        "operation": "smoke.error_logging_capture",
        "failure_class": "forced_db_failure",
        "component": "acme-api",
    }))
    .into_response()
}

/// Paths owned by the runtime family.
///
/// The version middleware consults this so a runtime endpoint under `/v1` is
/// never asked for a business API version. Keep it in step with
/// [`build_runtime_router`]: a runtime route missing from here would start
/// demanding a version header.
pub fn is_runtime_path(path: &str) -> bool {
    matches!(path, "/v1/health" | "/favicon.ico")
        || path == "/api/openapi.json"
        || path.starts_with("/api/docs")
        // Debug-only smoke endpoint; see `build_runtime_router`.
        || path == "/v1/dev/error-smoke"
}

/// Build the runtime family.
///
/// `include_docs` controls OpenAPI exposure. `main.rs` passes
/// `env.is_development()`, so staging, production, and any unrecognised
/// environment serve neither the JSON document nor Swagger UI. Changing that
/// is a deployment policy decision, not a route change.
pub fn build_runtime_router<S>(include_docs: bool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let router = Router::new();

    let router = if include_docs {
        router.merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
    } else {
        router
    };

    let router = router
        .route("/v1/health", get(health))
        // Return 204 rather than 404 so browsers stop logging a failed favicon.
        .route(
            "/favicon.ico",
            get(|| async { axum::http::StatusCode::NO_CONTENT }),
        );

    // Forces a structured 500 so `platform.error_log` capture can be verified
    // end to end. Debug builds only: never compiled into a release binary.
    #[cfg(debug_assertions)]
    let router = router.route("/v1/dev/error-smoke", axum::routing::post(error_smoke));

    router
}

#[cfg(test)]
#[path = "../tests/routes/runtime_tests.rs"]
mod tests;
