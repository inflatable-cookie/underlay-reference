use serde::Serialize;

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
