//! Admin routes for error log management.
//!
//! These endpoints provide visibility into HTTP errors for debugging and monitoring.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use underlay_core::{SingleResponse, Uuid};
use underlay_http::{
    count_error_logs, get_error_log_by_id, list_error_logs, ApiError, ErrorLogFilters,
};

use crate::state::{AdminUser, AppState, DB_POOL};

// ============================================================================
// DTOs
// ============================================================================

/// Summary of an error log for list views.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorLogSummaryDto {
    pub id: String,
    pub occurred_at: DateTime<Utc>,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub error_code: String,
    pub message: String,
    pub correlation_id: String,
}

/// Detailed error log information including context.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorLogDetailDto {
    pub id: String,
    pub occurred_at: DateTime<Utc>,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub error_code: String,
    pub message: String,
    pub correlation_id: String,
    pub context: serde_json::Value,
}

impl From<underlay_http::ErrorLogRow> for ErrorLogSummaryDto {
    fn from(row: underlay_http::ErrorLogRow) -> Self {
        Self {
            id: row.id.to_string(),
            occurred_at: row.occurred_at,
            endpoint: row.endpoint,
            method: row.method,
            status_code: row.status_code,
            error_code: row.error_code,
            message: row.message,
            correlation_id: row.correlation_id,
        }
    }
}

impl From<underlay_http::ErrorLogRow> for ErrorLogDetailDto {
    fn from(row: underlay_http::ErrorLogRow) -> Self {
        Self {
            id: row.id.to_string(),
            occurred_at: row.occurred_at,
            endpoint: row.endpoint,
            method: row.method,
            status_code: row.status_code,
            error_code: row.error_code,
            message: row.message,
            correlation_id: row.correlation_id,
            context: row.context,
        }
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListErrorLogsQuery {
    /// Filter by status code (e.g., 500, 404)
    pub status_code: Option<i32>,
    /// Filter by error code
    pub error_code: Option<String>,
    /// Filter by endpoint (exact match)
    pub endpoint: Option<String>,
    /// Filter by errors after this time
    pub since: Option<DateTime<Utc>>,
    /// Filter by errors before this time
    pub until: Option<DateTime<Utc>>,
    /// Maximum number of entries to return (default 50)
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Response for paginated error log lists.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PaginatedErrorLogsResponse {
    pub data: Vec<ErrorLogSummaryDto>,
    pub total: i64,
    pub has_more: bool,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// List error logs with optional filters.
///
/// GET /v1/admin/error-logs
pub async fn list_error_logs_handler(
    _user: AdminUser,
    State(_state): State<AppState>,
    Query(query): Query<ListErrorLogsQuery>,
) -> Result<Response, ApiError> {
    let Some(pool) = DB_POOL.get() else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Database not available",
        ));
    };

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let filters = ErrorLogFilters {
        since: query.since,
        until: query.until,
        status_code: query.status_code,
        error_code: query.error_code.clone(),
        endpoint: query.endpoint.clone(),
        limit,
        offset,
    };

    // Get total count for pagination
    let total = match count_error_logs(pool, &filters).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count error logs: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "error_log_count_failed",
                "Failed to count error logs",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "error_logs.count",
                "status_code": query.status_code,
                "limit": limit,
                "offset": offset
            })));
        }
    };

    // Get the error logs
    match list_error_logs(pool, filters).await {
        Ok(logs) => {
            let items: Vec<ErrorLogSummaryDto> = logs.into_iter().map(Into::into).collect();
            let has_more = (offset + items.len() as i64) < total;

            Ok(Json(PaginatedErrorLogsResponse {
                data: items,
                total,
                has_more,
            })
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list error logs: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "error_log_list_failed",
                "Failed to list error logs",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "error_logs.list",
                "status_code": query.status_code,
                "limit": limit,
                "offset": offset
            })))
        }
    }
}

/// Get details of a specific error log entry.
///
/// GET /v1/admin/error-logs/:id
pub async fn get_error_log_handler(
    _user: AdminUser,
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let Some(pool) = DB_POOL.get() else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Database not available",
        ));
    };

    match get_error_log_by_id(pool, id.into_inner()).await {
        Ok(Some(log)) => {
            let dto: ErrorLogDetailDto = log.into();
            Ok(Json(SingleResponse { data: dto }).into_response())
        }
        Ok(None) => Err(ApiError::not_found(
            "not_found",
            "Error log entry not found",
        )),
        Err(e) => {
            tracing::error!("Failed to get error log: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "error_log_get_failed",
                "Failed to get error log",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "error_logs.get",
                "error_log_id": id
            })))
        }
    }
}

/// Get error log statistics for the dashboard.
///
/// GET /v1/admin/error-logs/stats
pub async fn get_error_log_stats(
    _user: AdminUser,
    State(_state): State<AppState>,
) -> Result<Response, ApiError> {
    let Some(pool) = DB_POOL.get() else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Database not available",
        ));
    };

    let stats_row = sqlx::query(
        r#"
        SELECT
            COUNT(*)::bigint AS total_count,
            COUNT(*) FILTER (WHERE status_code >= 500 AND status_code < 600)::bigint AS server_error_count,
            COUNT(*) FILTER (WHERE status_code >= 400 AND status_code < 500)::bigint AS client_error_count
        FROM platform.error_log
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to load error log stats: {}", e);
        crate::db_errors::internal_with_diagnostics(
            "error_log_stats_failed",
            "Failed to load error log stats",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "error_logs.stats"
        }))
    })?;

    let total_count = stats_row.get::<i64, _>("total_count");
    let server_error_count = stats_row.get::<i64, _>("server_error_count");
    let client_error_count = stats_row.get::<i64, _>("client_error_count");

    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    struct ErrorLogStats {
        total_count: i64,
        server_error_count: i64,
        client_error_count: i64,
    }

    Ok(Json(SingleResponse {
        data: ErrorLogStats {
            total_count,
            server_error_count,
            client_error_count,
        },
    })
    .into_response())
}
