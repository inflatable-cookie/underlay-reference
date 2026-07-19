use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;
use sqlx::FromRow;
use std::collections::HashSet;
use underlay_http::query::QueryParams;
use underlay_query::{FieldMapping, WhereBuilder};
use uuid::Uuid;

use crate::DbPool;

#[derive(Debug)]
pub struct TaskListResponse {
    pub data: Vec<TaskWithLabelsRow>,
    pub total: i64,
    pub has_more: bool,
}

/// Row type for acme.tasks table.
#[derive(Debug, Clone, FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub notes: Option<Value>,
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
    pub notes: Option<Value>,
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
        .filter_only("project_id", "t.project_id::text")
}

/// Create a new task.
#[allow(clippy::too_many_arguments)]
pub async fn create_task(
    pool: &DbPool,
    id: Uuid,
    project_id: Uuid,
    title: &str,
    description: Option<&str>,
    notes: Option<&Value>,
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
        INSERT INTO acme.tasks (id, project_id, title, description, notes, priority, due_date, position)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, project_id, title, description, notes, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(title)
    .bind(description)
    .bind(notes)
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
        SELECT id, project_id, title, description, notes, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
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
            SELECT id, project_id, title, description, notes, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
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
            SELECT id, project_id, title, description, notes, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
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
    variant: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<TaskListResponse, sqlx::Error> {
    let mapping = task_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(2); // $1 is project_id
    where_builder.add_condition("t.project_id = $1");
    where_builder.add_condition("t.deleted_at IS NULL");
    match variant {
        Some("open") => where_builder.add_condition("t.status IN ('pending', 'in_progress')"),
        Some("completed") => where_builder.add_condition("t.status = 'completed'"),
        _ => {}
    }

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "t.position, t.created_at");

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM acme.tasks t
        WHERE {}
        "#,
        where_clause
    );

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(project_id);
    for value in &filter_values {
        count_query = count_query.bind(value);
    }
    let total = count_query.fetch_one(pool).await?;

    let sql = format!(
        r#"
        SELECT
            t.id, t.project_id, t.title, t.description, t.status, t.priority,
            t.notes,
            t.due_date, t.completed_at, t.position, t.weight,
            t.created_at, t.updated_at, t.deleted_at,
            COALESCE(COUNT(tl.label_id), 0) as label_count
        FROM acme.tasks t
        LEFT JOIN acme.task_labels tl ON tl.task_id = t.id
        WHERE {}
        GROUP BY t.id
        ORDER BY {}
        LIMIT ${}
        OFFSET ${}
        "#,
        where_clause,
        order_by,
        filter_values.len() + 2,
        filter_values.len() + 3
    );

    let mut query_builder = sqlx::query_as::<_, TaskWithLabelsRow>(&sql);
    query_builder = query_builder.bind(project_id);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let data = query_builder.fetch_all(pool).await?;
    let has_more = offset + (data.len() as i64) < total;

    Ok(TaskListResponse {
        data,
        total,
        has_more,
    })
}

/// Update a task.
#[allow(clippy::too_many_arguments)]
pub async fn update_task(
    pool: &DbPool,
    id: Uuid,
    project_id: Uuid,
    title: Option<&str>,
    description: Option<Option<&str>>,
    notes: Option<Option<&Value>>,
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
            notes = CASE WHEN $6 THEN $7 ELSE notes END,
            status = COALESCE($8, status),
            priority = COALESCE($9, priority),
            due_date = CASE WHEN $10 THEN $11 ELSE due_date END,
            completed_at = {},
            updated_at = NOW()
        WHERE id = $1 AND project_id = $2 AND deleted_at IS NULL
        RETURNING id, project_id, title, description, notes, status, priority, due_date, completed_at, position, weight, created_at, updated_at, deleted_at
        "#,
        completed_at_expr
    );

    sqlx::query_as::<_, TaskRow>(&query)
        .bind(id)
        .bind(project_id)
        .bind(title)
        .bind(description.is_some())
        .bind(description.flatten())
        .bind(notes.is_some())
        .bind(notes.flatten())
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

#[derive(Debug, Clone, Default)]
pub struct ReorderTasksResult {
    pub reordered_count: usize,
    pub missing_from_submission: Vec<Uuid>,
    pub not_found: Vec<Uuid>,
}

/// Reorder tasks within a project with conflict detection.
pub async fn reorder_tasks(
    pool: &DbPool,
    project_id: Uuid,
    task_ids: &[Uuid],
) -> Result<ReorderTasksResult, sqlx::Error> {
    let current_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM acme.tasks
        WHERE project_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let submitted_set: HashSet<Uuid> = task_ids.iter().copied().collect();
    let current_set: HashSet<Uuid> = current_ids.iter().copied().collect();

    let missing_from_submission: Vec<Uuid> =
        current_set.difference(&submitted_set).copied().collect();
    let not_found: Vec<Uuid> = submitted_set.difference(&current_set).copied().collect();

    if !missing_from_submission.is_empty() || !not_found.is_empty() {
        return Ok(ReorderTasksResult {
            reordered_count: 0,
            missing_from_submission,
            not_found,
        });
    }

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
    Ok(ReorderTasksResult {
        reordered_count: task_ids.len(),
        missing_from_submission: Vec::new(),
        not_found: Vec::new(),
    })
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
