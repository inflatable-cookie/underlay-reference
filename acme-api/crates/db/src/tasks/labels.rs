// conformance: allow — parent-scoped collection, small by design

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use underlay_http::query::QueryParams;
use underlay_query::{FieldMapping, WhereBuilder};
use uuid::Uuid;

use crate::DbPool;

#[derive(Debug)]
pub struct LabelListResponse {
    pub data: Vec<LabelRow>,
    pub total: i64,
    pub has_more: bool,
}

/// Row type for acme.labels table.
#[derive(Debug, Clone, FromRow)]
pub struct LabelRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
        RETURNING id, project_id, name, color, weight, created_at, updated_at, deleted_at
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
        SELECT id, project_id, name, color, weight, created_at, updated_at, deleted_at
        FROM acme.labels
        WHERE project_id = $1 AND deleted_at IS NULL
        ORDER BY weight, name
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// List labels for a project with filtering, sorting, and paging (admin).
pub async fn list_labels_admin(
    pool: &DbPool,
    project_id: Uuid,
    query: &QueryParams,
    limit: i64,
    offset: i64,
) -> Result<LabelListResponse, sqlx::Error> {
    let mapping = FieldMapping::new()
        .map("name", "name")
        .sort_only("weight", "weight")
        .sort_only("created_at", "created_at");
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(2); // $1 is project_id
    where_builder.add_condition("project_id = $1");
    where_builder.add_condition("deleted_at IS NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "weight, name");

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM acme.labels
        WHERE {}
        "#,
        where_clause
    );

    let mut count_query =
        sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_sql)).bind(project_id);
    for value in &filter_values {
        count_query = count_query.bind(value);
    }
    let total = count_query.fetch_one(pool).await?;

    let sql = format!(
        r#"
        SELECT id, project_id, name, color, weight, created_at, updated_at, deleted_at
        FROM acme.labels
        WHERE {}
        ORDER BY {}
        LIMIT ${}
        OFFSET ${}
        "#,
        where_clause,
        order_by,
        filter_values.len() + 2,
        filter_values.len() + 3
    );

    let mut query_builder = sqlx::query_as::<_, LabelRow>(sqlx::AssertSqlSafe(sql));
    query_builder = query_builder.bind(project_id);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let data = query_builder.fetch_all(pool).await?;
    let has_more = offset + (data.len() as i64) < total;

    Ok(LabelListResponse {
        data,
        total,
        has_more,
    })
}

/// Get a label by ID.
pub async fn get_label(pool: &DbPool, id: Uuid) -> Result<Option<LabelRow>, sqlx::Error> {
    sqlx::query_as::<_, LabelRow>(
        r#"
        SELECT id, project_id, name, color, weight, created_at, updated_at, deleted_at
        FROM acme.labels
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Update a label.
pub async fn update_label(
    pool: &DbPool,
    id: Uuid,
    project_id: Uuid,
    name: Option<&str>,
    color: Option<&str>,
) -> Result<Option<LabelRow>, sqlx::Error> {
    sqlx::query_as::<_, LabelRow>(
        r#"
        UPDATE acme.labels
        SET
            name = COALESCE($3, name),
            color = COALESCE($4, color),
            updated_at = NOW()
        WHERE id = $1 AND project_id = $2 AND deleted_at IS NULL
        RETURNING id, project_id, name, color, weight, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(name)
    .bind(color)
    .fetch_optional(pool)
    .await
}

/// Soft delete a label.
pub async fn soft_delete_label(
    pool: &DbPool,
    id: Uuid,
    batch_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE acme.labels
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
        SELECT l.id, l.project_id, l.name, l.color, l.weight, l.created_at, l.updated_at, l.deleted_at
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
