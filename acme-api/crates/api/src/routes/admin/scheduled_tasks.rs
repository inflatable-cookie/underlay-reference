//! Admin routes for scheduled task management.
//!
//! These endpoints provide visibility and control over cron scheduled tasks.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use underlay_core::{SingleResponse, Uuid};
use underlay_http::ApiError;
use underlay_jobs::JobConfig;

use crate::state::{AdminUser, AppState, DB_POOL};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ScheduledTaskSummaryDto {
    pub id: String,
    pub name: String,
    pub job_type: String,
    pub schedule: String,
    pub enabled: bool,
    pub last_scheduled_at: Option<DateTime<Utc>>,
    pub last_completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ScheduledTaskDetailDto {
    pub id: String,
    pub name: String,
    pub job_type: String,
    pub schedule: String,
    pub payload: serde_json::Value,
    pub max_attempts: i32,
    pub timeout_seconds: Option<i32>,
    pub allow_overlap: bool,
    pub priority: i32,
    pub enabled: bool,
    pub last_scheduled_at: Option<DateTime<Utc>>,
    pub last_completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskRow {
    id: uuid::Uuid,
    name: String,
    job_type: String,
    schedule: String,
    payload: serde_json::Value,
    max_attempts: i32,
    timeout_seconds: Option<i32>,
    allow_overlap: bool,
    priority: i32,
    enabled: bool,
    last_scheduled_at: Option<DateTime<Utc>>,
    last_completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ScheduledTaskSummaryDto {
    fn from_row(row: ScheduledTaskRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            job_type: row.job_type,
            schedule: row.schedule,
            enabled: row.enabled,
            last_scheduled_at: row.last_scheduled_at,
            last_completed_at: row.last_completed_at,
            created_at: row.created_at,
        }
    }
}

impl ScheduledTaskDetailDto {
    fn from_row(row: ScheduledTaskRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            job_type: row.job_type,
            schedule: row.schedule,
            payload: row.payload,
            max_attempts: row.max_attempts,
            timeout_seconds: row.timeout_seconds,
            allow_overlap: row.allow_overlap,
            priority: row.priority,
            enabled: row.enabled,
            last_scheduled_at: row.last_scheduled_at,
            last_completed_at: row.last_completed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListScheduledTasksQuery {
    pub enabled: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PaginatedScheduledTasksResponse {
    pub data: Vec<ScheduledTaskSummaryDto>,
    pub total: i64,
    pub has_more: bool,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// List scheduled tasks with optional filters.
///
/// GET /v1/admin/scheduled-tasks
pub async fn list_scheduled_tasks(
    _user: AdminUser,
    Query(query): Query<ListScheduledTasksQuery>,
) -> Result<Response, ApiError> {
    let pool = match DB_POOL.get() {
        Some(pool) => pool,
        None => {
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                "Scheduled tasks not available",
            ));
        }
    };

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let page = query.page.unwrap_or(1).max(1);
    let offset = ((page - 1) as i64) * limit;

    let total: Result<i64, sqlx::Error> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM platform.scheduled_task
        WHERE ($1::bool IS NULL OR enabled = $1)
        "#,
    )
    .bind(query.enabled)
    .fetch_one(pool)
    .await;

    let rows: Result<Vec<ScheduledTaskRow>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT *
        FROM platform.scheduled_task
        WHERE ($1::bool IS NULL OR enabled = $1)
        ORDER BY name
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(query.enabled)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let total = total.map_err(|e| {
                crate::db_errors::internal_with_diagnostics(
                    "scheduled_tasks_count_failed",
                    "Failed to count tasks",
                    &e,
                )
                .with_context(serde_json::json!({
                    "operation": "scheduled_tasks.count",
                    "enabled": query.enabled,
                    "page": page,
                    "limit": limit
                }))
            })?;
            let data: Vec<ScheduledTaskSummaryDto> = rows
                .into_iter()
                .map(ScheduledTaskSummaryDto::from_row)
                .collect();
            let has_more = offset + (data.len() as i64) < total;
            Ok((
                StatusCode::OK,
                Json(PaginatedScheduledTasksResponse {
                    data,
                    total,
                    has_more,
                }),
            )
                .into_response())
        }
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "scheduled_tasks_list_failed",
            "Failed to list tasks",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "scheduled_tasks.list",
            "enabled": query.enabled,
            "page": page,
            "limit": limit,
            "offset": offset
        }))),
    }
}

