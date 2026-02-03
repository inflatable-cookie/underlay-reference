//! Media Library DTOs.
//!
//! Data transfer objects for media library operations including upload,
//! metadata management, and usage tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use underlay_blob::UploadPlan;
use underlay_db::{MediaKind, MediaVisibility};
use uuid::Uuid;
use validator::Validate;

use acme_db::media::{
    MediaRenditionRow, MediaRow, MediaUsageRow, MediaVersionRow, MediaWithVersionRow,
};

// ============================================================================
// Media DTOs
// ============================================================================

/// Media item summary for list views.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSummaryDto {
    pub id: Uuid,
    pub kind: String,
    pub visibility: String,
    pub title: String,
    pub original_filename: Option<String>,
    pub current_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
    /// URL to thumbnail image (if available).
    pub thumbnail_url: Option<String>,
}

impl MediaSummaryDto {
    /// Create a summary DTO from a row with a thumbnail URL generator.
    pub fn from_row_with_thumbnail<F>(row: MediaWithVersionRow, url_fn: F) -> Self
    where
        F: FnOnce(&str) -> String,
    {
        let thumbnail_url = row.thumbnail_object_key.as_ref().map(|key| url_fn(key));
        Self {
            id: row.id,
            kind: row.kind,
            visibility: row.visibility,
            title: row.title,
            original_filename: row.original_filename,
            current_version_id: row.current_version_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            byte_size: row.byte_size,
            mime_type: row.mime_type,
            thumbnail_url,
        }
    }
}

impl From<MediaWithVersionRow> for MediaSummaryDto {
    fn from(m: MediaWithVersionRow) -> Self {
        Self {
            id: m.id,
            kind: m.kind,
            visibility: m.visibility,
            title: m.title,
            original_filename: m.original_filename,
            current_version_id: m.current_version_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            byte_size: m.byte_size,
            mime_type: m.mime_type,
            thumbnail_url: None, // Use from_row_with_thumbnail for URL generation
        }
    }
}

impl From<MediaRow> for MediaSummaryDto {
    fn from(m: MediaRow) -> Self {
        Self {
            id: m.id,
            kind: m.kind,
            visibility: m.visibility,
            title: m.title,
            original_filename: m.original_filename,
            current_version_id: m.current_version_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            byte_size: None,
            mime_type: None,
            thumbnail_url: None,
        }
    }
}

/// Full media item detail.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDetailDto {
    pub id: Uuid,
    pub kind: String,
    pub visibility: String,
    pub title: String,
    pub original_filename: Option<String>,
    pub current_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// Current version details (if available).
    pub current_version: Option<MediaVersionDto>,
    /// Total usage count.
    pub usage_count: i64,
}

impl MediaDetailDto {
    pub fn from_media(
        m: MediaRow,
        current_version: Option<MediaVersionRow>,
        usage_count: i64,
    ) -> Self {
        Self {
            id: m.id,
            kind: m.kind,
            visibility: m.visibility,
            title: m.title,
            original_filename: m.original_filename,
            current_version_id: m.current_version_id,
            created_at: m.created_at,
            created_by: m.created_by,
            updated_at: m.updated_at,
            deleted_at: m.deleted_at,
            current_version: current_version.map(MediaVersionDto::from),
            usage_count,
        }
    }
}

// ============================================================================
// Media Version DTOs
// ============================================================================

/// Media version DTO.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaVersionDto {
    pub id: Uuid,
    pub media_id: Uuid,
    pub state: String,
    pub byte_size: Option<i64>,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub storage_provider: Option<String>,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

impl From<MediaVersionRow> for MediaVersionDto {
    fn from(v: MediaVersionRow) -> Self {
        Self {
            id: v.id,
            media_id: v.media_id,
            state: v.state,
            byte_size: v.byte_size,
            mime_type: v.mime_type,
            sha256: v.sha256,
            storage_provider: v.storage_provider,
            bucket: v.bucket,
            object_key: v.object_key,
            created_at: v.created_at,
            created_by: v.created_by,
        }
    }
}

// ============================================================================
// Media Usage DTOs
// ============================================================================

