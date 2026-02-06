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
use underlay_http::ApiError;

use acme_core::Uuid;
use acme_db::tasks;

use crate::state::{AppState, AuthenticatedUser};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<tasks::ProjectRow> for ProjectResponse {
    fn from(row: tasks::ProjectRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            description: row.description,
            status: row.status,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<Option<NaiveDate>>,
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
            Err(
                ApiError::internal("projects.get_failed", "Failed to get project")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": operation,
                        "project_id": project_id
                    })),
            )
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
            let response: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({ "items": response })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            Err(
                ApiError::internal("projects.list_failed", "Failed to list projects")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "projects.list",
                        "user_id": user_id
                    })),
            )
        }
    }
}

/// Create a new project.
pub async fn create_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = Uuid::new_v7().into_inner();

    match tasks::create_project(
        pool,
        project_id,
        user_id,
        &req.name,
        req.description.as_deref(),
        None,
    )
    .await
    {
        Ok(project) => {
            let response: ProjectResponse = project.into();
            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create project: {}", e);
            Err(
                ApiError::internal("projects.create_failed", "Failed to create project")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "projects.create",
                        "project_id": project_id,
                        "user_id": user_id
                    })),
            )
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
            let response: ProjectResponse = project.into();
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
            Err(
                ApiError::internal("projects.get_failed", "Failed to get project")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "projects.get",
                        "project_id": project_id
                    })),
            )
        }
    }
}

/// Update a project.
pub async fn update_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "projects.update").await?;

    match tasks::update_project(
        pool,
        project_id,
        req.name.as_deref(),
        req.description.as_ref().map(|d| d.as_deref()),
        req.status.as_deref(),
        None, // category_id - not editable from user routes
    )
    .await
    {
        Ok(Some(project)) => {
            let response: ProjectResponse = project.into();
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
            Err(
                ApiError::internal("projects.update_failed", "Failed to update project")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "projects.update",
                        "project_id": project_id
                    })),
            )
        }
    }
}

/// Delete a project.
pub async fn delete_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "projects.delete").await?;

    match tasks::delete_project(pool, project_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT.into_response()),
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
            Err(
                ApiError::internal("projects.delete_failed", "Failed to delete project")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "projects.delete",
                        "project_id": project_id
                    })),
            )
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
            Ok(Json(serde_json::json!({ "items": response })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list tasks: {}", e);
            Err(
                ApiError::internal("tasks.list_failed", "Failed to list tasks")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "tasks.list",
                        "project_id": project_id
                    })),
            )
        }
    }
}

/// Create a new task.
pub async fn create_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Response, ApiError> {
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
        priority,
        req.due_date,
    )
    .await
    {
        Ok(task) => {
            let response: TaskResponse = task.into();
            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            Err(
                ApiError::internal("tasks.create_failed", "Failed to create task")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "tasks.create",
                        "project_id": project_id,
                        "task_id": task_id
                    })),
            )
        }
    }
}

/// Update a task.
pub async fn update_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();
    let task_id = task_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "tasks.update").await?;

    match tasks::update_task(
        pool,
        task_id,
        req.title.as_deref(),
        req.description.as_ref().map(|d| d.as_deref()),
        req.status.as_deref(),
        req.priority.as_deref(),
        req.due_date,
    )
    .await
    {
        Ok(Some(task)) => {
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
            Err(
                ApiError::internal("tasks.update_failed", "Failed to update task")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "tasks.update",
                        "task_id": task_id
                    })),
            )
        }
    }
}

/// Delete a task.
pub async fn delete_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();
    let task_id = task_id.into_inner();

    ensure_project_owned(pool, user_id, project_id, "tasks.delete").await?;

    match tasks::delete_task(pool, task_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT.into_response()),
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
            Err(
                ApiError::internal("tasks.delete_failed", "Failed to delete task")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "tasks.delete",
                        "task_id": task_id
                    })),
            )
        }
    }
}
