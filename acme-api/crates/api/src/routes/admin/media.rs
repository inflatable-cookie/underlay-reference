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
use underlay_http::query::QueryParams;
use underlay_jobs::JobConfig;
use underlay_media::storage::version_key;
use uuid::Uuid;
use validator::Validate;

use acme_core::Uuid as AcmeUuid;
use acme_db::{activity, media};

use crate::dto::media::{
    CheckDuplicateRequest, CheckDuplicateResponse, CreateMediaRequest, FinaliseUploadRequest,
    FinaliseUploadResponse, InitiateUploadRequest, InitiateUploadResponse, MediaDetailDto,
    MediaSummaryDto, MediaUsageDto, MediaVersionDto, UpdateMediaRequest,
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
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "create",
                    resource_type: "media",
                    resource_id: media_id,
                    details: Some(json!({ "title": req.title, "kind": req.kind })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            let detail = MediaDetailDto::from_media(row, None, 0);
            (StatusCode::CREATED, Json(json!({ "data": detail }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// List all media items with filtering and sorting (admin).
///
/// GET /v1/admin/media
///
/// Supports filtering and sorting via query parameters:
/// - `sort=title:asc,updatedAt:desc`
/// - `filter[kind]=image`
/// - `filter[visibility]=public`
/// - `filter[title][like]=%search%`
pub async fn list_media(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::list_media_admin(pool, &query).await {
        Ok(rows) => {
            let items: Vec<MediaSummaryDto> = rows
                .into_iter()
                .map(|row| {
                    MediaSummaryDto::from_row_with_thumbnail(row, |key| {
                        state.blob_adapter.public_url(key)
                    })
                })
                .collect();
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
            let items: Vec<MediaSummaryDto> = response
                .data
                .into_iter()
                .map(|row| {
                    MediaSummaryDto::from_row_with_thumbnail(row, |key| {
                        state.blob_adapter.public_url(key)
                    })
                })
                .collect();
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
            let items: Vec<MediaSummaryDto> = rows
                .into_iter()
                .map(|row| {
                    MediaSummaryDto::from_row_with_thumbnail(row, |key| {
                        state.blob_adapter.public_url(key)
                    })
                })
                .collect();
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
                media::get_media_version(pool, version_id)
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            };

            // Get renditions for the current version
            let renditions = if let Some(version_id) = row.current_version_id {
                media::list_media_renditions(pool, version_id)
                    .await
                    .unwrap_or_default()
            } else {
                vec![]
            };

            // Get usage count
            let usage_count = media::get_media_usage_count(pool, media_id)
                .await
                .unwrap_or(0);

            let detail = MediaDetailDto::from_media_with_urls(
                row,
                current_version,
                renditions,
                usage_count,
                |key| state.blob_adapter.public_url(key),
            );
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
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "update",
                    resource_type: "media",
                    resource_id: media_id,
                    details: Some(json!({ "title": req.title })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            // Get current version if set
            let current_version = if let Some(version_id) = row.current_version_id {
                media::get_media_version(pool, version_id)
                    .await
                    .ok()
                    .flatten()
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
        Ok(()) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "delete",
                    resource_type: "media",
                    resource_id: media_id,
                    details: None,
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            Json(json!({ "ok": true })).into_response()
        }
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
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    match media::restore_media(pool, media_id).await {
        Ok(()) => {
            // Log activity
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(user.user_id.0.into_inner()),
                    action: "restore",
                    resource_type: "media",
                    resource_id: media_id,
                    details: None,
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            Json(json!({ "ok": true })).into_response()
        }
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

    // Check declared file size before initiating upload
    if req.content_length > state.config.media.max_file_size_bytes {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "media.file_too_large",
                    "message": format!(
                        "File size ({:.1} MB) exceeds maximum allowed size ({})",
                        req.content_length as f64 / (1024.0 * 1024.0),
                        state.config.media.max_file_size_display()
                    )
                }
            })),
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

    // Generate object key using standardized storage pattern
    // Use original filename if available, otherwise generate from content type
    let filename = media_row.original_filename.clone().unwrap_or_else(|| {
        let ext = underlay_media::storage::mime_to_extension(&req.content_type);
        format!("file.{}", ext)
    });
    let object_key = version_key(media_id, version_id, &filename);

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

    // Generate object key (same as in initiate) using standardized storage pattern
    // Use original filename if available, otherwise generate from content type
    let filename = media_row.original_filename.clone().unwrap_or_else(|| {
        let ext = underlay_media::storage::mime_to_extension(&req.content_type);
        format!("file.{}", ext)
    });
    let object_key = version_key(media_id, version_id, &filename);

    // Finalise with blob adapter to get actual metadata
    let stored = match state.blob_adapter.finalise_upload(&object_key).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to finalise upload: {}", e);
            let _ = media::fail_media_version(pool, version_id).await;
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Check file size limit
    if stored.size > state.config.media.max_file_size_bytes {
        tracing::warn!(
            "Upload rejected: file size {} exceeds limit {}",
            stored.size,
            state.config.media.max_file_size_bytes
        );
        // Clean up: delete the uploaded blob and fail the version
        let _ = state.blob_adapter.delete(&object_key).await;
        let _ = media::fail_media_version(pool, version_id).await;
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "media.file_too_large",
                    "message": format!(
                        "File size ({:.1} MB) exceeds maximum allowed size ({})",
                        stored.size as f64 / (1024.0 * 1024.0),
                        state.config.media.max_file_size_display()
                    )
                }
            })),
        )
            .into_response();
    }

    // Magic byte detection: verify file content matches declared MIME type
    let file_bytes = match state.blob_adapter.get_bytes(&object_key).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read file for magic byte detection: {}", e);
            // Continue without magic byte check - don't block upload for this
            vec![]
        }
    };

    if !file_bytes.is_empty() {
        // Use infer crate to detect actual file type
        if let Some(detected) = infer::get(&file_bytes) {
            let detected_mime = detected.mime_type();
            let declared_mime = &stored.content_type;

            // Check if types are compatible
            let types_match = match (detected_mime, declared_mime.as_str()) {
                // Exact match
                (d, s) if d == s => true,
                // JPEG variations
                ("image/jpeg", "image/jpg") | ("image/jpg", "image/jpeg") => true,
                // SVG is XML-based, infer might detect as text/xml
                ("text/xml", "image/svg+xml") | ("application/xml", "image/svg+xml") => true,
                // PDF check (exact match already handled above, but keep for explicitness)
                ("application/pdf", "application/pdf") => true,
                // Generic image type checks (same category is OK)
                (d, s) if d.starts_with("image/") && s.starts_with("image/") => {
                    // Log mismatch but allow - some formats are flexible
                    if d != s {
                        tracing::info!(
                            detected = detected_mime,
                            declared = declared_mime,
                            "Minor MIME type mismatch (allowed)"
                        );
                    }
                    true
                }
                // Different categories - reject
                _ => {
                    tracing::warn!(
                        detected = detected_mime,
                        declared = declared_mime,
                        "MIME type mismatch detected"
                    );
                    false
                }
            };

            if !types_match {
                // Clean up: delete the uploaded blob and fail the version
                let _ = state.blob_adapter.delete(&object_key).await;
                let _ = media::fail_media_version(pool, version_id).await;
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "ok": false,
                        "error": {
                            "code": "media.content_type_mismatch",
                            "message": format!(
                                "File content does not match declared type. Detected: {}, Declared: {}",
                                detected_mime,
                                declared_mime
                            )
                        }
                    })),
                )
                    .into_response();
            }
        }
        // If infer returns None, allow the upload (unknown format)
    }

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

    // Enqueue thumbnail generation job for images
    if stored.content_type.starts_with("image/") && stored.content_type != "image/svg+xml" {
        if let Some(job_repo) = &state.job_repository {
            let payload = json!({
                "media_id": media_id,
                "version_id": version_id
            });
            let config = JobConfig {
                max_attempts: 3,
                ..Default::default()
            };
            if let Err(e) = job_repo
                .create("media.generate_thumbnail", payload, &config)
                .await
            {
                // Log but don't fail the request - thumbnail is not critical
                tracing::warn!("Failed to enqueue thumbnail job: {}", e);
            } else {
                tracing::info!(
                    media_id = %media_id,
                    version_id = %version_id,
                    "Enqueued thumbnail generation job"
                );
            }
        }
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

    let detail =
        MediaDetailDto::from_media(updated_media, Some(finalised_version.clone()), usage_count);
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
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                let renditions = media::list_media_renditions(pool, row.id)
                    .await
                    .unwrap_or_default();
                let dto = MediaVersionDto::from_row_with_urls(
                    row,
                    renditions,
                    |key| state.blob_adapter.public_url(key),
                );
                items.push(dto);
            }
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

// ============================================================================
// Batch Operations
// ============================================================================

/// Request for batch delete operation.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteMediaRequest {
    pub ids: Vec<Uuid>,
}

/// Batch delete media items.
///
/// POST /v1/admin/media:batch-delete
pub async fn batch_delete_media(
    AdminUser(user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<BatchDeleteMediaRequest>,
) -> impl IntoResponse {
    if req.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "validation.empty_ids",
                    "message": "At least one ID is required"
                }
            })),
        )
            .into_response();
    }

    let pool = state.local_auth.pool();
    let actor_id = user.user_id.0.into_inner();

    match media::batch_soft_delete_media(pool, &req.ids, Some(actor_id)).await {
        Ok(count) => {
            // Log activity for batch operation
            let batch_id = AcmeUuid::new_v7().into_inner();
            let _ = activity::log_activity(
                pool,
                activity::LogActivityParams {
                    user_id: Some(actor_id),
                    action: "batch_delete",
                    resource_type: "media",
                    resource_id: batch_id,
                    details: Some(json!({ "count": count, "ids": req.ids })),
                    correlation_id: None,
                    ip_address: None,
                },
            )
            .await;

            Json(json!({ "ok": true, "deleted": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to batch delete media: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
