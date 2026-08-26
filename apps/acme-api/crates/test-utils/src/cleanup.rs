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
#[path = "tests/cleanup_tests.rs"]
mod tests;
