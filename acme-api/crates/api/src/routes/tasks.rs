//! Task and project routes.
//!
//! Example domain routes demonstrating common patterns.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use underlay_http::context::RequestContext;
use underlay_http::ApiError;
use underlay_nightfire::NightfireValue;

use acme_core::Uuid;
use acme_db::{activity, tasks};

use crate::routes::project_description::{
    prepare_project_description, sync_project_description_media_usage,
};
use crate::state::{AppState, AuthenticatedUser};
use validator::Validate;

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectTaskSummaryResponse {
    pub total: i64,
    pub completed: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub description: Option<Value>,
    pub status: String,
    pub task_summary: ProjectTaskSummaryResponse,
    pub created_at: String,
    pub updated_at: String,
}

impl ProjectResponse {
    fn from_row(row: tasks::ProjectRow, task_summary: tasks::ProjectTaskSummary) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            description: row.description,
            status: row.status,
            task_summary: ProjectTaskSummaryResponse {
                total: task_summary.total,
                completed: task_summary.completed,
            },
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateProjectRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: String,
    pub description: Option<NightfireValue>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateProjectRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,
    pub description: Option<Option<NightfireValue>>,
    pub status: Option<String>,
}

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
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateTaskRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: String,
    #[validate(length(max = 5000, message = "Description must not exceed 5000 characters"))]
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateTaskRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: Option<String>,
    #[validate(length(max = 5000, message = "Description must not exceed 5000 characters"))]
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<Option<NaiveDate>>,
}

const VALID_PROJECT_STATUSES: &[&str] = &["active", "archived", "completed"];
const VALID_TASK_STATUSES: &[&str] = &["todo", "in_progress", "done"];
const VALID_TASK_PRIORITIES: &[&str] = &["low", "medium", "high"];

fn validate_project_status_input(status: Option<&str>) -> Result<(), ApiError> {
    if let Some(status) = status {
        if !VALID_PROJECT_STATUSES.contains(&status) {
            return Err(ApiError::bad_request(
                "validation.invalid_status",
                "Project status must be one of: active, archived, completed",
            ));
        }
    }
    Ok(())
}

fn validate_task_input(status: Option<&str>, priority: Option<&str>) -> Result<(), ApiError> {
    if let Some(status) = status {
        if !VALID_TASK_STATUSES.contains(&status) {
            return Err(ApiError::bad_request(
                "validation.invalid_status",
                "Task status must be one of: todo, in_progress, done",
            ));
        }
    }

    if let Some(priority) = priority {
        if !VALID_TASK_PRIORITIES.contains(&priority) {
            return Err(ApiError::bad_request(
                "validation.invalid_priority",
                "Task priority must be one of: low, medium, high",
            ));
        }
    }

    Ok(())
}

fn validation_failed(err: validator::ValidationErrors) -> ApiError {
    let mut field_errors = std::collections::HashMap::new();
    for (field, errors) in err.field_errors() {
        if let Some(first) = errors.first() {
            let message = first
                .message
                .clone()
                .unwrap_or_else(|| "Invalid value".into())
                .to_string();
            field_errors.insert(field.to_string(), message);
        }
    }

    ApiError::bad_request(
        "validation.failed",
        "There is a problem with one or more fields.",
    )
    .with_field_errors(field_errors)
}

// ============================================================================
// Project Handlers
// ============================================================================

async fn ensure_project_owned(
    pool: &acme_db::DbPool,
    user_id: uuid::Uuid,
    project_id: uuid::Uuid,
    operation: &'static str,
) -> Result<(), ApiError> {
    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => Ok(()),
        Ok(Some(_)) => Err(
            ApiError::new(StatusCode::FORBIDDEN, "auth.forbidden", "Forbidden").with_context(
                serde_json::json!({
                    "operation": operation,
                    "project_id": project_id
                }),
            ),
        ),
        Ok(None) => Err(
            ApiError::not_found("projects.not_found", "Project not found").with_context(
                serde_json::json!({
                    "operation": operation,
                    "project_id": project_id
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.get_failed",
                "Failed to get project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": operation,
                "project_id": project_id
            })))
        }
    }
}

/// List all projects for the authenticated user.
pub async fn list_projects(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();

    match tasks::list_projects_for_user(pool, user_id, false).await {
        Ok(projects) => {
            let response: Vec<ProjectResponse> = projects
                .into_iter()
                .map(|project| {
                    ProjectResponse::from_row(
                        project,
                        tasks::ProjectTaskSummary {
                            total: 0,
                            completed: 0,
                        },
                    )
                })
                .collect();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.list_failed",
                "Failed to list projects",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.list",
                "user_id": user_id
            })))
        }
    }
}

