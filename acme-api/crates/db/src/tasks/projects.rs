use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use underlay_http::query::QueryParams;
use underlay_query::{FieldMapping, WhereBuilder};
use uuid::Uuid;

use crate::DbPool;

#[derive(Debug)]
pub struct ProjectListResponse {
    pub data: Vec<ProjectWithCountsRow>,
    pub total: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectTaskSummary {
    pub total: i64,
    pub completed: i64,
}

/// Row type for acme.projects table.
#[derive(Debug, Clone, FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<Value>,
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
    pub description: Option<Value>,
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
        .sort_only("created_at", "p.created_at")
        .sort_only("updated_at", "p.updated_at")
        .sort_only("category_name", "c.name")
        .filter_only("category_id", "p.category_id::text")
        .filter_only("owner_id", "p.owner_id::text")
}

/// Create a new project.
pub async fn create_project(
    pool: &DbPool,
    id: Uuid,
    owner_id: Uuid,
    name: &str,
    description: Option<&Value>,
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

pub async fn get_project_task_summary(
    pool: &DbPool,
    project_id: Uuid,
) -> Result<ProjectTaskSummary, sqlx::Error> {
    let counts = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            COALESCE(COUNT(id) FILTER (WHERE deleted_at IS NULL), 0) AS total,
            COALESCE(COUNT(id) FILTER (WHERE deleted_at IS NULL AND status = 'completed'), 0) AS completed
        FROM acme.tasks
        WHERE project_id = $1
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    Ok(ProjectTaskSummary {
        total: counts.0,
        completed: counts.1,
    })
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
    limit: i64,
    offset: i64,
) -> Result<ProjectListResponse, sqlx::Error> {
    let mapping = project_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    where_builder.add_condition("p.deleted_at IS NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "p.weight, p.name");

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM acme.projects p
        WHERE {}
        "#,
        where_clause
    );

    let mut count_query = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_sql));
    for value in &filter_values {
        count_query = count_query.bind(value);
    }
    let total = count_query.fetch_one(pool).await?;

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
        LIMIT ${}
        OFFSET ${}
        "#,
        where_clause,
        order_by,
        filter_values.len() + 1,
        filter_values.len() + 2
    );

    let mut query_builder = sqlx::query_as::<_, ProjectWithCountsRow>(sqlx::AssertSqlSafe(sql));
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let data = query_builder.fetch_all(pool).await?;
    let has_more = offset + (data.len() as i64) < total;

    Ok(ProjectListResponse {
        data,
        total,
        has_more,
    })
}

/// Update a project.
pub async fn update_project(
    pool: &DbPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<Option<&Value>>,
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

#[derive(Debug, Clone, Default)]
pub struct ReorderProjectsResult {
    pub reordered_count: usize,
    pub missing_from_submission: Vec<Uuid>,
    pub not_found: Vec<Uuid>,
}

/// Reorder projects by weight with conflict detection.
pub async fn reorder_projects(
    pool: &DbPool,
    project_ids: &[Uuid],
) -> Result<ReorderProjectsResult, sqlx::Error> {
    let table = underlay_db::QualifiedTableName::parse("acme.projects")
        .expect("valid table name");
    let id_col = underlay_db::SqlIdentifier::parse("id").expect("valid column");
    let weight_col = underlay_db::SqlIdentifier::parse("weight").expect("valid column");
    let deleted_col = underlay_db::SqlIdentifier::parse("deleted_at").expect("valid column");

    match underlay_db::reorder_scoped(
        pool,
        &table,
        &id_col,
        &weight_col,
        underlay_db::ReorderScope::none().exclude_deleted(&deleted_col),
        project_ids,
    )
    .await
    {
        Ok(rows) => Ok(ReorderProjectsResult {
            reordered_count: rows as usize,
            missing_from_submission: Vec::new(),
            not_found: Vec::new(),
        }),
        Err(underlay_db::ReorderError::Conflict(conflict)) => Ok(ReorderProjectsResult {
            reordered_count: 0,
            missing_from_submission: conflict.removed_ids,
            not_found: conflict.added_ids,
        }),
        Err(underlay_db::ReorderError::DuplicateIds) => Err(sqlx::Error::InvalidArgument(
            "reorder submission contains duplicate ids".to_string(),
        )),
        Err(underlay_db::ReorderError::Db(err)) => Err(err),
    }
}

/// Batch soft delete projects.
///
/// Returns the number of projects deleted.
pub async fn batch_soft_delete_projects(
    pool: &DbPool,
    ids: &[Uuid],
    batch_id: Uuid,
) -> Result<u64, sqlx::Error> {
    if ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        UPDATE acme.projects
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
