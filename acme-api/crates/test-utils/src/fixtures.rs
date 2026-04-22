//! Test fixtures and factory functions.
//!
//! Provides helpers for creating test data in the database.
//! All fixtures use UUIDv7 for predictable, time-ordered IDs.

use chrono::{DateTime, Utc};
use rand::RngExt;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Test Data Types
// ============================================================================

/// A test user created by fixtures.
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub status: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A test project created by fixtures.
#[derive(Debug, Clone)]
pub struct TestProject {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// A test category created by fixtures.
#[derive(Debug, Clone)]
pub struct TestCategory {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

/// A test task created by fixtures.
#[derive(Debug, Clone)]
pub struct TestTask {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// User Fixtures
// ============================================================================

/// Options for creating a test user.
#[derive(Debug, Default)]
pub struct CreateUserOptions {
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub display_name: Option<String>,
}

/// Create a test user with default values.
///
/// By default creates an active user with role "user".
/// Use `CreateUserOptions` to customize.
///
/// # Example
///
/// ```ignore
/// // Create default user
/// let user = create_test_user(&pool, Default::default()).await;
///
/// // Create admin user
/// let admin = create_test_user(&pool, CreateUserOptions {
///     role: Some("admin".to_string()),
///     ..Default::default()
/// }).await;
/// ```
pub async fn create_test_user(pool: &PgPool, opts: CreateUserOptions) -> TestUser {
    let id = Uuid::now_v7();
    let random_suffix: u32 = rand::rng().random_range(10000..99999);
    let email = opts
        .email
        .unwrap_or_else(|| format!("test-user-{}@example.com", random_suffix));
    let role = opts.role.unwrap_or_else(|| "user".to_string());
    let status = opts.status.unwrap_or_else(|| "active".to_string());
    let display_name = opts.display_name;

    // Create a password hash (we don't need a real one for tests)
    let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$test$testpasswordhash";

    let row = sqlx::query_as::<_, (Uuid, String, String, String, Option<String>, DateTime<Utc>)>(
        r#"
        INSERT INTO auth.users (id, email, password_hash, role, status, display_name)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, email, role, status, display_name, created_at
        "#,
    )
    .bind(id)
    .bind(&email)
    .bind(password_hash)
    .bind(&role)
    .bind(&status)
    .bind(&display_name)
    .fetch_one(pool)
    .await
    .expect("Failed to create test user");

    TestUser {
        id: row.0,
        email: row.1,
        role: row.2,
        status: row.3,
        display_name: row.4,
        created_at: row.5,
    }
}

/// Create a test admin user.
///
/// Convenience function that wraps `create_test_user` with admin role.
pub async fn create_test_admin(pool: &PgPool) -> TestUser {
    create_test_user(
        pool,
        CreateUserOptions {
            role: Some("admin".to_string()),
            ..Default::default()
        },
    )
    .await
}

// ============================================================================
// Category Fixtures
// ============================================================================

/// Options for creating a test category.
#[derive(Debug, Default)]
pub struct CreateCategoryOptions {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
}

/// Create a test category.
pub async fn create_test_category(pool: &PgPool, opts: CreateCategoryOptions) -> TestCategory {
    let id = Uuid::now_v7();
    let random_suffix: u32 = rand::rng().random_range(10000..99999);
    let name = opts
        .name
        .unwrap_or_else(|| format!("Test Category {}", random_suffix));
    let slug = opts
        .slug
        .unwrap_or_else(|| format!("test-category-{}", random_suffix));
    let description = opts.description;

    let row = sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
        r#"
        INSERT INTO acme.categories (id, name, slug, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, slug, created_at
        "#,
    )
    .bind(id)
    .bind(&name)
    .bind(&slug)
    .bind(&description)
    .fetch_one(pool)
    .await
    .expect("Failed to create test category");

    TestCategory {
        id: row.0,
        name: row.1,
        slug: row.2,
        created_at: row.3,
    }
}

// ============================================================================
// Project Fixtures
// ============================================================================

/// Options for creating a test project.
#[derive(Debug, Default)]
pub struct CreateProjectOptions {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub status: Option<String>,
}

/// Create a test project.
///
/// Requires an owner_id. Use `create_test_user` first to get a user ID.
///
/// # Example
///
/// ```ignore
/// let user = create_test_user(&pool, Default::default()).await;
/// let project = create_test_project(&pool, user.id, Default::default()).await;
/// ```
pub async fn create_test_project(
    pool: &PgPool,
    owner_id: Uuid,
    opts: CreateProjectOptions,
) -> TestProject {
    let id = Uuid::now_v7();
    let random_suffix: u32 = rand::rng().random_range(10000..99999);
    let name = opts
        .name
        .unwrap_or_else(|| format!("Test Project {}", random_suffix));
    let description = opts.description;
    let category_id = opts.category_id;
    let status = opts.status.unwrap_or_else(|| "active".to_string());

    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            Option<Uuid>,
            String,
            Option<String>,
            String,
            DateTime<Utc>,
        ),
    >(
        r#"
        INSERT INTO acme.projects (id, owner_id, category_id, name, description, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, owner_id, category_id, name, description, status, created_at
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(category_id)
    .bind(&name)
    .bind(&description)
    .bind(&status)
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    TestProject {
        id: row.0,
        owner_id: row.1,
        category_id: row.2,
        name: row.3,
        description: row.4,
        status: row.5,
        created_at: row.6,
    }
}

// ============================================================================
// Task Fixtures
// ============================================================================

/// Options for creating a test task.
#[derive(Debug, Default)]
pub struct CreateTaskOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

/// Create a test task.
///
/// Requires a project_id. Use `create_test_project` first.
///
/// # Example
///
/// ```ignore
/// let user = create_test_user(&pool, Default::default()).await;
/// let project = create_test_project(&pool, user.id, Default::default()).await;
/// let task = create_test_task(&pool, project.id, Default::default()).await;
/// ```
pub async fn create_test_task(
    pool: &PgPool,
    project_id: Uuid,
    opts: CreateTaskOptions,
) -> TestTask {
    let id = Uuid::now_v7();
    let random_suffix: u32 = rand::rng().random_range(10000..99999);
    let title = opts
        .title
        .unwrap_or_else(|| format!("Test Task {}", random_suffix));
    let description = opts.description;
    let status = opts.status.unwrap_or_else(|| "pending".to_string());
    let priority = opts.priority.unwrap_or_else(|| "medium".to_string());

    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Option<String>,
            String,
            String,
            DateTime<Utc>,
        ),
    >(
        r#"
        INSERT INTO acme.tasks (id, project_id, title, description, status, priority)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, project_id, title, description, status, priority, created_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(&title)
    .bind(&description)
    .bind(&status)
    .bind(&priority)
    .fetch_one(pool)
    .await
    .expect("Failed to create test task");

    TestTask {
        id: row.0,
        project_id: row.1,
        title: row.2,
        description: row.3,
        status: row.4,
        priority: row.5,
        created_at: row.6,
    }
}

// ============================================================================
// Batch Fixtures
// ============================================================================

/// Create multiple test tasks for a project.
///
/// Useful for testing pagination and bulk operations.
pub async fn create_test_tasks(pool: &PgPool, project_id: Uuid, count: usize) -> Vec<TestTask> {
    let mut tasks = Vec::with_capacity(count);
    for i in 0..count {
        let task = create_test_task(
            pool,
            project_id,
            CreateTaskOptions {
                title: Some(format!("Task #{}", i + 1)),
                ..Default::default()
            },
        )
        .await;
        tasks.push(task);
    }
    tasks
}

/// Create a project with tasks.
///
/// Convenience function that creates a user, project, and multiple tasks.
pub async fn create_project_with_tasks(
    pool: &PgPool,
    task_count: usize,
) -> (TestUser, TestProject, Vec<TestTask>) {
    let user = create_test_user(pool, Default::default()).await;
    let project = create_test_project(pool, user.id, Default::default()).await;
    let tasks = create_test_tasks(pool, project.id, task_count).await;
    (user, project, tasks)
}

#[cfg(test)]
#[path = "tests/fixtures_tests.rs"]
mod tests;
