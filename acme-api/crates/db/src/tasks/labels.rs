// conformance: allow — parent-scoped collection, small by design

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DbPool;

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
