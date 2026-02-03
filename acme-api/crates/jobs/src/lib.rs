//! Background job system for Acme.
//!
//! This crate wraps `underlay-jobs` and provides Acme-specific job handlers.
//!
//! ## Job Handlers
//!
//! ### Platform (maintenance)
//! - `platform.jobs_cleanup` - Purge old completed/failed jobs from history
//! - `auth.cleanup_sessions` - Remove expired sessions and old refresh tokens
//!
//! ### Email
//! - `email.welcome` - Send welcome email to newly registered users
//!
//! ### Tasks (example business logic)
//! - `tasks.cleanup_completed` - Cleanup old completed tasks (batch processing)
//! - `tasks.send_reminder` - Send task reminder email (single-item processing)
//! - `tasks.check_due_reminders` - Check for upcoming due dates and enqueue reminders
//!
//! ### Projects
//! - `projects.generate_report` - Generate project report (long-running with progress)
//!
//! ### Media
//! - `media.generate_thumbnail` - Generate thumbnails for uploaded images
//! - `media.cleanup_orphans` - Soft-delete unused media files

use async_trait::async_trait;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::io::Cursor;
use std::sync::Arc;
use tracing::{info, warn};
use underlay_blob::{BlobAdapter, MediaConfig};

// Re-export everything from underlay-jobs.
pub use underlay_jobs::{
    BackoffStrategy, Job, JobConfig, JobErrorRecord, JobFilters, JobHandler, JobHandlerError,
    JobId, JobProgress, JobRegistry, JobRepository, JobResult, JobRunner, JobRunnerConfig,
    JobStatus, JobStore, PgJobNotifier, RepoError, ScheduledTask, ScheduledTaskDefinition,
    ScheduledTaskRepository, Scheduler, JOB_NOTIFY_CHANNEL, JOB_NOTIFY_SQL, JOB_TABLES_SQL,
};

// ============================================================================
// Job Handler: platform.jobs_cleanup
// ============================================================================

/// Purge old job history (completed/failed jobs).
///
/// Payload: `{ "days_old": 30 }`
///
/// This handler is portable across all Underlay apps since it uses the
/// JobRepository directly from underlay-jobs.
pub struct JobsCleanupHandler {
    pool: Arc<PgPool>,
}

impl JobsCleanupHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct JobsCleanupPayload {
    /// Days to retain completed/failed jobs (default: 30)
    days_old: Option<i32>,
}

#[async_trait]
impl JobHandler for JobsCleanupHandler {
    fn job_type(&self) -> &'static str {
        "platform.jobs_cleanup"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: JobsCleanupPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        let days_old = payload.days_old.unwrap_or(30);

        let job_repo = JobRepository::new((*self.pool).clone());
        let purged = job_repo
            .purge_history(days_old)
            .await
            .map_err(|e| JobHandlerError::new(format!("job history purge failed: {}", e)))?;

        info!(
            job_id = %job.id,
            purged = purged,
            days_old = days_old,
            "job history cleanup completed"
        );

        Ok(())
    }
}

// ============================================================================
// Job Handler: tasks.cleanup_completed
// ============================================================================

/// Cleanup completed tasks older than a specified number of days.
///
/// Payload: `{ "project_id": "uuid", "days_old": 30 }`
pub struct CleanupCompletedTasksHandler {
    pool: Arc<PgPool>,
}

impl CleanupCompletedTasksHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct CleanupCompletedTasksPayload {
    project_id: uuid::Uuid,
    days_old: Option<i32>,
}

#[async_trait]
impl JobHandler for CleanupCompletedTasksHandler {
    fn job_type(&self) -> &'static str {
        "tasks.cleanup_completed"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: CleanupCompletedTasksPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        let days_old = payload.days_old.unwrap_or(30);
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old as i64);

        let result = sqlx::query(
            r#"
            DELETE FROM tasks.tasks
            WHERE project_id = $1
              AND status = 'completed'
              AND completed_at < $2
            "#,
        )
        .bind(payload.project_id)
        .bind(cutoff)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        info!(
            job_id = %job.id,
            project_id = %payload.project_id,
            deleted = result.rows_affected(),
            "cleaned up completed tasks"
        );

        Ok(())
    }
}

// ============================================================================
// Job Handler: tasks.send_reminder
// ============================================================================

