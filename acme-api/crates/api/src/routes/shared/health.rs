use axum::response::IntoResponse;
use serde::Serialize;
use underlay_http::ApiError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
