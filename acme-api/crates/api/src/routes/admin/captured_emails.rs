//! Admin routes for captured emails (dev-only).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use underlay_core::{AppError, ListResponse, SingleResponse, Uuid};

use acme_db::infra::{self, CapturedEmailFilters};

use crate::error::error_response;
use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedEmailSummaryDto {
    pub id: String,
    pub email_id: String,
    pub to_addresses: Vec<String>,
    pub from_address: String,
    pub subject: String,
    pub captured_at: DateTime<Utc>,
    pub was_delivered: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedEmailDetailDto {
    pub id: String,
    pub email_id: String,
    pub to_addresses: Vec<String>,
    pub from_address: String,
    pub reply_to: Option<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub headers_json: Option<serde_json::Value>,
    pub captured_at: DateTime<Utc>,
    pub was_delivered: bool,
    pub delivery_error: Option<String>,
}

impl From<infra::CapturedEmailRow> for CapturedEmailSummaryDto {
    fn from(row: infra::CapturedEmailRow) -> Self {
        Self {
            id: row.id.to_string(),
            email_id: row.email_id.to_string(),
            to_addresses: row.to_addresses,
            from_address: row.from_address,
            subject: row.subject,
            captured_at: row.captured_at,
            was_delivered: row.was_delivered,
        }
    }
}

impl From<infra::CapturedEmailRow> for CapturedEmailDetailDto {
    fn from(row: infra::CapturedEmailRow) -> Self {
        Self {
            id: row.id.to_string(),
            email_id: row.email_id.to_string(),
            to_addresses: row.to_addresses,
            from_address: row.from_address,
            reply_to: row.reply_to,
            cc_addresses: row.cc_addresses,
            bcc_addresses: row.bcc_addresses,
            subject: row.subject,
            text_body: row.text_body,
            html_body: row.html_body,
            headers_json: row.headers_json,
            captured_at: row.captured_at,
            was_delivered: row.was_delivered,
            delivery_error: row.delivery_error,
        }
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCapturedEmailsQuery {
    pub since: Option<String>,
    pub until: Option<String>,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// List captured emails with optional filters.
///
/// GET /v1/admin/captured-emails
pub async fn list_captured_emails(
    _user: AdminUser,
    State(state): State<AppState>,
    Query(query): Query<ListCapturedEmailsQuery>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    let parse_datetime = |value: Option<String>, label: &str| {
        value
            .map(|raw| {
                DateTime::parse_from_rfc3339(&raw)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|_| {
                        AppError::new(
                            "captured_email_invalid_filter",
                            format!("Invalid {label} date format"),
                        )
                    })
            })
            .transpose()
    };

    let since = match parse_datetime(query.since, "since") {
        Ok(value) => value,
        Err(err) => {
            return error_response(StatusCode::BAD_REQUEST, err);
        }
    };

    let until = match parse_datetime(query.until, "until") {
        Ok(value) => value,
        Err(err) => {
            return error_response(StatusCode::BAD_REQUEST, err);
        }
    };

    let filters = CapturedEmailFilters {
        since,
        until,
        to_address: query.to_address,
        from_address: query.from_address,
        limit: query.limit.unwrap_or(50).clamp(1, 200),
        offset: query.offset.unwrap_or(0).max(0),
    };

    match infra::list_captured_emails(pool, filters).await {
        Ok(rows) => {
            let data: Vec<CapturedEmailSummaryDto> =
                rows.into_iter().map(CapturedEmailSummaryDto::from).collect();
            (StatusCode::OK, Json(ListResponse { data })).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::new(
                "captured_email_query_failed",
                format!("Failed to list captured emails: {}", e),
            ),
        ),
    }
}

/// Get captured email detail by ID.
///
/// GET /v1/admin/captured-emails/:id
pub async fn get_captured_email(
    _user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&id) {
        Ok(id) => id.into_inner(),
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("captured_email_invalid_id", "Invalid captured email id"),
            )
        }
    };

    let pool = state.local_auth.pool();

    match infra::get_captured_email_by_id(pool, uuid).await {
        Ok(Some(row)) => {
            let dto = CapturedEmailDetailDto::from(row);
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            AppError::new("captured_email_not_found", "Captured email not found"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::new(
                "captured_email_query_failed",
                format!("Failed to get captured email: {}", e),
            ),
        ),
    }
}

/// Delete a captured email by ID.
///
/// DELETE /v1/admin/captured-emails/:id
pub async fn delete_captured_email(
    _user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&id) {
        Ok(id) => id.into_inner(),
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                AppError::new("captured_email_invalid_id", "Invalid captured email id"),
            )
        }
    };

    let pool = state.local_auth.pool();

    match infra::delete_captured_email(pool, uuid).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => error_response(
            StatusCode::NOT_FOUND,
            AppError::new("captured_email_not_found", "Captured email not found"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::new(
                "captured_email_delete_failed",
                format!("Failed to delete captured email: {}", e),
            ),
        ),
    }
}
