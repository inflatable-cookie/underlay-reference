//! User admin database functions.
//!
//! Query and management functions for user administration.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use underlay_http::query::QueryParams;
use underlay_query::{FieldMapping, WhereBuilder};
use uuid::Uuid;

use crate::DbPool;

// ============================================================================
// Row Types
// ============================================================================

/// Row type for auth.users table (admin view).
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub status: String,
    pub display_name: Option<String>,
    pub failed_login_count: i32,
    pub lockout_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Extended user row with session count.
#[derive(Debug, Clone, FromRow)]
pub struct UserWithSessionCountRow {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub status: String,
    pub display_name: Option<String>,
    pub failed_login_count: i32,
    pub lockout_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active_session_count: i64,
}

/// Paginated response for user lists.
#[derive(Debug)]
pub struct UserListResponse {
    pub data: Vec<UserRow>,
    pub has_more: bool,
    pub total: i64,
}

// ============================================================================
// Query Functions
// ============================================================================

/// Get a single user by ID (admin view).
pub async fn get_user_admin(pool: &DbPool, user_id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        SELECT
            id,
            email,
            role,
            status,
            display_name,
            failed_login_count,
            lockout_until,
            created_at,
            updated_at
        FROM auth.users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Get a single user by ID with active session count (admin detail view).
pub async fn get_user_with_session_count(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Option<UserWithSessionCountRow>, sqlx::Error> {
    sqlx::query_as::<_, UserWithSessionCountRow>(
        r#"
        SELECT
            u.id,
            u.email,
            u.role,
            u.status,
            u.display_name,
            u.failed_login_count,
            u.lockout_until,
            u.created_at,
            u.updated_at,
            COALESCE(
                (SELECT COUNT(*) FROM auth.sessions s
                 WHERE s.user_id = u.id
                   AND s.status = 'active'
                   AND s.refresh_token_expires_at > NOW()),
                0
            ) AS active_session_count
        FROM auth.users u
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// List users with pagination (admin).
///
/// Supports filtering by role and status, and search by email.
/// Uses offset-based pagination for admin user management.
pub async fn list_users_admin(
    pool: &DbPool,
    query: &QueryParams,
    limit: i64,
    offset: i64,
) -> Result<UserListResponse, sqlx::Error> {
    let mapping = FieldMapping::new()
        .map("email", "email")
        .map("display_name", "display_name")
        .filter_only("query", "COALESCE(display_name, '') || ' ' || email")
        .map("role", "role")
        .map("status", "status")
        .sort_only("created_at", "created_at")
        .sort_only("updated_at", "updated_at");
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let where_sql = if where_clause.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clause)
    };
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "created_at DESC");

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM auth.users
        {}
        "#,
        where_sql
    );

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for value in &filter_values {
        count_query = count_query.bind(value);
    }
    let total = count_query.fetch_one(pool).await?;

    let sql = format!(
        r#"
        SELECT
            id,
            email,
            role,
            status,
            display_name,
            failed_login_count,
            lockout_until,
            created_at,
            updated_at
        FROM auth.users
        {}
        ORDER BY {}
        LIMIT ${}
        OFFSET ${}
        "#,
        where_sql,
        order_by,
        filter_values.len() + 1,
        filter_values.len() + 2
    );

    let mut data_query = sqlx::query_as::<_, UserRow>(&sql);
    for value in filter_values {
        data_query = data_query.bind(value);
    }
    let data = data_query.bind(limit).bind(offset).fetch_all(pool).await?;

    let has_more = (offset + data.len() as i64) < total;

    Ok(UserListResponse {
        data,
        has_more,
        total,
    })
}

/// Update a user's role.
pub async fn update_user_role(
    pool: &DbPool,
    user_id: Uuid,
    new_role: &str,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE auth.users
        SET role = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            email,
            role,
            status,
            display_name,
            failed_login_count,
            lockout_until,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(new_role)
    .fetch_optional(pool)
    .await
}

/// Update a user's status.
pub async fn update_user_status(
    pool: &DbPool,
    user_id: Uuid,
    new_status: &str,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE auth.users
        SET status = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            email,
            role,
            status,
            display_name,
            failed_login_count,
            lockout_until,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(new_status)
    .fetch_optional(pool)
    .await
}

/// Create a user (admin).
///
/// Creates the auth.users row only (no credentials). Use password reset flow
/// to let the user set an initial password.
pub async fn create_user_admin(
    pool: &DbPool,
    user_id: Uuid,
    email: &str,
    role: &str,
    status: &str,
    display_name: Option<&str>,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO auth.users (id, email, role, status, display_name)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            email,
            role,
            status,
            display_name,
            failed_login_count,
            lockout_until,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(role)
    .bind(status)
    .bind(display_name)
    .fetch_one(pool)
    .await
}

/// Update a user (admin).
///
/// Allows updating display name, role, and status.
#[allow(clippy::too_many_arguments)]
pub async fn update_user_admin(
    pool: &DbPool,
    user_id: Uuid,
    email_update: bool,
    email: Option<&str>,
    display_name_update: bool,
    display_name: Option<&str>,
    role: Option<&str>,
    status: Option<&str>,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE auth.users
        SET
            email = CASE WHEN $2 THEN COALESCE($3, email) ELSE email END,
            display_name = CASE WHEN $4 THEN $5 ELSE display_name END,
            role = COALESCE($6, role),
            status = COALESCE($7, status),
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            email,
            role,
            status,
            display_name,
            failed_login_count,
            lockout_until,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(email_update)
    .bind(email)
    .bind(display_name_update)
    .bind(display_name)
    .bind(role)
    .bind(status)
    .fetch_optional(pool)
    .await
}

/// Count active sessions for a user.
pub async fn count_active_sessions_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM auth.sessions
        WHERE user_id = $1
          AND status = 'active'
          AND refresh_token_expires_at > NOW()
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Revoke all sessions for a user.
///
/// Useful when suspending a user to force logout.
pub async fn revoke_all_user_sessions(
    pool: &DbPool,
    user_id: Uuid,
    reason: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE auth.sessions
        SET status = 'revoked',
            revoked_at = NOW(),
            revocation_reason = $2,
            is_active = false
        WHERE user_id = $1
          AND status = 'active'
        "#,
    )
    .bind(user_id)
    .bind(reason)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============================================================================
// Admin Session Management
// ============================================================================

/// Row type for session listing (admin view).
#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
}

/// List all sessions for a user (admin view).
///
/// Returns all sessions (active, expired, revoked) for administrative purposes.
pub async fn list_sessions_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<SessionRow>, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT
            id,
            user_id,
            status,
            ip_address,
            user_agent,
            created_at,
            last_used_at,
            access_token_expires_at,
            refresh_token_expires_at,
            revoked_at,
            revocation_reason
        FROM auth.sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Revoke a specific session (admin action).
///
/// Returns true if the session was found and revoked, false if not found or already revoked.
pub async fn revoke_session_admin(
    pool: &DbPool,
    user_id: Uuid,
    session_id: Uuid,
    reason: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE auth.sessions
        SET status = 'revoked',
            revoked_at = NOW(),
            revocation_reason = $3,
            is_active = false
        WHERE id = $2
          AND user_id = $1
          AND status = 'active'
        "#,
    )
    .bind(user_id)
    .bind(session_id)
    .bind(reason)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
