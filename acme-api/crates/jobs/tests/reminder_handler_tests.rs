use std::{env, sync::Arc};

use acme_jobs::{CheckDueRemindersHandler, Job, JobConfig, JobHandler, JobRepository, JobStatus};
use acme_test_utils::{
    cleanup,
    fixtures::{create_test_project, create_test_task, create_test_user},
    setup_test_db,
};
use chrono::{Duration, Utc};
use serde_json::json;

/// Serializes the two handler tests: `check_due_reminders` scans every due
/// task, so concurrent runs enqueue reminders for each other's fixtures and
/// break the count assertions.
static HANDLER_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn skip_without_db() -> bool {
    env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err()
}

fn make_job(job_type: &str, payload: serde_json::Value) -> Job {
    let now = Utc::now();
    Job {
        id: acme_jobs::JobId::new_v7(),
        job_type: job_type.to_string(),
        status: JobStatus::Pending,
        payload,
        attempts: 0,
        max_attempts: 1,
        scheduled_for: None,
        priority: 0,
        claimed_at: None,
        claimed_by: None,
        started_at: None,
        finished_at: None,
        heartbeat_at: None,
        progress: None,
        last_error: None,
        created_at: now,
        updated_at: now,
    }
}

async fn set_task_due_tomorrow(pool: &sqlx::PgPool, task_id: uuid::Uuid) {
    let due_date = Utc::now().date_naive() + Duration::days(1);

    sqlx::query(
        r#"
        UPDATE acme.tasks
        SET due_date = $2
        WHERE id = $1
        "#,
    )
    .bind(task_id)
    .bind(due_date)
    .execute(pool)
    .await
    .expect("should set due date");
}

async fn assign_task_to_user(pool: &sqlx::PgPool, task_id: uuid::Uuid, user_id: uuid::Uuid) {
    sqlx::query(
        r#"
        INSERT INTO acme.task_assignees (task_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (task_id, user_id) DO NOTHING
        "#,
    )
    .bind(task_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("should assign task");
}

async fn delete_reminder_jobs(pool: &sqlx::PgPool, task_id: uuid::Uuid, user_email: &str) {
    sqlx::query(
        r#"
        DELETE FROM platform.job
        WHERE job_type = 'tasks.send_reminder'
          AND payload->>'task_id' = $1
          AND payload->>'user_email' = $2
        "#,
    )
    .bind(task_id.to_string())
    .bind(user_email)
    .execute(pool)
    .await
    .expect("should cleanup reminder jobs");
}

#[tokio::test]
async fn check_due_reminders_skips_existing_reminder_jobs() {
    let _guard = HANDLER_TEST_LOCK.lock().await;
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let pool = db.pool();

    let user = create_test_user(pool, Default::default()).await;
    let project = create_test_project(pool, user.id, Default::default()).await;
    let task = create_test_task(pool, project.id, Default::default()).await;

    set_task_due_tomorrow(pool, task.id).await;
    assign_task_to_user(pool, task.id, user.id).await;
    delete_reminder_jobs(pool, task.id, &user.email).await;

    let job_repo = JobRepository::new(db.pool_clone());
    let config = JobConfig::new().with_max_attempts(5);

    job_repo
        .create(
            "tasks.send_reminder",
            json!({ "task_id": task.id, "user_email": user.email }),
            &config,
        )
        .await
        .expect("should create seed reminder job");

    let handler = CheckDueRemindersHandler::new(Arc::new(db.pool_clone()));
    let job = make_job("tasks.check_due_reminders", json!({ "days_ahead": 1 }));

    handler
        .handle(job)
        .await
        .expect("check due reminders should succeed");

    let reminders_for_task: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM platform.job
        WHERE job_type = 'tasks.send_reminder'
          AND payload->>'task_id' = $1
          AND payload->>'user_email' = $2
        "#,
    )
    .bind(task.id.to_string())
    .bind(&user.email)
    .fetch_one(pool)
    .await
    .expect("should count reminder jobs");

    assert_eq!(reminders_for_task, 1);

    delete_reminder_jobs(pool, task.id, &user.email).await;
    cleanup::delete_user(pool, user.id)
        .await
        .expect("cleanup should succeed");
}

#[tokio::test]
async fn check_due_reminders_enqueues_with_expected_retry_attempts() {
    let _guard = HANDLER_TEST_LOCK.lock().await;
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let pool = db.pool();

    let user = create_test_user(pool, Default::default()).await;
    let project = create_test_project(pool, user.id, Default::default()).await;
    let task = create_test_task(pool, project.id, Default::default()).await;

    set_task_due_tomorrow(pool, task.id).await;
    assign_task_to_user(pool, task.id, user.id).await;
    delete_reminder_jobs(pool, task.id, &user.email).await;

    let handler = CheckDueRemindersHandler::new(Arc::new(db.pool_clone()));
    let job = make_job("tasks.check_due_reminders", json!({ "days_ahead": 1 }));

    handler
        .handle(job)
        .await
        .expect("check due reminders should succeed");

    let reminder_attempts: Vec<i32> = sqlx::query_scalar(
        r#"
        SELECT max_attempts
        FROM platform.job
        WHERE job_type = 'tasks.send_reminder'
          AND payload->>'task_id' = $1
          AND payload->>'user_email' = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(task.id.to_string())
    .bind(&user.email)
    .fetch_all(pool)
    .await
    .expect("should load reminder job attempts");

    assert_eq!(reminder_attempts.len(), 1);
    assert_eq!(reminder_attempts[0], 5);

    delete_reminder_jobs(pool, task.id, &user.email).await;
    cleanup::delete_user(pool, user.id)
        .await
        .expect("cleanup should succeed");
}
