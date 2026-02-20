//! Project admin routes.
//!
//! Demonstrates:
//! - Filtering by category, status
//! - Sorting by name, weight, dates
//! - List with task counts
//! - Soft delete

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use underlay_http::{context::RequestContext, query::QueryParams, ApiError};

use acme_core::Uuid;
use acme_db::{activity, tasks};
use serde_json::json;

use crate::routes::admin::freshness::{
    build_etag_cache_headers, detail_etag, if_match_mismatch, maybe_not_modified,
    precondition_failed_error,
};
use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub owner_id: Option<Uuid>, // Admin can create for other users
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub category_id: Option<Option<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReorderRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchDeleteRequest {
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match tasks::list_projects_admin(pool, &query).await {
        Ok(projects) => {
            let response: Vec<ProjectWithCountsResponse> =
                projects.into_iter().map(Into::into).collect();
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
                "operation": "projects.list"
            })))
        }
    }
}

/// Get a single project (admin).
pub async fn get_project(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let project_id = project_id.into_inner();

    match tasks::get_project_admin(pool, project_id).await {
        Ok(Some(project)) => {
            let etag = detail_etag(
                "project",
                &project.id.to_string(),
                &project.updated_at.to_rfc3339(),
            );
            if let Some(not_modified) = maybe_not_modified(&headers, &etag) {
                return Ok(not_modified);
            }
            let response: ProjectResponse = project.into();
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
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

/// Create a project (admin).
///
/// Admin can create projects for any user by specifying `ownerId`.
/// If not specified, the project is created for the admin user.
pub async fn create_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Response, ApiError> {
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: ProjectResponse = project.into();
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
                "owner_id": owner_id,
                "category_id": req.category_id.map(|id| id.into_inner()),
                "name": &req.name
            })))
        }
    }
}

/// Update a project (admin).
pub async fn update_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let pid = project_id.into_inner();

    let current = match tasks::get_project_admin(pool, pid).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Err(
                ApiError::not_found("projects.not_found", "Project not found").with_context(
                    serde_json::json!({
                        "operation": "projects.update",
                        "project_id": pid
                    }),
                ),
            );
        }
        Err(e) => {
            tracing::error!("Failed to load current project before update: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "projects.update_failed",
                "Failed to update project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.update",
                "project_id": pid
            })));
        }
    };

    let current_etag = detail_etag(
        "project",
        &current.id.to_string(),
        &current.updated_at.to_rfc3339(),
    );
    if if_match_mismatch(&headers, &current_etag) {
        return Err(precondition_failed_error().with_context(serde_json::json!({
            "operation": "projects.update",
            "project_id": pid
        })));
    }

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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: ProjectResponse = project.into();
            let etag = detail_etag("project", &response.id, &response.updated_at);
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("projects.not_found", "Project not found").with_context(
                serde_json::json!({
                    "operation": "projects.update",
                    "project_id": pid
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
                "project_id": pid
            })))
        }
    }
}

/// Soft delete a project (admin).
pub async fn soft_delete_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ApiError> {
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Ok(false) => Err(
            ApiError::not_found("projects.not_found", "Project not found").with_context(
                serde_json::json!({
                    "operation": "projects.soft_delete",
                    "project_id": pid
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to soft delete project: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.soft_delete_failed",
                "Failed to delete project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.soft_delete",
                "project_id": pid,
                "batch_id": batch_id
            })))
        }
    }
}

/// Restore a soft-deleted project (admin).
pub async fn restore_project(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ApiError> {
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: ProjectResponse = project.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("projects.not_found", "Project not found").with_context(
                serde_json::json!({
                    "operation": "projects.restore",
                    "project_id": pid
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to restore project: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.restore_failed",
                "Failed to restore project",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.restore",
                "project_id": pid
            })))
        }
    }
}

/// Reorder projects.
pub async fn reorder_projects(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<ReorderRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match tasks::reorder_projects(pool, &ids).await {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true })).into_response()),
        Err(e) => {
            tracing::error!("Failed to reorder projects: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.reorder_failed",
                "Failed to reorder projects",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.reorder",
                "count": ids.len()
            })))
        }
    }
}

/// Batch delete projects.
///
/// POST /v1/admin/projects:batch-delete
pub async fn batch_delete_projects(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Response, ApiError> {
    if req.ids.is_empty() {
        return Err(ApiError::bad_request(
            "validation.empty_ids",
            "At least one ID is required",
        ));
    }

    let pool = state.local_auth.pool();
    let batch_id = Uuid::new_v7().into_inner();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match tasks::batch_soft_delete_projects(pool, &ids, batch_id).await {
        Ok(count) => {
            // Log activity for batch operation
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "batch_delete",
                    resource_type: "project",
                    resource_id: batch_id,
                    details: Some(json!({ "count": count, "ids": req.ids })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(Json(json!({ "ok": true, "deleted": count })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to batch delete projects: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "projects.batch_delete_failed",
                "Failed to batch delete projects",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "projects.batch_delete",
                "count": ids.len(),
                "batch_id": batch_id
            })))
        }
    }
}