/// Send a reminder email for an overdue task.
///
/// Payload: `{ "task_id": "uuid", "user_email": "user@example.com" }`
pub struct SendTaskReminderHandler {
    pool: Arc<PgPool>,
}

impl SendTaskReminderHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct SendTaskReminderPayload {
    task_id: uuid::Uuid,
    user_email: String,
}

#[async_trait]
impl JobHandler for SendTaskReminderHandler {
    fn job_type(&self) -> &'static str {
        "tasks.send_reminder"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 5,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: SendTaskReminderPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        // Fetch task details
        let task: Option<(String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT title, due_date::text
            FROM tasks.tasks
            WHERE id = $1
            "#,
        )
        .bind(payload.task_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        let Some((title, due_date)) = task else {
            warn!(task_id = %payload.task_id, "task not found, skipping reminder");
            return Ok(()); // Task was deleted, nothing to do
        };

        // In a real app, send email via email service
        // For now, just log the reminder
        info!(
            job_id = %job.id,
            task_id = %payload.task_id,
            email = %payload.user_email,
            task_title = %title,
            due_date = ?due_date,
            "would send task reminder email"
        );

        Ok(())
    }
}

// ============================================================================
// Job Handler: projects.generate_report
// ============================================================================

/// Generate a summary report for a project.
///
/// Payload: `{ "project_id": "uuid" }`
///
/// This handler demonstrates progress tracking for long-running jobs.
pub struct GenerateProjectReportHandler {
    pool: Arc<PgPool>,
}

impl GenerateProjectReportHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct GenerateProjectReportPayload {
    project_id: uuid::Uuid,
}

#[derive(Debug, Serialize)]
struct ProjectReport {
    project_name: String,
    total_tasks: i64,
    completed_tasks: i64,
    pending_tasks: i64,
    in_progress_tasks: i64,
    completion_rate: f64,
    overdue_tasks: i64,
    generated_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
impl JobHandler for GenerateProjectReportHandler {
    fn job_type(&self) -> &'static str {
        "projects.generate_report"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            timeout_seconds: Some(300), // 5 minute timeout
            tracks_progress: true,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: GenerateProjectReportPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        // Fetch project name
        let project_name: Option<String> =
            sqlx::query_scalar(r#"SELECT name FROM tasks.projects WHERE id = $1"#)
                .bind(payload.project_id)
                .fetch_optional(self.pool.as_ref())
                .await
                .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        let Some(project_name) = project_name else {
            return Err(JobHandlerError::permanent("project not found"));
        };

        // Count tasks by status
        let stats: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress
            FROM tasks.tasks
            WHERE project_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(payload.project_id)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        let (total, completed, pending, in_progress) = stats;

        // Count overdue tasks
        let overdue: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM tasks.tasks
            WHERE project_id = $1
              AND deleted_at IS NULL
              AND status != 'completed'
              AND due_date < CURRENT_DATE
            "#,
        )
        .bind(payload.project_id)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        let completion_rate = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let report = ProjectReport {
            project_name,
            total_tasks: total,
            completed_tasks: completed,
            pending_tasks: pending,
            in_progress_tasks: in_progress,
            completion_rate,
            overdue_tasks: overdue,
            generated_at: chrono::Utc::now(),
        };

        info!(
            job_id = %job.id,
            project_id = %payload.project_id,
            report = ?report,
            "generated project report"
        );

        // In a real app, you might store the report or send it somewhere
        // For now, we just log it

        Ok(())
    }
}

// ============================================================================
// Job Handler: media.generate_thumbnail
// ============================================================================

/// Generate a thumbnail for an uploaded image.
///
/// Payload: `{ "media_id": "uuid", "version_id": "uuid" }`
///
/// This handler:
/// 1. Fetches the original image from blob storage
/// 2. Resizes it to a thumbnail (size from MediaConfig)
/// 3. Stores the thumbnail in blob storage
/// 4. Creates a rendition record in the database
pub struct GenerateThumbnailHandler {
    pool: Arc<PgPool>,
    blob_adapter: Arc<dyn BlobAdapter>,
    media_config: MediaConfig,
}

