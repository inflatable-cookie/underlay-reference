use super::*;

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
    sqlx::query_as::<_, RawMediaRenditionRow>(
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
    .await?
    .try_into()
}

/// List renditions for a version.
pub async fn list_media_renditions(
    pool: &DbPool,
    media_version_id: Uuid,
) -> Result<Vec<MediaRenditionRow>, sqlx::Error> {
    sqlx::query_as::<_, RawMediaRenditionRow>(
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
    .await?
    .into_iter()
    .map(TryInto::try_into)
    .collect()
}
