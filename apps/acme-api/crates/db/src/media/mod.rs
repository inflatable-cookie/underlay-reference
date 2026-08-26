//! Database layer for the Media Library.
//!
//! Provides raw database operations for media, media versions, renditions, and usage tracking.

use crate::DbPool;
use sqlx::FromRow;
use underlay_http::query::QueryParams;
use underlay_media::BlobObjectKey;
use underlay_query::{FieldMapping, WhereBuilder};
use uuid::Uuid;

mod queries;
mod renditions;
mod usage;
mod versions;

pub use queries::*;
pub use renditions::*;
pub use usage::*;
pub use versions::*;

// ============================================================================
// Row Types
// ============================================================================

/// Raw DB representation of a media item.
#[derive(Debug, Clone, FromRow)]
pub struct MediaRow {
    pub id: Uuid,
    pub kind: String,
    pub visibility: String,
    pub title: String,
    pub original_filename: Option<String>,
    pub current_version_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Option<Uuid>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_by: Option<Uuid>,
}

/// Raw DB representation of a media version.
#[derive(Debug, Clone)]
pub struct MediaVersionRow {
    pub id: Uuid,
    pub media_id: Uuid,
    pub state: String,
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub storage_provider: Option<String>,
    pub bucket: Option<String>,
    pub object_key: Option<BlobObjectKey>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
}

/// Raw DB representation of a media rendition.
#[derive(Debug, Clone)]
pub struct MediaRenditionRow {
    pub id: Uuid,
    pub media_version_id: Uuid,
    pub kind: String,
    pub byte_size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub storage_provider: String,
    pub bucket: String,
    pub object_key: BlobObjectKey,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Raw DB representation of a media usage record.
#[derive(Debug, Clone, FromRow)]
pub struct MediaUsageRow {
    pub id: Uuid,
    pub media_id: Uuid,
    pub used_by_type: String,
    pub used_by_id: Option<Uuid>,
    pub owner_field: Option<String>,
    pub content_kind: String,
    pub locator_kind: String,
    pub locator_key: String,
    pub usage_role: String,
    pub provenance_kind: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct MediaListResponse {
    pub data: Vec<MediaWithVersionRow>,
    pub total: i64,
    pub has_more: bool,
}

/// Media with current version info for list views.
#[derive(Debug, Clone)]
pub struct MediaWithVersionRow {
    pub id: Uuid,
    pub kind: String,
    pub visibility: String,
    pub title: String,
    pub original_filename: Option<String>,
    pub current_version_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    // From current version
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
    // Thumbnail rendition object key (if available)
    pub thumbnail_object_key: Option<BlobObjectKey>,
}

#[derive(Debug, Clone, FromRow)]
pub(crate) struct RawMediaVersionRow {
    pub id: Uuid,
    pub media_id: Uuid,
    pub state: String,
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub storage_provider: Option<String>,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, FromRow)]
pub(crate) struct RawMediaRenditionRow {
    pub id: Uuid,
    pub media_version_id: Uuid,
    pub kind: String,
    pub byte_size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub storage_provider: String,
    pub bucket: String,
    pub object_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub(crate) struct RawMediaWithVersionRow {
    pub id: Uuid,
    pub kind: String,
    pub visibility: String,
    pub title: String,
    pub original_filename: Option<String>,
    pub current_version_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
    pub thumbnail_object_key: Option<String>,
}

impl TryFrom<RawMediaVersionRow> for MediaVersionRow {
    type Error = sqlx::Error;

    fn try_from(row: RawMediaVersionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            media_id: row.media_id,
            state: row.state,
            byte_size: row.byte_size,
            mime_type: row.mime_type,
            sha256: row.sha256,
            storage_provider: row.storage_provider,
            bucket: row.bucket,
            object_key: parse_optional_object_key(row.object_key)?,
            created_at: row.created_at,
            created_by: row.created_by,
        })
    }
}

impl TryFrom<RawMediaRenditionRow> for MediaRenditionRow {
    type Error = sqlx::Error;

    fn try_from(row: RawMediaRenditionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            media_version_id: row.media_version_id,
            kind: row.kind,
            byte_size: row.byte_size,
            mime_type: row.mime_type,
            width: row.width,
            height: row.height,
            storage_provider: row.storage_provider,
            bucket: row.bucket,
            object_key: parse_object_key(row.object_key)?,
            created_at: row.created_at,
        })
    }
}

impl TryFrom<RawMediaWithVersionRow> for MediaWithVersionRow {
    type Error = sqlx::Error;

    fn try_from(row: RawMediaWithVersionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            kind: row.kind,
            visibility: row.visibility,
            title: row.title,
            original_filename: row.original_filename,
            current_version_id: row.current_version_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
            byte_size: row.byte_size,
            mime_type: row.mime_type,
            thumbnail_object_key: parse_optional_object_key(row.thumbnail_object_key)?,
        })
    }
}

fn parse_optional_object_key(value: Option<String>) -> Result<Option<BlobObjectKey>, sqlx::Error> {
    value.map(parse_object_key).transpose()
}

fn parse_object_key(value: String) -> Result<BlobObjectKey, sqlx::Error> {
    BlobObjectKey::parse(value).map_err(|err| sqlx::Error::Decode(Box::new(err)))
}

// ============================================================================
// Field Mapping
// ============================================================================

/// Get field mapping for media queries.
///
/// Supports filtering by kind, visibility, and title (search).
/// Supports sorting by title, kind, updated_at, created_at.
pub fn media_field_mapping() -> FieldMapping {
    FieldMapping::new()
        .map("title", "m.title")
        .map("kind", "m.kind")
        .map("visibility", "m.visibility")
        .sort_only("updated_at", "m.updated_at")
        .sort_only("created_at", "m.created_at")
}
