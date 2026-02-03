//! Task and project database operations.
//!
//! Example domain queries demonstrating common patterns including:
//! - Filtering and sorting via QueryParams
//! - Soft delete with batch IDs
//! - Relations (projects → tasks, tasks → labels)
//! - Admin queries with counts

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use underlay_http::query::{FieldMapping, QueryParams, WhereBuilder};
use uuid::Uuid;

use crate::DbPool;

// ============================================================================
// Projects
// ============================================================================

/// Row type for acme.projects table.
#[derive(Debug, Clone, FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Project with task counts (for admin list views).
#[derive(Debug, Clone, FromRow)]
pub struct ProjectWithCountsRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub task_count: i64,
    pub completed_task_count: i64,
}

/// Get field mapping for project queries.
pub fn project_field_mapping() -> FieldMapping {
    FieldMapping::new()
        .map("name", "p.name")
        .map("status", "p.status")
        .sort_only("weight", "p.weight")
        .sort_only("createdAt", "p.created_at")
        .sort_only("updatedAt", "p.updated_at")
        .sort_only("categoryName", "c.name")
        .filter_only("categoryId", "p.category_id")
        .filter_only("ownerId", "p.owner_id")
}

/// Create a new project.
pub async fn create_project(
    pool: &DbPool,
    id: Uuid,
    owner_id: Uuid,
    name: &str,
    description: Option<&str>,
    category_id: Option<Uuid>,
) -> Result<ProjectRow, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        INSERT INTO acme.projects (id, owner_id, name, description, category_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, owner_id, category_id, name, description, status, weight, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(name)
    .bind(description)
    .bind(category_id)
    .fetch_one(pool)
    .await
}

