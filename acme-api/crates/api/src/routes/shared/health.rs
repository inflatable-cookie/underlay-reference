use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthData {
    pub status: &'static str,
}

pub async fn health() -> axum::response::Response {
    underlay_http::ok(HealthData { status: "ok" })
}
