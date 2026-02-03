//! User admin routes.
//!
//! Provides user management endpoints for administrators.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use acme_db::users;

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

/// User response for list view.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub display_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<users::UserRow> for UserResponse {
    fn from(row: users::UserRow) -> Self {
        Self {
            id: row.id.to_string(),
            email: row.email,
            role: row.role,
            status: row.status,
            display_name: row.display_name,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

/// User detail response with session info.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailResponse {
    pub id: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub display_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub active_session_count: i64,
    pub failed_login_count: i32,
    pub lockout_until: Option<String>,
}

impl From<users::UserWithSessionCountRow> for UserDetailResponse {
    fn from(row: users::UserWithSessionCountRow) -> Self {
        Self {
            id: row.id.to_string(),
            email: row.email,
            role: row.role,
            status: row.status,
            display_name: row.display_name,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            active_session_count: row.active_session_count,
            failed_login_count: row.failed_login_count,
            lockout_until: row.lockout_until.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// Query parameters for user listing.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersQuery {
    /// Filter by role
    pub role: Option<String>,
    /// Filter by status
    pub status: Option<String>,
    /// Search by email
    pub search: Option<String>,
    /// Limit (default 50)
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Request to update user role.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRoleRequest {
    #[validate(length(min = 1))]
    pub role: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// List users (admin).
///
/// GET /v1/admin/users
pub async fn list_users(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match users::list_users_admin(
        pool,
        limit,
        offset,
        query.role.as_deref(),
        query.status.as_deref(),
        query.search.as_deref(),
    )
    .await
    {
        Ok(response) => {
            let items: Vec<UserResponse> = response.data.into_iter().map(Into::into).collect();
            Json(serde_json::json!({
                "data": items,
                "hasMore": response.has_more,
                "total": response.total
            }))
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get a single user (admin).
///
/// GET /v1/admin/users/:user_id
pub async fn get_user(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match users::get_user_with_session_count(pool, user_id).await {
        Ok(Some(user)) => {
            let response: UserDetailResponse = user.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Update a user's role.
///
/// PUT /v1/admin/users/:user_id/role
pub async fn update_user_role(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "validation.failed",
                    "message": e.to_string()
                }
            })),
        )
            .into_response();
    }

    // Validate role value
    let valid_roles = ["student", "tester", "tutor", "editor", "admin", "support", "superadmin"];
    if !valid_roles.contains(&req.role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "validation.invalid_role",
                    "message": "Invalid role value"
                }
            })),
        )
            .into_response();
    }

    let pool = state.local_auth.pool();

    match users::update_user_role(pool, user_id, &req.role).await {
        Ok(Some(user)) => {
            let response: UserResponse = user.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to update user role: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Suspend a user.
///
/// POST /v1/admin/users/:user_id/suspend
pub async fn suspend_user(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    // Update status to suspended
    match users::update_user_status(pool, user_id, "suspended").await {
        Ok(Some(user)) => {
            // Revoke all active sessions
            if let Err(e) =
                users::revoke_all_user_sessions(pool, user_id, "User suspended by admin").await
            {
                tracing::warn!("Failed to revoke sessions for suspended user: {}", e);
            }

            let response: UserResponse = user.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to suspend user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Unsuspend (reactivate) a user.
///
/// POST /v1/admin/users/:user_id/unsuspend
pub async fn unsuspend_user(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match users::update_user_status(pool, user_id, "active").await {
        Ok(Some(user)) => {
            let response: UserResponse = user.into();
            Json(serde_json::json!({ "data": response })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to unsuspend user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