/// Get scheduled task details.
///
/// GET /v1/admin/scheduled-tasks/:task_id
pub async fn get_scheduled_task(
    _user: AdminUser,
    Path(task_id): Path<String>,
) -> Result<Response, ApiError> {
    let uuid = match Uuid::parse_str(&task_id) {
        Ok(id) => id.into_inner(),
        Err(_) => {
            return Err(ApiError::bad_request(
                "invalid_id",
                "Invalid scheduled task id",
            ));
        }
    };

    let pool = match DB_POOL.get() {
        Some(pool) => pool,
        None => {
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                "Scheduled tasks not available",
            ));
        }
    };

    let row: Result<Option<ScheduledTaskRow>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT *
        FROM platform.scheduled_task
        WHERE id = $1
        "#,
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(row)) => {
            let dto = ScheduledTaskDetailDto::from_row(row);
            Ok((StatusCode::OK, Json(SingleResponse { data: dto })).into_response())
        }
        Ok(None) => Err(ApiError::not_found("not_found", "Scheduled task not found")),
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "scheduled_task_get_failed",
            "Failed to get task",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "scheduled_tasks.get",
            "task_id": uuid
        }))),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ToggleScheduledTaskRequest {
    pub enabled: bool,
}

/// Enable or disable a scheduled task.
///
/// POST /v1/admin/scheduled-tasks/:task_id/toggle
pub async fn toggle_scheduled_task(
    _user: AdminUser,
    Path(task_id): Path<String>,
    Json(payload): Json<ToggleScheduledTaskRequest>,
) -> Result<Response, ApiError> {
    let uuid = match Uuid::parse_str(&task_id) {
        Ok(id) => id.into_inner(),
        Err(_) => {
            return Err(ApiError::bad_request(
                "invalid_id",
                "Invalid scheduled task id",
            ));
        }
    };

    let pool = match DB_POOL.get() {
        Some(pool) => pool,
        None => {
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                "Scheduled tasks not available",
            ));
        }
    };

    let result = sqlx::query(
        r#"
        UPDATE platform.scheduled_task
        SET enabled = $2
        WHERE id = $1
        "#,
    )
    .bind(uuid)
    .bind(payload.enabled)
    .execute(pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => Ok(StatusCode::NO_CONTENT.into_response()),
        Ok(_) => Err(ApiError::not_found("not_found", "Scheduled task not found")),
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "scheduled_task_toggle_failed",
            "Failed to toggle task",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "scheduled_tasks.toggle",
            "task_id": uuid,
            "enabled": payload.enabled
        }))),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TriggerScheduledTaskResponse {
    pub job_id: String,
}

/// Trigger a scheduled task immediately.
///
/// POST /v1/admin/scheduled-tasks/:task_id/trigger
pub async fn trigger_scheduled_task(
    _user: AdminUser,
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Response, ApiError> {
    let uuid = match Uuid::parse_str(&task_id) {
        Ok(id) => id.into_inner(),
        Err(_) => {
            return Err(ApiError::bad_request(
                "invalid_id",
                "Invalid scheduled task id",
            ));
        }
    };

    let Some(ref job_repo) = state.job_repository else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Job system not available",
        ));
    };

    let pool = match DB_POOL.get() {
        Some(pool) => pool,
        None => {
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                "Scheduled tasks not available",
            ));
        }
    };

    let row: Result<Option<ScheduledTaskRow>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT *
        FROM platform.scheduled_task
        WHERE id = $1
        "#,
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await;

    let task = match row {
        Ok(Some(task)) => task,
        Ok(None) => {
            return Err(ApiError::not_found("not_found", "Scheduled task not found"));
        }
        Err(e) => {
            return Err(crate::db_errors::internal_with_diagnostics(
                "scheduled_task_get_failed",
                "Failed to get task",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "scheduled_tasks.trigger",
                "task_id": uuid
            })));
        }
    };

    let config = JobConfig::new()
        .with_max_attempts(task.max_attempts as u32)
        .with_optional_timeout(task.timeout_seconds.map(|s| s as u32))
        .with_allow_overlap(task.allow_overlap)
        .with_priority(task.priority);

    match job_repo.create(&task.job_type, task.payload, &config).await {
        Ok(job_id) => {
            let body = SingleResponse {
                data: TriggerScheduledTaskResponse {
                    job_id: job_id.to_string(),
                },
            };
            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "scheduled_task_trigger_failed",
            "Failed to trigger task",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "scheduled_tasks.trigger",
            "task_id": uuid,
            "job_type": task.job_type
        }))),
    }
}

#[cfg(test)]
#[path = "../../tests/routes/admin/scheduled_tasks_tests.rs"]
mod tests;
