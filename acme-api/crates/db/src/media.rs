//! Database layer for the Media Library.
//!
//! Provides raw database operations for media, media versions, renditions, and usage tracking.

use crate::DbPool;
use sqlx::FromRow;
use underlay_db::pagination::{Cursor, PaginatedResponse, PaginationBuilder, PaginationParams};
use underlay_http::query::{FieldMapping, QueryParams, WhereBuilder};
use uuid::Uuid;

// ============================================================================
// Row Types
// ============================================================================

/// Raw DB representation of a media item.
#[derive(Debug, Clone, FromRow)]
pub struct MediaRow {
    pub id: Uuid,
    pub kind: String,
    pub visibility: String,
    pub title: String,
    pub original_filename: Option<String>,
    pub current_version_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Option<Uuid>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_by: Option<Uuid>,
}

/// Raw DB representation of a media version.
#[derive(Debug, Clone, FromRow)]
pub struct MediaVersionRow {
    pub id: Uuid,
    pub media_id: Uuid,
    pub state: String,
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub storage_provider: Option<String>,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
}

/// Raw DB representation of a media rendition.
#[derive(Debug, Clone, FromRow)]
pub struct MediaRenditionRow {
    pub id: Uuid,
    pub media_version_id: Uuid,
    pub kind: String,
    pub byte_size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub storage_provider: String,
    pub bucket: String,
    pub object_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Raw DB representation of a media usage record.
#[derive(Debug, Clone, FromRow)]
pub struct MediaUsageRow {
    pub id: Uuid,
    pub media_id: Uuid,
    pub used_by_type: String,
    pub used_by_id: Uuid,
    pub field: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Media with current version info for list views.
#[derive(Debug, Clone, FromRow)]
pub struct MediaWithVersionRow {
    pub id: Uuid,
    pub kind: String,
    pub visibility: String,
    pub title: String,
    pub original_filename: Option<String>,
    pub current_version_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    // From current version
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
}

// ============================================================================
// Field Mapping
// ============================================================================

/// Get field mapping for media queries.
///
/// Supports filtering by kind, visibility, and title (search).
/// Supports sorting by title, kind, updatedAt, createdAt.
pub fn media_field_mapping() -> FieldMapping {
    FieldMapping::new()
        .map("title", "m.title")
        .map("kind", "m.kind")
        .map("visibility", "m.visibility")
        .sort_only("updatedAt", "m.updated_at")
        .sort_only("createdAt", "m.created_at")
}

// ============================================================================
// Media Queries
// ============================================================================

/// Create a new media item.
pub async fn create_media(
    pool: &DbPool,
    id: Uuid,
    kind: &str,
    visibility: &str,
    title: &str,
    original_filename: Option<&str>,
    created_by: Option<Uuid>,
) -> Result<MediaRow, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        INSERT INTO media.media (id, kind, visibility, title, original_filename, created_by, updated_by)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        RETURNING id, kind, visibility, title, original_filename, current_version_id,
                  created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        "#,
    )
    .bind(id)
    .bind(kind)
    .bind(visibility)
    .bind(title)
    .bind(original_filename)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

/// Get a media item by ID.
pub async fn get_media(pool: &DbPool, id: Uuid) -> Result<Option<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        SELECT id, kind, visibility, title, original_filename, current_version_id,
               created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        FROM media.media
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Get a media item by ID (admin, excluding deleted).
pub async fn get_media_admin(pool: &DbPool, id: Uuid) -> Result<Option<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        SELECT id, kind, visibility, title, original_filename, current_version_id,
               created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        FROM media.media
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// List all media items with filtering and sorting (admin, excluding deleted and incomplete uploads).
pub async fn list_media_admin(
    pool: &DbPool,
    query: &QueryParams,
) -> Result<Vec<MediaWithVersionRow>, sqlx::Error> {
    let mapping = media_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    where_builder.add_condition("m.deleted_at IS NULL");
    where_builder.add_condition("m.current_version_id IS NOT NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "m.updated_at DESC");

    let sql = format!(
        r#"
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.updated_at, m.deleted_at,
               v.byte_size, v.mime_type
        FROM media.media m
        LEFT JOIN media.media_version v ON m.current_version_id = v.id
        WHERE {}
        ORDER BY {}
        "#,
        where_clause, order_by
    );

    let mut query_builder = sqlx::query_as::<_, MediaWithVersionRow>(&sql);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }

    query_builder.fetch_all(pool).await
}

/// List media items with pagination (admin, excluding deleted and incomplete uploads).
pub async fn list_media_admin_paginated(
    pool: &DbPool,
    params: PaginationParams,
) -> Result<PaginatedResponse<MediaWithVersionRow>, sqlx::Error> {
    let builder = PaginationBuilder::new(params.clone());
    let cursor = builder.decode_timestamp_cursor().ok().flatten();

    let items = if let Some(cursor) = cursor {
        match params.direction {
            underlay_db::pagination::PaginationDirection::Forward => {
                sqlx::query_as::<_, MediaWithVersionRow>(
                    r#"
                    SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
                           m.created_at, m.updated_at, m.deleted_at,
                           v.byte_size, v.mime_type
                    FROM media.media m
                    LEFT JOIN media.media_version v ON m.current_version_id = v.id
                    WHERE m.deleted_at IS NULL
                      AND m.current_version_id IS NOT NULL
                      AND (m.updated_at, m.id) < ($1, $2)
                    ORDER BY m.updated_at DESC, m.id DESC
                    LIMIT $3
                    "#,
                )
                .bind(cursor.timestamp)
                .bind(cursor.id)
                .bind(builder.query_limit())
                .fetch_all(pool)
                .await?
            }
            underlay_db::pagination::PaginationDirection::Backward => {
                let mut items = sqlx::query_as::<_, MediaWithVersionRow>(
                    r#"
                    SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
                           m.created_at, m.updated_at, m.deleted_at,
                           v.byte_size, v.mime_type
                    FROM media.media m
                    LEFT JOIN media.media_version v ON m.current_version_id = v.id
                    WHERE m.deleted_at IS NULL
                      AND m.current_version_id IS NOT NULL
                      AND (m.updated_at, m.id) > ($1, $2)
                    ORDER BY m.updated_at ASC, m.id ASC
                    LIMIT $3
                    "#,
                )
                .bind(cursor.timestamp)
                .bind(cursor.id)
                .bind(builder.query_limit())
                .fetch_all(pool)
                .await?;
                items.reverse();
                items
            }
        }
    } else {
        sqlx::query_as::<_, MediaWithVersionRow>(
            r#"
            SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
                   m.created_at, m.updated_at, m.deleted_at,
                   v.byte_size, v.mime_type
            FROM media.media m
            LEFT JOIN media.media_version v ON m.current_version_id = v.id
            WHERE m.deleted_at IS NULL
              AND m.current_version_id IS NOT NULL
            ORDER BY m.updated_at DESC, m.id DESC
            LIMIT $1
            "#,
        )
        .bind(builder.query_limit())
        .fetch_all(pool)
        .await?
    };

    let total = if params.include_total {
        Some(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM media.media WHERE deleted_at IS NULL AND current_version_id IS NOT NULL",
            )
            .fetch_one(pool)
            .await?,
        )
    } else {
        None
    };

