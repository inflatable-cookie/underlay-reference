use super::*;
use crate::db::setup_test_db;
use std::env;

fn skip_without_db() -> bool {
    env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err()
}

#[tokio::test]
async fn test_create_user_fixture() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let user = create_test_user(db.pool(), Default::default()).await;

    assert!(!user.email.is_empty());
    assert_eq!(user.role, "user");
    assert_eq!(user.status, "active");
}

#[tokio::test]
async fn test_create_admin_fixture() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let admin = create_test_admin(db.pool()).await;

    assert_eq!(admin.role, "admin");
}

#[tokio::test]
async fn test_create_project_fixture() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let user = create_test_user(db.pool(), Default::default()).await;
    let project = create_test_project(db.pool(), user.id, Default::default()).await;

    assert_eq!(project.owner_id, user.id);
    assert_eq!(project.status, "active");
}

#[tokio::test]
async fn test_create_task_fixture() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let user = create_test_user(db.pool(), Default::default()).await;
    let project = create_test_project(db.pool(), user.id, Default::default()).await;
    let task = create_test_task(db.pool(), project.id, Default::default()).await;

    assert_eq!(task.project_id, project.id);
    assert_eq!(task.status, "pending");
    assert_eq!(task.priority, "medium");
}
