//! Test data cleanup utilities.
//!
//! Helpers for cleaning up test data after tests complete.
//! Use these to maintain database hygiene in shared test environments.

use sqlx::PgPool;
use uuid::Uuid;

/// Delete a test user and all associated data.
///
/// This cascades to delete:
/// - User's sessions
/// - User's projects
/// - Tasks in user's projects
///
/// # Example
///
/// ```ignore
/// use acme_test_utils::cleanup;
///
/// // ... after test ...
/// cleanup::delete_user(&pool, user.id).await;
/// ```
pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    // Delete tasks in user's projects first
    sqlx::query(
        r#"
        DELETE FROM acme.tasks
        WHERE project_id IN (
            SELECT id FROM acme.projects WHERE owner_id = $1
        )
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Delete user's projects
    sqlx::query(
        r#"
        DELETE FROM acme.projects WHERE owner_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Delete user's sessions
    sqlx::query(
        r#"
        DELETE FROM auth.sessions WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Delete user
    sqlx::query(
        r#"
        DELETE FROM auth.users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a test project and all associated data.
///
/// This cascades to delete:
/// - Project's tasks
/// - Task labels
///
/// # Example
///
/// ```ignore
/// use acme_test_utils::cleanup;
///
/// cleanup::delete_project(&pool, project.id).await;
/// ```
pub async fn delete_project(pool: &PgPool, project_id: Uuid) -> Result<(), sqlx::Error> {
    // Delete tasks first
    sqlx::query(
        r#"
        DELETE FROM acme.tasks WHERE project_id = $1
        "#,
    )
    .bind(project_id)
    .execute(pool)
    .await?;

    // Delete project
    sqlx::query(
        r#"
        DELETE FROM acme.projects WHERE id = $1
        "#,
    )
    .bind(project_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a test task.
pub async fn delete_task(pool: &PgPool, task_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM acme.tasks WHERE id = $1
        "#,
    )
    .bind(task_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a test category.
///
/// Note: This will fail if projects still reference the category.
/// Either delete projects first or set their category_id to NULL.
pub async fn delete_category(pool: &PgPool, category_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM acme.categories WHERE id = $1
        "#,
    )
    .bind(category_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete all test data created with a specific email prefix.
///
/// Useful for cleaning up after parallel tests that use unique email prefixes.
///
/// # Example
///
/// ```ignore
/// // In test setup, use unique prefix
/// let prefix = format!("test-{}-", uuid::Uuid::new_v4());
/// let user = create_test_user(&pool, CreateUserOptions {
///     email: Some(format!("{}user@example.com", prefix)),
///     ..Default::default()
/// }).await;
///
/// // ... run test ...
///
/// // Clean up by prefix
/// cleanup::delete_users_by_email_prefix(&pool, &prefix).await;
/// ```
pub async fn delete_users_by_email_prefix(pool: &PgPool, prefix: &str) -> Result<u64, sqlx::Error> {
    // Get user IDs first
    let user_ids: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM auth.users WHERE email LIKE $1 || '%'
        "#,
    )
    .bind(prefix)
    .fetch_all(pool)
    .await?;

    let mut deleted = 0;
    for (user_id,) in user_ids {
        delete_user(pool, user_id).await?;
        deleted += 1;
    }

    Ok(deleted)
}

/// Clean up all test data (use with caution!).
///
/// This deletes all data that looks like test data:
/// - Users with "test-" prefix in email
/// - Categories with "test-" prefix in slug
///
/// **Warning**: Only use this in isolated test databases!
pub async fn cleanup_all_test_data(pool: &PgPool) -> Result<CleanupStats, sqlx::Error> {
    // Clean up test users
    let users = delete_users_by_email_prefix(pool, "test-").await?;

    // Clean up test categories
    let result = sqlx::query(
        r#"
        DELETE FROM acme.categories WHERE slug LIKE 'test-%'
        "#,
    )
    .execute(pool)
    .await?;

    Ok(CleanupStats {
        users,
        categories: result.rows_affected(),
        ..Default::default()
    })
}

/// Statistics from cleanup operations.
#[derive(Debug, Default)]
pub struct CleanupStats {
    pub users: u64,
    pub categories: u64,
    pub projects: u64,
    pub tasks: u64,
}

#[cfg(test)]
mod tests {
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
}
