use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use underlay_http::query::{FieldMapping, QueryParams, WhereBuilder};
use uuid::Uuid;

use crate::DbPool;

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
        .sort_only("due_date", "t.due_date")
        .sort_only("created_at", "t.created_at")
        .filter_only("project_id", "t.project_id")
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
    project_id: Uuid,
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
            title = COALESCE($3, title),
            description = CASE WHEN $4 THEN $5 ELSE description END,
            status = COALESCE($6, status),
            priority = COALESCE($7, priority),
            due_date = CASE WHEN $8 THEN $9 ELSE due_date END,
            completed_at = {},
            updated_at = NOW()
        WHERE id = $1 AND project_id = $2 AND deleted_at IS NULL
        RETURNING id, project_id, title, description, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
        "#,
        completed_at_expr
    );

    sqlx::query_as::<_, TaskRow>(&query)
        .bind(id)
        .bind(project_id)
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
pub async fn delete_task(pool: &DbPool, id: Uuid, project_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM acme.tasks WHERE id = $1 AND project_id = $2")
        .bind(id)
        .bind(project_id)
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

/// Batch soft delete tasks.
///
/// Returns the number of tasks deleted.
pub async fn batch_soft_delete_tasks(
    pool: &DbPool,
    ids: &[Uuid],
    batch_id: Uuid,
) -> Result<u64, sqlx::Error> {
    if ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        UPDATE acme.tasks
        SET deleted_at = NOW(), delete_batch_id = $1, updated_at = NOW()
        WHERE id = ANY($2) AND deleted_at IS NULL
        "#,
    )
    .bind(batch_id)
    .bind(ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Batch update task status.
///
/// Returns the number of tasks updated.
pub async fn batch_update_task_status(
    pool: &DbPool,
    ids: &[Uuid],
    status: &str,
) -> Result<u64, sqlx::Error> {
    if ids.is_empty() {
        return Ok(0);
    }

    // Handle completed_at based on status
    let completed_at_expr = match status {
        "completed" => "NOW()",
        _ => "NULL",
    };

    let query = format!(
        r#"
        UPDATE acme.tasks
        SET status = $1, completed_at = {}, updated_at = NOW()
        WHERE id = ANY($2) AND deleted_at IS NULL
        "#,
        completed_at_expr
    );

    let result = sqlx::query(&query)
        .bind(status)
        .bind(ids)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
