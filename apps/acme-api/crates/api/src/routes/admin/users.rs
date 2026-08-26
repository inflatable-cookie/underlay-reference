//! User admin routes.
//!
//! Provides user management endpoints for administrators.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use underlay_http::{context::RequestContext, query::QueryParams, ApiError};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use acme_auth::UserPrincipal;
use acme_core::Uuid as UnderlayUuid;
use acme_db::{activity, users};

use crate::routes::admin::freshness::{
    build_etag_cache_headers, detail_etag, if_match_mismatch, maybe_not_modified,
    precondition_failed_error,
};
use crate::state::{AdminUser, AppState};

fn is_email_unique_violation(err: &sqlx::Error) -> bool {
    let Some(db_err) = err.as_database_error() else {
        return false;
    };

    if db_err.code().as_deref() != Some("23505") {
        return false;
    }

    if let Some(constraint) = db_err.constraint() {
        return constraint == "users_email_key";
    }

    db_err.message().to_ascii_lowercase().contains("email")
}

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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListUsersQuery {
    #[serde(flatten)]
    pub query: QueryParams,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Limit (default 50)
    pub limit: Option<u32>,
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
        "display_name" => "display_name",
        "send_password_reset" => "send_password_reset",
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

    let limit = query.limit.unwrap_or(50).clamp(1, 100) as i64;
    let page = query.page.unwrap_or(1).max(1) as i64;
    let offset = (page - 1) * limit;

    match users::list_users_admin(pool, &query.query, limit, offset).await {
        Ok(response) => {
            let items: Vec<UserResponse> = response.data.into_iter().map(Into::into).collect();
            Ok(Json(underlay_http::PageList {
                data: items,
                total: response.total as u64,
                has_more: response.has_more,
            })
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.list_failed",
                "Failed to list users",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.list",
                "limit": limit,
                "offset": offset
            })))
        }
    }
}

/// Create a user (admin).
///
/// POST /v1/admin/users
pub async fn create_user(
    AdminUser(admin): AdminUser,
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
        return Err(ApiError::bad_request(
            "admin.users.validation_failed",
            "There is a problem with one or more fields.",
        )
        .with_field_errors(field_errors));
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
        return Err(ApiError::bad_request(
            "admin.users.validation_failed",
            "There is a problem with one or more fields.",
        )
        .with_field_errors(field_errors));
    }

    // Role hierarchy: callers may only create users below their own level.
    can_assign_role(&admin, role)?;

    let display_name = payload
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let send_password_reset = payload.send_password_reset.unwrap_or(true);

    let pool = state.local_auth.pool();
    let user_id = Uuid::now_v7();

    let user =
        match users::create_user_admin(pool, user_id, &email, role, status, display_name).await {
            Ok(user) => user,
            Err(e) => {
                if is_email_unique_violation(&e) {
                    let mut field_errors = std::collections::HashMap::new();
                    field_errors.insert("email".to_string(), "Email is already in use".to_string());
                    return Err(ApiError::conflict(
                        "admin.users.email_not_unique",
                        "Email is already in use.",
                    )
                    .with_field_errors(field_errors));
                }

                tracing::error!("Failed to create user: {}", e);
                return Err(crate::db_errors::internal_with_diagnostics(
                    "admin.users.create_failed",
                    "Failed to create user",
                    &e,
                )
                .with_context(json!({
                    "operation": "admin.users.create",
                    "user_id": user_id,
                    "email": &email
                })));
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
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match users::get_user_with_session_count(pool, user_id).await {
        Ok(Some(user)) => {
            let etag = detail_etag("user", &user.id.to_string(), &user.updated_at.to_rfc3339());
            if let Some(not_modified) = maybe_not_modified(&headers, &etag) {
                return Ok(not_modified);
            }
            let response: UserDetailResponse = user.into();
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Ok(None) => Err(ApiError::not_found(
            "admin.users.not_found",
            "User not found",
        )),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.get_failed",
                "Failed to get user",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.get",
                "user_id": user_id
            })))
        }
    }
}

