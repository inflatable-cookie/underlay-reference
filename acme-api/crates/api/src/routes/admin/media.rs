//! Media Library admin routes.
//!
//! Provides admin endpoints for managing media items, versions, and uploads.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use underlay_blob::UploadRequest;
use underlay_db::pagination::PaginationParams;
use underlay_http::{query::QueryParams, ApiError};
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
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::bad_request("validation.failed", e.to_string()));
    }

    let pool = state.local_auth.pool();

    match media::find_media_by_hash(pool, &req.sha256).await {
        Ok(Some(row)) => {
            let summary: MediaSummaryDto = row.into();
            Ok(Json(CheckDuplicateResponse {
                exists: true,
                media: Some(summary),
            })
            .into_response())
        }
        Ok(None) => Ok(Json(CheckDuplicateResponse {
            exists: false,
            media: None,
        })
        .into_response()),
        Err(e) => {
            tracing::error!("Failed to check for duplicate: {}", e);
            Err(
                ApiError::internal("media.duplicate_check_failed", "Failed to check duplicate")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.check_duplicate",
                        "sha256": &req.sha256
                    })),
            )
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
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::bad_request("validation.failed", e.to_string()));
    }

    // Validate kind and visibility
    let Some(_kind) = req.media_kind() else {
        return Err(ApiError::bad_request(
            "validation.invalid_kind",
            "Invalid media kind",
        ));
    };

    let Some(_visibility) = req.media_visibility() else {
        return Err(ApiError::bad_request(
            "validation.invalid_visibility",
            "Invalid visibility",
        ));
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
            Ok((StatusCode::CREATED, Json(json!({ "data": detail }))).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create media: {}", e);
            Err(
                ApiError::internal("media.create_failed", "Failed to create media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.create",
                        "media_id": media_id,
                        "kind": &req.kind
                    })),
            )
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
) -> Result<Response, ApiError> {
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
            Ok(Json(json!({ "data": items })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list media: {}", e);
            Err(
                ApiError::internal("media.list_failed", "Failed to list media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.list"
                    })),
            )
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
) -> Result<Response, ApiError> {
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
            Ok(Json(json!({
                "data": items,
                "nextCursor": response.next_cursor,
                "prevCursor": response.prev_cursor,
                "hasMore": response.has_more,
                "total": response.total
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list media paginated: {}", e);
            Err(
                ApiError::internal("media.list_paginated_failed", "Failed to list media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.list_paginated"
                    })),
            )
        }
    }
}

