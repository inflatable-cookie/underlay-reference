use async_trait::async_trait;
use underlay_media::{
    MediaContentKind, MediaId, MediaLocatorKind, MediaUsageEdge, MediaUsageEdgeInput,
    MediaUsageEdgeKey, MediaUsageProvenanceKind, MediaUsageRole, MediaUsageSyncRepository,
};

use super::*;

#[derive(Clone)]
pub struct AcmeMediaUsageSyncRepo {
    pool: DbPool,
}

impl AcmeMediaUsageSyncRepo {
    pub fn new(pool: &DbPool) -> Self {
        Self { pool: pool.clone() }
    }
}

/// Add or retain one locator-aware media usage edge.
pub async fn upsert_media_usage_edge(
    pool: &DbPool,
    id: Uuid,
    usage: &MediaUsageEdgeInput,
) -> Result<MediaUsageRow, sqlx::Error> {
    sqlx::query_as::<_, MediaUsageRow>(
        r#"
        INSERT INTO media.media_usage (
            id,
            media_id,
            used_by_type,
            used_by_id,
            owner_field,
            content_kind,
            locator_kind,
            locator_key,
            usage_role,
            provenance_kind
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (
            media_id,
            used_by_type,
            used_by_id,
            owner_field,
            locator_kind,
            locator_key,
            provenance_kind
        ) DO UPDATE
        SET id = media.media_usage.id
        RETURNING
            id,
            media_id,
            used_by_type,
            used_by_id,
            owner_field,
            content_kind,
            locator_kind,
            locator_key,
            usage_role,
            provenance_kind,
            created_at
        "#,
    )
    .bind(id)
    .bind(usage.media_id.into_inner())
    .bind(&usage.used_by_type)
    .bind(usage.used_by_id)
    .bind(&usage.owner_field)
    .bind(usage.content_kind.as_str())
    .bind(usage.locator_kind.as_str())
    .bind(&usage.locator_key)
    .bind(usage.usage_role.as_str())
    .bind(usage.provenance_kind.as_str())
    .fetch_one(pool)
    .await
}

/// Remove one locator-aware media usage edge by natural key.
pub async fn remove_media_usage_edge(
    pool: &DbPool,
    key: &MediaUsageEdgeKey,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM media.media_usage
        WHERE media_id = $1
          AND used_by_type = $2
          AND used_by_id IS NOT DISTINCT FROM $3
          AND owner_field IS NOT DISTINCT FROM $4
          AND locator_kind = $5
          AND locator_key = $6
          AND provenance_kind = $7
        "#,
    )
    .bind(key.media_id.into_inner())
    .bind(&key.used_by_type)
    .bind(key.used_by_id)
    .bind(&key.owner_field)
    .bind(key.locator_kind.as_str())
    .bind(&key.locator_key)
    .bind(key.provenance_kind.as_str())
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// List all usage rows for a media item.
pub async fn list_media_usages(
    pool: &DbPool,
    media_id: Uuid,
) -> Result<Vec<MediaUsageRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaUsageRow>(
        r#"
        SELECT
            id,
            media_id,
            used_by_type,
            used_by_id,
            owner_field,
            content_kind,
            locator_kind,
            locator_key,
            usage_role,
            provenance_kind,
            created_at
        FROM media.media_usage
        WHERE media_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(media_id)
    .fetch_all(pool)
    .await
}

/// List all managed usage edges for one owner/provenance scope.
pub async fn list_usage_edges_for_owner(
    pool: &DbPool,
    used_by_type: &str,
    used_by_id: Uuid,
    provenance_kind: &MediaUsageProvenanceKind,
) -> Result<Vec<MediaUsageEdge>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MediaUsageRow>(
        r#"
        SELECT
            id,
            media_id,
            used_by_type,
            used_by_id,
            owner_field,
            content_kind,
            locator_kind,
            locator_key,
            usage_role,
            provenance_kind,
            created_at
        FROM media.media_usage
        WHERE used_by_type = $1
          AND used_by_id = $2
          AND provenance_kind = $3
        ORDER BY created_at DESC
        "#,
    )
    .bind(used_by_type)
    .bind(used_by_id)
    .bind(provenance_kind.as_str())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(media_usage_edge_from_row).collect()
}

/// Get usage count for a media item.
pub async fn get_media_usage_count(pool: &DbPool, media_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media.media_usage WHERE media_id = $1")
        .bind(media_id)
        .fetch_one(pool)
        .await
}

