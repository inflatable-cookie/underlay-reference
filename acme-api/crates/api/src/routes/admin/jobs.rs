//! Admin routes for background jobs management.
//!
//! These endpoints provide visibility into the job queue for administrators.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use underlay_core::{ListResponse, SingleResponse, Uuid};
use underlay_http::ApiError;
use underlay_jobs::{Job, JobFilters, JobStatus};

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

/// Summary of a job for list views.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JobSummaryDto {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Detailed job information including payload.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JobDetailDto {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub payload: serde_json::Value,
    pub progress: Option<JobProgressDto>,
    pub created_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Progress information for long-running jobs.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JobProgressDto {
    pub current: u64,
    pub total: u64,
    pub percent: f64,
    pub message: Option<String>,
}

impl JobSummaryDto {
    fn from_job(job: &Job) -> Self {
        Self {
            id: job.id.to_string(),
            job_type: job.job_type.clone(),
            status: job.status.as_str().to_string(),
            attempts: job.attempts,
            max_attempts: job.max_attempts,
            created_at: job.created_at,
            started_at: job.started_at,
            finished_at: job.finished_at,
            error_message: job.last_error.clone(),
        }
    }
}

impl JobDetailDto {
    fn from_job(job: Job) -> Self {
        let progress = job.progress.as_ref().map(|p| JobProgressDto {
            current: p.current,
            total: p.total,
            percent: p.percentage(),
            message: p.message.clone(),
        });

        Self {
            id: job.id.to_string(),
            job_type: job.job_type.clone(),
            status: job.status.as_str().to_string(),
            attempts: job.attempts,
            max_attempts: job.max_attempts,
            payload: job.payload.clone(),
            progress,
            created_at: job.created_at,
            scheduled_for: job.scheduled_for,
            started_at: job.started_at,
            finished_at: job.finished_at,
            error_message: job.last_error.clone(),
        }
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListJobsQuery {
    /// Filter by status: pending, running, succeeded, failed, cancelled
    pub status: Option<String>,
    /// Filter by job type
    pub job_type: Option<String>,
    /// Maximum number of jobs to return
    pub limit: Option<usize>,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// List jobs with optional filters.
///
/// GET /v1/admin/jobs
pub async fn list_jobs(
    _user: AdminUser,
    State(state): State<AppState>,
    Query(query): Query<ListJobsQuery>,
) -> Result<Response, ApiError> {
    let Some(ref job_repo) = state.job_repository else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Job system not available",
        ));
    };

    let status = query.status.as_ref().and_then(|s| match s.as_str() {
        "pending" => Some(JobStatus::Pending),
        "running" => Some(JobStatus::Running),
        "succeeded" => Some(JobStatus::Succeeded),
        "failed" => Some(JobStatus::Failed),
        "cancelled" => Some(JobStatus::Cancelled),
        _ => None,
    });

    let filters = JobFilters {
        status,
        job_type: query.job_type.clone(),
        limit: query.limit.unwrap_or(50),
        ..Default::default()
    };

    match job_repo.list(filters).await {
        Ok(jobs) => {
            let items: Vec<JobSummaryDto> = jobs.iter().map(JobSummaryDto::from_job).collect();
            Ok(Json(ListResponse { data: items }).into_response())
        }
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "job_list_failed",
            "Failed to list jobs",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "jobs.list",
            "status": query.status,
            "job_type": query.job_type
        }))),
    }
}

/// Get details of a specific job.
///
/// GET /v1/admin/jobs/:jobId
pub async fn get_job(
    _user: AdminUser,
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let Some(ref job_repo) = state.job_repository else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Job system not available",
        ));
    };

    match job_repo.get(job_id).await {
        Ok(Some(job)) => {
            let dto = JobDetailDto::from_job(job);
            Ok(Json(SingleResponse { data: dto }).into_response())
        }
        Ok(None) => Err(ApiError::not_found("not_found", "Job not found")),
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "job_get_failed",
            "Failed to get job",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "jobs.get",
            "job_id": job_id
        }))),
    }
}

