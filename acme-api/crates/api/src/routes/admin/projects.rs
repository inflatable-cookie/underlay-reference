//! Project admin routes.
//!
//! Demonstrates:
//! - Filtering by category, status
//! - Sorting by name, weight, dates
//! - List with task counts
//! - Soft delete

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
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
pub struct ProjectResponse {
    pub id: String,
    pub owner_id: String,
    pub category_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub weight: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<tasks::ProjectRow> for ProjectResponse {
    fn from(row: tasks::ProjectRow) -> Self {
        Self {
            id: row.id.to_string(),
            owner_id: row.owner_id.to_string(),
            category_id: row.category_id.map(|id| id.to_string()),
            name: row.name,
            description: row.description,
            status: row.status,
            weight: row.weight,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWithCountsResponse {
    pub id: String,
    pub owner_id: String,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub weight: i32,
    pub created_at: String,
    pub updated_at: String,
    pub task_count: i64,
    pub completed_task_count: i64,
}

impl From<tasks::ProjectWithCountsRow> for ProjectWithCountsResponse {
    fn from(row: tasks::ProjectWithCountsRow) -> Self {
        Self {
            id: row.id.to_string(),
            owner_id: row.owner_id.to_string(),
            category_id: row.category_id.map(|id| id.to_string()),
            category_name: row.category_name,
            name: row.name,
            description: row.description,
            status: row.status,
            weight: row.weight,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            task_count: row.task_count,
            completed_task_count: row.completed_task_count,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub owner_id: Option<Uuid>, // Admin can create for other users
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub category_id: Option<Option<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderRequest {
    pub ids: Vec<Uuid>,
}

// ============================================================================
// Handlers
// ============================================================================

/// List projects with counts (admin).
///
/// Supports filtering and sorting via query parameters:
/// - `sort=name:asc,weight:desc,categoryName:asc`
/// - `filter[categoryId]=<uuid>`
/// - `filter[status]=active`
/// - `filter[ownerId]=<uuid>`
pub async fn list_projects(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match tasks::list_projects_admin(pool, &query).await {
        Ok(projects) => {
            let response: Vec<ProjectWithCountsResponse> =
                projects.into_iter().map(Into::into).collect();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get a single project (admin).
pub async fn get_project(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match tasks::get_project_admin(pool, project_id.into_inner()).await {
        Ok(Some(project)) => {
            let response: ProjectResponse = project.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a project (admin).
///
/// Admin can create projects for any user by specifying `ownerId`.
/// If not specified, the project is created for the admin user.
pub async fn create_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let project_id = Uuid::new_v7().into_inner();
    let owner_id = req
        .owner_id
        .map(|id| id.into_inner())
        .unwrap_or_else(|| user.user_id.0.into_inner());

    match tasks::create_project(
        pool,
        project_id,
        owner_id,
        &req.name,
        req.description.as_deref(),
        req.category_id.map(|id| id.into_inner()),
    )
    .await
    {
        Ok(project) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "create",
                    resource_type: "project",
                    resource_id: project_id,
                    details: Some(serde_json::json!({ "name": req.name })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

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

/// Update a project (admin).
pub async fn update_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let pid = project_id.into_inner();

    match tasks::update_project(
        pool,
        pid,
        req.name.as_deref(),
        req.description.as_ref().map(|d| d.as_deref()),
        req.status.as_deref(),
        req.category_id
            .as_ref()
            .map(|opt| opt.as_ref().map(|id| id.into_inner())),
    )
    .await
    {
        Ok(Some(project)) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "update",
                    resource_type: "project",
                    resource_id: pid,
                    details: Some(serde_json::json!({ "name": project.name })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

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

/// Soft delete a project (admin).
pub async fn soft_delete_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let batch_id = Uuid::new_v7().into_inner();
    let pid = project_id.into_inner();

    match tasks::soft_delete_project(pool, pid, batch_id).await {
        Ok(true) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "delete",
                    resource_type: "project",
                    resource_id: pid,
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
            tracing::error!("Failed to soft delete project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Restore a soft-deleted project (admin).
pub async fn restore_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let pid = project_id.into_inner();

    match tasks::restore_project(pool, pid).await {
        Ok(Some(project)) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "restore",
                    resource_type: "project",
                    resource_id: pid,
                    details: Some(serde_json::json!({ "name": project.name })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: ProjectResponse = project.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to restore project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Reorder projects.
pub async fn reorder_projects(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<ReorderRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match tasks::reorder_projects(pool, &ids).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to reorder projects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
