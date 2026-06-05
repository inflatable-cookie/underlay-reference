//! Background job system for Acme.
//!
//! This crate wraps `underlay-jobs` and provides Acme-specific job handlers.
//!
//! ## Job Handlers
//!
//! ### Platform Maintenance (from underlay-jobs-postgres)
//!
//! Standard maintenance tasks are provided by `underlay_jobs_postgres::tasks`:
//! - `purge_expired_sessions` - Remove expired sessions
//! - `purge_auth_states` - Remove expired auth states
//! - `purge_login_attempts` - Remove old login attempts
//! - `purge_rate_limit_entries` - Remove old rate limit entries
//! - `purge_email_totp_codes` - Remove expired/used TOTP codes
//! - `purge_verification_sessions` - Remove expired/used verification sessions
//! - `archive_completed_jobs` - Move old jobs to history table
//! - `purge_job_history` - Remove old job history
//! - `recover_abandoned_jobs` - Reset stalled jobs
//! - `purge_error_logs` - Remove old error logs
//!
//! ### Acme-specific Handlers
//!
//! #### Email
//! - `email.welcome` - Send welcome email to newly registered users
//!
//! #### Tasks (example business logic)
//! - `tasks.cleanup_completed` - Cleanup old completed tasks (batch processing)
//! - `tasks.send_reminder` - Send task reminder email (single-item processing)
//! - `tasks.check_due_reminders` - Check for upcoming due dates and enqueue reminders
//!
//! #### Projects
//! - `projects.generate_report` - Generate project report (long-running with progress)
//! - `projects.generate_reports` - Enqueue report jobs for all projects
//!
//! #### Media
//! - `media.generate_thumbnail` - Generate thumbnails for uploaded images
//! - `media.cleanup_orphans` - Soft-delete unused media files

use std::sync::Arc;

use sqlx::PgPool;
use underlay_blob::{BlobAdapter, MediaConfig};

// Re-export everything from underlay-jobs-postgres.
pub use underlay_jobs::{
    BackoffStrategy, Job, JobConfig, JobErrorRecord, JobFilters, JobHandler, JobHandlerError,
    JobId, JobProgress, JobRegistry, JobResult, JobRunner, JobRunnerConfig, JobStatus, JobStore,
    ScheduledTask, ScheduledTaskDefinition,
};
pub use underlay_jobs_postgres::{
    JobRepository, PgDeadLetterRepository, PgJobNotifier, PostgresJobRunnerExt, RepoError,
    ScheduledTaskRepository, Scheduler, DOMAIN_EVENT_NOTIFY_SQL, JOB_DEAD_LETTERS_SQL,
    JOB_NOTIFY_CHANNEL, JOB_NOTIFY_SQL, JOB_TABLES_SQL,
};

// Re-export outbox components from underlay-jobs
pub use underlay_jobs_postgres::outbox::{
    OutboxConfig, OutboxEvent, OutboxNotifier, OutboxProcessor, DOMAIN_EVENT_NOTIFY_CHANNEL,
};

// Re-export standard platform maintenance tasks from underlay-jobs
pub use underlay_jobs_postgres::tasks::{
    // Job maintenance
    ArchiveCompletedJobsJob,
    // Auth cleanup
    PurgeAuthStatesJob,
    PurgeEmailTotpCodesJob,
    PurgeErrorLogsJob,
    PurgeExpiredSessionsJob,
    PurgeJobHistoryJob,
    PurgeLoginAttemptsJob,
    PurgeRateLimitEntriesJob,
    PurgeVerificationSessionsJob,
    RecoverAbandonedJobsJob,
};

mod handlers;
pub use handlers::*;

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

    // ========================================================================
    // Standard platform maintenance tasks (from underlay-jobs-postgres)
    // ========================================================================

    // Auth cleanup
    registry.register(PurgeExpiredSessionsJob::new((*pool).clone()));
    registry.register(PurgeAuthStatesJob::new((*pool).clone()));
    registry.register(PurgeLoginAttemptsJob::new((*pool).clone()));
    registry.register(PurgeRateLimitEntriesJob::new((*pool).clone()));
    registry.register(PurgeEmailTotpCodesJob::new((*pool).clone()));
    registry.register(PurgeVerificationSessionsJob::new((*pool).clone()));

    // Job system maintenance
    registry.register(ArchiveCompletedJobsJob::new((*pool).clone()));
    registry.register(PurgeJobHistoryJob::new((*pool).clone()));
    registry.register(RecoverAbandonedJobsJob::new((*pool).clone()));

    // Log cleanup
    registry.register(PurgeErrorLogsJob::new((*pool).clone()));
    // ========================================================================
    // Acme-specific handlers
    // ========================================================================

    // Email handlers
    registry.register(WelcomeEmailHandler::new(pool.clone()));

    // Task handlers
    registry.register(CleanupCompletedTasksHandler::new(pool.clone()));
    registry.register(SendTaskReminderHandler::new(pool.clone()));
    registry.register(CheckDueRemindersHandler::new(pool.clone()));
    registry.register(GenerateProjectReportsHandler::new(pool.clone()));
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
        // Platform maintenance tasks (from underlay-jobs-postgres)
        // ====================================================================

        // Purge expired sessions - every 15 minutes
        ScheduledTaskDefinition {
            name: "purge_expired_sessions",
            job_type: "purge_expired_sessions",
            schedule: "0 */15 * * * *", // Every 15 minutes
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Purge expired auth states - hourly
        ScheduledTaskDefinition {
            name: "purge_auth_states",
            job_type: "purge_auth_states",
            schedule: "0 0 * * * *", // Hourly
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Purge old login attempts - daily at 3 AM
        ScheduledTaskDefinition {
            name: "purge_login_attempts",
            job_type: "purge_login_attempts",
            schedule: "0 0 3 * * *", // 3:00 AM daily
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Purge old rate limit entries - hourly
        ScheduledTaskDefinition {
            name: "purge_rate_limit_entries",
            job_type: "purge_rate_limit_entries",
            schedule: "0 5 * * * *", // 5 minutes past each hour
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Purge expired email TOTP codes - hourly
        ScheduledTaskDefinition {
            name: "purge_email_totp_codes",
            job_type: "purge_email_totp_codes",
            schedule: "0 10 * * * *", // 10 minutes past each hour
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Purge expired verification sessions - hourly
        ScheduledTaskDefinition {
            name: "purge_verification_sessions",
            job_type: "purge_verification_sessions",
            schedule: "0 15 * * * *", // 15 minutes past each hour
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Archive completed jobs - daily at 5 AM
        ScheduledTaskDefinition {
            name: "archive_completed_jobs",
            job_type: "archive_completed_jobs",
            schedule: "0 0 5 * * *", // 5:00 AM daily
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Purge old job history - weekly on Sunday at 5 AM
        ScheduledTaskDefinition {
            name: "purge_job_history",
            job_type: "purge_job_history",
            schedule: "0 0 5 * * SUN", // 5:00 AM every Sunday
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Recover abandoned jobs - every 5 minutes
        ScheduledTaskDefinition {
            name: "recover_abandoned_jobs",
            job_type: "recover_abandoned_jobs",
            schedule: "0 */5 * * * *", // Every 5 minutes
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // Purge old error logs - daily at 4 AM
        ScheduledTaskDefinition {
            name: "purge_error_logs",
            job_type: "purge_error_logs",
            schedule: "0 0 4 * * *", // 4:00 AM daily
            payload: serde_json::json!({}),
            config: JobConfig::maintenance(),
        },
        // ====================================================================
        // Acme-specific tasks
        // ====================================================================

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
            job_type: "projects.generate_reports",
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
