use super::*;

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
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media.media_usage WHERE media_id = $1")
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
