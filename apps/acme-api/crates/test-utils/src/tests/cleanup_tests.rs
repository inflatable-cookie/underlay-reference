use super::*;
use crate::db::setup_test_db;
use crate::fixtures::{create_test_project, create_test_task, create_test_user};
use std::env;

fn skip_without_db() -> bool {
    env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err()
}

#[tokio::test]
async fn test_delete_user_cascade() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;

    // Create user, project, and task
    let user = create_test_user(db.pool(), Default::default()).await;
    let project = create_test_project(db.pool(), user.id, Default::default()).await;
    let _task = create_test_task(db.pool(), project.id, Default::default()).await;

    // Delete user (should cascade)
    delete_user(db.pool(), user.id)
        .await
        .expect("delete should succeed");

    // Verify user is gone
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM auth.users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(db.pool())
        .await
        .expect("query should succeed");
    assert!(row.is_none(), "user should be deleted");

    // Verify project is gone
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM acme.projects WHERE id = $1")
        .bind(project.id)
        .fetch_optional(db.pool())
        .await
        .expect("query should succeed");
    assert!(row.is_none(), "project should be deleted");
}
