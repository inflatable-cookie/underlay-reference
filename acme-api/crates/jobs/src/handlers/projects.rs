use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};
use underlay_jobs::{Job, JobConfig, JobHandler, JobHandlerError};

// ============================================================================
// Job Handler: projects.generate_reports
// ============================================================================

/// Enqueue report generation jobs for all projects.
///
/// Payload: `{ "project_ids": ["uuid", ...] }` (optional)
///
/// This scheduled job fans out into individual `projects.generate_report` jobs.
pub struct GenerateProjectReportsHandler {
    pool: Arc<PgPool>,
}

impl GenerateProjectReportsHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct GenerateProjectReportsPayload {
    project_ids: Option<Vec<uuid::Uuid>>,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)] // Fields used by sqlx::FromRow derive
struct ProjectIdRow {
    id: uuid::Uuid,
}

#[async_trait]
impl JobHandler for GenerateProjectReportsHandler {
    fn job_type(&self) -> &'static str {
        "projects.generate_reports"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 3,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: GenerateProjectReportsPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        let project_ids = if let Some(ids) = payload.project_ids {
            ids
        } else {
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

            rows.into_iter().map(|row| row.id).collect()
        };

        let mut reports_generated = 0;
        let mut projects_missing = 0;
        let mut errors = 0;

        for project_id in &project_ids {
            match build_project_report(self.pool.as_ref(), *project_id).await {
                Ok(Some(report)) => {
                    reports_generated += 1;
                    info!(
                        job_id = %job.id,
                        project_id = %project_id,
                        report = ?report,
                        "generated project report"
                    );
                }
                Ok(None) => {
                    projects_missing += 1;
                    warn!(project_id = %project_id, "project not found, skipping report");
                }
                Err(e) => {
                    errors += 1;
                    warn!(project_id = %project_id, error = %e, "failed to generate report");
                }
            }
        }

        info!(
            job_id = %job.id,
            projects_found = project_ids.len(),
            reports_generated = reports_generated,
            projects_missing = projects_missing,
            errors = errors,
            "project report batch completed"
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
    project_id: Option<uuid::Uuid>,
    project_ids: Option<Vec<uuid::Uuid>>,
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

async fn build_project_report(
    pool: &PgPool,
    project_id: uuid::Uuid,
) -> Result<Option<ProjectReport>, JobHandlerError> {
    let project_name: Option<String> =
        sqlx::query_scalar(r#"SELECT name FROM acme.projects WHERE id = $1"#)
            .bind(project_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

    let Some(project_name) = project_name else {
        return Ok(None);
    };

    let stats: (i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'completed') as completed,
            COUNT(*) FILTER (WHERE status = 'pending') as pending,
            COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress
        FROM acme.tasks
        WHERE project_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

    let (total, completed, pending, in_progress) = stats;

    let overdue: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM acme.tasks
        WHERE project_id = $1
          AND deleted_at IS NULL
          AND status != 'completed'
          AND due_date < CURRENT_DATE
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| JobHandlerError::new(format!("database error: {}", e)))?;

    let completion_rate = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(Some(ProjectReport {
        project_name,
        total_tasks: total,
        completed_tasks: completed,
        pending_tasks: pending,
        in_progress_tasks: in_progress,
        completion_rate,
        overdue_tasks: overdue,
        generated_at: chrono::Utc::now(),
    }))
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

        let Some(project_id) = payload.project_id else {
            let project_ids = if let Some(ids) = payload.project_ids {
                ids
            } else {
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

                rows.into_iter().map(|row| row.id).collect()
            };

            let mut reports_generated = 0;
            let mut projects_missing = 0;
            let mut errors = 0;

            for project_id in &project_ids {
                match build_project_report(self.pool.as_ref(), *project_id).await {
                    Ok(Some(report)) => {
                        reports_generated += 1;
                        info!(
                            job_id = %job.id,
                            project_id = %project_id,
                            report = ?report,
                            "generated project report"
                        );
                    }
                    Ok(None) => {
                        projects_missing += 1;
                        warn!(project_id = %project_id, "project not found, skipping report");
                    }
                    Err(e) => {
                        errors += 1;
                        warn!(project_id = %project_id, error = %e, "failed to generate report");
                    }
                }
            }

            info!(
                job_id = %job.id,
                projects_found = project_ids.len(),
                reports_generated = reports_generated,
                projects_missing = projects_missing,
                errors = errors,
                "project report batch completed"
            );

            return Ok(());
        };

        let Some(report) = build_project_report(self.pool.as_ref(), project_id).await? else {
            return Err(JobHandlerError::permanent("project not found"));
        };

        info!(
            job_id = %job.id,
            project_id = %project_id,
            report = ?report,
            "generated project report"
        );

        // In a real app, you might store the report or send it somewhere
        // For now, we just log it

        Ok(())
    }
}
