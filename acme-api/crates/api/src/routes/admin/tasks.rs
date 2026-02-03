//! Task admin routes.
//!
//! Demonstrates:
//! - Nested resource routes (projects/:id/tasks)
//! - Filtering by status, priority
//! - Reordering within parent
//! - Label management (many-to-many relations)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use underlay_http::query::QueryParams;

use acme_core::Uuid;
use acme_db::{activity, tasks};

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub position: i32,
    pub weight: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<tasks::TaskRow> for TaskResponse {
    fn from(row: tasks::TaskRow) -> Self {
        Self {
            id: row.id.to_string(),
            project_id: row.project_id.to_string(),
            title: row.title,
            description: row.description,
            status: row.status,
            priority: row.priority,
            due_date: row.due_date.map(|d| d.to_string()),
            completed_at: row.completed_at.map(|dt| dt.to_rfc3339()),
            position: row.position,
            weight: row.weight,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskWithLabelsResponse {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub position: i32,
    pub weight: i32,
    pub created_at: String,
    pub updated_at: String,
    pub label_count: i64,
}

impl From<tasks::TaskWithLabelsRow> for TaskWithLabelsResponse {
    fn from(row: tasks::TaskWithLabelsRow) -> Self {
        Self {
            id: row.id.to_string(),
            project_id: row.project_id.to_string(),
            title: row.title,
            description: row.description,
            status: row.status,
            priority: row.priority,
            due_date: row.due_date.map(|d| d.to_string()),
            completed_at: row.completed_at.map(|dt| dt.to_rfc3339()),
            position: row.position,
            weight: row.weight,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            label_count: row.label_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub color: String,
    pub weight: i32,
    pub created_at: String,
}

impl From<tasks::LabelRow> for LabelResponse {
    fn from(row: tasks::LabelRow) -> Self {
        Self {
            id: row.id.to_string(),
            project_id: row.project_id.to_string(),
            name: row.name,
            color: row.color,
            weight: row.weight,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub label_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<Option<NaiveDate>>,
    pub label_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLabelsRequest {
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteTasksRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateTaskStatusRequest {
    pub ids: Vec<Uuid>,
    pub status: String,
}

// ============================================================================
// Task Handlers
// ============================================================================

/// List tasks for a project (admin).
///
/// Supports filtering and sorting:
/// - `sort=position:asc,dueDate:asc`
/// - `filter[status]=pending`
/// - `filter[priority]=high`
pub async fn list_tasks(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match tasks::list_tasks_admin(pool, project_id.into_inner(), &query).await {
        Ok(task_list) => {
            let response: Vec<TaskWithLabelsResponse> =
                task_list.into_iter().map(Into::into).collect();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get a single task.
pub async fn get_task(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match tasks::get_task(pool, task_id.into_inner()).await {
        Ok(Some(task)) => {
            let response: TaskResponse = task.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a task (admin).
pub async fn create_task(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let task_id = Uuid::new_v7().into_inner();
    let project_id = project_id.into_inner();
    let priority = req.priority.as_deref().unwrap_or("medium");

    // Create the task
    match tasks::create_task(
        pool,
        task_id,
        project_id,
        &req.title,
        req.description.as_deref(),
        priority,
        req.due_date,
    )
    .await
    {
        Ok(task) => {
            // Set labels if provided
            if let Some(label_ids) = req.label_ids {
                let ids: Vec<_> = label_ids.iter().map(|id| id.into_inner()).collect();
                if let Err(e) = tasks::set_task_labels(pool, task_id, &ids).await {
                    tracing::warn!("Failed to set task labels: {}", e);
                }
            }

            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "create",
                    resource_type: "task",
                    resource_id: task_id,
                    details: Some(serde_json::json!({ "title": req.title, "projectId": project_id.to_string() })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: TaskResponse = task.into();
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Update a task (admin).
pub async fn update_task(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let task_id_inner = task_id.into_inner();

    match tasks::update_task(
        pool,
        task_id_inner,
        req.title.as_deref(),
        req.description.as_ref().map(|d| d.as_deref()),
        req.status.as_deref(),
        req.priority.as_deref(),
        req.due_date,
    )
    .await
    {
        Ok(Some(task)) => {
            // Update labels if provided
            if let Some(label_ids) = req.label_ids {
                let ids: Vec<_> = label_ids.iter().map(|id| id.into_inner()).collect();
                if let Err(e) = tasks::set_task_labels(pool, task_id_inner, &ids).await {
                    tracing::warn!("Failed to update task labels: {}", e);
                }
            }

            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "update",
                    resource_type: "task",
                    resource_id: task_id_inner,
                    details: Some(serde_json::json!({ "title": task.title })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: TaskResponse = task.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to update task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Soft delete a task (admin).
pub async fn soft_delete_task(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let batch_id = Uuid::new_v7().into_inner();
    let tid = task_id.into_inner();

    match tasks::soft_delete_task(pool, tid, batch_id).await {
        Ok(true) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "delete",
                    resource_type: "task",
                    resource_id: tid,
                    details: None,
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to soft delete task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Reorder tasks within a project.
pub async fn reorder_tasks(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<ReorderRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match tasks::reorder_tasks(pool, project_id.into_inner(), &ids).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to reorder tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ============================================================================
// Label Handlers
// ============================================================================

/// List labels for a project.
pub async fn list_labels(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match tasks::list_labels_for_project(pool, project_id.into_inner()).await {
        Ok(labels) => {
            let response: Vec<LabelResponse> = labels.into_iter().map(Into::into).collect();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list labels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a label for a project.
pub async fn create_label(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateLabelRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let label_id = Uuid::new_v7().into_inner();
    let color = req.color.as_deref().unwrap_or("#6366f1");

    match tasks::create_label(pool, label_id, project_id.into_inner(), &req.name, color).await {
        Ok(label) => {
            let response: LabelResponse = label.into();
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create label: {}", e);

            // Check for unique constraint violation
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({
                            "ok": false,
                            "error": {
                                "code": "label.name_exists",
                                "message": "A label with this name already exists in this project"
                            }
                        })),
                    )
                        .into_response();
                }
            }

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get labels for a task.
pub async fn get_task_labels(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match tasks::get_labels_for_task(pool, task_id.into_inner()).await {
        Ok(labels) => {
            let response: Vec<LabelResponse> = labels.into_iter().map(Into::into).collect();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get task labels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Set labels for a task (replaces all).
pub async fn set_task_labels(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<SetLabelsRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let ids: Vec<_> = req.label_ids.iter().map(|id| id.into_inner()).collect();

    match tasks::set_task_labels(pool, task_id.into_inner(), &ids).await {
        Ok(()) => {
            // Return the updated labels
            match tasks::get_labels_for_task(pool, task_id.into_inner()).await {
                Ok(labels) => {
                    let response: Vec<LabelResponse> = labels.into_iter().map(Into::into).collect();
                    Json(serde_json::json!({ "data": response })).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to get updated labels: {}", e);
                    Json(serde_json::json!({ "ok": true })).into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to set task labels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Batch delete tasks.
///
/// POST /v1/admin/projects/:project_id/tasks:batch-delete
pub async fn batch_delete_tasks(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BatchDeleteTasksRequest>,
) -> impl IntoResponse {
    if req.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "validation.empty_ids",
                    "message": "At least one ID is required"
                }
            })),
        )
            .into_response();
    }

    let pool = state.local_auth.pool();
    let batch_id = Uuid::new_v7().into_inner();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match tasks::batch_soft_delete_tasks(pool, &ids, batch_id).await {
        Ok(count) => {
            // Log activity for batch operation
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "batch_delete",
                    resource_type: "task",
                    resource_id: batch_id,
                    details: Some(serde_json::json!({
                        "count": count,
                        "ids": req.ids,
                        "projectId": project_id.to_string()
                    })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            Json(serde_json::json!({ "ok": true, "deleted": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to batch delete tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Batch update task status.
///
/// POST /v1/admin/projects/:project_id/tasks:batch-update
pub async fn batch_update_task_status(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BatchUpdateTaskStatusRequest>,
) -> impl IntoResponse {
    if req.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "validation.empty_ids",
                    "message": "At least one ID is required"
                }
            })),
        )
            .into_response();
    }

    // Validate status value
    let valid_statuses = ["pending", "in_progress", "completed", "cancelled"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "validation.invalid_status",
                    "message": "Invalid status value"
                }
            })),
        )
            .into_response();
    }

    let pool = state.local_auth.pool();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match tasks::batch_update_task_status(pool, &ids, &req.status).await {
        Ok(count) => {
            // Log activity for batch operation
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "batch_update",
                    resource_type: "task",
                    resource_id: Uuid::new_v7().into_inner(),
                    details: Some(serde_json::json!({
                        "count": count,
                        "ids": req.ids,
                        "status": req.status,
                        "projectId": project_id.to_string()
                    })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            Json(serde_json::json!({ "ok": true, "updated": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to batch update task status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
