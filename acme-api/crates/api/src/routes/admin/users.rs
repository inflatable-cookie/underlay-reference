//! User admin routes.
//!
//! Provides user management endpoints for administrators.

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use underlay_http::ApiError;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use acme_core::Uuid as UnderlayUuid;
use acme_db::{activity, users};

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

/// User response for list view.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ListUsersQuery {
    /// Filter by role
    pub role: Option<String>,
    /// Filter by status
    pub status: Option<String>,
    /// Search by email
    pub search: Option<String>,
    /// Search by display name
    pub display_name: Option<String>,
    /// Limit (default 50)
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Request to update user role.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateUserRoleRequest {
    #[validate(length(min = 1))]
    pub role: String,
}

/// Request to create a user (admin).
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 320), email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub role: String,
    #[validate(length(min = 1))]
    pub status: String,
    #[validate(length(max = 100))]
    pub display_name: Option<String>,
    /// If true, send a password reset email so the user can set an initial password.
    pub send_password_reset: Option<bool>,
}

/// Request to update a user (admin).
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 320), email)]
    pub email: Option<String>,
    #[validate(length(max = 100))]
    pub display_name: Option<String>,
    #[validate(length(min = 1))]
    pub role: Option<String>,
    #[validate(length(min = 1))]
    pub status: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

fn is_valid_role(role: &str) -> bool {
    matches!(
        role,
        "user" | "tester" | "editor" | "admin" | "support" | "superadmin"
    )
}

fn is_valid_status(status: &str) -> bool {
    matches!(status, "active" | "suspended" | "deleted")
}

fn field_name_for_errors(field: &str) -> &str {
    match field {
        "display_name" => "displayName",
        "send_password_reset" => "sendPasswordReset",
        other => other,
    }
}

/// List users (admin).
///
/// GET /v1/admin/users
pub async fn list_users(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Response, ApiError> {
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
        query.display_name.as_deref(),
    )
    .await
    {
        Ok(response) => {
            let items: Vec<UserResponse> = response.data.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({
                "data": items,
                "has_more": response.has_more,
                "total": response.total
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            Err(
                ApiError::internal("admin.users.list_failed", "Failed to list users")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "admin.users.list",
                        "limit": limit,
                        "offset": offset
                    })),
            )
        }
    }
}

/// Create a user (admin).
///
/// POST /v1/admin/users
pub async fn create_user(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Response, ApiError> {
    if let Err(validation_err) = payload.validate() {
        let mut field_errors = std::collections::HashMap::new();
        for (field, errors) in validation_err.field_errors() {
            if let Some(err) = errors.first() {
                let msg = err
                    .message
                    .clone()
                    .unwrap_or_else(|| "Invalid value".into());
                field_errors.insert(
                    field_name_for_errors(field.as_ref()).to_string(),
                    msg.to_string(),
                );
            }
        }
        return Err(
            ApiError::bad_request(
                "admin.users.validation_failed",
                "There is a problem with one or more fields.",
            )
            .with_field_errors(field_errors),
        );
    }

    let email = payload.email.trim().to_lowercase();
    let role = payload.role.trim();
    let status = payload.status.trim();

    let mut field_errors = std::collections::HashMap::new();
    if !is_valid_role(role) {
        field_errors.insert("role".to_string(), "Invalid role".to_string());
    }
    if !is_valid_status(status) {
        field_errors.insert("status".to_string(), "Invalid status".to_string());
    }
    if !field_errors.is_empty() {
        return Err(
            ApiError::bad_request(
                "admin.users.validation_failed",
                "There is a problem with one or more fields.",
            )
            .with_field_errors(field_errors),
        );
    }

    let display_name = payload
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let send_password_reset = payload.send_password_reset.unwrap_or(true);

    let pool = state.local_auth.pool();
    let user_id = Uuid::now_v7();

    let user = match users::create_user_admin(pool, user_id, &email, role, status, display_name)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            // Avoid coupling the API crate to sqlx just to match on error codes.
            // This string match is intentionally defensive (schema/index names may differ).
            let msg = e.to_string();
            if msg.contains("duplicate key value violates unique constraint")
                && (msg.contains("email") || msg.contains("users_email_key"))
            {
                let mut field_errors = std::collections::HashMap::new();
                field_errors.insert("email".to_string(), "Email is already in use".to_string());
                return Err(
                    ApiError::conflict("admin.users.email_not_unique", "Email is already in use.")
                        .with_field_errors(field_errors),
                );
            }

            tracing::error!("Failed to create user: {}", e);
            return Err(
                ApiError::internal("admin.users.create_failed", "Failed to create user")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "admin.users.create",
                        "user_id": user_id,
                        "email": &email
                    })),
            );
        }
    };

    if send_password_reset {
        // Let the user set an initial password via the existing password reset flow.
        if let Err(e) = state
            .email_totp
            .request_code(
                UnderlayUuid(user_id),
                &email,
                acme_db::auth::EmailTotpPurpose::PasswordReset,
            )
            .await
        {
            tracing::warn!("Failed to send password reset email for new user: {}", e);
        }
    }

    Ok(Json(serde_json::json!({ "data": UserResponse::from(user) })).into_response())
}