    Ok(builder.build_response(items, total, |row| {
        Cursor::new()
            .with_timestamp("t", row.updated_at)
            .with_id(row.id)
    }))
}

/// List soft-deleted media items (trash).
pub async fn list_media_trash(pool: &DbPool) -> Result<Vec<MediaWithVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaWithVersionRow>(
        r#"
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.updated_at, m.deleted_at,
               v.byte_size, v.mime_type
        FROM media.media m
        LEFT JOIN media.media_version v ON m.current_version_id = v.id
        WHERE m.deleted_at IS NOT NULL
          AND m.current_version_id IS NOT NULL
        ORDER BY m.deleted_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Update a media item.
pub async fn update_media(
    pool: &DbPool,
    id: Uuid,
    title: &str,
    original_filename: Option<&str>,
    visibility: &str,
    updated_by: Option<Uuid>,
) -> Result<MediaRow, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        UPDATE media.media
        SET title = $2,
            original_filename = COALESCE($3, original_filename),
            visibility = $4,
            updated_by = $5,
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, kind, visibility, title, original_filename, current_version_id,
                  created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(original_filename)
    .bind(visibility)
    .bind(updated_by)
    .fetch_one(pool)
    .await
}

/// Soft delete a media item.
pub async fn soft_delete_media(
    pool: &DbPool,
    id: Uuid,
    deleted_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE media.media
        SET deleted_at = NOW(), deleted_by = $2
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(deleted_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// Restore a soft-deleted media item.
pub async fn restore_media(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE media.media
        SET deleted_at = NULL, deleted_by = NULL
        WHERE id = $1 AND deleted_at IS NOT NULL
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Hard delete a media item (for purge).
pub async fn purge_media(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM media.media WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================================================
// Media Version Queries
// ============================================================================

/// Create a new media version (in uploading state).
pub async fn create_media_version(
    pool: &DbPool,
    id: Uuid,
    media_id: Uuid,
    created_by: Option<Uuid>,
) -> Result<MediaVersionRow, sqlx::Error> {
    sqlx::query_as::<_, MediaVersionRow>(
        r#"
        INSERT INTO media.media_version (id, media_id, state, created_by)
        VALUES ($1, $2, 'uploading', $3)
        RETURNING id, media_id, state, byte_size, mime_type, sha256,
                  storage_provider, bucket, object_key, created_at, created_by
        "#,
    )
    .bind(id)
    .bind(media_id)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

/// Get a media version by ID.
pub async fn get_media_version(
    pool: &DbPool,
    id: Uuid,
) -> Result<Option<MediaVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaVersionRow>(
        r#"
        SELECT id, media_id, state, byte_size, mime_type, sha256,
               storage_provider, bucket, object_key, created_at, created_by
        FROM media.media_version
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// List all versions for a media item.
pub async fn list_media_versions(
    pool: &DbPool,
    media_id: Uuid,
) -> Result<Vec<MediaVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaVersionRow>(
        r#"
        SELECT id, media_id, state, byte_size, mime_type, sha256,
               storage_provider, bucket, object_key, created_at, created_by
        FROM media.media_version
        WHERE media_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(media_id)
    .fetch_all(pool)
    .await
}

/// Finalise a media version (transition to ready state).
#[allow(clippy::too_many_arguments)]
pub async fn finalise_media_version(
    pool: &DbPool,
    id: Uuid,
    byte_size: i64,
    mime_type: &str,
    sha256: &str,
    storage_provider: &str,
    bucket: &str,
    object_key: &str,
) -> Result<MediaVersionRow, sqlx::Error> {
    sqlx::query_as::<_, MediaVersionRow>(
        r#"
        UPDATE media.media_version
        SET state = 'ready',
            byte_size = $2,
            mime_type = $3,
            sha256 = $4,
            storage_provider = $5,
            bucket = $6,
            object_key = $7
        WHERE id = $1 AND state = 'uploading'
        RETURNING id, media_id, state, byte_size, mime_type, sha256,
                  storage_provider, bucket, object_key, created_at, created_by
        "#,
    )
    .bind(id)
    .bind(byte_size)
    .bind(mime_type)
    .bind(sha256)
    .bind(storage_provider)
    .bind(bucket)
    .bind(object_key)
    .fetch_one(pool)
    .await
}

/// Update media's current_version_id.
pub async fn set_current_version(
    pool: &DbPool,
    media_id: Uuid,
    version_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE media.media
        SET current_version_id = $2, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(media_id)
    .bind(version_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark a version as failed.
pub async fn fail_media_version(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE media.media_version
        SET state = 'failed'
        WHERE id = $1 AND state = 'uploading'
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete a media version.
///
/// Note: The caller should ensure this version is not the current version
/// and should also delete any associated blob storage objects.
pub async fn delete_media_version(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM media.media_version
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Find a media version by SHA-256 hash (for deduplication).
/// Returns the media that owns the version (via its current_version_id).
pub async fn find_media_by_hash(
    pool: &DbPool,
    sha256: &str,
) -> Result<Option<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.created_by, m.updated_at, m.updated_by, m.deleted_at, m.deleted_by
        FROM media.media m
        JOIN media.media_version v ON m.current_version_id = v.id
        WHERE v.sha256 = $1
          AND v.state = 'ready'
          AND m.deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(sha256)
    .fetch_optional(pool)
    .await
}

// ============================================================================
// Media Usage Queries
// ============================================================================

/// Add a usage record.
pub async fn add_media_usage(
    pool: &DbPool,
    id: Uuid,
    media_id: Uuid,
    used_by_type: &str,
    used_by_id: Uuid,
    field: &str,
) -> Result<MediaUsageRow, sqlx::Error> {
    sqlx::query_as::<_, MediaUsageRow>(
        r#"
        INSERT INTO media.media_usage (id, media_id, used_by_type, used_by_id, field)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (media_id, used_by_type, used_by_id, field) DO UPDATE
        SET id = media.media_usage.id
        RETURNING id, media_id, used_by_type, used_by_id, field, created_at
        "#,
    )
    .bind(id)
    .bind(media_id)
    .bind(used_by_type)
    .bind(used_by_id)
    .bind(field)
    .fetch_one(pool)
    .await
}

/// Remove a usage record.
pub async fn remove_media_usage(
    pool: &DbPool,
    media_id: Uuid,
    used_by_type: &str,
    used_by_id: Uuid,
    field: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM media.media_usage
        WHERE media_id = $1 AND used_by_type = $2 AND used_by_id = $3 AND field = $4
        "#,
    )
    .bind(media_id)
    .bind(used_by_type)
    .bind(used_by_id)
    .bind(field)
    .execute(pool)
    .await?;
    Ok(())
}

/// List all usages for a media item.
pub async fn list_media_usages(
    pool: &DbPool,
    media_id: Uuid,
) -> Result<Vec<MediaUsageRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaUsageRow>(
        r#"
        SELECT id, media_id, used_by_type, used_by_id, field, created_at
        FROM media.media_usage
        WHERE media_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(media_id)
    .fetch_all(pool)
    .await
}

/// Get usage count for a media item.
pub async fn get_media_usage_count(pool: &DbPool, media_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM media.media_usage WHERE media_id = $1",
    )
    .bind(media_id)
    .fetch_one(pool)
    .await
}

/// List media items with zero usages (excludes incomplete uploads).
pub async fn list_unused_media(pool: &DbPool) -> Result<Vec<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.created_by, m.updated_at, m.updated_by, m.deleted_at, m.deleted_by
        FROM media.media m
        LEFT JOIN media.media_usage u ON m.id = u.media_id
        WHERE m.deleted_at IS NULL
          AND m.current_version_id IS NOT NULL
          AND u.id IS NULL
        ORDER BY m.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// List all usages for a specific entity (used_by_type, used_by_id, field).
pub async fn list_usages_by_entity(
    pool: &DbPool,
    used_by_type: &str,
    used_by_id: Uuid,
    field: &str,
) -> Result<Vec<MediaUsageRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaUsageRow>(
        r#"
        SELECT id, media_id, used_by_type, used_by_id, field, created_at
        FROM media.media_usage
        WHERE used_by_type = $1 AND used_by_id = $2 AND field = $3
        ORDER BY created_at DESC
        "#,
    )
    .bind(used_by_type)
    .bind(used_by_id)
    .bind(field)
    .fetch_all(pool)
    .await
}

// ============================================================================
// Media Rendition Queries
// ============================================================================

/// Create a rendition.
#[allow(clippy::too_many_arguments)]
pub async fn create_media_rendition(
    pool: &DbPool,
    id: Uuid,
    media_version_id: Uuid,
    kind: &str,
    byte_size: i64,
    mime_type: &str,
    width: Option<i32>,
    height: Option<i32>,
    storage_provider: &str,
    bucket: &str,
    object_key: &str,
) -> Result<MediaRenditionRow, sqlx::Error> {
    sqlx::query_as::<_, MediaRenditionRow>(
        r#"
        INSERT INTO media.media_rendition
            (id, media_version_id, kind, byte_size, mime_type, width, height,
             storage_provider, bucket, object_key)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, media_version_id, kind, byte_size, mime_type, width, height,
                  storage_provider, bucket, object_key, created_at
        "#,
    )
    .bind(id)
    .bind(media_version_id)
    .bind(kind)
    .bind(byte_size)
    .bind(mime_type)
    .bind(width)
    .bind(height)
    .bind(storage_provider)
    .bind(bucket)
    .bind(object_key)
    .fetch_one(pool)
    .await
}

/// List renditions for a version.
pub async fn list_media_renditions(
    pool: &DbPool,
    media_version_id: Uuid,
) -> Result<Vec<MediaRenditionRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRenditionRow>(
        r#"
        SELECT id, media_version_id, kind, byte_size, mime_type, width, height,
               storage_provider, bucket, object_key, created_at
        FROM media.media_rendition
        WHERE media_version_id = $1
        ORDER BY kind
        "#,
    )
    .bind(media_version_id)
    .fetch_all(pool)
    .await
}
