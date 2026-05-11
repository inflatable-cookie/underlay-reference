//! Category admin routes.
//!
//! Demonstrates:
//! - Filtering and sorting via QueryParams
//! - Admin list with counts
//! - Soft delete with batch IDs
//! - Reordering

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
use acme_db::{activity, categories};

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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub is_active: Option<bool>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListCategoriesQuery {
    #[serde(flatten)]
    pub query: QueryParams,
    pub page: Option<u32>,
    pub limit: Option<u32>,
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
    Query(params): Query<ListCategoriesQuery>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page.saturating_sub(1) * limit) as i64;

    match categories::list_categories_with_counts(pool, &params.query, limit as i64, offset).await {
        Ok(cats) => {
            let response: Vec<CategoryWithCountsResponse> =
                cats.data.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({
                "data": response,
                "total": cats.total,
                "has_more": cats.has_more
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list categories: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "categories.list_failed",
                "Failed to list categories",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.list"
            })))
        }
    }
}

/// Get a single category.
pub async fn get_category(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(category_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let category_id = category_id.into_inner();

    match categories::get_category(pool, category_id).await {
        Ok(Some(category)) => {
            let etag = detail_etag(
                "category",
                &category.id.to_string(),
                &category.updated_at.to_rfc3339(),
            );
            if let Some(not_modified) = maybe_not_modified(&headers, &etag) {
                return Ok(not_modified);
            }
            let response: CategoryResponse = category.into();
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("categories.not_found", "Category not found").with_context(
                serde_json::json!({
                    "operation": "categories.get",
                    "category_id": category_id
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to get category: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "categories.get_failed",
                "Failed to get category",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.get",
                "category_id": category_id
            })))
        }
    }
}

/// Create a new category.
pub async fn create_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Response, ApiError> {
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: CategoryResponse = category.into();
            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create category: {}", e);

            // Check for unique constraint violation
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return Err(ApiError::conflict(
                        "category.slug_exists",
                        "A category with this slug already exists",
                    )
                    .with_context(serde_json::json!({
                        "operation": "categories.create",
                        "slug": &req.slug
                    })));
                }
            }

            Err(crate::db_errors::internal_with_diagnostics(
                "categories.create_failed",
                "Failed to create category",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.create",
                "slug": &req.slug
            })))
        }
    }
}

/// Update a category.
pub async fn update_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    ctx: RequestContext,
    Path(category_id): Path<Uuid>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let cid = category_id.into_inner();

    let current = match categories::get_category(pool, cid).await {
        Ok(Some(category)) => category,
        Ok(None) => {
            return Err(
                ApiError::not_found("categories.not_found", "Category not found").with_context(
                    serde_json::json!({
                        "operation": "categories.update",
                        "category_id": cid
                    }),
                ),
            );
        }
        Err(e) => {
            tracing::error!("Failed to load current category before update: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "categories.update_failed",
                "Failed to update category",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.update",
                "category_id": cid
            })));
        }
    };

    let current_etag = detail_etag(
        "category",
        &current.id.to_string(),
        &current.updated_at.to_rfc3339(),
    );
    if if_match_mismatch(&headers, &current_etag) {
        return Err(precondition_failed_error().with_context(serde_json::json!({
            "operation": "categories.update",
            "category_id": cid
        })));
    }

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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: CategoryResponse = category.into();
            let etag = detail_etag("category", &response.id, &response.updated_at);
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("categories.not_found", "Category not found").with_context(
                serde_json::json!({
                    "operation": "categories.update",
                    "category_id": cid
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to update category: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "categories.update_failed",
                "Failed to update category",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.update",
                "category_id": cid
            })))
        }
    }
}

/// Soft delete a category.
pub async fn soft_delete_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(category_id): Path<Uuid>,
) -> Result<Response, ApiError> {
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Ok(false) => Err(
            ApiError::not_found("categories.not_found", "Category not found").with_context(
                serde_json::json!({
                    "operation": "categories.soft_delete",
                    "category_id": cid
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to soft delete category: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "categories.soft_delete_failed",
                "Failed to delete category",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.soft_delete",
                "category_id": cid,
                "batch_id": batch_id
            })))
        }
    }
}

/// Batch delete categories.
///
/// POST /v1/admin/categories:batch-delete
pub async fn batch_delete_categories(
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

    match categories::batch_soft_delete_categories(pool, &ids, batch_id).await {
        Ok(count) => {
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "batch_delete",
                    resource_type: "category",
                    resource_id: batch_id,
                    details: Some(serde_json::json!({ "count": count, "ids": req.ids })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(Json(serde_json::json!({ "ok": true, "deleted": count })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to batch delete categories: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "categories.batch_delete_failed",
                "Failed to batch delete categories",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.batch_delete",
                "count": ids.len(),
                "batch_id": batch_id
            })))
        }
    }
}

/// Restore a soft-deleted category.
pub async fn restore_category(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(category_id): Path<Uuid>,
) -> Result<Response, ApiError> {
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
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: CategoryResponse = category.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("categories.not_found", "Category not found").with_context(
                serde_json::json!({
                    "operation": "categories.restore",
                    "category_id": cid
                }),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to restore category: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "categories.restore_failed",
                "Failed to restore category",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.restore",
                "category_id": cid
            })))
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let ids: Vec<_> = req.ids.iter().map(|id| id.into_inner()).collect();

    match categories::reorder_categories(pool, &ids).await {
        Ok(result) => map_reorder_categories_result(ids.len(), result),
        Err(e) => {
            tracing::error!("Failed to reorder categories: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "categories.reorder_failed",
                "Failed to reorder categories",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "categories.reorder",
                "count": ids.len()
            })))
        }
    }
}

fn map_reorder_categories_result(
    submitted_count: usize,
    result: categories::ReorderCategoriesResult,
) -> Result<Response, ApiError> {
    if !result.missing_from_submission.is_empty() || !result.not_found.is_empty() {
        let added_ids: Vec<String> = result
            .missing_from_submission
            .iter()
            .map(ToString::to_string)
            .collect();
        let removed_ids: Vec<String> = result.not_found.iter().map(ToString::to_string).collect();

        return Err(reorder_conflict_error(
            "categories.reorder_conflict",
            "categories.reorder",
            submitted_count,
            added_ids,
            removed_ids,
            serde_json::json!({}),
        ));
    }

    Ok(
        Json(serde_json::json!({ "ok": true, "reordered_count": result.reordered_count }))
            .into_response(),
    )
}

#[cfg(test)]
#[path = "../../tests/routes/admin/categories_tests.rs"]
mod tests;
