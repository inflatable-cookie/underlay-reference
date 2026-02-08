use async_trait::async_trait;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};
use underlay_jobs::{Job, JobConfig, JobHandler, JobHandlerError, JobRepository};

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
    project_id: Option<uuid::Uuid>,
    days_old: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)] // Fields used by sqlx::FromRow derive
struct ProjectIdRow {
    id: uuid::Uuid,
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

        let Some(project_id) = payload.project_id else {
            let rows: Vec<ProjectIdRow> = sqlx::query_as(
                r#"
                SELECT id
                FROM acme.projects
                WHERE deleted_at IS NULL
                "#,
            )
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

            let project_ids: Vec<uuid::Uuid> = rows.into_iter().map(|row| row.id).collect();
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old as i64);
            let mut total_deleted = 0;
            let mut errors = 0;

            for project_id in &project_ids {
                match sqlx::query(
                    r#"
                    DELETE FROM acme.tasks
                    WHERE project_id = $1
                      AND status = 'completed'
                      AND completed_at < $2
                    "#,
                )
                .bind(project_id)
                .bind(cutoff)
                .execute(self.pool.as_ref())
                .await
                {
                    Ok(result) => total_deleted += result.rows_affected(),
                    Err(e) => {
                        errors += 1;
                        warn!(
                            project_id = %project_id,
                            error = %e,
                            "failed to cleanup completed tasks"
                        );
                    }
                }
            }

            info!(
                job_id = %job.id,
                projects_found = project_ids.len(),
                deleted = total_deleted,
                errors = errors,
                "cleanup completed tasks batch completed"
            );

            return Ok(());
        };

        let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old as i64);

        let result = sqlx::query(
            r#"
            DELETE FROM acme.tasks
            WHERE project_id = $1
              AND status = 'completed'
              AND completed_at < $2
            "#,
        )
        .bind(project_id)
        .bind(cutoff)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

        info!(
            job_id = %job.id,
            project_id = %project_id,
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
            FROM acme.tasks
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
            SELECT t.id, t.title, t.due_date, ta.user_id, u.email as user_email
            FROM acme.tasks t
            JOIN acme.task_assignees ta ON ta.task_id = t.id
            JOIN auth.users u ON u.id = ta.user_id
            WHERE t.due_date = $1
              AND t.status != 'completed'
              AND t.deleted_at IS NULL
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
