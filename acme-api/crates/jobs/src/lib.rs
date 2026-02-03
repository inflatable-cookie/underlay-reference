//! Background job system for Acme.
//!
//! This crate wraps `underlay-jobs` and provides Acme-specific job handlers.
//!
//! ## Job Handlers
//!
//! ### Platform (maintenance)
//! - `platform.jobs_cleanup` - Purge old completed/failed jobs from history
//!
//! ### Domain (example business logic)
//! - `tasks.cleanup_completed` - Cleanup old completed tasks (batch processing)
//! - `tasks.send_reminder` - Send task reminder email (single-item processing)
//! - `projects.generate_report` - Generate project report (long-running with progress)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};

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
        let project_name: Option<String> = sqlx::query_scalar(
            r#"SELECT name FROM tasks.projects WHERE id = $1"#,
        )
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
// Registry Builder
// ============================================================================

/// Create a job registry with all Acme job handlers registered.
pub fn create_registry(pool: Arc<PgPool>) -> JobRegistry {
    let mut registry = JobRegistry::new();

    // Platform handlers
    registry.register(JobsCleanupHandler::new(pool.clone()));

    // Domain handlers
    registry.register(CleanupCompletedTasksHandler::new(pool.clone()));
    registry.register(SendTaskReminderHandler::new(pool.clone()));
    registry.register(GenerateProjectReportHandler::new(pool));

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
