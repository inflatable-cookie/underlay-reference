
use super::*;
use acme_auth::{UserId, UserPrincipal, UserRole};
use acme_test_utils::setup_test_db;
use serde_json::json;

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err() && std::env::var("TEST_DATABASE_URL").is_err()
}

fn admin_user() -> AdminUser {
    AdminUser(UserPrincipal {
        user_id: UserId(acme_core::Uuid::new_v7()),
        roles: vec![UserRole::Admin],
        email: Some("admin@example.com".to_string()),
        display_name: Some("Admin".to_string()),
    })
}

async fn insert_scheduled_task(pool: &sqlx::PgPool, enabled: bool) -> uuid::Uuid {
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
    )
    .bind(format!("test_scheduled_task_{}", uuid::Uuid::now_v7()))
    .bind("projects.generate_reports")
    .bind("0 0 * * * *")
    .bind(json!({ "source": "test" }))
    .bind(3_i32)
    .bind(Some(120_i32))
    .bind(false)
    .bind(0_i32)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .expect("should insert scheduled task")
}

async fn delete_scheduled_task(pool: &sqlx::PgPool, task_id: uuid::Uuid) {
    sqlx::query("DELETE FROM platform.scheduled_task WHERE id = $1")
        .bind(task_id)
        .execute(pool)
        .await
        .expect("should delete scheduled task");
}

async fn response_json(response: Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should read");
    serde_json::from_slice(&body).expect("response should be valid json")
}

#[tokio::test]
async fn list_scheduled_tasks_respects_enabled_filter() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let pool = db.pool();
    let _ = DB_POOL.set(db.pool_clone());

    let enabled_id = insert_scheduled_task(pool, true).await;
    let disabled_id = insert_scheduled_task(pool, false).await;

    let response = list_scheduled_tasks(
        admin_user(),
        Query(ListScheduledTasksQuery {
            enabled: Some(true),
            limit: Some(200),
            offset: Some(0),
        }),
    )
    .await
    .expect("list should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["data"].as_array().expect("data should be an array");

    let enabled_id_str = enabled_id.to_string();
    let disabled_id_str = disabled_id.to_string();

    let has_enabled = items.iter().any(|item| item["id"] == enabled_id_str);
    let has_disabled = items.iter().any(|item| item["id"] == disabled_id_str);

    assert!(has_enabled);
    assert!(!has_disabled);

    delete_scheduled_task(pool, enabled_id).await;
    delete_scheduled_task(pool, disabled_id).await;
}

#[tokio::test]
async fn toggle_scheduled_task_updates_enabled_state() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let pool = db.pool();
    let _ = DB_POOL.set(db.pool_clone());

    let task_id = insert_scheduled_task(pool, true).await;

    let response = toggle_scheduled_task(
        admin_user(),
        Path(task_id.to_string()),
        Json(ToggleScheduledTaskRequest { enabled: false }),
    )
    .await
    .expect("toggle should succeed");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let enabled: bool = sqlx::query_scalar(
        r#"
            SELECT enabled
            FROM platform.scheduled_task
            WHERE id = $1
            "#,
    )
    .bind(task_id)
    .fetch_one(pool)
    .await
    .expect("should fetch updated task");

    assert!(!enabled);

    delete_scheduled_task(pool, task_id).await;
}

#[tokio::test]
async fn get_scheduled_task_rejects_invalid_uuid() {
    let result = get_scheduled_task(admin_user(), Path("not-a-uuid".to_string())).await;

    let error = result.expect_err("invalid task id should fail");
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_scheduled_task_returns_inserted_task() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let pool = db.pool();
    let _ = DB_POOL.set(db.pool_clone());

    let task_id = insert_scheduled_task(pool, true).await;

    let response = get_scheduled_task(admin_user(), Path(task_id.to_string()))
        .await
        .expect("get should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["data"]["id"], task_id.to_string());

    delete_scheduled_task(pool, task_id).await;
}