/// Get a single user (admin).
///
/// GET /v1/admin/users/:user_id
pub async fn get_user(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match users::get_user_with_session_count(pool, user_id).await {
        Ok(Some(user)) => {
            let response: UserDetailResponse = user.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(ApiError::not_found("admin.users.not_found", "User not found")),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            Err(
                ApiError::internal("admin.users.get_failed", "Failed to get user")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "admin.users.get",
                        "user_id": user_id
                    })),
            )
        }
    }
}

/// Update a user (admin).
///
/// PUT /v1/admin/users/:user_id
pub async fn update_user(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Response, ApiError> {
    if let Err(validation_err) = payload.validate() {
        let mut field_errors = std::collections::HashMap::new();
        for (field, errors) in validation_err.field_errors() {
            if let Some(err) = errors.first() {
                let msg = err
                    .message
                    .clone()
                    .unwrap_or_else(|| "Invalid value".into());
                field_errors.insert(
                    field_name_for_errors(field.as_ref()).to_string(),
                    msg.to_string(),
                );
            }
        }
        return Err(
            ApiError::bad_request(
                "admin.users.validation_failed",
                "There is a problem with one or more fields.",
            )
            .with_field_errors(field_errors),
        );
    }

    let email = payload.email.as_deref().map(str::trim);
    let role = payload.role.as_deref().map(str::trim);
    let status = payload.status.as_deref().map(str::trim);

    let mut field_errors = std::collections::HashMap::new();
    if let Some(email) = email {
        if email.is_empty() {
            field_errors.insert("email".to_string(), "Email is required".to_string());
        }
    }
    if let Some(role) = role {
        if !is_valid_role(role) {
            field_errors.insert("role".to_string(), "Invalid role".to_string());
        }
    }
    if let Some(status) = status {
        if !is_valid_status(status) {
            field_errors.insert("status".to_string(), "Invalid status".to_string());
        }
    }
    if !field_errors.is_empty() {
        return Err(
            ApiError::bad_request(
                "admin.users.validation_failed",
                "There is a problem with one or more fields.",
            )
            .with_field_errors(field_errors),
        );
    }

    let email_update = payload.email.is_some();
    let normalized_email = email
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase());
    let display_name_update = payload.display_name.is_some();
    let display_name = payload
        .display_name
        .as_deref()
        .map(str::trim)
        .and_then(|s| if s.is_empty() { None } else { Some(s) });

    let pool = state.local_auth.pool();
    match users::update_user_admin(
        pool,
        user_id,
        email_update,
        normalized_email.as_deref(),
        display_name_update,
        display_name,
        role,
        status,
    )
    .await
    {
        Ok(Some(user)) => Ok(Json(serde_json::json!({ "data": UserResponse::from(user) })).into_response()),
        Ok(None) => Err(ApiError::not_found("admin.users.not_found", "User not found")),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("duplicate key value violates unique constraint")
                && (msg.contains("email") || msg.contains("users_email_key"))
            {
                let mut field_errors = std::collections::HashMap::new();
                field_errors.insert("email".to_string(), "Email is already in use".to_string());
                return Err(
                    ApiError::conflict("admin.users.email_not_unique", "Email is already in use.")
                        .with_field_errors(field_errors),
                );
            }

            tracing::error!("Failed to update user: {}", e);
            Err(
                ApiError::internal("admin.users.update_failed", "Failed to update user")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "admin.users.update",
                        "user_id": user_id
                    })),
            )
        }
    }
}

/// Update a user's role.
///
/// PUT /v1/admin/users/:user_id/role
pub async fn update_user_role(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::bad_request("validation.failed", e.to_string()));
    }

    // Validate role value
    let valid_roles = [
        "user",
        "tester",
        "editor",
        "admin",
        "support",
        "superadmin",
    ];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(ApiError::bad_request(
            "validation.invalid_role",
            "Invalid role value",
        ));
    }

    let pool = state.local_auth.pool();

    match users::update_user_role(pool, user_id, &req.role).await {
        Ok(Some(user)) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "role_change",
                    resource_type: "user",
                    resource_id: user_id,
                    details: Some(serde_json::json!({ "new_role": req.role })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: UserResponse = user.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(ApiError::not_found("admin.users.not_found", "User not found")),
        Err(e) => {
            tracing::error!("Failed to update user role: {}", e);
            Err(
                ApiError::internal("admin.users.update_role_failed", "Failed to update user role")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "admin.users.update_role",
                        "user_id": user_id,
                        "role": &req.role
                    })),
            )
        }
    }
}

