// conformance: allow — parent-scoped collection, small by design

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DbPool;

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