/// List media items with zero usages (excludes incomplete uploads).
pub async fn list_unused_media(pool: &DbPool) -> Result<Vec<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        r#"
        SELECT m.id, m.kind, m.visibility, m.title, m.original_filename, m.current_version_id,
               m.created_at, m.created_by, m.updated_at, m.updated_by, m.deleted_at, m.deleted_by
        FROM media.media m
        LEFT JOIN media.media_usage u ON m.id = u.media_id
        WHERE m.deleted_at IS NULL
          AND m.current_version_id IS NOT NULL
          AND u.id IS NULL
        ORDER BY m.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// List all usage rows for a specific entity field.
pub async fn list_usages_by_entity(
    pool: &DbPool,
    used_by_type: &str,
    used_by_id: Uuid,
    owner_field: &str,
) -> Result<Vec<MediaUsageRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaUsageRow>(
        r#"
        SELECT
            id,
            media_id,
            used_by_type,
            used_by_id,
            owner_field,
            content_kind,
            locator_kind,
            locator_key,
            usage_role,
            provenance_kind,
            created_at
        FROM media.media_usage
        WHERE used_by_type = $1
          AND used_by_id = $2
          AND owner_field = $3
        ORDER BY created_at DESC
        "#,
    )
    .bind(used_by_type)
    .bind(used_by_id)
    .bind(owner_field)
    .fetch_all(pool)
    .await
}

fn media_usage_edge_from_row(row: MediaUsageRow) -> Result<MediaUsageEdge, sqlx::Error> {
    Ok(MediaUsageEdge {
        id: row.id,
        media_id: MediaId::from(row.media_id),
        used_by_type: row.used_by_type,
        used_by_id: row.used_by_id,
        owner_field: row.owner_field,
        content_kind: parse_content_kind(&row.content_kind),
        locator_kind: parse_locator_kind(&row.locator_kind),
        locator_key: row.locator_key,
        usage_role: parse_usage_role(&row.usage_role),
        provenance_kind: parse_provenance_kind(&row.provenance_kind),
        created_at: row.created_at,
    })
}

fn parse_content_kind(value: &str) -> MediaContentKind {
    match value {
        "record_field" => MediaContentKind::RecordField,
        "structured_content" => MediaContentKind::StructuredContent,
        "external" => MediaContentKind::External,
        other => MediaContentKind::Custom(other.to_string()),
    }
}

fn parse_locator_kind(value: &str) -> MediaLocatorKind {
    match value {
        "field" => MediaLocatorKind::Field,
        "block_id" => MediaLocatorKind::BlockId,
        "path" => MediaLocatorKind::Path,
        "external_ref" => MediaLocatorKind::ExternalRef,
        other => MediaLocatorKind::Custom(other.to_string()),
    }
}

fn parse_usage_role(value: &str) -> MediaUsageRole {
    match value {
        "primary" => MediaUsageRole::Primary,
        "attachment" => MediaUsageRole::Attachment,
        "embedded" => MediaUsageRole::Embedded,
        "external" => MediaUsageRole::External,
        "derived" => MediaUsageRole::Derived,
        other => MediaUsageRole::Custom(other.to_string()),
    }
}

fn parse_provenance_kind(value: &str) -> MediaUsageProvenanceKind {
    match value {
        "content_sync" => MediaUsageProvenanceKind::ContentSync,
        "legacy_migration" => MediaUsageProvenanceKind::LegacyMigration,
        "manual" => MediaUsageProvenanceKind::Manual,
        "system_generated" => MediaUsageProvenanceKind::SystemGenerated,
        other => MediaUsageProvenanceKind::Custom(other.to_string()),
    }
}

#[async_trait]
impl MediaUsageSyncRepository for AcmeMediaUsageSyncRepo {
    async fn list_usage_edges_for_owner(
        &self,
        used_by_type: &str,
        used_by_id: Uuid,
        provenance_kind: &MediaUsageProvenanceKind,
    ) -> underlay_media::MediaResult<Vec<MediaUsageEdge>> {
        list_usage_edges_for_owner(&self.pool, used_by_type, used_by_id, provenance_kind)
            .await
            .map_err(|err| underlay_media::MediaError::storage(err.to_string()))
    }

    async fn upsert_usage_edge(
        &self,
        usage: &MediaUsageEdgeInput,
    ) -> underlay_media::MediaResult<()> {
        upsert_media_usage_edge(&self.pool, Uuid::now_v7(), usage)
            .await
            .map(|_| ())
            .map_err(|err| underlay_media::MediaError::storage(err.to_string()))
    }

    async fn remove_usage_edge(
        &self,
        key: &MediaUsageEdgeKey,
    ) -> underlay_media::MediaResult<bool> {
        remove_media_usage_edge(&self.pool, key)
            .await
            .map_err(|err| underlay_media::MediaError::storage(err.to_string()))
    }
}