/// Create a new project.
pub async fn create_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(validation_failed(e));
    }

    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = Uuid::new_v7().into_inner();
    let prepared_description = prepare_project_description(req.description)?;
    let description = prepared_description.as_ref().map(|value| value.json());

    match tasks::create_project(
        pool,
        project_id,
        user_id,
        &req.name,
        description,
        None,
    )
    .await
    {
        Ok(project) => {
            let task_summary = tasks::get_project_task_summary(pool, project_id)
                .await
                .map_err(|e| {
                    crate::db_errors::internal_with_diagnostics(
                        "projects.create_failed",
                        "Failed to create project",
                        &e,
                    )
                    .with_context(serde_json::json!({
                        "operation": "projects.create_task_summary",
                        "project_id": project_id
                    }))
                })?;
            sync_project_description_media_usage(
                pool,
                project_id,
                prepared_description.as_ref().map(|value| value.value()),
            )
            .await?;

            let ip_address = ctx.ip_address().map(|ip| ip.to_string());
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user_id),
                    action: "create",
                    resource_type: "project",
                    resource_id: project_id,
                    details: Some(serde_json::json!({
                        "name": project.name,
                        "status": project.status,
                    })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: ip_address.as_deref(),
                },
            )
            .await;

            let response = ProjectResponse::from_row(project, task_summary);
            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create project: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.create_failed",
                "Failed to create project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.create",
                "project_id": project_id,
                "user_id": user_id
            })))
        }
    }
}

/// Get a single project.
pub async fn get_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {
            let task_summary = tasks::get_project_task_summary(pool, project_id)
                .await
                .map_err(|e| {
                    crate::db_errors::internal_with_diagnostics(
                        "projects.get_failed",
                        "Failed to get project",
                        &e,
                    )
                    .with_context(serde_json::json!({
                        "operation": "projects.get_task_summary",
                        "project_id": project_id
                    }))
                })?;
            let response = ProjectResponse::from_row(project, task_summary);
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(Some(_)) => Err(
            ApiError::new(StatusCode::FORBIDDEN, "auth.forbidden", "Forbidden").with_context(
                serde_json::json!({
                    "operation": "projects.get",
                    "project_id": project_id,
                    "user_id": user_id
                }),
            ),
        ),
        Ok(None) => Err(
            ApiError::not_found("projects.not_found", "Project not found").with_context(
                serde_json::json!({
                    "operation": "projects.get",
                    "project_id": project_id
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.get_failed",
                "Failed to get project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.get",
                "project_id": project_id
            })))
        }
    }
}

/// Update a project.
pub async fn update_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(validation_failed(e));
    }
    validate_project_status_input(req.status.as_deref())?;

    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();
    let prepared_description = match req.description {
        Some(description) => Some(prepare_project_description(description)?),
        None => None,
    };
    let description = prepared_description
        .as_ref()
        .map(|entry| entry.as_ref().map(|value| value.json()));

    ensure_project_owned(pool, user_id, project_id, "projects.update").await?;

    match tasks::update_project(
        pool,
        project_id,
        req.name.as_deref(),
        description,
        req.status.as_deref(),
        None, // category_id - not editable from user routes
    )
    .await
    {
        Ok(Some(project)) => {
            let task_summary = tasks::get_project_task_summary(pool, project_id)
                .await
                .map_err(|e| {
                    crate::db_errors::internal_with_diagnostics(
                        "projects.update_failed",
                        "Failed to update project",
                        &e,
                    )
                    .with_context(serde_json::json!({
                        "operation": "projects.update_task_summary",
                        "project_id": project_id
                    }))
                })?;
            if let Some(prepared_description) = prepared_description.as_ref() {
                sync_project_description_media_usage(
                    pool,
                    project_id,
                    prepared_description.as_ref().map(|value| value.value()),
                )
                .await?;
            }

            let ip_address = ctx.ip_address().map(|ip| ip.to_string());
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user_id),
                    action: "update",
                    resource_type: "project",
                    resource_id: project_id,
                    details: Some(serde_json::json!({
                        "name": project.name,
                        "status": project.status,
                    })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: ip_address.as_deref(),
                },
            )
            .await;

            let response = ProjectResponse::from_row(project, task_summary);
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("projects.not_found", "Project not found").with_context(
                serde_json::json!({
                    "operation": "projects.update",
                    "project_id": project_id
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to update project: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.update_failed",
                "Failed to update project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.update",
                "project_id": project_id
            })))
        }
    }
}

