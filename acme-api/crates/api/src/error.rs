//! Error handling utilities for the Acme API.

use acme_core::AppError;
use axum::{http::StatusCode, response::Response};

pub use underlay_http::{ApiError, ApiResult};

/// Legacy compatibility helper.
///
/// Prefer returning `ApiError` directly from handlers.
#[deprecated(note = "Prefer ApiError/ApiResult directly in route handlers")]
pub fn error_response(status: StatusCode, err: AppError) -> Response {
    underlay_http::error_response(status, err)
}
