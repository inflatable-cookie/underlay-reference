use super::*;

/// Create a new media item.
pub async fn create_media(
    pool: &DbPool,
    id: Uuid,
    kind: &str,
    visibility: &str,
    title: &str,
    original_filename: Option<&str>,
    created_by: Option<Uuid>,
) -> Result<MediaRow, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        INSERT INTO media.media (id, kind, visibility, title, original_filename, created_by, updated_by)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        RETURNING id, kind, visibility, title, original_filename, current_version_id,
                  created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        "#,
    )
    .bind(id)
    .bind(kind)
    .bind(visibility)
    .bind(title)
    .bind(original_filename)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

/// Get a media item by ID.
pub async fn get_media(pool: &DbPool, id: Uuid) -> Result<Option<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        SELECT id, kind, visibility, title, original_filename, current_version_id,
               created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        FROM media.media
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Get a media item by ID (admin, excluding deleted).
pub async fn get_media_admin(pool: &DbPool, id: Uuid) -> Result<Option<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        SELECT id, kind, visibility, title, original_filename, current_version_id,
               created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        FROM media.media
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// List all media items with filtering and sorting (admin, excluding deleted and incomplete uploads).
pub async fn list_media_admin(
    pool: &DbPool,
    query: &QueryParams,
) -> Result<Vec<MediaWithVersionRow>, sqlx::Error> {
    let mapping = media_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    where_builder.add_condition("m.deleted_at IS NULL");
    where_builder.add_condition("m.current_version_id IS NOT NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "m.updated_at DESC");

    let sql = format!(
        r#"
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.updated_at, m.deleted_at,
               v.byte_size, v.mime_type,
               r.object_key AS thumbnail_object_key
        FROM media.media m
        LEFT JOIN media.media_version v ON m.current_version_id = v.id
        LEFT JOIN media.media_rendition r ON v.id = r.media_version_id AND r.kind = 'thumbnail'
        WHERE {}
        ORDER BY {}
        "#,
        where_clause, order_by
    );

    let mut query_builder = sqlx::query_as::<_, MediaWithVersionRow>(&sql);
    for value in filter_values {
        query_builder = query_builder.bind(value);
    }

    query_builder.fetch_all(pool).await
}

/// List media items with pagination (admin, excluding deleted and incomplete uploads).
pub async fn list_media_admin_paged(
    pool: &DbPool,
    query: &QueryParams,
    limit: i64,
    offset: i64,
) -> Result<MediaListResponse, sqlx::Error> {
    let mapping = media_field_mapping();
    let filters = query.filter_fields();

    let mut where_builder = WhereBuilder::new(1);
    where_builder.add_condition("m.deleted_at IS NULL");
    where_builder.add_condition("m.current_version_id IS NOT NULL");

    for filter in &filters {
        where_builder.add_filter(filter, &mapping.filter_map());
    }

    let (where_clause, filter_values) = where_builder.build();
    let order_by = query.sql_order_by_or(&mapping.sort_map(), "m.updated_at DESC, m.id DESC");

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM media.media m
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
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.updated_at, m.deleted_at,
               v.byte_size, v.mime_type,
               r.object_key AS thumbnail_object_key
        FROM media.media m
        LEFT JOIN media.media_version v ON m.current_version_id = v.id
        LEFT JOIN media.media_rendition r ON v.id = r.media_version_id AND r.kind = 'thumbnail'
        WHERE {}
        ORDER BY {}
        LIMIT ${} OFFSET ${}
        "#,
        where_clause,
        order_by,
        filter_values.len() + 1,
        filter_values.len() + 2
    );

    let mut items_query = sqlx::query_as::<_, MediaWithVersionRow>(&sql);
    for value in &filter_values {
        items_query = items_query.bind(value);
    }

    let items = items_query
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(MediaListResponse {
        has_more: offset + limit < total,
        data: items,
        total,
    })
}

/// List soft-deleted media items (trash).
pub async fn list_media_trash(pool: &DbPool) -> Result<Vec<MediaWithVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaWithVersionRow>(
        r#"
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.updated_at, m.deleted_at,
               v.byte_size, v.mime_type,
               r.object_key AS thumbnail_object_key
        FROM media.media m
        LEFT JOIN media.media_version v ON m.current_version_id = v.id
        LEFT JOIN media.media_rendition r ON v.id = r.media_version_id AND r.kind = 'thumbnail'
        WHERE m.deleted_at IS NOT NULL
          AND m.current_version_id IS NOT NULL
        ORDER BY m.deleted_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Update a media item.
pub async fn update_media(
    pool: &DbPool,
    id: Uuid,
    title: &str,
    original_filename: Option<&str>,
    visibility: &str,
    updated_by: Option<Uuid>,
) -> Result<MediaRow, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        UPDATE media.media
        SET title = $2,
            original_filename = COALESCE($3, original_filename),
            visibility = $4,
            updated_by = $5,
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, kind, visibility, title, original_filename, current_version_id,
                  created_at, created_by, updated_at, updated_by, deleted_at, deleted_by
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(original_filename)
    .bind(visibility)
    .bind(updated_by)
    .fetch_one(pool)
    .await
}

/// Soft delete a media item.
pub async fn soft_delete_media(
    pool: &DbPool,
    id: Uuid,
    deleted_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE media.media
        SET deleted_at = NOW(), deleted_by = $2
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(deleted_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// Restore a soft-deleted media item.
pub async fn restore_media(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE media.media
        SET deleted_at = NULL, deleted_by = NULL
        WHERE id = $1 AND deleted_at IS NOT NULL
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Hard delete a media item (for purge).
pub async fn purge_media(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM media.media WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Batch soft delete media items.
///
/// Returns the number of items deleted.
pub async fn batch_soft_delete_media(
    pool: &DbPool,
    ids: &[Uuid],
    deleted_by: Option<Uuid>,
) -> Result<u64, sqlx::Error> {
    if ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        UPDATE media.media
        SET deleted_at = NOW(), deleted_by = $1
        WHERE id = ANY($2) AND deleted_at IS NULL
        "#,
    )
    .bind(deleted_by)
    .bind(ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
