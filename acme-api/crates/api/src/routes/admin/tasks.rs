//! Task admin routes.
//!
//! Demonstrates:
//! - Nested resource routes (projects/:id/tasks)
//! - Filtering by status, priority
//! - Reordering within parent
//! - Label management (many-to-many relations)

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use underlay_http::{context::RequestContext, query::QueryParams, ApiError};
use uuid::Uuid as RawUuid;

use acme_core::Uuid;
use acme_db::{activity, tasks};

use crate::routes::admin::freshness::{
    build_etag_cache_headers, detail_etag, if_match_mismatch, maybe_not_modified,
    precondition_failed_error,
};
use crate::routes::admin::reorder_conflict::reorder_conflict_error;
use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub label_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<Option<NaiveDate>>,
    pub label_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReorderRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetLabelsRequest {
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchDeleteTasksRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let project_id = project_id.into_inner();

    match tasks::list_tasks_admin(pool, project_id, &query).await {
        Ok(task_list) => {
            let response: Vec<TaskWithLabelsResponse> =
                task_list.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list tasks: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.list_failed",
                "Failed to list tasks",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.list",
                "project_id": project_id
            })))
        }
    }
}

/// Get a single task.
pub async fn get_task(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let task_id = task_id.into_inner();

    match tasks::get_task(pool, task_id).await {
        Ok(Some(task)) => {
            let etag = detail_etag("task", &task.id.to_string(), &task.updated_at.to_rfc3339());
            if let Some(not_modified) = maybe_not_modified(&headers, &etag) {
                return Ok(not_modified);
            }
            let response: TaskResponse = task.into();
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("tasks.not_found", "Task not found").with_context(
                serde_json::json!({
                    "operation": "tasks.get",
                    "task_id": task_id
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to get task: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.get_failed",
                "Failed to get task",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.get",
                "task_id": task_id
            })))
        }
    }
}

/// Create a task (admin).
pub async fn create_task(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Response, ApiError> {
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
                    details: Some(serde_json::json!({ "title": req.title, "project_id": project_id.to_string() })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: TaskResponse = task.into();
            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.create_failed",
                "Failed to create task",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.create",
                "task_id": task_id,
                "project_id": project_id,
                "title": &req.title
            })))
        }
    }
}

/// Update a task (admin).
pub async fn update_task(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    ctx: RequestContext,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let project_id = project_id.into_inner();
    let task_id_inner = task_id.into_inner();

    let current = match tasks::get_task(pool, task_id_inner).await {
        Ok(Some(task)) => task,
        Ok(None) => {
            return Err(
                ApiError::not_found("tasks.not_found", "Task not found").with_context(
                    serde_json::json!({
                        "operation": "tasks.update",
                        "task_id": task_id_inner
                    }),
                ),
            );
        }
        Err(e) => {
            tracing::error!("Failed to load current task before update: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "tasks.update_failed",
                "Failed to update task",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.update",
                "task_id": task_id_inner
            })));
        }
    };

    let current_etag = detail_etag(
        "task",
        &current.id.to_string(),
        &current.updated_at.to_rfc3339(),
    );
    if if_match_mismatch(&headers, &current_etag) {
        return Err(precondition_failed_error().with_context(serde_json::json!({
            "operation": "tasks.update",
            "task_id": task_id_inner
        })));
    }

    match tasks::update_task(
        pool,
        task_id_inner,
        project_id,
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: TaskResponse = task.into();
            let etag = detail_etag("task", &response.id, &response.updated_at);
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("tasks.not_found", "Task not found").with_context(
                serde_json::json!({
                    "operation": "tasks.update",
                    "task_id": task_id_inner
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to update task: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.update_failed",
                "Failed to update task",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.update",
                "task_id": task_id_inner
            })))
        }
    }
}

/// Soft delete a task (admin).
pub async fn soft_delete_task(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Ok(false) => Err(
            ApiError::not_found("tasks.not_found", "Task not found").with_context(
                serde_json::json!({
                    "operation": "tasks.soft_delete",
                    "task_id": tid
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to soft delete task: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.soft_delete_failed",
                "Failed to delete task",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.soft_delete",
                "task_id": tid,
                "batch_id": batch_id
            })))
        }
    }
}

/// Reorder tasks within a project.
pub async fn reorder_tasks(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<ReorderRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let project_id = project_id.into_inner();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match tasks::reorder_tasks(pool, project_id, &ids).await {
        Ok(result) => map_reorder_tasks_result(project_id, ids.len(), result),
        Err(e) => {
            tracing::error!("Failed to reorder tasks: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.reorder_failed",
                "Failed to reorder tasks",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.reorder",
                "project_id": project_id,
                "count": ids.len()
            })))
        }
    }
}

