//! Error handling utilities for the Acme API.

use axum::{
    http::StatusCode,
    response::Response,
};
use acme_core::AppError;
use serde::Serialize;
use utoipa::ToSchema;

/// Error body within the response envelope.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    /// Optional field-specific errors, keyed by field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<std::collections::HashMap<String, String>>,
}

/// Standard error response envelope.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

/// Construct a standardised error response.
///
/// Delegates to Underlay's `error_response`.
pub fn error_response(status: StatusCode, err: AppError) -> Response {
    underlay_http::error_response(status, err)
}
