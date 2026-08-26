//! Dashboard statistics database functions.
//!
//! Aggregate queries for admin dashboard metrics.

use sqlx::FromRow;

use crate::DbPool;

// ============================================================================
// Stats Types
// ============================================================================

/// User counts by status.
#[derive(Debug, Clone, FromRow)]
pub struct UserCounts {
    pub active: i64,
    pub suspended: i64,
    pub deleted: i64,
    pub total: i64,
}

// ============================================================================
// Query Functions
// ============================================================================

/// Get user counts by status.
pub async fn get_user_counts(pool: &DbPool) -> Result<UserCounts, sqlx::Error> {
    sqlx::query_as::<_, UserCounts>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'active') AS active,
            COUNT(*) FILTER (WHERE status = 'suspended') AS suspended,
            COUNT(*) FILTER (WHERE status = 'deleted') AS deleted,
            COUNT(*) AS total
        FROM auth.users
        "#,
    )
    .fetch_one(pool)
    .await
}

/// Get total media count (non-deleted).
pub async fn get_media_count(pool: &DbPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM media.media
        WHERE deleted_at IS NULL
        "#,
    )
    .fetch_one(pool)
    .await
}

/// Get count of new user registrations in the last N days.
pub async fn get_recent_registrations(pool: &DbPool, days: i32) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM auth.users
        WHERE created_at > NOW() - make_interval(days => $1)
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await
}

/// Get count of active sessions across all users.
pub async fn get_active_sessions_count(pool: &DbPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM auth.sessions
        WHERE status = 'active'
          AND refresh_token_expires_at > NOW()
        "#,
    )
    .fetch_one(pool)
    .await
}
