//! Task and project routes.
//!
//! Example domain routes demonstrating common patterns.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use acme_core::Uuid;
use acme_db::tasks;

use crate::state::{AppState, AuthenticatedUser};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
}

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
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// List all projects for the authenticated user.
pub async fn list_projects(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();

    match tasks::list_projects_for_user(pool, user_id, false).await {
        Ok(projects) => {
            let response: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();
            Json(serde_json::json!({ "items": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a new project.
pub async fn create_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> impl IntoResponse {
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
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get a single project.
pub async fn get_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {
            let response: ProjectResponse = project.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(Some(_)) => StatusCode::FORBIDDEN.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Update a project.
pub async fn update_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    // First check ownership
    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {}
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

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
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to update project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Delete a project.
pub async fn delete_project(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    // First check ownership
    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {}
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    match tasks::delete_project(pool, project_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    // Verify project ownership
    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {}
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    match tasks::list_tasks_for_project(pool, project_id, false).await {
        Ok(task_list) => {
            let response: Vec<TaskResponse> = task_list.into_iter().map(Into::into).collect();
            Json(serde_json::json!({ "items": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a new task.
pub async fn create_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();

    // Verify project ownership
    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {}
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

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

/// Update a task.
pub async fn update_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();
    let task_id = task_id.into_inner();

    // Verify project ownership
    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {}
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

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
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to update task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Delete a task.
pub async fn delete_task(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let user_id = user.user_id.0.into_inner();
    let project_id = project_id.into_inner();
    let task_id = task_id.into_inner();

    // Verify project ownership
    match tasks::get_project(pool, project_id).await {
        Ok(Some(project)) if project.owner_id == user_id => {}
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    match tasks::delete_task(pool, task_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
