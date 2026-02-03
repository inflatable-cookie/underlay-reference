//! Activity log database functions.
//!
//! Query and insertion functions for the platform.audit_log table.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

use crate::DbPool;

// ============================================================================
// Row Types
// ============================================================================

/// Activity log entry row.
#[derive(Debug, Clone, FromRow)]
pub struct ActivityRow {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub details: JsonValue,
    pub correlation_id: Option<String>,
    pub ip_address: Option<String>,
}

/// Activity log entry with actor info (joined with users).
#[derive(Debug, Clone, FromRow)]
pub struct ActivityWithActorRow {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_display_name: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub details: JsonValue,
    pub correlation_id: Option<String>,
    pub ip_address: Option<String>,
}

/// Paginated response for activity lists.
#[derive(Debug)]
pub struct ActivityListResponse {
    pub data: Vec<ActivityWithActorRow>,
    pub has_more: bool,
    pub total: i64,
}

// ============================================================================
// Query Functions
// ============================================================================

/// List all activity (global feed) with pagination.
pub async fn list_activity(
    pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<ActivityListResponse, sqlx::Error> {
    let data = sqlx::query_as::<_, ActivityWithActorRow>(
        r#"
        SELECT
            a.id,
            a.occurred_at,
            a.user_id,
            u.email AS actor_email,
            u.display_name AS actor_display_name,
            a.action,
            a.resource_type,
            a.resource_id,
            a.details,
            a.correlation_id,
            a.ip_address
        FROM platform.audit_log a
        LEFT JOIN auth.users u ON a.user_id = u.id
        ORDER BY a.occurred_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM platform.audit_log
        "#,
    )
    .fetch_one(pool)
    .await?;

    let has_more = (offset + data.len() as i64) < total;

    Ok(ActivityListResponse {
        data,
        has_more,
        total,
    })
}

/// List activity for a specific entity.
pub async fn list_activity_for_entity(
    pool: &DbPool,
    resource_type: &str,
    resource_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<ActivityListResponse, sqlx::Error> {
    let data = sqlx::query_as::<_, ActivityWithActorRow>(
        r#"
        SELECT
            a.id,
            a.occurred_at,
            a.user_id,
            u.email AS actor_email,
            u.display_name AS actor_display_name,
            a.action,
            a.resource_type,
            a.resource_id,
            a.details,
            a.correlation_id,
            a.ip_address
        FROM platform.audit_log a
        LEFT JOIN auth.users u ON a.user_id = u.id
        WHERE a.resource_type = $1 AND a.resource_id = $2
        ORDER BY a.occurred_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(resource_type)
    .bind(resource_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM platform.audit_log
        WHERE resource_type = $1 AND resource_id = $2
        "#,
    )
    .bind(resource_type)
    .bind(resource_id)
    .fetch_one(pool)
    .await?;

    let has_more = (offset + data.len() as i64) < total;

    Ok(ActivityListResponse {
        data,
        has_more,
        total,
    })
}

/// List activity performed by a specific user.
pub async fn list_activity_for_user(
    pool: &DbPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<ActivityListResponse, sqlx::Error> {
    let data = sqlx::query_as::<_, ActivityWithActorRow>(
        r#"
        SELECT
            a.id,
            a.occurred_at,
            a.user_id,
            u.email AS actor_email,
            u.display_name AS actor_display_name,
            a.action,
            a.resource_type,
            a.resource_id,
            a.details,
            a.correlation_id,
            a.ip_address
        FROM platform.audit_log a
        LEFT JOIN auth.users u ON a.user_id = u.id
        WHERE a.user_id = $1
        ORDER BY a.occurred_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM platform.audit_log
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let has_more = (offset + data.len() as i64) < total;

    Ok(ActivityListResponse {
        data,
        has_more,
        total,
    })
}

// ============================================================================
// Insert Functions
// ============================================================================

/// Parameters for logging an activity.
pub struct LogActivityParams<'a> {
    pub user_id: Option<Uuid>,
    pub action: &'a str,
    pub resource_type: &'a str,
    pub resource_id: Uuid,
    pub details: Option<JsonValue>,
    pub correlation_id: Option<&'a str>,
    pub ip_address: Option<&'a str>,
}

/// Log an activity to the audit log.
pub async fn log_activity(
    pool: &DbPool,
    params: LogActivityParams<'_>,
) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO platform.audit_log (
            user_id,
            action,
            resource_type,
            resource_id,
            details,
            correlation_id,
            ip_address
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(params.user_id)
    .bind(params.action)
    .bind(params.resource_type)
    .bind(params.resource_id)
    .bind(params.details.unwrap_or(serde_json::json!({})))
    .bind(params.correlation_id)
    .bind(params.ip_address)
    .fetch_one(pool)
    .await?;

    Ok(id)
}