/// Cancel a pending or running job.
///
/// POST /v1/admin/jobs/:jobId/cancel
pub async fn cancel_job(
    _user: AdminUser,
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let Some(ref job_repo) = state.job_repository else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Job system not available",
        ));
    };

    // Get the job first to check its status
    let job = match job_repo.get(job_id).await {
        Ok(Some(job)) => job,
        Ok(None) => return Err(ApiError::not_found("not_found", "Job not found")),
        Err(e) => {
            return Err(crate::db_errors::internal_with_diagnostics(
                "job_get_failed",
                "Failed to get job",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "jobs.cancel.get",
                "job_id": job_id
            })))
        }
    };

    // Only pending or running jobs can be cancelled
    match job.status {
        JobStatus::Pending | JobStatus::Running => {}
        _ => {
            return Err(ApiError::bad_request(
                "invalid_status",
                format!("Cannot cancel job with status {}", job.status.as_str()),
            ))
        }
    }

    match job_repo.cancel(job_id).await {
        Ok(()) => {
            // Return the updated job
            match job_repo.get(job_id).await {
                Ok(Some(job)) => {
                    let dto = JobDetailDto::from_job(job);
                    Ok(Json(SingleResponse { data: dto }).into_response())
                }
                Ok(None) => Err(ApiError::not_found(
                    "not_found",
                    "Job not found after cancel",
                )),
                Err(e) => Err(crate::db_errors::internal_with_diagnostics(
                    "job_get_failed",
                    "Failed to get job after cancel",
                    &e,
                )
                .with_context(serde_json::json!({
                    "operation": "jobs.cancel.get_after",
                    "job_id": job_id
                }))),
            }
        }
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "job_cancel_failed",
            "Failed to cancel job",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "jobs.cancel",
            "job_id": job_id
        }))),
    }
}

/// Retry a failed job by creating a new job with the same payload.
///
/// POST /v1/admin/jobs/:jobId/retry
pub async fn retry_job(
    _user: AdminUser,
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let Some(ref job_repo) = state.job_repository else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Job system not available",
        ));
    };

    // Get the original job
    let job = match job_repo.get(job_id).await {
        Ok(Some(job)) => job,
        Ok(None) => return Err(ApiError::not_found("not_found", "Job not found")),
        Err(e) => {
            return Err(crate::db_errors::internal_with_diagnostics(
                "job_get_failed",
                "Failed to get job",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "jobs.retry.get",
                "job_id": job_id
            })))
        }
    };

    // Only failed or cancelled jobs can be retried
    match job.status {
        JobStatus::Failed | JobStatus::Cancelled => {}
        _ => {
            return Err(ApiError::bad_request(
                "invalid_status",
                format!("Cannot retry job with status {}", job.status.as_str()),
            ))
        }
    }

    // Create a new job with the same payload and execution policy fields
    let config = underlay_jobs::JobConfig {
        max_attempts: job.max_attempts as u32,
        priority: job.priority,
        ..Default::default()
    };

    match job_repo
        .create_scheduled(
            &job.job_type,
            job.payload.clone(),
            &config,
            job.scheduled_for,
        )
        .await
    {
        Ok(new_job_id) => {
            // Return the new job
            match job_repo.get(new_job_id).await {
                Ok(Some(new_job)) => {
                    let dto = JobDetailDto::from_job(new_job);
                    Ok((StatusCode::CREATED, Json(SingleResponse { data: dto })).into_response())
                }
                Ok(None) => Err(ApiError::not_found(
                    "not_found",
                    "New job not found after create",
                )),
                Err(e) => Err(crate::db_errors::internal_with_diagnostics(
                    "job_get_failed",
                    "Failed to get new job",
                    &e,
                )
                .with_context(serde_json::json!({
                    "operation": "jobs.retry.get_new",
                    "job_id": new_job_id
                }))),
            }
        }
        Err(e) => Err(crate::db_errors::internal_with_diagnostics(
            "job_create_failed",
            "Failed to create retry job",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "jobs.retry.create",
            "job_id": job_id
        }))),
    }
}

/// Get job statistics for the dashboard.
///
/// GET /v1/admin/jobs/stats
pub async fn get_job_stats(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let Some(ref job_repo) = state.job_repository else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Job system not available",
        ));
    };

    // Get counts for each status
    let pending = job_repo
        .list(JobFilters {
            status: Some(JobStatus::Pending),
            limit: 1000,
            ..Default::default()
        })
        .await
        .map(|jobs| jobs.len() as i64)
        .unwrap_or(0);

    let running = job_repo
        .list(JobFilters {
            status: Some(JobStatus::Running),
            limit: 1000,
            ..Default::default()
        })
        .await
        .map(|jobs| jobs.len() as i64)
        .unwrap_or(0);

    let failed = job_repo
        .list(JobFilters {
            status: Some(JobStatus::Failed),
            limit: 1000,
            ..Default::default()
        })
        .await
        .map(|jobs| jobs.len() as i64)
        .unwrap_or(0);

    let succeeded_recent = job_repo
        .list(JobFilters {
            status: Some(JobStatus::Succeeded),
            limit: 100,
            ..Default::default()
        })
        .await
        .map(|jobs| jobs.len() as i64)
        .unwrap_or(0);

    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    struct JobStats {
        pending: i64,
        running: i64,
        failed: i64,
        succeeded_recent: i64,
    }

    Ok(Json(SingleResponse {
        data: JobStats {
            pending,
            running,
            failed,
            succeeded_recent,
        },
    })
    .into_response())
}
