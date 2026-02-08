//! Database layer for the Media Library.
//!
//! Provides raw database operations for media, media versions, renditions, and usage tracking.

use crate::DbPool;
use sqlx::FromRow;
use underlay_db::pagination::{Cursor, PaginatedResponse, PaginationBuilder, PaginationParams};
use underlay_http::query::{FieldMapping, QueryParams, WhereBuilder};
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
#[derive(Debug, Clone, FromRow)]
pub struct MediaVersionRow {
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

/// Raw DB representation of a media rendition.
#[derive(Debug, Clone, FromRow)]
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
    pub object_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Raw DB representation of a media usage record.
#[derive(Debug, Clone, FromRow)]
pub struct MediaUsageRow {
    pub id: Uuid,
    pub media_id: Uuid,
    pub used_by_type: String,
    pub used_by_id: Uuid,
    pub field: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Media with current version info for list views.
#[derive(Debug, Clone, FromRow)]
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
    pub thumbnail_object_key: Option<String>,
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
