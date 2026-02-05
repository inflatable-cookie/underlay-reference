//! Category admin routes.
//!
//! Demonstrates:
//! - Filtering and sorting via QueryParams
//! - Admin list with counts
//! - Soft delete with batch IDs
//! - Reordering

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use underlay_core::AppError;
use underlay_http::{error_response, query::QueryParams};

use acme_core::Uuid;
use acme_db::{activity, categories};

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub weight: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<categories::CategoryRow> for CategoryResponse {
    fn from(row: categories::CategoryRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            slug: row.slug,
            description: row.description,
            color: row.color,
            weight: row.weight,
            is_active: row.is_active,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryWithCountsResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub weight: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub project_count: i64,
}

impl From<categories::CategoryWithCountsRow> for CategoryWithCountsResponse {
    fn from(row: categories::CategoryWithCountsRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            slug: row.slug,
            description: row.description,
            color: row.color,
            weight: row.weight,
            is_active: row.is_active,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            project_count: row.project_count,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderRequest {
    pub ids: Vec<Uuid>,
}

// ============================================================================
// Handlers
// ============================================================================

/// List categories with counts (admin).
///
/// Supports filtering and sorting via query parameters:
/// - `sort=name:asc,weight:desc`
/// - `filter[isActive]=true`
pub async fn list_categories(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match categories::list_categories_with_counts(pool, &query).await {
        Ok(cats) => {
            let response: Vec<CategoryWithCountsResponse> =
                cats.into_iter().map(Into::into).collect();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list categories: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::new("categories.list_failed", format!("Failed to list categories: {}", e)),
            )
        }
    }
}

/// Get a single category.
pub async fn get_category(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match categories::get_category(pool, category_id.into_inner()).await {
        Ok(Some(category)) => {
            let response: CategoryResponse = category.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a new category.
pub async fn create_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateCategoryRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let category_id = Uuid::new_v7().into_inner();

    match categories::create_category(
        pool,
        category_id,
        &req.name,
        &req.slug,
        req.description.as_deref(),
        req.color.as_deref(),
    )
    .await
    {
        Ok(category) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "create",
                    resource_type: "category",
                    resource_id: category_id,
                    details: Some(serde_json::json!({ "name": req.name, "slug": req.slug })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: CategoryResponse = category.into();
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create category: {}", e);

            // Check for unique constraint violation
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({
                            "ok": false,
                            "error": {
                                "code": "category.slug_exists",
                                "message": "A category with this slug already exists"
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

/// Update a category.
pub async fn update_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
    Json(req): Json<UpdateCategoryRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let cid = category_id.into_inner();

    match categories::update_category(
        pool,
        cid,
        req.name.as_deref(),
        req.slug.as_deref(),
        req.description.as_ref().map(|d| d.as_deref()),
        req.color.as_ref().map(|c| c.as_deref()),
        req.is_active,
    )
    .await
    {
        Ok(Some(category)) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "update",
                    resource_type: "category",
                    resource_id: cid,
                    details: Some(serde_json::json!({ "name": category.name })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: CategoryResponse = category.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to update category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Soft delete a category.
pub async fn soft_delete_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let batch_id = Uuid::new_v7().into_inner();
    let cid = category_id.into_inner();

    match categories::soft_delete_category(pool, cid, batch_id).await {
        Ok(true) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "delete",
                    resource_type: "category",
                    resource_id: cid,
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
            tracing::error!("Failed to soft delete category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Restore a soft-deleted category.
pub async fn restore_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let cid = category_id.into_inner();

    match categories::restore_category(pool, cid).await {
        Ok(Some(category)) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "restore",
                    resource_type: "category",
                    resource_id: cid,
                    details: Some(serde_json::json!({ "name": category.name })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: CategoryResponse = category.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to restore category: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Reorder categories.
///
/// Accepts an array of category IDs in the desired order.
/// Sets weight values 0, 1, 2, ... in the order provided.
pub async fn reorder_categories(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<ReorderRequest>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match categories::reorder_categories(pool, &ids).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to reorder categories: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
