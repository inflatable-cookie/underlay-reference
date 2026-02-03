//! Media Library admin routes.
//!
//! Provides admin endpoints for managing media items, versions, and uploads.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use underlay_blob::UploadRequest;
use underlay_db::pagination::PaginationParams;
use uuid::Uuid;
use validator::Validate;

use acme_core::Uuid as AcmeUuid;
use acme_db::media;

use crate::dto::media::{
    CheckDuplicateRequest, CheckDuplicateResponse, CreateMediaRequest, FinaliseUploadRequest,
    FinaliseUploadResponse, InitiateUploadRequest, InitiateUploadResponse, MediaDetailDto,
    MediaListQuery, MediaSummaryDto, MediaUsageDto, MediaVersionDto, UpdateMediaRequest,
};
use crate::state::{AdminUser, AppState};

// ============================================================================
// Deduplication
// ============================================================================

/// Check if a file with the given hash already exists.
///
/// POST /v1/admin/media/check-duplicate
pub async fn check_duplicate(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CheckDuplicateRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.failed", "message": e.to_string() } })),
        )
            .into_response();
    }

    let pool = state.local_auth.pool();

    match media::find_media_by_hash(pool, &req.sha256).await {
        Ok(Some(row)) => {
            let summary: MediaSummaryDto = row.into();
            Json(CheckDuplicateResponse {
                exists: true,
                media: Some(summary),
            })
            .into_response()
        }
        Ok(None) => Json(CheckDuplicateResponse {
            exists: false,
            media: None,
        })
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to check for duplicate: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ============================================================================
// CRUD Operations
// ============================================================================

/// Create a new media item (metadata only, no file upload yet).
///
/// POST /v1/admin/media
pub async fn create_media(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateMediaRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.failed", "message": e.to_string() } })),
        )
            .into_response();
    }

    // Validate kind and visibility
    let Some(_kind) = req.media_kind() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.invalid_kind", "message": "Invalid media kind" } })),
        )
            .into_response();
    };

    let Some(_visibility) = req.media_visibility() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.invalid_visibility", "message": "Invalid visibility" } })),
        )
            .into_response();
    };

    let pool = state.local_auth.pool();
    let media_id = AcmeUuid::new_v7().into_inner();

    match media::create_media(
        pool,
        media_id,
        &req.kind,
        &req.visibility,
        &req.title,
        req.original_filename.as_deref(),
        Some(user.user_id.0.into_inner()),
    )
    .await
    {
        Ok(row) => {
            let detail = MediaDetailDto::from_media(row, None, 0);
            (StatusCode::CREATED, Json(json!({ "data": detail }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// List all media items (admin).
///
/// GET /v1/admin/media
pub async fn list_media(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(_query): Query<MediaListQuery>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::list_media_admin(pool).await {
        Ok(rows) => {
            let items: Vec<MediaSummaryDto> = rows.into_iter().map(Into::into).collect();
            Json(json!({ "data": items })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// List media items with pagination (admin).
///
/// GET /v1/admin/media/paginated
pub async fn list_media_paginated(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::list_media_admin_paginated(pool, params).await {
        Ok(response) => {
            let items: Vec<MediaSummaryDto> = response.data.into_iter().map(Into::into).collect();
            Json(json!({
                "data": items,
                "nextCursor": response.next_cursor,
                "prevCursor": response.prev_cursor,
                "hasMore": response.has_more,
                "total": response.total
            }))
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list media paginated: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// List soft-deleted media items (trash).
///
/// GET /v1/admin/media/trash
pub async fn list_media_trash(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::list_media_trash(pool).await {
        Ok(rows) => {
            let items: Vec<MediaSummaryDto> = rows.into_iter().map(Into::into).collect();
            Json(json!({ "data": items })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list media trash: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get a single media item.
///
/// GET /v1/admin/media/:media_id
pub async fn get_media(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::get_media_admin(pool, media_id).await {
        Ok(Some(row)) => {
            // Get current version if set
            let current_version = if let Some(version_id) = row.current_version_id {
                media::get_media_version(pool, version_id).await.ok().flatten()
            } else {
                None
            };

            // Get usage count
            let usage_count = media::get_media_usage_count(pool, media_id)
                .await
                .unwrap_or(0);

            let detail = MediaDetailDto::from_media(row, current_version, usage_count);
            Json(json!({ "data": detail })).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Update a media item.
///
/// PUT /v1/admin/media/:media_id
pub async fn update_media(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
    Json(req): Json<UpdateMediaRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.failed", "message": e.to_string() } })),
        )
            .into_response();
    }

    let Some(_visibility) = req.media_visibility() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.invalid_visibility", "message": "Invalid visibility" } })),
        )
            .into_response();
    };

    let pool = state.local_auth.pool();

    match media::update_media(
        pool,
        media_id,
        &req.title,
        req.original_filename.as_deref(),
        &req.visibility,
        Some(user.user_id.0.into_inner()),
    )
    .await
    {
        Ok(row) => {
            // Get current version if set
            let current_version = if let Some(version_id) = row.current_version_id {
                media::get_media_version(pool, version_id).await.ok().flatten()
            } else {
                None
            };

            // Get usage count
            let usage_count = media::get_media_usage_count(pool, media_id)
                .await
                .unwrap_or(0);

            let detail = MediaDetailDto::from_media(row, current_version, usage_count);
            Json(json!({ "data": detail })).into_response()
        }
        Err(e) if e.to_string().contains("no rows") => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to update media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Soft delete a media item.
///
/// POST /v1/admin/media/:media_id/soft-delete
pub async fn soft_delete_media(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::soft_delete_media(pool, media_id, Some(user.user_id.0.into_inner())).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to soft delete media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Restore a soft-deleted media item.
///
/// POST /v1/admin/media/:media_id/restore
pub async fn restore_media(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::restore_media(pool, media_id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to restore media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Hard delete (purge) a media item.
///
/// DELETE /v1/admin/media/:media_id
pub async fn purge_media(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    // Check if media is in use
    let usage_count = match media::get_media_usage_count(pool, media_id).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get usage count: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if usage_count > 0 {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "media.in_use",
                    "message": format!("Media is still in use ({} references)", usage_count)
                }
            })),
        )
            .into_response();
    }

    // Get all versions to delete blobs
    let versions = match media::list_media_versions(pool, media_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to list versions: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Delete blobs for all versions
    for version in &versions {
        if let Some(ref object_key) = version.object_key {
            if let Err(e) = state.blob_adapter.delete(object_key).await {
                tracing::warn!("Failed to delete blob {}: {}", object_key, e);
            }
        }
    }

    // Purge from database
    match media::purge_media(pool, media_id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to purge media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ============================================================================
// Upload Flow
// ============================================================================

/// Initiate an upload for a media item.
///
/// POST /v1/admin/media/:media_id/versions/initiate-upload
pub async fn initiate_upload(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
    Json(req): Json<InitiateUploadRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.failed", "message": e.to_string() } })),
        )
            .into_response();
    }

    let pool = state.local_auth.pool();

    // Verify media exists
    let media_row = match media::get_media_admin(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Create version in uploading state
    let version_id = AcmeUuid::new_v7().into_inner();
    let version = match media::create_media_version(
        pool,
        version_id,
        media_id,
        Some(user.user_id.0.into_inner()),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to create version: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Generate object key
    let ext = media_row
        .original_filename
        .as_ref()
        .and_then(|f| f.rsplit('.').next())
        .unwrap_or("bin");
    let object_key = format!("media/{}/versions/{}/file.{}", media_id, version_id, ext);

    // Request upload URL from blob adapter
    let upload_request = UploadRequest::new(&object_key, &req.content_type, req.content_length);
    let upload_plan = match state.blob_adapter.initiate_upload(upload_request).await {
        Ok(plan) => plan,
        Err(e) => {
            tracing::error!("Failed to initiate upload: {}", e);
            // Mark version as failed
            let _ = media::fail_media_version(pool, version_id).await;
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(InitiateUploadResponse {
        version_id: version.id,
        upload_plan,
    })
    .into_response()
}

/// Finalise an upload.
///
/// POST /v1/admin/media/:media_id/versions/:version_id/finalise-upload
pub async fn finalise_upload(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((media_id, version_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<FinaliseUploadRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "validation.failed", "message": e.to_string() } })),
        )
            .into_response();
    }

    let pool = state.local_auth.pool();

    // Verify version exists and belongs to this media
    let version = match media::get_media_version(pool, version_id).await {
        Ok(Some(v)) if v.media_id == media_id => v,
        Ok(Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": { "code": "version.wrong_media", "message": "Version does not belong to this media" } })),
            )
                .into_response()
        }
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if version.state != "uploading" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "version.not_uploading", "message": "Version is not in uploading state" } })),
        )
            .into_response();
    }

    // Get media for filename
    let media_row = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Generate object key (same as in initiate)
    let ext = media_row
        .original_filename
        .as_ref()
        .and_then(|f| f.rsplit('.').next())
        .unwrap_or("bin");
    let object_key = format!("media/{}/versions/{}/file.{}", media_id, version_id, ext);

    // Finalise with blob adapter to get actual metadata
    let stored = match state.blob_adapter.finalise_upload(&object_key).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to finalise upload: {}", e);
            let _ = media::fail_media_version(pool, version_id).await;
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Update version with storage info
    let finalised_version = match media::finalise_media_version(
        pool,
        version_id,
        stored.size as i64,
        &stored.content_type,
        &req.sha256,
        "local", // storage provider
        "media", // bucket
        &object_key,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to finalise version: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Set as current version
    if let Err(e) = media::set_current_version(pool, media_id, version_id).await {
        tracing::error!("Failed to set current version: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Get updated media
    let updated_media = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get updated media: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let usage_count = media::get_media_usage_count(pool, media_id)
        .await
        .unwrap_or(0);

    let detail = MediaDetailDto::from_media(updated_media, Some(finalised_version.clone()), usage_count);
    let version_dto = MediaVersionDto::from(finalised_version);

    Json(FinaliseUploadResponse {
        media: detail,
        version: version_dto,
    })
    .into_response()
}

// ============================================================================
// Version Management
// ============================================================================

/// List all versions for a media item.
///
/// GET /v1/admin/media/:media_id/versions
pub async fn list_versions(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::list_media_versions(pool, media_id).await {
        Ok(rows) => {
            let items: Vec<MediaVersionDto> = rows.into_iter().map(Into::into).collect();
            Json(json!({ "data": items })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list versions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Set a version as the current version.
///
/// POST /v1/admin/media/:media_id/versions/:version_id/activate
pub async fn activate_version(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((media_id, version_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    // Verify version exists, belongs to media, and is ready
    let version = match media::get_media_version(pool, version_id).await {
        Ok(Some(v)) if v.media_id == media_id => v,
        Ok(Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": { "code": "version.wrong_media", "message": "Version does not belong to this media" } })),
            )
                .into_response()
        }
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if version.state != "ready" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "version.not_ready", "message": "Version is not ready" } })),
        )
            .into_response();
    }

    // Check if already current
    let media_row = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if media_row.current_version_id == Some(version_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "version.already_current", "message": "Version is already the current version" } })),
        )
            .into_response();
    }

    match media::set_current_version(pool, media_id, version_id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to set current version: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Delete a specific version.
///
/// DELETE /v1/admin/media/:media_id/versions/:version_id
pub async fn delete_version(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((media_id, version_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    // Verify version exists and belongs to media
    let version = match media::get_media_version(pool, version_id).await {
        Ok(Some(v)) if v.media_id == media_id => v,
        Ok(Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": { "code": "version.wrong_media", "message": "Version does not belong to this media" } })),
            )
                .into_response()
        }
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Can't delete current version
    let media_row = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if media_row.current_version_id == Some(version_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": { "code": "version.is_current", "message": "Cannot delete the current version" } })),
        )
            .into_response();
    }

    // Delete blob if exists
    if let Some(ref object_key) = version.object_key {
        if let Err(e) = state.blob_adapter.delete(object_key).await {
            tracing::warn!("Failed to delete blob {}: {}", object_key, e);
        }
    }

    match media::delete_media_version(pool, version_id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete version: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ============================================================================
// Usage Tracking
// ============================================================================

/// List all usages for a media item.
///
/// GET /v1/admin/media/:media_id/usage
pub async fn list_usage(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::list_media_usages(pool, media_id).await {
        Ok(rows) => {
            let items: Vec<MediaUsageDto> = rows.into_iter().map(Into::into).collect();
            Json(json!({ "data": items })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list usage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
