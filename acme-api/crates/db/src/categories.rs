//! Category database operations.
//!
//! Categories are top-level organizational containers for projects.
//! Demonstrates hierarchical relationships and soft delete patterns.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use underlay_http::query::QueryParams;
use underlay_query::{FieldMapping, WhereBuilder};
use uuid::Uuid;

use crate::DbPool;

#[derive(Debug)]
pub struct CategoryListResponse {
    pub data: Vec<CategoryWithCountsRow>,
    pub total: i64,
    pub has_more: bool,
}

// ============================================================================
// Row Types
// ============================================================================

/// Row type for acme.categories table.
#[derive(Debug, Clone, FromRow)]
pub struct CategoryRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub weight: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Category with project counts (for admin list views).
#[derive(Debug, Clone, FromRow)]
pub struct CategoryWithCountsRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub weight: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub project_count: i64,
}

// ============================================================================
// Query Functions
// ============================================================================

/// Get field mapping for category queries.
pub fn category_field_mapping() -> FieldMapping {
    FieldMapping::new()
        .map("name", "name")
        .map("slug", "slug")
        .map("is_active", "is_active")
        .sort_only("weight", "weight")
        .sort_only("created_at", "created_at")
}

/// List categories with filtering and sorting.
pub async fn list_categories(
    pool: &DbPool,
    query: &QueryParams,
    include_deleted: bool,
) -> Result<Vec<CategoryRow>, sqlx::Error> {
    let mapping = category_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    if !include_deleted {
        where_builder.add_condition("deleted_at IS NULL");
    }

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "weight, name");

    let where_sql = if where_clause.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clause)
    };

    let sql = format!(
        r#"
        SELECT id, name, slug, description, color, weight, is_active, created_at, updated_at, deleted_at
        FROM acme.categories
        {}
        ORDER BY {}
        "#,
        where_sql, order_by
    );

    let mut query_builder = sqlx::query_as::<_, CategoryRow>(&sql);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }

    query_builder.fetch_all(pool).await
}

/// List categories with project counts (for admin).
pub async fn list_categories_with_counts(
    pool: &DbPool,
    query: &QueryParams,
    limit: i64,
    offset: i64,
) -> Result<CategoryListResponse, sqlx::Error> {
    let mapping = FieldMapping::new()
        .map("name", "c.name")
        .map("slug", "c.slug")
        .map("is_active", "c.is_active")
        .sort_only("weight", "c.weight")
        .sort_only("created_at", "c.created_at")
        .sort_only("project_count", "project_count");
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    where_builder.add_condition("c.deleted_at IS NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "c.weight, c.name");

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM acme.categories c
        WHERE {}
        "#,
        where_clause
    );

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for value in &filter_values {
        count_query = count_query.bind(value);
    }
    let total = count_query.fetch_one(pool).await?;

    let sql = format!(
        r#"
        SELECT
            c.id, c.name, c.slug, c.description, c.color, c.weight,
            c.is_active, c.created_at, c.updated_at, c.deleted_at,
            COALESCE(COUNT(p.id) FILTER (WHERE p.deleted_at IS NULL), 0) as project_count
        FROM acme.categories c
        LEFT JOIN acme.projects p ON p.category_id = c.id
        WHERE {}
        GROUP BY c.id
        ORDER BY {}
        LIMIT ${}
        OFFSET ${}
        "#,
        where_clause,
        order_by,
        filter_values.len() + 1,
        filter_values.len() + 2
    );

    let mut query_builder = sqlx::query_as::<_, CategoryWithCountsRow>(&sql);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let data = query_builder.fetch_all(pool).await?;
    let has_more = offset + (data.len() as i64) < total;

    Ok(CategoryListResponse {
        data,
        total,
        has_more,
    })
}

/// Get a category by ID.
pub async fn get_category(pool: &DbPool, id: Uuid) -> Result<Option<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"
        SELECT id, name, slug, description, color, weight, is_active, created_at, updated_at, deleted_at
        FROM acme.categories
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Get a category by slug.
pub async fn get_category_by_slug(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"
        SELECT id, name, slug, description, color, weight, is_active, created_at, updated_at, deleted_at
        FROM acme.categories
        WHERE slug = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

/// Create a new category.
pub async fn create_category(
    pool: &DbPool,
    id: Uuid,
    name: &str,
    slug: &str,
    description: Option<&str>,
    color: Option<&str>,
) -> Result<CategoryRow, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"
        INSERT INTO acme.categories (id, name, slug, description, color)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, slug, description, color, weight, is_active, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(color)
    .fetch_one(pool)
    .await
}

/// Update a category.
pub async fn update_category(
    pool: &DbPool,
    id: Uuid,
    name: Option<&str>,
    slug: Option<&str>,
    description: Option<Option<&str>>,
    color: Option<Option<&str>>,
    is_active: Option<bool>,
) -> Result<Option<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"
        UPDATE acme.categories
        SET
            name = COALESCE($2, name),
            slug = COALESCE($3, slug),
            description = CASE WHEN $4 THEN $5 ELSE description END,
            color = CASE WHEN $6 THEN $7 ELSE color END,
            is_active = COALESCE($8, is_active),
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, name, slug, description, color, weight, is_active, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(slug)
    .bind(description.is_some())
    .bind(description.flatten())
    .bind(color.is_some())
    .bind(color.flatten())
    .bind(is_active)
    .fetch_optional(pool)
    .await
}

/// Soft delete a category.
pub async fn soft_delete_category(
    pool: &DbPool,
    id: Uuid,
    batch_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE acme.categories
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

/// Batch soft delete categories.
pub async fn batch_soft_delete_categories(
    pool: &DbPool,
    ids: &[Uuid],
    batch_id: Uuid,
) -> Result<u64, sqlx::Error> {
    if ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        UPDATE acme.categories
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

/// Restore a soft-deleted category.
pub async fn restore_category(pool: &DbPool, id: Uuid) -> Result<Option<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"
        UPDATE acme.categories
        SET deleted_at = NULL, delete_batch_id = NULL, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, slug, description, color, weight, is_active, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, Default)]
pub struct ReorderCategoriesResult {
    pub reordered_count: usize,
    pub missing_from_submission: Vec<Uuid>,
    pub not_found: Vec<Uuid>,
}

/// Reorder categories by setting weights with conflict detection.
///
/// Delegates to the canonical underlay reorder helper: one transaction,
/// single weight-rewrite statement, set-conflict on drift. Weights are
/// assigned 1..=N in the submitted order.
pub async fn reorder_categories(
    pool: &DbPool,
    category_ids: &[Uuid],
) -> Result<ReorderCategoriesResult, sqlx::Error> {
    let table = underlay_db::QualifiedTableName::parse("acme.categories")
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
        category_ids,
    )
    .await
    {
        Ok(rows) => Ok(ReorderCategoriesResult {
            reordered_count: rows as usize,
            missing_from_submission: Vec::new(),
            not_found: Vec::new(),
        }),
        Err(underlay_db::ReorderError::Conflict(conflict)) => Ok(ReorderCategoriesResult {
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

/// Validate that a slug is unique (for async field validation).
pub async fn is_slug_available(
    pool: &DbPool,
    slug: &str,
    exclude_id: Option<Uuid>,
) -> Result<bool, sqlx::Error> {
    let result: (i64,) = if let Some(id) = exclude_id {
        sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM acme.categories
            WHERE slug = $1 AND id != $2 AND deleted_at IS NULL
            "#,
        )
        .bind(slug)
        .bind(id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM acme.categories
            WHERE slug = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(slug)
        .fetch_one(pool)
        .await?
    };

    Ok(result.0 == 0)
}