impl GenerateThumbnailHandler {
    pub fn new(
        pool: Arc<PgPool>,
        blob_adapter: Arc<dyn BlobAdapter>,
        media_config: MediaConfig,
    ) -> Self {
        Self {
            pool,
            blob_adapter,
            media_config,
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
            .get_bytes(&object_key)
            .await
            .map_err(|e| JobHandlerError::new(format!("failed to download original: {}", e)))?;

        // Decode and resize image
        let img = image::load_from_memory(&original_bytes)
            .map_err(|e| JobHandlerError::permanent(format!("failed to decode image: {}", e)))?;

        let thumb_size = self.media_config.thumbnail_max_dimension;
        let thumbnail = img.thumbnail(thumb_size, thumb_size);
        let (width, height) = (thumbnail.width(), thumbnail.height());

        // Encode as WebP for efficient storage
        let mut thumbnail_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut thumbnail_bytes);
        thumbnail
            .write_to(&mut cursor, ImageFormat::WebP)
            .map_err(|e| JobHandlerError::new(format!("failed to encode thumbnail: {}", e)))?;

        // Generate thumbnail object key
        let thumb_object_key = format!(
            "media/{}/versions/{}/thumbnail.webp",
            payload.media_id, payload.version_id
        );

        // Upload thumbnail
        self.blob_adapter
            .put_bytes(&thumb_object_key, &thumbnail_bytes, "image/webp")
            .await
            .map_err(|e| JobHandlerError::new(format!("failed to upload thumbnail: {}", e)))?;

        // Create rendition record
        let rendition_id = uuid::Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO media.media_rendition
                (id, media_version_id, kind, byte_size, mime_type, width, height,
                 storage_provider, bucket, object_key)
            VALUES ($1, $2, 'thumbnail', $3, 'image/webp', $4, $5, 'local', 'media', $6)
            "#,
        )
        .bind(rendition_id)
        .bind(payload.version_id)
        .bind(thumbnail_bytes.len() as i64)
        .bind(width as i32)
        .bind(height as i32)
        .bind(&thumb_object_key)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("failed to create rendition: {}", e)))?;

        info!(
            job_id = %job.id,
            media_id = %payload.media_id,
            version_id = %payload.version_id,
            width = width,
            height = height,
            size = thumbnail_bytes.len(),
            "generated thumbnail"
        );

        Ok(())
    }
}

// ============================================================================
// Job Handler: email.welcome
// ============================================================================

/// Send a welcome email to a newly registered user.
///
/// Payload: `{ "user_id": "uuid", "email": "user@example.com", "display_name": "John" }`
pub struct WelcomeEmailHandler {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl WelcomeEmailHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct WelcomeEmailPayload {
    user_id: uuid::Uuid,
    email: String,
    display_name: Option<String>,
}

#[async_trait]
impl JobHandler for WelcomeEmailHandler {
    fn job_type(&self) -> &'static str {
        "email.welcome"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 5,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: WelcomeEmailPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        // In a real app, send email via email service (underlay-email)
        // For the reference implementation, we just log it
        info!(
            job_id = %job.id,
            user_id = %payload.user_id,
            email = %payload.email,
            display_name = ?payload.display_name,
            "would send welcome email"
        );

        Ok(())
    }
}

// ============================================================================
// Job Handler: auth.cleanup_sessions
// ============================================================================

/// Cleanup expired sessions and old refresh tokens.
///
/// Payload: `{ "expired_sessions_days": 7, "old_tokens_days": 30 }`
pub struct SessionCleanupHandler {
    pool: Arc<PgPool>,
}

impl SessionCleanupHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct SessionCleanupPayload {
    /// Days after which expired sessions should be purged (default: 7)
    expired_sessions_days: Option<i32>,
    /// Days after which unused refresh tokens should be purged (default: 30)
    old_tokens_days: Option<i32>,
}

#[async_trait]
impl JobHandler for SessionCleanupHandler {
    fn job_type(&self) -> &'static str {
        "auth.cleanup_sessions"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: SessionCleanupPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        let expired_days = payload.expired_sessions_days.unwrap_or(7);
        let tokens_days = payload.old_tokens_days.unwrap_or(30);

