use super::*;

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
                "next_cursor": response.next_cursor,
                "prev_cursor": response.prev_cursor,
                "has_more": response.has_more,
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
        Err(e) if e.to_string().contains("no rows") => Err(ApiError::not_found(
            "media.not_found",
            "Media item not found",
        )
        .with_context(json!({
            "operation": "media.update",
            "media_id": media_id
        }))),
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