/// Delete a project.
pub async fn delete_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "projects.delete").await?;

    match tasks::delete_project(pool, project_id).await {
        Ok(true) => {
            sync_project_description_media_usage(pool, project_id, None).await?;

            let ip_address = ctx.ip_address().map(|ip| ip.to_string());
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user_id),
                    action: "delete",
                    resource_type: "project",
                    resource_id: project_id,
                    details: None,
                    correlation_id: Some(ctx.request_id()),
                    ip_address: ip_address.as_deref(),
                },
            )
            .await;

            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Ok(false) => Err(
            ApiError::not_found("projects.not_found", "Project not found").with_context(
                serde_json::json!({
                    "operation": "projects.delete",
                    "project_id": project_id
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.delete_failed",
                "Failed to delete project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.delete",
                "project_id": project_id
            })))
        }
    }
}

// ============================================================================
// Task Handlers
// ============================================================================

/// List tasks for a project.
pub async fn list_tasks(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "tasks.list").await?;

    match tasks::list_tasks_for_project(pool, project_id, false).await {
        Ok(task_list) => {
            let response: Vec<TaskResponse> = task_list.into_iter().map(Into::into).collect();
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

/// Create a new task.
pub async fn create_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(validation_failed(e));
    }
    validate_task_input(None, req.priority.as_deref())?;

    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "tasks.create").await?;

    let task_id = Uuid::new_v7().into_inner();
    let priority = req.priority.as_deref().unwrap_or("medium");

    match tasks::create_task(
        pool,
        task_id,
        project_id,
        &req.title,
        req.description.as_deref(),
        None,
        priority,
        req.due_date,
    )
    .await
    {
        Ok(task) => {
            let ip_address = ctx.ip_address().map(|ip| ip.to_string());
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user_id),
                    action: "create",
                    resource_type: "task",
                    resource_id: task_id,
                    details: Some(serde_json::json!({
                        "project_id": project_id,
                        "title": task.title,
                        "status": task.status,
                        "priority": task.priority,
                    })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: ip_address.as_deref(),
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
                "project_id": project_id,
                "task_id": task_id
            })))
        }
    }
}

/// Update a task.
pub async fn update_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(validation_failed(e));
    }
    validate_task_input(req.status.as_deref(), req.priority.as_deref())?;

    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();
    let task_id = task_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "tasks.update").await?;

    match tasks::update_task(
        pool,
        task_id,
        project_id,
        req.title.as_deref(),
        req.description.as_ref().map(|d| d.as_deref()),
        None,
        req.status.as_deref(),
        req.priority.as_deref(),
        req.due_date,
    )
    .await
    {
        Ok(Some(task)) => {
            let ip_address = ctx.ip_address().map(|ip| ip.to_string());
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user_id),
                    action: "update",
                    resource_type: "task",
                    resource_id: task_id,
                    details: Some(serde_json::json!({
                        "project_id": project_id,
                        "title": task.title,
                        "status": task.status,
                        "priority": task.priority,
                    })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: ip_address.as_deref(),
                },
            )
            .await;

            let response: TaskResponse = task.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("tasks.not_found", "Task not found").with_context(
                serde_json::json!({
                    "operation": "tasks.update",
                    "task_id": task_id
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
                "task_id": task_id
            })))
        }
    }
}

/// Delete a task.
pub async fn delete_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();
    let task_id = task_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "tasks.delete").await?;

    match tasks::delete_task(pool, task_id, project_id).await {
        Ok(true) => {
            let ip_address = ctx.ip_address().map(|ip| ip.to_string());
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user_id),
                    action: "delete",
                    resource_type: "task",
                    resource_id: task_id,
                    details: Some(serde_json::json!({
                        "project_id": project_id,
                    })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: ip_address.as_deref(),
                },
            )
            .await;

            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Ok(false) => Err(
            ApiError::not_found("tasks.not_found", "Task not found").with_context(
                serde_json::json!({
                    "operation": "tasks.delete",
                    "task_id": task_id
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to delete task: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "tasks.delete_failed",
                "Failed to delete task",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "tasks.delete",
                "task_id": task_id
            })))
        }
    }
}
