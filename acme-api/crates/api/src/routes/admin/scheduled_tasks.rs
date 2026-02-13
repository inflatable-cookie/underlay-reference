//! Admin routes for scheduled task management.
//!
//! These endpoints provide visibility and control over cron scheduled tasks.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use underlay_core::{ListResponse, SingleResponse, Uuid};
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
    pub limit: Option<i64>,
    pub offset: Option<i64>,
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
            ))
        }
    };

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

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
            let data: Vec<ScheduledTaskSummaryDto> = rows
                .into_iter()
                .map(ScheduledTaskSummaryDto::from_row)
                .collect();
            Ok((StatusCode::OK, Json(ListResponse { data })).into_response())
        }
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "scheduled_tasks_list_failed",
            "Failed to list tasks",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "scheduled_tasks.list",
            "enabled": query.enabled,
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
            ))
        }
    };

    let pool = match DB_POOL.get() {
        Some(pool) => pool,
        None => {
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                "Scheduled tasks not available",
            ))
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
            ))
        }
    };

    let pool = match DB_POOL.get() {
        Some(pool) => pool,
        None => {
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                "Scheduled tasks not available",
            ))
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
            ))
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
            ))
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

    let config = JobConfig {
        max_attempts: task.max_attempts as u32,
        timeout_seconds: task.timeout_seconds.map(|s| s as u32),
        allow_overlap: task.allow_overlap,
        priority: task.priority,
        ..Default::default()
    };

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
mod tests {
    use super::*;
    use acme_auth::{UserId, UserPrincipal, UserRole};
    use acme_test_utils::setup_test_db;
    use serde_json::json;

    fn skip_without_db() -> bool {
        std::env::var("DATABASE_URL").is_err() && std::env::var("TEST_DATABASE_URL").is_err()
    }

    fn admin_user() -> AdminUser {
        AdminUser(UserPrincipal {
            user_id: UserId(acme_core::Uuid::new_v7()),
            roles: vec![UserRole::Admin],
            email: Some("admin@example.com".to_string()),
            display_name: Some("Admin".to_string()),
        })
    }

    async fn insert_scheduled_task(pool: &sqlx::PgPool, enabled: bool) -> uuid::Uuid {
        sqlx::query_scalar(
            r#"
            INSERT INTO platform.scheduled_task (
                name,
                job_type,
                schedule,
                payload,
                max_attempts,
                timeout_seconds,
                allow_overlap,
                priority,
                enabled
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(format!("test_scheduled_task_{}", uuid::Uuid::now_v7()))
        .bind("projects.generate_reports")
        .bind("0 0 * * * *")
        .bind(json!({ "source": "test" }))
        .bind(3_i32)
        .bind(Some(120_i32))
        .bind(false)
        .bind(0_i32)
        .bind(enabled)
        .fetch_one(pool)
        .await
        .expect("should insert scheduled task")
    }

    async fn delete_scheduled_task(pool: &sqlx::PgPool, task_id: uuid::Uuid) {
        sqlx::query("DELETE FROM platform.scheduled_task WHERE id = $1")
            .bind(task_id)
            .execute(pool)
            .await
            .expect("should delete scheduled task");
    }

    async fn response_json(response: Response) -> serde_json::Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should read");
        serde_json::from_slice(&body).expect("response should be valid json")
    }

    #[tokio::test]
    async fn list_scheduled_tasks_respects_enabled_filter() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool();
        let _ = DB_POOL.set(db.pool_clone());

        let enabled_id = insert_scheduled_task(pool, true).await;
        let disabled_id = insert_scheduled_task(pool, false).await;

        let response = list_scheduled_tasks(
            admin_user(),
            Query(ListScheduledTasksQuery {
                enabled: Some(true),
                limit: Some(200),
                offset: Some(0),
            }),
        )
        .await
        .expect("list should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        let items = body["data"].as_array().expect("data should be an array");

        let enabled_id_str = enabled_id.to_string();
        let disabled_id_str = disabled_id.to_string();

        let has_enabled = items.iter().any(|item| item["id"] == enabled_id_str);
        let has_disabled = items.iter().any(|item| item["id"] == disabled_id_str);

        assert!(has_enabled);
        assert!(!has_disabled);

        delete_scheduled_task(pool, enabled_id).await;
        delete_scheduled_task(pool, disabled_id).await;
    }

    #[tokio::test]
    async fn toggle_scheduled_task_updates_enabled_state() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool();
        let _ = DB_POOL.set(db.pool_clone());

        let task_id = insert_scheduled_task(pool, true).await;

        let response = toggle_scheduled_task(
            admin_user(),
            Path(task_id.to_string()),
            Json(ToggleScheduledTaskRequest { enabled: false }),
        )
        .await
        .expect("toggle should succeed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let enabled: bool = sqlx::query_scalar(
            r#"
            SELECT enabled
            FROM platform.scheduled_task
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .fetch_one(pool)
        .await
        .expect("should fetch updated task");

        assert!(!enabled);

        delete_scheduled_task(pool, task_id).await;
    }

    #[tokio::test]
    async fn get_scheduled_task_rejects_invalid_uuid() {
        let result = get_scheduled_task(admin_user(), Path("not-a-uuid".to_string())).await;

        let error = result.expect_err("invalid task id should fail");
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_scheduled_task_returns_inserted_task() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool();
        let _ = DB_POOL.set(db.pool_clone());

        let task_id = insert_scheduled_task(pool, true).await;

        let response = get_scheduled_task(admin_user(), Path(task_id.to_string()))
            .await
            .expect("get should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["data"]["id"], task_id.to_string());

        delete_scheduled_task(pool, task_id).await;
    }
}