fn map_reorder_tasks_result(
    project_id: RawUuid,
    submitted_count: usize,
    result: tasks::ReorderTasksResult,
) -> Result<Response, ApiError> {
    if !result.missing_from_submission.is_empty() || !result.not_found.is_empty() {
        let added_ids: Vec<String> = result
            .missing_from_submission
            .iter()
            .map(ToString::to_string)
            .collect();
        let removed_ids: Vec<String> = result.not_found.iter().map(ToString::to_string).collect();

        return Err(reorder_conflict_error(
            "tasks.reorder_conflict",
            "tasks.reorder",
            submitted_count,
            added_ids,
            removed_ids,
            serde_json::json!({ "project_id": project_id }),
        ));
    }

    Ok(
        Json(serde_json::json!({ "ok": true, "reordered_count": result.reordered_count }))
            .into_response(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reorder_tasks_conflict_contains_added_removed_ids_and_project_id() {
        let project_id = Uuid::new_v7().into_inner();
        let err = map_reorder_tasks_result(
            project_id,
            4,
            tasks::ReorderTasksResult {
                reordered_count: 0,
                missing_from_submission: vec![Uuid::new_v7().into_inner()],
                not_found: vec![Uuid::new_v7().into_inner()],
            },
        )
        .expect_err("expected conflict");

        assert_eq!(err.status, StatusCode::CONFLICT);
        assert_eq!(err.err.code, "tasks.reorder_conflict");
        assert!(err.context["added_ids"].is_array());
        assert!(err.context["removed_ids"].is_array());
        assert_eq!(
            err.context["project_id"],
            serde_json::json!(project_id.to_string())
        );
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let project_id = project_id.into_inner();

    match tasks::list_labels_for_project(pool, project_id).await {
        Ok(labels) => {
            let response: Vec<LabelResponse> = labels.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list labels: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "labels.list_failed",
                "Failed to list labels",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "labels.list",
                "project_id": project_id
            })))
        }
    }
}

/// Create a label for a project.
pub async fn create_label(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateLabelRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let label_id = Uuid::new_v7().into_inner();
    let color = req.color.as_deref().unwrap_or("#6366f1");
    let project_id = project_id.into_inner();

    match tasks::create_label(pool, label_id, project_id, &req.name, color).await {
        Ok(label) => {
            let response: LabelResponse = label.into();
            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create label: {}", e);

            // Check for unique constraint violation
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return Err(ApiError::conflict(
                        "label.name_exists",
                        "A label with this name already exists in this project",
                    )
                    .with_context(serde_json::json!({
                        "operation": "labels.create",
                        "project_id": project_id,
                        "name": &req.name
                    })));
                }
            }

            Err(crate::db_errors::internal_with_diagnostics(
                "labels.create_failed",
                "Failed to create label",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "labels.create",
                "project_id": project_id,
                "name": &req.name
            })))
        }
    }
}

/// Get labels for a task.
pub async fn get_task_labels(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let task_id = task_id.into_inner();

    match tasks::get_labels_for_task(pool, task_id).await {
        Ok(labels) => {
            let response: Vec<LabelResponse> = labels.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to get task labels: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "labels.get_for_task_failed",
                "Failed to get task labels",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "labels.get_for_task",
                "task_id": task_id
            })))
        }
    }
}

/// Set labels for a task (replaces all).
pub async fn set_task_labels(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<SetLabelsRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let task_id = task_id.into_inner();
    let ids: Vec<_> = req.label_ids.iter().map(|id| id.into_inner()).collect();

    match tasks::set_task_labels(pool, task_id, &ids).await {
        Ok(()) => {
            // Return the updated labels
            match tasks::get_labels_for_task(pool, task_id).await {
                Ok(labels) => {
                    let response: Vec<LabelResponse> = labels.into_iter().map(Into::into).collect();
                    Ok(Json(serde_json::json!({ "data": response })).into_response())
                }
                Err(e) => {
                    tracing::error!("Failed to get updated labels: {}", e);
                    Ok(Json(serde_json::json!({ "ok": true })).into_response())
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to set task labels: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "labels.set_for_task_failed",
                "Failed to set task labels",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "labels.set_for_task",
                "task_id": task_id,
                "count": ids.len()
            })))
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
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BatchDeleteTasksRequest>,
) -> Result<Response, ApiError> {
    let project_id = project_id.into_inner();
    if req.ids.is_empty() {
        return Err(ApiError::bad_request(
            "validation.empty_ids",
            "At least one ID is required",
        ));
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
                        "project_id": project_id.to_string()
                    })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(Json(serde_json::json!({ "ok": true, "deleted": count })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to batch delete tasks: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.batch_delete_failed",
                "Failed to batch delete tasks",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.batch_delete",
                "project_id": project_id,
                "batch_id": batch_id,
                "count": ids.len()
            })))
        }
    }
}

/// Batch update task status.
///
/// POST /v1/admin/projects/:project_id/tasks:batch-update
pub async fn batch_update_task_status(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BatchUpdateTaskStatusRequest>,
) -> Result<Response, ApiError> {
    let project_id = project_id.into_inner();
    if req.ids.is_empty() {
        return Err(ApiError::bad_request(
            "validation.empty_ids",
            "At least one ID is required",
        ));
    }

    // Validate status value
    let valid_statuses = ["pending", "in_progress", "completed", "cancelled"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(ApiError::bad_request(
            "validation.invalid_status",
            "Invalid status value",
        ));
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
                        "project_id": project_id.to_string()
                    })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(Json(serde_json::json!({ "ok": true, "updated": count })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to batch update task status: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.batch_update_status_failed",
                "Failed to batch update task status",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.batch_update_status",
                "project_id": project_id,
                "count": ids.len(),
                "status": &req.status
            })))
        }
    }
}
