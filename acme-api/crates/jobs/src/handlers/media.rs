use async_trait::async_trait;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};
use underlay_blob::{BlobAdapter, BlobAdapterObjectKeyExt, BlobObjectKey};
use underlay_jobs::{Job, JobConfig, JobHandler, JobHandlerError};
use underlay_media::image::{generate_thumbnail, ThumbnailConfig};
use underlay_media::renditions::RenditionConfig;
use underlay_media::storage::rendition_object_key;

// ============================================================================
// Job Handler: media.generate_thumbnail
// ============================================================================

/// Generate a thumbnail for an uploaded image.
///
/// Payload: `{ "media_id": "uuid", "version_id": "uuid" }`
///
/// This handler:
/// 1. Fetches the original image from blob storage
/// 2. Resizes it to a thumbnail (size from RenditionConfig)
/// 3. Stores the thumbnail in blob storage
/// 4. Creates a rendition record in the database
pub struct GenerateThumbnailHandler {
    pool: Arc<PgPool>,
    blob_adapter: Arc<dyn BlobAdapter>,
    rendition_config: RenditionConfig,
}

impl GenerateThumbnailHandler {
    pub fn new(
        pool: Arc<PgPool>,
        blob_adapter: Arc<dyn BlobAdapter>,
        rendition_config: RenditionConfig,
    ) -> Self {
        Self {
            pool,
            blob_adapter,
            rendition_config,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GenerateThumbnailPayload {
    media_id: uuid::Uuid,
    version_id: uuid::Uuid,
}

#[async_trait]
impl JobHandler for GenerateThumbnailHandler {
    fn job_type(&self) -> &'static str {
        "media.generate_thumbnail"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            timeout_seconds: Some(60), // 1 minute timeout
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: GenerateThumbnailPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        // Get version info
        let version: Option<(String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT object_key, mime_type
            FROM media.media_version
            WHERE id = $1 AND state = 'ready'
            "#,
        )
        .bind(payload.version_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        let Some((object_key, mime_type)) = version else {
            warn!(version_id = %payload.version_id, "version not found or not ready");
            return Ok(()); // Nothing to do
        };
        let object_key = BlobObjectKey::parse(object_key)
            .map_err(|e| JobHandlerError::permanent(format!("invalid source key: {e}")))?;

        // Only process images
        let mime = mime_type.as_deref().unwrap_or("");
        if !mime.starts_with("image/") || mime == "image/svg+xml" {
            info!(version_id = %payload.version_id, mime_type = mime, "skipping non-raster image");
            return Ok(());
        }

        // Check if thumbnail already exists
        let existing: Option<uuid::Uuid> = sqlx::query_scalar(
            r#"
            SELECT id FROM media.media_rendition
            WHERE media_version_id = $1 AND kind = 'thumbnail'
            "#,
        )
        .bind(payload.version_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        if existing.is_some() {
            info!(version_id = %payload.version_id, "thumbnail already exists");
            return Ok(());
        }

        // Download original image
        let original_bytes = self
            .blob_adapter
            .get_object_bytes(&object_key)
            .await
            .map_err(|e| JobHandlerError::new(format!("failed to download original: {}", e)))?;

        // Generate thumbnail using underlay-image (Lanczos3 resampling for quality)
        let thumb_size = self.rendition_config.thumbnail_max_dimension();
        let config = ThumbnailConfig::new(thumb_size, thumb_size)
            .with_quality(self.rendition_config.jpeg_quality());

        let result = generate_thumbnail(&original_bytes, &config).map_err(|e| {
            JobHandlerError::permanent(format!("failed to generate thumbnail: {}", e))
        })?;

        // Generate thumbnail object key using standardized storage pattern
        let thumb_object_key = rendition_object_key(payload.media_id, payload.version_id, "thumb")
            .map_err(|e| JobHandlerError::permanent(format!("invalid thumbnail key: {e}")))?;

        // Upload thumbnail
        self.blob_adapter
            .put_object_bytes(&thumb_object_key, &result.data, result.mime_type)
            .await
            .map_err(|e| JobHandlerError::new(format!("failed to upload thumbnail: {}", e)))?;

        // Create rendition record
        let rendition_id = uuid::Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO media.media_rendition
                (id, media_version_id, kind, byte_size, mime_type, width, height,
                 storage_provider, bucket, object_key)
            VALUES ($1, $2, 'thumbnail', $3, $4, $5, $6, 'local', 'media', $7)
            "#,
        )
        .bind(rendition_id)
        .bind(payload.version_id)
        .bind(result.data.len() as i64)
        .bind(result.mime_type)
        .bind(result.width as i32)
        .bind(result.height as i32)
        .bind(thumb_object_key.as_str())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("failed to create rendition: {}", e)))?;

        info!(
            job_id = %job.id,
            media_id = %payload.media_id,
            version_id = %payload.version_id,
            width = result.width,
            height = result.height,
            size = result.data.len(),
            "generated thumbnail"
        );

        Ok(())
    }
}

// ============================================================================
// Job Handler: media.cleanup_orphans
// ============================================================================

/// Soft-delete media items that have never been used (attached to content).
///
/// Payload: `{ "unused_days": 7 }`
///
/// This helps clean up media that was uploaded but never actually used,
/// such as abandoned uploads or test files.
pub struct OrphanMediaCleanupHandler {
    pool: Arc<PgPool>,
}

impl OrphanMediaCleanupHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct OrphanMediaCleanupPayload {
    /// Days after which unused media should be soft-deleted (default: 7)
    unused_days: Option<i32>,
}

#[async_trait]
impl JobHandler for OrphanMediaCleanupHandler {
    fn job_type(&self) -> &'static str {
        "media.cleanup_orphans"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: OrphanMediaCleanupPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        let unused_days = payload.unused_days.unwrap_or(7);
        let cutoff = chrono::Utc::now() - chrono::Duration::days(unused_days as i64);

        // Soft-delete media that:
        // - Has no usages (not attached to any content)
        // - Was created more than N days ago
        // - Is not already deleted
        //
        // Note: In a real app, you'd join with a media_usage table to check
        // if media is referenced. For now, we check if it's been viewed/updated
        // since creation as a proxy for "used".
        let result = sqlx::query(
            r#"
            UPDATE media.media
            SET deleted_at = NOW()
            WHERE deleted_at IS NULL
              AND created_at < $1
              AND updated_at = created_at
              AND NOT EXISTS (
                  SELECT 1 FROM media.media_version mv
                  WHERE mv.media_id = media.id
                    AND mv.state = 'ready'
              )
            "#,
        )
        .bind(cutoff)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        info!(
            job_id = %job.id,
            soft_deleted = result.rows_affected(),
            cutoff_date = %cutoff,
            "orphan media cleanup completed"
        );

        Ok(())
    }
}