/// Update a user (admin).
///
/// PUT /v1/admin/users/:user_id
pub async fn update_user(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    headers: HeaderMap,
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
        return Err(ApiError::bad_request(
            "admin.users.validation_failed",
            "There is a problem with one or more fields.",
        )
        .with_field_errors(field_errors));
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
        return Err(ApiError::bad_request(
            "admin.users.validation_failed",
            "There is a problem with one or more fields.",
        )
        .with_field_errors(field_errors));
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
        .filter(|s| !s.is_empty());

    let pool = state.local_auth.pool();
    let current = match users::get_user_admin(pool, user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(ApiError::not_found(
                "admin.users.not_found",
                "User not found",
            ));
        }
        Err(e) => {
            tracing::error!("Failed to load current user before update: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.update_failed",
                "Failed to update user",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.update",
                "user_id": user_id
            })));
        }
    };

    let current_etag = detail_etag(
        "user",
        &current.id.to_string(),
        &current.updated_at.to_rfc3339(),
    );
    if if_match_mismatch(&headers, &current_etag) {
        return Err(precondition_failed_error().with_context(json!({
            "operation": "admin.users.update",
            "user_id": user_id
        })));
    }

    // Enforce the same role-hierarchy rules as update_user_role/suspend_user:
    // no self-management, no managing peers/superiors, no promotion beyond
    // the caller's own level.
    let is_self = admin.user_id.0.into_inner() == user_id;
    can_manage_user(&admin, &current.role, is_self)?;
    if let Some(new_role) = role {
        can_manage_user(&admin, new_role, is_self)?;
    }

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
        Ok(Some(user)) => {
            let response = UserResponse::from(user);
            let etag = detail_etag("user", &response.id, &response.updated_at);
            let response_headers = build_etag_cache_headers(&etag);
            Ok((
                response_headers,
                Json(serde_json::json!({ "data": response })),
            )
                .into_response())
        }
        Ok(None) => Err(ApiError::not_found(
            "admin.users.not_found",
            "User not found",
        )),
        Err(e) => {
            if is_email_unique_violation(&e) {
                let mut field_errors = std::collections::HashMap::new();
                field_errors.insert("email".to_string(), "Email is already in use".to_string());
                return Err(ApiError::conflict(
                    "admin.users.email_not_unique",
                    "Email is already in use.",
                )
                .with_field_errors(field_errors));
            }

            tracing::error!("Failed to update user: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.update_failed",
                "Failed to update user",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.update",
                "user_id": user_id
            })))
        }
    }
}

/// Role hierarchy for privilege checking, delegated to the canonical
/// underlay implementation. Local code only maps the typed role enum to
/// role-name strings and converts errors to ApiError.
fn role_name(role: &acme_auth::UserRole) -> &'static str {
    use acme_auth::UserRole;
    match role {
        UserRole::User => "user",
        UserRole::Tester => "tester",
        UserRole::Support => "support",
        UserRole::Admin => "admin",
        UserRole::Superadmin => "superadmin",
    }
}

fn caller_roles(admin: &UserPrincipal) -> Vec<&'static str> {
    admin.roles.iter().map(role_name).collect()
}

fn hierarchy_error(err: underlay_auth::RoleHierarchyError) -> ApiError {
    use axum::http::StatusCode;
    use underlay_auth::RoleHierarchyError as E;
    let code = match &err {
        E::CannotManageSelf => "admin.users.cannot_manage_self",
        E::CannotManageSuperRole => "admin.users.cannot_manage_superadmin",
        E::InsufficientPrivileges { .. } => "admin.users.insufficient_privileges",
        E::CannotPromoteToSuperRole => "admin.users.cannot_promote_to_superadmin",
    };
    ApiError::new(StatusCode::FORBIDDEN, code, err.to_string())
}

/// Check if an admin can manage a target user based on roles.
// ApiError is the canonical error type here; boxing it would force
// map_err at every `?` call site (matches underlay-http house style).
#[allow(clippy::result_large_err)]
fn can_manage_user(
    admin: &UserPrincipal,
    target_role: &str,
    is_self: bool,
) -> Result<(), ApiError> {
    underlay_auth::RoleHierarchy::standard()
        .can_manage(&caller_roles(admin), target_role, is_self)
        .map_err(hierarchy_error)
}

/// Check if an admin can create/assign a user with the given role.
/// Superadmins may assign any role; others only roles below their own level.
#[allow(clippy::result_large_err)]
fn can_assign_role(admin: &UserPrincipal, role: &str) -> Result<(), ApiError> {
    underlay_auth::RoleHierarchy::standard()
        .can_assign(&caller_roles(admin), role)
        .map_err(hierarchy_error)
}

/// Update a user's role.
///
/// PUT /v1/admin/users/:user_id/role
pub async fn update_user_role(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        let mut field_errors = std::collections::HashMap::new();
        if e.field_errors().contains_key("role") {
            field_errors.insert("role".to_string(), "Role is required".to_string());
        }
        return Err(ApiError::bad_request(
            "admin.users.validation_failed",
            "There is a problem with one or more fields.",
        )
        .with_field_errors(field_errors));
    }

    // Validate role value
    let valid_roles = ["user", "tester", "editor", "admin", "support", "superadmin"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(ApiError::bad_request(
            "validation.invalid_role",
            "Invalid role value",
        ));
    }

    // Check privilege escalation
    let is_self = admin.user_id.0.into_inner() == user_id;
    can_manage_user(&admin, &req.role, is_self)?;

    // Get current user to check their current role
    let pool = state.local_auth.pool();
    let current_user = users::get_user_admin(pool, user_id).await.map_err(|e| {
        crate::db_errors::internal_with_diagnostics(
            "admin.users.fetch_failed",
            "Failed to fetch user",
            &e,
        )
    })?;

    if let Some(ref user) = current_user {
        // Check if trying to demote a higher-privileged user
        can_manage_user(&admin, &user.role, is_self)?;
    }

    let pool = state.local_auth.pool();

    match users::update_user_role(pool, user_id, &req.role).await {
        Ok(Some(user)) => {
            // Log activity
            activity::log_activity_reported(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "role_change",
                    resource_type: "user",
                    resource_id: user_id,
                    details: Some(serde_json::json!({ "new_role": req.role })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: UserResponse = user.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(ApiError::not_found(
            "admin.users.not_found",
            "User not found",
        )),
        Err(e) => {
            tracing::error!("Failed to update user role: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.update_role_failed",
                "Failed to update user role",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.update_role",
                "user_id": user_id,
                "role": &req.role
            })))
        }
    }
}

