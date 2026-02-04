//! Development-only blob storage endpoints.
//!
//! **WARNING**: These endpoints are for development only and must NOT be enabled in production.
//! They provide direct filesystem access for the LocalAdapter blob storage.

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use underlay_blob::LocalAdapter;

/// State for dev blob endpoints.
#[derive(Clone)]
pub struct DevBlobState {
    pub adapter: Arc<LocalAdapter>,
}

/// Upload a file (PUT /v1/dev-blobs/*key).
///
/// This endpoint receives the raw file bytes and writes them to the local filesystem
/// using the LocalAdapter.
pub async fn upload(
    State(state): State<DevBlobState>,
    Path(key): Path<String>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let content_type = guess_content_type(&key);

    state
        .adapter
        .write_file(&key, &body, &content_type)
        .await
        .map_err(|e| {
            tracing::error!("Failed to write file {}: {}", key, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write file: {}", e),
            )
        })?;

    tracing::debug!("Uploaded file: {}", key);
    Ok(StatusCode::OK)
}

/// Download a file (GET /v1/dev-blobs/*key).
///
/// This endpoint serves files from the local filesystem for development.
pub async fn download(
    State(state): State<DevBlobState>,
    Path(key): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let data = state.adapter.read_file(&key).await.map_err(|e| {
        let status = if e.to_string().contains("NotFound") || e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, format!("Failed to read file: {}", e))
    })?;

    let content_type = guess_content_type(&key);

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, content_type)], data).into_response())
}

/// Guess the content type from a file extension.
fn guess_content_type(key: &str) -> String {
    let extension = key.rsplit('.').next().map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        Some("json") => "application/json",
        Some("txt") => "text/plain",
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        _ => "application/octet-stream",
    }
    .to_string()
}
