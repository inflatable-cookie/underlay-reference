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
use sqlx::FromRow;
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

#[derive(Debug, FromRow)]
struct RetryPolicyRow {
    max_attempts: i32,
    timeout_seconds: Option<i32>,
    allow_overlap: bool,
    priority: i32,
}

fn retry_job_config(
    job: &Job,
    scheduled_policy: Option<&RetryPolicyRow>,
) -> underlay_jobs::JobConfig {
    if let Some(policy) = scheduled_policy {
        return underlay_jobs::JobConfig {
            max_attempts: policy.max_attempts as u32,
            timeout_seconds: policy.timeout_seconds.map(|s| s as u32),
            allow_overlap: policy.allow_overlap,
            priority: policy.priority,
            ..Default::default()
        };
    }

    underlay_jobs::JobConfig {
        max_attempts: job.max_attempts as u32,
        priority: job.priority,
        ..Default::default()
    }
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

    // Preserve execution policy metadata where possible.
    // For scheduled jobs, source timeout/overlap/priority from scheduled_task.
    let pool = state.local_auth.pool();
    let scheduled_policy: Option<RetryPolicyRow> = sqlx::query_as(
        r#"
        SELECT max_attempts, timeout_seconds, allow_overlap, priority
        FROM platform.scheduled_task
        WHERE job_type = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&job.job_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        crate::db_errors::internal_with_diagnostics(
            "job_retry_policy_fetch_failed",
            "Failed to load retry policy",
            &e,
        )
        .with_context(serde_json::json!({
            "operation": "jobs.retry.policy",
            "job_id": job_id,
            "job_type": job.job_type,
        }))
    })?;

    let config = retry_job_config(&job, scheduled_policy.as_ref());

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, sync::Arc};

    use acme_auth::{
        AcmeLocalAuthProvider, AcmeLocalAuthService, EmailTotpService, UserId, UserPrincipal,
        UserRole,
    };
    use acme_db::infra::DbEmailStore;
    use acme_test_utils::setup_test_db;
    use chrono::{Duration, Utc};
    use serde_json::json;
    use underlay_auth::AuthProvider;
    use underlay_blob::NoopAdapter;
    use underlay_jobs::{JobConfig, JobRepository};

    static TEST_ENV_ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();

    fn skip_without_db() -> bool {
        env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err()
    }

    fn ensure_test_env() {
        TEST_ENV_ONCE.get_or_init(|| {
            let (jwt_cfg, _) =
                underlay_auth_jwt::JwtConfig::generate().expect("should generate test JWT keys");

            env::set_var("AUTH_JWT_PRIVATE_KEY", jwt_cfg.private_key_b64);
            env::set_var("AUTH_JWT_PUBLIC_KEY", jwt_cfg.public_key_b64);
            env::set_var("ENVIRONMENT", "test");
            env::set_var("WEBAUTHN_RP_ID", "localhost");
            env::set_var("WEBAUTHN_RP_ORIGIN", "http://localhost:3000");
            env::set_var("WEBAUTHN_RP_NAME", "Acme Test");
        });
    }

    fn admin_user() -> AdminUser {
        AdminUser(UserPrincipal {
            user_id: UserId(acme_core::Uuid::new_v7()),
            roles: vec![UserRole::Admin],
            email: Some("admin@example.com".to_string()),
            display_name: Some("Admin".to_string()),
        })
    }

    async fn build_test_state(pool: sqlx::PgPool) -> AppState {
        ensure_test_env();

        let local_auth = Arc::new(
            AcmeLocalAuthService::from_env(pool.clone()).expect("should create local auth service"),
        );
        let auth_provider: Arc<dyn AuthProvider> =
            Arc::new(AcmeLocalAuthProvider::new(local_auth.clone()));

        let app_cfg = acme_infra::AppConfig::from_env();
        let email_manager = Arc::new(
            acme_infra::create_email_manager::<DbEmailStore>(&app_cfg.email, None)
                .expect("should create email manager"),
        );
        let email_templates = Arc::new(
            acme_infra::create_template_engine(&app_cfg.email)
                .expect("should create email template engine"),
        );
        let email_totp = Arc::new(EmailTotpService::new(
            pool.clone(),
            email_manager.clone(),
            email_templates.clone(),
            app_cfg.email.clone(),
        ));

        AppState {
            local_auth,
            auth_provider,
            cookie_config: underlay_http::AuthCookieConfig::default(),
            email_manager,
            email_templates,
            email_totp,
            email_config: app_cfg.email,
            blob_adapter: Arc::new(NoopAdapter::new()),
            job_repository: Some(Arc::new(JobRepository::new(pool))),
            config: crate::config::AcmeConfig::default(),
            trusted_proxy_config: acme_infra::TrustedProxyConfig::from_env(),
        }
    }

    async fn response_json(response: Response) -> serde_json::Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        serde_json::from_slice(&body).expect("response body should be valid json")
    }

    async fn mark_job_failed(pool: &sqlx::PgPool, job_id: Uuid) {
        sqlx::query(
            r#"
            UPDATE platform.job
            SET status = 'failed',
                finished_at = NOW(),
                last_error = 'forced test failure'
            WHERE id = $1
            "#,
        )
        .bind(job_id.into_inner())
        .execute(pool)
        .await
        .expect("should mark job as failed");
    }

    async fn delete_job(pool: &sqlx::PgPool, job_id: Uuid) {
        sqlx::query("DELETE FROM platform.job WHERE id = $1")
            .bind(job_id.into_inner())
            .execute(pool)
            .await
            .expect("should delete job");
    }

    async fn insert_scheduled_policy(pool: &sqlx::PgPool, job_type: &str) -> uuid::Uuid {
        sqlx::query_scalar(
            r#"
            INSERT INTO platform.scheduled_task (
                name,
                job_type,
                schedule,
                payload,
                max_attempts,
                timeout_seconds,
                allow_overlap,
                priority,
                enabled
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
            RETURNING id
            "#,
        )
        .bind(format!("test_job_policy_{}", uuid::Uuid::now_v7()))
        .bind(job_type)
        .bind("0 * * * * *")
        .bind(json!({ "source": "retry_test" }))
        .bind(9_i32)
        .bind(Some(180_i32))
        .bind(true)
        .bind(33_i32)
        .fetch_one(pool)
        .await
        .expect("should insert scheduled policy")
    }

    async fn delete_scheduled_policy(pool: &sqlx::PgPool, task_id: uuid::Uuid) {
        sqlx::query("DELETE FROM platform.scheduled_task WHERE id = $1")
            .bind(task_id)
            .execute(pool)
            .await
            .expect("should delete scheduled policy");
    }

    fn make_job(max_attempts: i32, priority: i32) -> Job {
        let now = Utc::now();
        Job {
            id: Uuid::new_v7(),
            job_type: "projects.generate_reports".to_string(),
            status: JobStatus::Failed,
            payload: serde_json::json!({}),
            attempts: 1,
            max_attempts,
            scheduled_for: None,
            priority,
            claimed_at: None,
            claimed_by: None,
            started_at: None,
            finished_at: None,
            heartbeat_at: None,
            progress: None,
            last_error: Some("boom".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn retry_job_config_uses_scheduled_policy_when_present() {
        let job = make_job(2, 1);
        let policy = RetryPolicyRow {
            max_attempts: 7,
            timeout_seconds: Some(900),
            allow_overlap: true,
            priority: 42,
        };

        let config = retry_job_config(&job, Some(&policy));

        assert_eq!(config.max_attempts, 7);
        assert_eq!(config.timeout_seconds, Some(900));
        assert!(config.allow_overlap);
        assert_eq!(config.priority, 42);
    }

    #[test]
    fn retry_job_config_falls_back_to_job_values_without_policy() {
        let job = make_job(4, 3);

        let config = retry_job_config(&job, None);

        assert_eq!(config.max_attempts, 4);
        assert_eq!(config.timeout_seconds, None);
        assert!(!config.allow_overlap);
        assert_eq!(config.priority, 3);
    }

    #[tokio::test]
    async fn list_jobs_filters_by_status() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool_clone();
        let state = build_test_state(pool.clone()).await;
        let repo = state.job_repository.clone().expect("job repo should exist");

        let job_config = JobConfig {
            max_attempts: 2,
            ..Default::default()
        };
        let pending_id = repo
            .create(
                "admin.jobs.test.pending",
                json!({ "kind": "pending" }),
                &job_config,
            )
            .await
            .expect("should create pending job");
        let failed_id = repo
            .create(
                "admin.jobs.test.failed",
                json!({ "kind": "failed" }),
                &job_config,
            )
            .await
            .expect("should create failed job");
        mark_job_failed(&pool, failed_id).await;

        let response = list_jobs(
            admin_user(),
            State(state),
            Query(ListJobsQuery {
                status: Some("pending".to_string()),
                job_type: None,
                limit: Some(200),
            }),
        )
        .await
        .expect("list jobs should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        let items = body["data"].as_array().expect("data should be array");

        let has_pending = items
            .iter()
            .any(|item| item["id"] == pending_id.to_string());
        let has_failed = items.iter().any(|item| item["id"] == failed_id.to_string());

        assert!(has_pending);
        assert!(!has_failed);

        delete_job(&pool, pending_id).await;
        delete_job(&pool, failed_id).await;
    }

    #[tokio::test]
    async fn get_job_returns_job_detail() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool_clone();
        let state = build_test_state(pool.clone()).await;
        let repo = state.job_repository.clone().expect("job repo should exist");

        let job_id = repo
            .create(
                "admin.jobs.test.get",
                json!({ "source": "get" }),
                &JobConfig::default(),
            )
            .await
            .expect("should create job");

        let response = get_job(admin_user(), State(state), Path(job_id))
            .await
            .expect("get job should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["data"]["id"], job_id.to_string());
        assert_eq!(body["data"]["job_type"], "admin.jobs.test.get");

        delete_job(&pool, job_id).await;
    }

    #[tokio::test]
    async fn cancel_job_updates_job_status() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool_clone();
        let state = build_test_state(pool.clone()).await;
        let repo = state.job_repository.clone().expect("job repo should exist");

        let job_id = repo
            .create(
                "admin.jobs.test.cancel",
                json!({ "source": "cancel" }),
                &JobConfig::default(),
            )
            .await
            .expect("should create job");

        let response = cancel_job(admin_user(), State(state.clone()), Path(job_id))
            .await
            .expect("cancel job should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["data"]["status"], "cancelled");

        let cancelled = state
            .job_repository
            .as_ref()
            .expect("job repo should exist")
            .get(job_id)
            .await
            .expect("job lookup should succeed")
            .expect("job should still exist");
        assert_eq!(cancelled.status, JobStatus::Cancelled);

        delete_job(&pool, job_id).await;
    }

    #[tokio::test]
    async fn retry_job_creates_new_job_with_scheduled_policy_fields() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool_clone();
        let state = build_test_state(pool.clone()).await;
        let repo = state.job_repository.clone().expect("job repo should exist");

        let scheduled_for = Utc::now() + Duration::hours(2);
        let original_id = repo
            .create_scheduled(
                "admin.jobs.test.retry",
                json!({ "source": "retry" }),
                &JobConfig {
                    max_attempts: 2,
                    priority: 1,
                    ..Default::default()
                },
                Some(scheduled_for),
            )
            .await
            .expect("should create scheduled job");
        mark_job_failed(&pool, original_id).await;

        let policy_id = insert_scheduled_policy(&pool, "admin.jobs.test.retry").await;

        let response = retry_job(admin_user(), State(state.clone()), Path(original_id))
            .await
            .expect("retry should succeed");

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response_json(response).await;

        let retried_id = Uuid::parse_str(
            body["data"]["id"]
                .as_str()
                .expect("retried id should be a string"),
        )
        .expect("retried id should parse");

        let retried = state
            .job_repository
            .as_ref()
            .expect("job repo should exist")
            .get(retried_id)
            .await
            .expect("retried job lookup should succeed")
            .expect("retried job should exist");

        assert_eq!(retried.max_attempts, 9);
        assert_eq!(retried.priority, 33);
        assert_eq!(retried.scheduled_for, Some(scheduled_for));

        delete_job(&pool, original_id).await;
        delete_job(&pool, retried_id).await;
        delete_scheduled_policy(&pool, policy_id).await;
    }

    #[tokio::test]
    async fn get_job_stats_returns_numeric_fields() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let pool = db.pool_clone();
        let state = build_test_state(pool.clone()).await;
        let repo = state.job_repository.clone().expect("job repo should exist");

        let pending_id = repo
            .create(
                "admin.jobs.test.stats.pending",
                json!({ "source": "stats" }),
                &JobConfig::default(),
            )
            .await
            .expect("should create pending job");
        let failed_id = repo
            .create(
                "admin.jobs.test.stats.failed",
                json!({ "source": "stats" }),
                &JobConfig::default(),
            )
            .await
            .expect("should create failed job");
        mark_job_failed(&pool, failed_id).await;

        let response = get_job_stats(admin_user(), State(state))
            .await
            .expect("job stats should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;

        assert!(body["data"]["pending"].is_number());
        assert!(body["data"]["running"].is_number());
        assert!(body["data"]["failed"].is_number());
        assert!(body["data"]["succeeded_recent"].is_number());

        delete_job(&pool, pending_id).await;
        delete_job(&pool, failed_id).await;
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
