use super::*;

/// Create a new media version (in uploading state) with its staging object key.
pub async fn create_media_version(
    pool: &DbPool,
    id: Uuid,
    media_id: Uuid,
    created_by: Option<Uuid>,
    object_key: &str,
) -> Result<MediaVersionRow, sqlx::Error> {
    sqlx::query_as::<_, RawMediaVersionRow>(
        r#"
        INSERT INTO media.media_version (id, media_id, state, created_by, object_key)
        VALUES ($1, $2, 'uploading', $3, $4)
        RETURNING id, media_id, state, byte_size, mime_type, sha256,
                  storage_provider, bucket, object_key, created_at, created_by
        "#,
    )
    .bind(id)
    .bind(media_id)
    .bind(created_by)
    .bind(object_key)
    .fetch_one(pool)
    .await?
    .try_into()
}

/// Persist server-derived promotion facts while the version stays uploading
/// and `object_key` remains the staging identity.
#[allow(clippy::too_many_arguments)]
pub async fn record_verified_promotion(
    pool: &DbPool,
    id: Uuid,
    byte_size: i64,
    mime_type: &str,
    sha256: &str,
    storage_provider: &str,
    bucket: &str,
) -> Result<MediaVersionRow, sqlx::Error> {
    sqlx::query_as::<_, RawMediaVersionRow>(
        r#"
        UPDATE media.media_version
        SET byte_size = $2,
            mime_type = $3,
            sha256 = $4,
            storage_provider = $5,
            bucket = $6
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
    .fetch_one(pool)
    .await?
    .try_into()
}

/// Get a media version by ID.
pub async fn get_media_version(
    pool: &DbPool,
    id: Uuid,
) -> Result<Option<MediaVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, RawMediaVersionRow>(
        r#"
        SELECT id, media_id, state, byte_size, mime_type, sha256,
               storage_provider, bucket, object_key, created_at, created_by
        FROM media.media_version
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .map(TryInto::try_into)
    .transpose()
}

/// List all versions for a media item.
pub async fn list_media_versions(
    pool: &DbPool,
    media_id: Uuid,
) -> Result<Vec<MediaVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, RawMediaVersionRow>(
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
    .await?
    .into_iter()
    .map(TryInto::try_into)
    .collect()
}

/// Atomically mark a version ready and commit `media.current_version_id`.
///
/// Both writes share one transaction. Failure leaves the version uploading
/// and the current pointer unchanged.
#[allow(clippy::too_many_arguments)]
pub async fn activate_ready_current(
    pool: &DbPool,
    id: Uuid,
    media_id: Uuid,
    byte_size: i64,
    mime_type: &str,
    sha256: &str,
    storage_provider: &str,
    bucket: &str,
    object_key: &str,
) -> Result<MediaVersionRow, sqlx::Error> {
    activate_ready_current_inner(
        pool,
        id,
        media_id,
        byte_size,
        mime_type,
        sha256,
        storage_provider,
        bucket,
        object_key,
        false,
    )
    .await
}

/// Same transaction as [`activate_ready_current`], but raises a Postgres error
/// after the version-ready write and before the current-pointer write so the
/// open transaction rolls back both statements.
#[allow(clippy::too_many_arguments)]
pub async fn activate_ready_current_failing_after_version_ready(
    pool: &DbPool,
    id: Uuid,
    media_id: Uuid,
    byte_size: i64,
    mime_type: &str,
    sha256: &str,
    storage_provider: &str,
    bucket: &str,
    object_key: &str,
) -> Result<MediaVersionRow, sqlx::Error> {
    activate_ready_current_inner(
        pool,
        id,
        media_id,
        byte_size,
        mime_type,
        sha256,
        storage_provider,
        bucket,
        object_key,
        true,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn activate_ready_current_inner(
    pool: &DbPool,
    id: Uuid,
    media_id: Uuid,
    byte_size: i64,
    mime_type: &str,
    sha256: &str,
    storage_provider: &str,
    bucket: &str,
    object_key: &str,
    fail_after_version_ready: bool,
) -> Result<MediaVersionRow, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let version = sqlx::query_as::<_, RawMediaVersionRow>(
        r#"
        UPDATE media.media_version
        SET state = 'ready',
            byte_size = $2,
            mime_type = $3,
            sha256 = $4,
            storage_provider = $5,
            bucket = $6,
            object_key = $7
        WHERE id = $1 AND media_id = $8 AND state = 'uploading'
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
    .bind(media_id)
    .fetch_one(&mut *tx)
    .await?;

    if fail_after_version_ready {
        sqlx::query("SELECT 1 / 0").execute(&mut *tx).await?;
    }

    let updated = sqlx::query(
        r#"
        UPDATE media.media
        SET current_version_id = $2, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(media_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() != 1 {
        return Err(sqlx::Error::RowNotFound);
    }

    tx.commit().await?;
    version.try_into()
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
