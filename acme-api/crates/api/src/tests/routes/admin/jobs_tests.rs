use super::*;
use std::{env, sync::Arc};

use acme_auth::{
    AcmeLocalAuthProvider, AcmeLocalAuthService, EmailTotpService, UserId, UserPrincipal, UserRole,
};
use acme_test_utils::setup_test_db;
use chrono::{Duration, Utc};
use serde_json::json;
use underlay_auth::AuthProvider;
use underlay_blob::NoopAdapter;
use underlay_jobs::JobConfig;
use underlay_jobs_postgres::JobRepository;

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
        acme_infra::create_email_manager(&app_cfg.email).expect("should create email manager"),
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
    assert!(body["data"]["succeeded"].is_number());

    delete_job(&pool, pending_id).await;
    delete_job(&pool, failed_id).await;
}