/// List soft-deleted media items (trash).
///
/// GET /v1/admin/media/trash
pub async fn list_media_trash(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
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
            Ok(Json(json!({ "data": items })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list media trash: {}", e);
            Err(
                ApiError::internal("media.list_trash_failed", "Failed to list media trash")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.list_trash"
                    })),
            )
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
) -> Result<Response, ApiError> {
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
            Ok(Json(json!({ "data": detail })).into_response())
        }
        Ok(None) => Err(
            ApiError::not_found("media.not_found", "Media item not found").with_context(json!({
                "operation": "media.get",
                "media_id": media_id
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            Err(
                ApiError::internal("media.get_failed", "Failed to get media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.get",
                        "media_id": media_id
                    })),
            )
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
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::bad_request("validation.failed", e.to_string()));
    }

    let Some(_visibility) = req.media_visibility() else {
        return Err(ApiError::bad_request(
            "validation.invalid_visibility",
            "Invalid visibility",
        ));
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
            Ok(Json(json!({ "data": detail })).into_response())
        }
        Err(e) if e.to_string().contains("no rows") => Err(
            ApiError::not_found("media.not_found", "Media item not found").with_context(json!({
                "operation": "media.update",
                "media_id": media_id
            })),
        ),
        Err(e) => {
            tracing::error!("Failed to update media: {}", e);
            Err(
                ApiError::internal("media.update_failed", "Failed to update media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.update",
                        "media_id": media_id
                    })),
            )
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
) -> Result<Response, ApiError> {
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

            Ok(Json(json!({ "ok": true })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to soft delete media: {}", e);
            Err(
                ApiError::internal("media.soft_delete_failed", "Failed to delete media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.soft_delete",
                        "media_id": media_id
                    })),
            )
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
) -> Result<Response, ApiError> {
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

            Ok(Json(json!({ "ok": true })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to restore media: {}", e);
            Err(
                ApiError::internal("media.restore_failed", "Failed to restore media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.restore",
                        "media_id": media_id
                    })),
            )
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    // Check if media is in use
    let usage_count = match media::get_media_usage_count(pool, media_id).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get usage count: {}", e);
            return Err(
                ApiError::internal("media.usage_count_failed", "Failed to purge media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.purge",
                        "media_id": media_id
                    })),
            );
        }
    };

    if usage_count > 0 {
        return Err(ApiError::conflict(
            "media.in_use",
            format!("Media is still in use ({} references)", usage_count),
        ));
    }

    // Get all versions to delete blobs
    let versions = match media::list_media_versions(pool, media_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to list versions: {}", e);
            return Err(
                ApiError::internal("media.versions_list_failed", "Failed to purge media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.purge",
                        "media_id": media_id
                    })),
            );
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
        Ok(()) => Ok(Json(json!({ "ok": true })).into_response()),
        Err(e) => {
            tracing::error!("Failed to purge media: {}", e);
            Err(
                ApiError::internal("media.purge_failed", "Failed to purge media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.purge",
                        "media_id": media_id
                    })),
            )
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
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::bad_request("validation.failed", e.to_string()));
    }

    // Check declared file size before initiating upload
    if req.content_length > state.config.media.max_file_size_bytes {
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "media.file_too_large",
            format!(
                "File size ({:.1} MB) exceeds maximum allowed size ({})",
                req.content_length as f64 / (1024.0 * 1024.0),
                state.config.media.max_file_size_display()
            ),
        ));
    }

    let pool = state.local_auth.pool();

    // Verify media exists
    let media_row = match media::get_media_admin(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(ApiError::not_found("media.not_found", "Media item not found")),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return Err(
                ApiError::internal("media.get_failed", "Failed to initiate upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.initiate_upload",
                        "media_id": media_id
                    })),
            );
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
            return Err(
                ApiError::internal("media.version_create_failed", "Failed to initiate upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.initiate_upload",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
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
            return Err(
                ApiError::internal("media.upload_initiate_failed", "Failed to initiate upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.initiate_upload",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    Ok(Json(InitiateUploadResponse {
        version_id: version.id,
        upload_plan,
    })
    .into_response())
}

/// Finalise an upload.
///
/// POST /v1/admin/media/:media_id/versions/:version_id/finalise-upload
pub async fn finalise_upload(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((media_id, version_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<FinaliseUploadRequest>,
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::bad_request("validation.failed", e.to_string()));
    }

    let pool = state.local_auth.pool();

    // Verify version exists and belongs to this media
    let version = match media::get_media_version(pool, version_id).await {
        Ok(Some(v)) if v.media_id == media_id => v,
        Ok(Some(_)) => return Err(ApiError::bad_request("version.wrong_media", "Version does not belong to this media")),
        Ok(None) => return Err(ApiError::not_found("version.not_found", "Version not found")),
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return Err(
                ApiError::internal("media.version_get_failed", "Failed to finalise upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.finalise_upload",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    if version.state != "uploading" {
        return Err(ApiError::bad_request(
            "version.not_uploading",
            "Version is not in uploading state",
        ));
    }

    // Get media for filename
    let media_row = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(ApiError::not_found("media.not_found", "Media item not found")),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return Err(
                ApiError::internal("media.get_failed", "Failed to finalise upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.finalise_upload",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
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
            return Err(
                ApiError::internal("media.upload_finalise_failed", "Failed to finalise upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.finalise_upload",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
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
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "media.file_too_large",
            format!(
                "File size ({:.1} MB) exceeds maximum allowed size ({})",
                stored.size as f64 / (1024.0 * 1024.0),
                state.config.media.max_file_size_display()
            ),
        ));
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
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "media.content_type_mismatch",
                    format!(
                        "File content does not match declared type. Detected: {}, Declared: {}",
                        detected_mime,
                        declared_mime
                    ),
                ));
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
            return Err(
                ApiError::internal("media.version_finalise_failed", "Failed to finalise upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.finalise_upload",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    // Set as current version
    if let Err(e) = media::set_current_version(pool, media_id, version_id).await {
        tracing::error!("Failed to set current version: {}", e);
        return Err(
            ApiError::internal("media.set_current_version_failed", "Failed to finalise upload")
                .with_cause(&e)
                .with_context(json!({
                    "operation": "media.finalise_upload",
                    "media_id": media_id,
                    "version_id": version_id
                })),
        );
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
        Ok(None) => return Err(ApiError::not_found("media.not_found", "Media item not found")),
        Err(e) => {
            tracing::error!("Failed to get updated media: {}", e);
            return Err(
                ApiError::internal("media.get_failed", "Failed to finalise upload")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.finalise_upload",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    let usage_count = media::get_media_usage_count(pool, media_id)
        .await
        .unwrap_or(0);

    let detail =
        MediaDetailDto::from_media(updated_media, Some(finalised_version.clone()), usage_count);
    let version_dto = MediaVersionDto::from(finalised_version);

    Ok(Json(FinaliseUploadResponse {
        media: detail,
        version: version_dto,
    })
    .into_response())
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
) -> Result<Response, ApiError> {
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
            Ok(Json(json!({ "data": items })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list versions: {}", e);
            Err(
                ApiError::internal("media.list_versions_failed", "Failed to list versions")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.list_versions",
                        "media_id": media_id
                    })),
            )
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    // Verify version exists, belongs to media, and is ready
    let version = match media::get_media_version(pool, version_id).await {
        Ok(Some(v)) if v.media_id == media_id => v,
        Ok(Some(_)) => return Err(ApiError::bad_request("version.wrong_media", "Version does not belong to this media")),
        Ok(None) => return Err(ApiError::not_found("version.not_found", "Version not found")),
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return Err(
                ApiError::internal("media.version_get_failed", "Failed to activate version")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.activate_version",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    if version.state != "ready" {
        return Err(ApiError::bad_request("version.not_ready", "Version is not ready"));
    }

    // Check if already current
    let media_row = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(ApiError::not_found("media.not_found", "Media item not found")),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return Err(
                ApiError::internal("media.get_failed", "Failed to activate version")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.activate_version",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    if media_row.current_version_id == Some(version_id) {
        return Err(ApiError::bad_request(
            "version.already_current",
            "Version is already the current version",
        ));
    }

    match media::set_current_version(pool, media_id, version_id).await {
        Ok(()) => Ok(Json(json!({ "ok": true })).into_response()),
        Err(e) => {
            tracing::error!("Failed to set current version: {}", e);
            Err(
                ApiError::internal("media.set_current_version_failed", "Failed to activate version")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.activate_version",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            )
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    // Verify version exists and belongs to media
    let version = match media::get_media_version(pool, version_id).await {
        Ok(Some(v)) if v.media_id == media_id => v,
        Ok(Some(_)) => return Err(ApiError::bad_request("version.wrong_media", "Version does not belong to this media")),
        Ok(None) => return Err(ApiError::not_found("version.not_found", "Version not found")),
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return Err(
                ApiError::internal("media.version_get_failed", "Failed to delete version")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.delete_version",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    // Can't delete current version
    let media_row = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(ApiError::not_found("media.not_found", "Media item not found")),
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return Err(
                ApiError::internal("media.get_failed", "Failed to delete version")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.delete_version",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            );
        }
    };

    if media_row.current_version_id == Some(version_id) {
        return Err(ApiError::bad_request(
            "version.is_current",
            "Cannot delete the current version",
        ));
    }

    // Delete blob if exists
    if let Some(ref object_key) = version.object_key {
        if let Err(e) = state.blob_adapter.delete(object_key).await {
            tracing::warn!("Failed to delete blob {}: {}", object_key, e);
        }
    }

    match media::delete_media_version(pool, version_id).await {
        Ok(()) => Ok(Json(json!({ "ok": true })).into_response()),
        Err(e) => {
            tracing::error!("Failed to delete version: {}", e);
            Err(
                ApiError::internal("media.delete_version_failed", "Failed to delete version")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.delete_version",
                        "media_id": media_id,
                        "version_id": version_id
                    })),
            )
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
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    match media::list_media_usages(pool, media_id).await {
        Ok(rows) => {
            let items: Vec<MediaUsageDto> = rows.into_iter().map(Into::into).collect();
            Ok(Json(json!({ "data": items })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list usage: {}", e);
            Err(
                ApiError::internal("media.list_usage_failed", "Failed to list media usage")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.list_usage",
                        "media_id": media_id
                    })),
            )
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
) -> Result<Response, ApiError> {
    if req.ids.is_empty() {
        return Err(ApiError::bad_request(
            "validation.empty_ids",
            "At least one ID is required",
        ));
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

            Ok(Json(json!({ "ok": true, "deleted": count })).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to batch delete media: {}", e);
            Err(
                ApiError::internal("media.batch_delete_failed", "Failed to batch delete media")
                    .with_cause(&e)
                    .with_context(json!({
                        "operation": "media.batch_delete",
                        "count": req.ids.len()
                    })),
            )
        }
    }
}