/// Suspend a user.
///
/// POST /v1/admin/users/:user_id/suspend
pub async fn suspend_user(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
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

            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "suspend",
                    resource_type: "user",
                    resource_id: user_id,
                    details: None,
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: UserResponse = user.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(ApiError::not_found("admin.users.not_found", "User not found")),
        Err(e) => {
            tracing::error!("Failed to suspend user: {}", e);
            Err(
                ApiError::internal("admin.users.suspend_failed", "Failed to suspend user")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "admin.users.suspend",
                        "user_id": user_id
                    })),
            )
        }
    }
}

/// Unsuspend (reactivate) a user.
///
/// POST /v1/admin/users/:user_id/unsuspend
pub async fn unsuspend_user(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match users::update_user_status(pool, user_id, "active").await {
        Ok(Some(user)) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "unsuspend",
                    resource_type: "user",
                    resource_id: user_id,
                    details: None,
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let response: UserResponse = user.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(ApiError::not_found("admin.users.not_found", "User not found")),
        Err(e) => {
            tracing::error!("Failed to unsuspend user: {}", e);
            Err(
                ApiError::internal("admin.users.unsuspend_failed", "Failed to unsuspend user")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "admin.users.unsuspend",
                        "user_id": user_id
                    })),
            )
        }
    }
}

// ============================================================================
// Session Management
// ============================================================================

/// Session response for admin view.
///
/// Matches the common Session type from the TypeScript client.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
    pub last_used_at: String,
    pub access_token_expires_at: String,
    pub refresh_token_expires_at: String,
    pub revoked_at: Option<String>,
    pub revocation_reason: Option<String>,
}

impl From<users::SessionRow> for SessionResponse {
    fn from(row: users::SessionRow) -> Self {
        Self {
            id: row.id.to_string(),
            user_id: row.user_id.to_string(),
            status: row.status,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            created_at: row.created_at.to_rfc3339(),
            last_used_at: row.last_used_at.to_rfc3339(),
            access_token_expires_at: row.access_token_expires_at.to_rfc3339(),
            refresh_token_expires_at: row.refresh_token_expires_at.to_rfc3339(),
            revoked_at: row.revoked_at.map(|dt| dt.to_rfc3339()),
            revocation_reason: row.revocation_reason,
        }
    }
}

/// Path parameters for session operations.
#[derive(Debug, Deserialize)]
pub struct UserSessionPath {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

/// List all sessions for a user (admin).
///
/// GET /v1/admin/users/:user_id/sessions
pub async fn list_user_sessions(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match users::list_sessions_for_user(pool, user_id).await {
        Ok(sessions) => {
            let items: Vec<SessionResponse> = sessions.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({ "data": items })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list sessions for user: {}", e);
            Err(
                ApiError::internal(
                    "admin.users.list_sessions_failed",
                    "Failed to list user sessions",
                )
                .with_cause(&e)
                .with_context(json!({
                    "operation": "admin.users.list_sessions",
                    "user_id": user_id
                })),
            )
        }
    }
}

/// Revoke a specific session for a user (admin).
///
/// POST /v1/admin/users/:user_id/sessions/:session_id/revoke
pub async fn revoke_user_session(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    Path(path): Path<UserSessionPath>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match users::revoke_session_admin(pool, path.user_id, path.session_id, "Revoked by admin").await
    {
        Ok(true) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "revoke_session",
                    resource_type: "user",
                    resource_id: path.user_id,
                    details: Some(serde_json::json!({ "session_id": path.session_id.to_string() })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            Ok(Json(serde_json::json!({ "ok": true })).into_response())
        }
        Ok(false) => {
            Err(ApiError::not_found(
                "session.not_found",
                "Session not found or already revoked",
            ))
        }
        Err(e) => {
            tracing::error!("Failed to revoke session: {}", e);
            Err(
                ApiError::internal(
                    "admin.users.revoke_session_failed",
                    "Failed to revoke session",
                )
                .with_cause(&e)
                .with_context(json!({
                    "operation": "admin.users.revoke_session",
                    "user_id": path.user_id,
                    "session_id": path.session_id
                })),
            )
        }
    }
}