        // Delete sessions that expired more than N days ago
        let session_cutoff = chrono::Utc::now() - chrono::Duration::days(expired_days as i64);
        let sessions_result = sqlx::query(
            r#"
            DELETE FROM auth.session
            WHERE expires_at < $1
            "#,
        )
        .bind(session_cutoff)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("session cleanup failed: {}", e)))?;

        // Delete refresh tokens not used for N days
        let tokens_cutoff = chrono::Utc::now() - chrono::Duration::days(tokens_days as i64);
        let tokens_result = sqlx::query(
            r#"
            DELETE FROM auth.refresh_token
            WHERE last_used_at < $1 OR (last_used_at IS NULL AND created_at < $1)
            "#,
        )
        .bind(tokens_cutoff)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("token cleanup failed: {}", e)))?;

        info!(
            job_id = %job.id,
            sessions_deleted = sessions_result.rows_affected(),
            tokens_deleted = tokens_result.rows_affected(),
            "auth session cleanup completed"
        );

        Ok(())
    }
}

// ============================================================================
// Job Handler: tasks.check_due_reminders
// ============================================================================

/// Check for tasks with upcoming due dates and enqueue individual reminder jobs.
///
/// Payload: `{ "days_ahead": 1 }`
///
/// This is a scheduled job that runs daily and creates individual
/// `tasks.send_reminder` jobs for each task due within the window.
pub struct CheckDueRemindersHandler {
    pool: Arc<PgPool>,
}

impl CheckDueRemindersHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct CheckDueRemindersPayload {
    /// Days ahead to check for due tasks (default: 1)
    days_ahead: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)] // Fields used by sqlx::FromRow derive
struct DueTask {
    id: uuid::Uuid,
    title: String,
    due_date: chrono::NaiveDate,
    user_id: uuid::Uuid,
    user_email: String,
}

#[async_trait]
impl JobHandler for CheckDueRemindersHandler {
    fn job_type(&self) -> &'static str {
        "tasks.check_due_reminders"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: CheckDueRemindersPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        let days_ahead = payload.days_ahead.unwrap_or(1);
        let target_date =
            chrono::Utc::now().date_naive() + chrono::Duration::days(days_ahead as i64);

        // Find tasks due on target date with assigned users
        let due_tasks: Vec<DueTask> = sqlx::query_as(
            r#"
            SELECT t.id, t.title, t.due_date, t.assignee_id as user_id, u.email as user_email
            FROM tasks.tasks t
            JOIN auth.user u ON u.id = t.assignee_id
            WHERE t.due_date = $1
              AND t.status != 'completed'
              AND t.deleted_at IS NULL
              AND t.assignee_id IS NOT NULL
            "#,
        )
        .bind(target_date)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        // Enqueue individual reminder jobs
        let job_repo = JobRepository::new((*self.pool).clone());
        let mut enqueued = 0;

        for task in &due_tasks {
            let reminder_payload = serde_json::json!({
                "task_id": task.id,
                "user_email": task.user_email,
            });

            match job_repo
                .create(
                    "tasks.send_reminder",
                    reminder_payload,
                    &JobConfig::default(),
                )
                .await
            {
                Ok(_) => enqueued += 1,
                Err(e) => {
                    warn!(
                        task_id = %task.id,
                        error = %e,
                        "failed to create reminder job"
                    );
                }
            }
        }