/// Get a project by ID.
pub async fn get_project(pool: &DbPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, owner_id, category_id, name, description, status, weight, created_at, updated_at, deleted_at
        FROM acme.projects
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Get a project by ID (admin view - includes deleted).
pub async fn get_project_admin(pool: &DbPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, owner_id, category_id, name, description, status, weight, created_at, updated_at, deleted_at
        FROM acme.projects
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// List projects for a user (non-admin, active only).
pub async fn list_projects_for_user(
    pool: &DbPool,
    owner_id: Uuid,
    include_archived: bool,
) -> Result<Vec<ProjectRow>, sqlx::Error> {
    if include_archived {
        sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, owner_id, category_id, name, description, status, weight, created_at, updated_at, deleted_at
            FROM acme.projects
            WHERE owner_id = $1 AND deleted_at IS NULL
            ORDER BY weight, created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, owner_id, category_id, name, description, status, weight, created_at, updated_at, deleted_at
            FROM acme.projects
            WHERE owner_id = $1 AND status = 'active' AND deleted_at IS NULL
            ORDER BY weight, created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    }
}

/// List projects with filtering and sorting (admin).
pub async fn list_projects_admin(
    pool: &DbPool,
    query: &QueryParams,
) -> Result<Vec<ProjectWithCountsRow>, sqlx::Error> {
    let mapping = project_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    where_builder.add_condition("p.deleted_at IS NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "p.weight, p.name");

    let sql = format!(
        r#"
        SELECT
            p.id, p.owner_id, p.category_id, c.name as category_name,
            p.name, p.description, p.status, p.weight,
            p.created_at, p.updated_at, p.deleted_at,
            COALESCE(COUNT(t.id) FILTER (WHERE t.deleted_at IS NULL), 0) as task_count,
            COALESCE(COUNT(t.id) FILTER (WHERE t.deleted_at IS NULL AND t.status = 'completed'), 0) as completed_task_count
        FROM acme.projects p
        LEFT JOIN acme.categories c ON c.id = p.category_id
        LEFT JOIN acme.tasks t ON t.project_id = p.id
        WHERE {}
        GROUP BY p.id, c.name
        ORDER BY {}
        "#,
        where_clause, order_by
    );

    let mut query_builder = sqlx::query_as::<_, ProjectWithCountsRow>(&sql);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }

    query_builder.fetch_all(pool).await
}

/// Update a project.
pub async fn update_project(
    pool: &DbPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<Option<&str>>,
    status: Option<&str>,
    category_id: Option<Option<Uuid>>,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        UPDATE acme.projects
        SET
            name = COALESCE($2, name),
            description = CASE WHEN $3 THEN $4 ELSE description END,
            status = COALESCE($5, status),
            category_id = CASE WHEN $6 THEN $7 ELSE category_id END,
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, owner_id, category_id, name, description, status, weight, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description.is_some())
    .bind(description.flatten())
    .bind(status)
    .bind(category_id.is_some())
    .bind(category_id.flatten())
    .fetch_optional(pool)
    .await
}

/// Soft delete a project.
pub async fn soft_delete_project(
    pool: &DbPool,
    id: Uuid,
    batch_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE acme.projects
        SET deleted_at = NOW(), delete_batch_id = $2, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(batch_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Hard delete a project (use with caution).
pub async fn delete_project(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM acme.projects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Restore a soft-deleted project.
pub async fn restore_project(pool: &DbPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        UPDATE acme.projects
        SET deleted_at = NULL, delete_batch_id = NULL, updated_at = NOW()
        WHERE id = $1
        RETURNING id, owner_id, category_id, name, description, status, weight, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Reorder projects by weight.
pub async fn reorder_projects(pool: &DbPool, project_ids: &[Uuid]) -> Result<(), sqlx::Error> {
    for (weight, project_id) in project_ids.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE acme.projects
            SET weight = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(weight as i32)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ============================================================================
// Tasks
// ============================================================================

/// Row type for acme.tasks table.
#[derive(Debug, Clone, FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub position: i32,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Task with label info (for list views).
#[derive(Debug, Clone, FromRow)]
pub struct TaskWithLabelsRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub position: i32,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub label_count: i64,
}

/// Get field mapping for task queries.
pub fn task_field_mapping() -> FieldMapping {
    FieldMapping::new()
        .map("title", "t.title")
        .map("status", "t.status")
        .map("priority", "t.priority")
        .sort_only("position", "t.position")
        .sort_only("dueDate", "t.due_date")
        .sort_only("createdAt", "t.created_at")
        .filter_only("projectId", "t.project_id")
}

/// Create a new task.
pub async fn create_task(
    pool: &DbPool,
    id: Uuid,
    project_id: Uuid,
    title: &str,
    description: Option<&str>,
    priority: &str,
    due_date: Option<NaiveDate>,
) -> Result<TaskRow, sqlx::Error> {
    // Get the next position
    let next_position: i32 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(MAX(position), -1) + 1
        FROM acme.tasks
        WHERE project_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    sqlx::query_as::<_, TaskRow>(
        r#"
        INSERT INTO acme.tasks (id, project_id, title, description, priority, due_date, position)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, project_id, title, description, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(title)
    .bind(description)
    .bind(priority)
    .bind(due_date)
    .bind(next_position)
    .fetch_one(pool)
    .await
}

/// Get a task by ID.
pub async fn get_task(pool: &DbPool, id: Uuid) -> Result<Option<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        r#"
        SELECT id, project_id, title, description, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
        FROM acme.tasks
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// List tasks for a project (non-admin).
pub async fn list_tasks_for_project(
    pool: &DbPool,
    project_id: Uuid,
    include_completed: bool,
) -> Result<Vec<TaskRow>, sqlx::Error> {
    if include_completed {
        sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, project_id, title, description, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
            FROM acme.tasks
            WHERE project_id = $1 AND deleted_at IS NULL
            ORDER BY position
            "#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, project_id, title, description, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
            FROM acme.tasks
            WHERE project_id = $1 AND status NOT IN ('completed', 'cancelled') AND deleted_at IS NULL
            ORDER BY position
            "#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }
}

/// List tasks with filtering and sorting (admin).
pub async fn list_tasks_admin(
    pool: &DbPool,
    project_id: Uuid,
    query: &QueryParams,
) -> Result<Vec<TaskWithLabelsRow>, sqlx::Error> {
    let mapping = task_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(2); // $1 is project_id
    where_builder.add_condition("t.project_id = $1");
    where_builder.add_condition("t.deleted_at IS NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "t.position, t.created_at");

    let sql = format!(
        r#"
        SELECT
            t.id, t.project_id, t.title, t.description, t.status, t.priority,
            t.due_date, t.completed_at, t.position, t.weight,
            t.created_at, t.updated_at, t.deleted_at,
            COALESCE(COUNT(tl.label_id), 0) as label_count
        FROM acme.tasks t
        LEFT JOIN acme.task_labels tl ON tl.task_id = t.id
        WHERE {}
        GROUP BY t.id
        ORDER BY {}
        "#,
        where_clause, order_by
    );

    let mut query_builder = sqlx::query_as::<_, TaskWithLabelsRow>(&sql);
    query_builder = query_builder.bind(project_id);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }

    query_builder.fetch_all(pool).await
}

/// Update a task.
pub async fn update_task(
    pool: &DbPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<Option<&str>>,
    status: Option<&str>,
    priority: Option<&str>,
    due_date: Option<Option<NaiveDate>>,
) -> Result<Option<TaskRow>, sqlx::Error> {
    // Handle completed_at based on status change
    let completed_at_expr = match status {
        Some("completed") => "NOW()",
        Some(_) => "NULL",
        None => "completed_at",
    };

    let query = format!(
        r#"
        UPDATE acme.tasks
        SET
            title = COALESCE($2, title),
            description = CASE WHEN $3 THEN $4 ELSE description END,
            status = COALESCE($5, status),
            priority = COALESCE($6, priority),
            due_date = CASE WHEN $7 THEN $8 ELSE due_date END,
            completed_at = {},
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, project_id, title, description, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
        "#,
        completed_at_expr
    );

    sqlx::query_as::<_, TaskRow>(&query)
        .bind(id)
        .bind(title)
        .bind(description.is_some())
        .bind(description.flatten())
        .bind(status)
        .bind(priority)
        .bind(due_date.is_some())
        .bind(due_date.flatten())
        .fetch_optional(pool)
        .await
}

/// Soft delete a task.
pub async fn soft_delete_task(
    pool: &DbPool,
    id: Uuid,
    batch_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE acme.tasks
        SET deleted_at = NOW(), delete_batch_id = $2, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(batch_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Hard delete a task.
pub async fn delete_task(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM acme.tasks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Reorder tasks within a project.
pub async fn reorder_tasks(
    pool: &DbPool,
    project_id: Uuid,
    task_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    for (position, task_id) in task_ids.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE acme.tasks
            SET position = $3, updated_at = NOW()
            WHERE id = $1 AND project_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(task_id)
        .bind(project_id)
        .bind(position as i32)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ============================================================================
// Labels
// ============================================================================

/// Row type for acme.labels table.
#[derive(Debug, Clone, FromRow)]
pub struct LabelRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Create a label for a project.
pub async fn create_label(
    pool: &DbPool,
    id: Uuid,
    project_id: Uuid,
    name: &str,
    color: &str,
) -> Result<LabelRow, sqlx::Error> {
    sqlx::query_as::<_, LabelRow>(
        r#"
        INSERT INTO acme.labels (id, project_id, name, color)
        VALUES ($1, $2, $3, $4)
        RETURNING id, project_id, name, color, weight, created_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(name)
    .bind(color)
    .fetch_one(pool)
    .await
}

/// List labels for a project.
pub async fn list_labels_for_project(
    pool: &DbPool,
    project_id: Uuid,
) -> Result<Vec<LabelRow>, sqlx::Error> {
    sqlx::query_as::<_, LabelRow>(
        r#"
        SELECT id, project_id, name, color, weight, created_at, deleted_at
        FROM acme.labels
        WHERE project_id = $1 AND deleted_at IS NULL
        ORDER BY weight, name
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Get a label by ID.
pub async fn get_label(pool: &DbPool, id: Uuid) -> Result<Option<LabelRow>, sqlx::Error> {
    sqlx::query_as::<_, LabelRow>(
        r#"
        SELECT id, project_id, name, color, weight, created_at, deleted_at
        FROM acme.labels
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Assign a label to a task.
pub async fn assign_label_to_task(
    pool: &DbPool,
    task_id: Uuid,
    label_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO acme.task_labels (task_id, label_id)
        VALUES ($1, $2)
        ON CONFLICT (task_id, label_id) DO NOTHING
        "#,
    )
    .bind(task_id)
    .bind(label_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a label from a task.
pub async fn remove_label_from_task(
    pool: &DbPool,
    task_id: Uuid,
    label_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM acme.task_labels
        WHERE task_id = $1 AND label_id = $2
        "#,
    )
    .bind(task_id)
    .bind(label_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Get labels for a task.
pub async fn get_labels_for_task(
    pool: &DbPool,
    task_id: Uuid,
) -> Result<Vec<LabelRow>, sqlx::Error> {
    sqlx::query_as::<_, LabelRow>(
        r#"
        SELECT l.id, l.project_id, l.name, l.color, l.weight, l.created_at, l.deleted_at
        FROM acme.labels l
        INNER JOIN acme.task_labels tl ON tl.label_id = l.id
        WHERE tl.task_id = $1 AND l.deleted_at IS NULL
        ORDER BY l.weight, l.name
        "#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await
}

/// Set labels for a task (replaces all existing).
pub async fn set_task_labels(
    pool: &DbPool,
    task_id: Uuid,
    label_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    // Remove all existing labels
    sqlx::query("DELETE FROM acme.task_labels WHERE task_id = $1")
        .bind(task_id)
        .execute(pool)
        .await?;

    // Add new labels
    for label_id in label_ids {
        sqlx::query(
            r#"
            INSERT INTO acme.task_labels (task_id, label_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(task_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ============================================================================
// Task Comments
// ============================================================================

/// Row type for acme.task_comments table.
#[derive(Debug, Clone, FromRow)]
pub struct TaskCommentRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create a comment on a task.
pub async fn create_task_comment(
    pool: &DbPool,
    id: Uuid,
    task_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> Result<TaskCommentRow, sqlx::Error> {
    sqlx::query_as::<_, TaskCommentRow>(
        r#"
        INSERT INTO acme.task_comments (id, task_id, author_id, body)
        VALUES ($1, $2, $3, $4)
        RETURNING id, task_id, author_id, body, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(task_id)
    .bind(author_id)
    .bind(body)
    .fetch_one(pool)
    .await
}

/// List comments for a task.
pub async fn list_task_comments(
    pool: &DbPool,
    task_id: Uuid,
) -> Result<Vec<TaskCommentRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskCommentRow>(
        r#"
        SELECT id, task_id, author_id, body, created_at, updated_at
        FROM acme.task_comments
        WHERE task_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await
}

// ============================================================================
// Validation
// ============================================================================

/// Check if a label name is unique within a project.
pub async fn is_label_name_available(
    pool: &DbPool,
    project_id: Uuid,
    name: &str,
    exclude_id: Option<Uuid>,
) -> Result<bool, sqlx::Error> {
    let result: (i64,) = if let Some(id) = exclude_id {
        sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM acme.labels
            WHERE project_id = $1 AND LOWER(name) = LOWER($2) AND id != $3 AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(name)
        .bind(id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM acme.labels
            WHERE project_id = $1 AND LOWER(name) = LOWER($2) AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(name)
        .fetch_one(pool)
        .await?
    };

    Ok(result.0 == 0)
}