/// Media usage record DTO.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaUsageDto {
    pub id: Uuid,
    pub media_id: Uuid,
    pub used_by_type: String,
    pub used_by_id: Uuid,
    pub field: String,
    pub created_at: DateTime<Utc>,
}

impl From<MediaUsageRow> for MediaUsageDto {
    fn from(u: MediaUsageRow) -> Self {
        Self {
            id: u.id,
            media_id: u.media_id,
            used_by_type: u.used_by_type,
            used_by_id: u.used_by_id,
            field: u.field,
            created_at: u.created_at,
        }
    }
}

// ============================================================================
// Media Rendition DTOs
// ============================================================================

/// Media rendition DTO.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRenditionDto {
    pub id: Uuid,
    pub media_version_id: Uuid,
    pub kind: String,
    pub byte_size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<MediaRenditionRow> for MediaRenditionDto {
    fn from(r: MediaRenditionRow) -> Self {
        Self {
            id: r.id,
            media_version_id: r.media_version_id,
            kind: r.kind,
            byte_size: r.byte_size,
            mime_type: r.mime_type,
            width: r.width,
            height: r.height,
            created_at: r.created_at,
        }
    }
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new media item.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateMediaRequest {
    /// Media kind: "image" or "pdf".
    #[validate(length(min = 1, max = 20))]
    pub kind: String,

    /// Visibility: "public" or "restricted".
    #[validate(length(min = 1, max = 20))]
    pub visibility: String,

    /// Human-readable title.
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    /// Original filename from upload.
    #[validate(length(max = 255))]
    pub original_filename: Option<String>,
}

impl CreateMediaRequest {
    /// Parse the kind into a MediaKind enum.
    pub fn media_kind(&self) -> Option<MediaKind> {
        self.kind.parse().ok()
    }

    /// Parse the visibility into a MediaVisibility enum.
    pub fn media_visibility(&self) -> Option<MediaVisibility> {
        self.visibility.parse().ok()
    }
}

/// Request to update a media item.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMediaRequest {
    /// Human-readable title.
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    /// Original filename (for downloads). Optional - if None, keeps current value.
    #[validate(length(max = 255))]
    pub original_filename: Option<String>,

    /// Visibility: "public" or "restricted".
    #[validate(length(min = 1, max = 20))]
    pub visibility: String,
}

impl UpdateMediaRequest {
    /// Parse the visibility into a MediaVisibility enum.
    pub fn media_visibility(&self) -> Option<MediaVisibility> {
        self.visibility.parse().ok()
    }
}

/// Request to check for duplicate media by hash.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CheckDuplicateRequest {
    /// SHA-256 hash of the file (hex-encoded, 64 characters).
    #[validate(length(equal = 64))]
    pub sha256: String,
}

/// Response for duplicate check.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDuplicateResponse {
    /// Whether a duplicate was found.
    pub exists: bool,
    /// The existing media item if found.
    pub media: Option<MediaSummaryDto>,
}

/// Request to initiate an upload.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct InitiateUploadRequest {
    /// Expected content type (MIME type).
    #[validate(length(min = 1, max = 100))]
    pub content_type: String,

    /// Expected file size in bytes.
    pub content_length: u64,
}

/// Response for upload initiation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiateUploadResponse {
    /// The version ID for this upload.
    pub version_id: Uuid,
    /// The upload plan with pre-signed URL and constraints.
    pub upload_plan: UploadPlan,
}

/// Request to finalise an upload.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct FinaliseUploadRequest {
    /// SHA-256 hash of the uploaded file (hex-encoded, 64 characters).
    #[validate(length(equal = 64))]
    pub sha256: String,
}

/// Response for upload finalisation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinaliseUploadResponse {
    /// The finalised media item.
    pub media: MediaDetailDto,
    /// The finalised version.
    pub version: MediaVersionDto,
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for listing media.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaListQuery {
    /// Filter by kind ("image", "pdf").
    pub kind: Option<String>,
    /// Filter by visibility ("public", "restricted").
    pub visibility: Option<String>,
    /// Search query (matches title).
    pub q: Option<String>,
    /// Include deleted items.
    #[serde(default)]
    pub include_deleted: bool,
    /// Only show unused items.
    #[serde(default)]
    pub unused_only: bool,
}