        info!(
            job_id = %job.id,
            tasks_found = due_tasks.len(),
            reminders_enqueued = enqueued,
            target_date = %target_date,
            "due date reminder check completed"
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

// ============================================================================
// Registry Builder
// ============================================================================

/// Create a job registry with all Acme job handlers registered.
///
/// The `blob_adapter` is optional for backwards compatibility; if not provided,
/// the media thumbnail handler will not be registered.
pub fn create_registry(pool: Arc<PgPool>) -> JobRegistry {
    create_registry_with_media(pool, None, MediaConfig::default())
}

/// Create a job registry with blob adapter for media processing jobs.
pub fn create_registry_with_blob(
    pool: Arc<PgPool>,
    blob_adapter: Option<Arc<dyn BlobAdapter>>,
) -> JobRegistry {
    create_registry_with_media(pool, blob_adapter, MediaConfig::default())
}

/// Create a job registry with blob adapter and media config for media processing jobs.
pub fn create_registry_with_media(
    pool: Arc<PgPool>,
    blob_adapter: Option<Arc<dyn BlobAdapter>>,
    media_config: MediaConfig,
) -> JobRegistry {
    let mut registry = JobRegistry::new();

    // Platform handlers
    registry.register(JobsCleanupHandler::new(pool.clone()));
    registry.register(SessionCleanupHandler::new(pool.clone()));

    // Email handlers
    registry.register(WelcomeEmailHandler::new(pool.clone()));

    // Task handlers
    registry.register(CleanupCompletedTasksHandler::new(pool.clone()));
    registry.register(SendTaskReminderHandler::new(pool.clone()));
    registry.register(CheckDueRemindersHandler::new(pool.clone()));
    registry.register(GenerateProjectReportHandler::new(pool.clone()));

    // Media handlers (require blob adapter)
    if let Some(blob) = blob_adapter.clone() {
        registry.register(GenerateThumbnailHandler::new(
            pool.clone(),
            blob,
            media_config,
        ));
    }
    registry.register(OrphanMediaCleanupHandler::new(pool));

    registry
}

// ============================================================================
// Scheduled Task Definitions
// ============================================================================

/// Returns the list of scheduled tasks to register on startup.
///
/// These definitions are upserted into the database by `Scheduler::register_tasks()`.
/// Tasks not in this list will be disabled (preserving data but preventing execution).
///
/// # Adding New Scheduled Tasks
///
/// 1. Add a new `ScheduledTaskDefinition` to this list
/// 2. Create a corresponding job handler if one doesn't exist
/// 3. The task will be automatically registered on next worker startup
///
/// # Cron Schedule Format
///
/// Uses standard cron format: `sec min hour day-of-month month day-of-week`
/// - `0 0 3 * * *` = 3:00 AM daily
/// - `0 0 2 * * 0` = 2:00 AM every Sunday
/// - `0 */15 * * * *` = Every 15 minutes
///
/// See <https://crates.io/crates/cron> for full syntax.
pub fn scheduled_task_definitions() -> Vec<ScheduledTaskDefinition> {
    vec![
        // ====================================================================
        // Platform maintenance tasks
        // ====================================================================

        // Job history cleanup - daily at 2:30 AM
        // Purges old completed/failed jobs
        ScheduledTaskDefinition {
            name: "jobs_cleanup",
            job_type: "platform.jobs_cleanup",
            schedule: "0 30 2 * * *", // 2:30 AM daily
            payload: serde_json::json!({ "days_old": 30 }),
            config: JobConfig {
                max_attempts: 3,
                ..Default::default()
            },
        },
        // Session cleanup - daily at 2:45 AM
        // Removes expired sessions and old refresh tokens
        ScheduledTaskDefinition {
            name: "session_cleanup",
            job_type: "auth.cleanup_sessions",
            schedule: "0 45 2 * * *", // 2:45 AM daily
            payload: serde_json::json!({
                "expired_sessions_days": 7,
                "old_tokens_days": 30
            }),
            config: JobConfig {
                max_attempts: 3,
                ..Default::default()
            },
        },
        // Orphan media cleanup - daily at 3:30 AM
        // Soft-deletes media that was never used
        ScheduledTaskDefinition {
            name: "orphan_media_cleanup",
            job_type: "media.cleanup_orphans",
            schedule: "0 30 3 * * *", // 3:30 AM daily
            payload: serde_json::json!({ "unused_days": 7 }),
            config: JobConfig {
                max_attempts: 3,
                ..Default::default()
            },
        },
        // ====================================================================
        // Domain-specific tasks (examples)
        // ====================================================================

        // Cleanup completed tasks - daily at 3 AM
        ScheduledTaskDefinition {
            name: "cleanup_completed_tasks",
            job_type: "tasks.cleanup_completed",
            schedule: "0 0 3 * * *", // 3:00 AM daily
            payload: serde_json::json!({ "days_old": 30 }),
            config: JobConfig {
                max_attempts: 3,
                ..Default::default()
            },
        },
        // Check for due date reminders - daily at 8 AM
        // Enqueues individual reminder jobs for tasks due tomorrow
        ScheduledTaskDefinition {
            name: "check_due_reminders",
            job_type: "tasks.check_due_reminders",
            schedule: "0 0 8 * * *", // 8:00 AM daily
            payload: serde_json::json!({ "days_ahead": 1 }),
            config: JobConfig {
                max_attempts: 3,
                ..Default::default()
            },
        },
        // Generate project reports - weekly on Sunday at 4 AM
        ScheduledTaskDefinition {
            name: "weekly_project_reports",
            job_type: "projects.generate_report",
            schedule: "0 0 4 * * SUN", // 4:00 AM every Sunday
            payload: serde_json::json!({}),
            config: JobConfig {
                max_attempts: 3,
                timeout_seconds: Some(600), // 10 minutes
                ..Default::default()
            },
        },
    ]
}