/// Suspend a user.
///
/// POST /v1/admin/users/:user_id/suspend
pub async fn suspend_user(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    use axum::http::StatusCode;

    // Check privilege escalation
    let is_self = admin.user_id.0.into_inner() == user_id;
    if is_self {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "admin.users.cannot_suspend_self",
            "You cannot suspend your own account.",
        ));
    }

    let pool = state.local_auth.pool();

    // Get current user to check their role
    let current_user = users::get_user_admin(pool, user_id).await.map_err(|e| {
        crate::db_errors::internal_with_diagnostics(
            "admin.users.fetch_failed",
            "Failed to fetch user",
            &e,
        )
    })?;

    if let Some(ref user) = current_user {
        // Prevent admins from suspending other admins
        can_manage_user(&admin, &user.role, is_self)?;
    }

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
            activity::log_activity_reported(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "suspend",
                    resource_type: "user",
                    resource_id: user_id,
                    details: None,
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: UserResponse = user.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(ApiError::not_found(
            "admin.users.not_found",
            "User not found",
        )),
        Err(e) => {
            tracing::error!("Failed to suspend user: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.suspend_failed",
                "Failed to suspend user",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.suspend",
                "user_id": user_id
            })))
        }
    }
}

/// Unsuspend (reactivate) a user.
///
/// POST /v1/admin/users/:user_id/unsuspend
pub async fn unsuspend_user(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    use axum::http::StatusCode;

    // Check privilege escalation
    let is_self = admin.user_id.0.into_inner() == user_id;
    if is_self {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "admin.users.cannot_unsuspend_self",
            "You cannot unsuspend your own account.",
        ));
    }

    let pool = state.local_auth.pool();

    // Get current user to check their role
    let current_user = users::get_user_admin(pool, user_id).await.map_err(|e| {
        crate::db_errors::internal_with_diagnostics(
            "admin.users.fetch_failed",
            "Failed to fetch user",
            &e,
        )
    })?;

    if let Some(ref user) = current_user {
        // Prevent admins from unsuspending other admins
        can_manage_user(&admin, &user.role, is_self)?;
    }

    match users::update_user_status(pool, user_id, "active").await {
        Ok(Some(user)) => {
            // Log activity
            activity::log_activity_reported(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "unsuspend",
                    resource_type: "user",
                    resource_id: user_id,
                    details: None,
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            let response: UserResponse = user.into();
            Ok(Json(serde_json::json!({ "data": response })).into_response())
        }
        Ok(None) => Err(ApiError::not_found(
            "admin.users.not_found",
            "User not found",
        )),
        Err(e) => {
            tracing::error!("Failed to unsuspend user: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.unsuspend_failed",
                "Failed to unsuspend user",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.unsuspend",
                "user_id": user_id
            })))
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
            let total = items.len();
            Ok(Json(serde_json::json!({
                "data": items,
                "total": total,
                "has_more": false
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list sessions for user: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.list_sessions_failed",
                "Failed to list user sessions",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.list_sessions",
                "user_id": user_id
            })))
        }
    }
}

/// Revoke a specific session for a user (admin).
///
/// POST /v1/admin/users/:user_id/sessions/:session_id/revoke
pub async fn revoke_user_session(
    AdminUser(admin): AdminUser,
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(path): Path<UserSessionPath>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match users::revoke_session_admin(pool, path.user_id, path.session_id, "Revoked by admin").await
    {
        Ok(true) => {
            // Log activity
            activity::log_activity_reported(
                pool,
                activity::LogActivityParams {
                    user_id: Some(admin.user_id.0.into_inner()),
                    action: "revoke_session",
                    resource_type: "user",
                    resource_id: path.user_id,
                    details: Some(serde_json::json!({ "session_id": path.session_id.to_string() })),
                    correlation_id: Some(ctx.request_id()),
                    ip_address: None,
                },
            )
            .await;

            Ok(Json(serde_json::json!({ "ok": true })).into_response())
        }
        Ok(false) => Err(ApiError::not_found(
            "session.not_found",
            "Session not found or already revoked",
        )),
        Err(e) => {
            tracing::error!("Failed to revoke session: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "admin.users.revoke_session_failed",
                "Failed to revoke session",
                &e,
            )
            .with_context(json!({
                "operation": "admin.users.revoke_session",
                "user_id": path.user_id,
                "session_id": path.session_id
            })))
        }
    }
}
