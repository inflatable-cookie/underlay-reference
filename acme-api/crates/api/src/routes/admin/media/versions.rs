use super::*;

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
                let dto = MediaVersionDto::from_row_with_urls(row, renditions, |key| {
                    state.blob_adapter.public_url(key)
                });
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
        Ok(Some(_)) => {
            return Err(ApiError::bad_request(
                "version.wrong_media",
                "Version does not belong to this media",
            ))
        }
        Ok(None) => {
            return Err(ApiError::not_found(
                "version.not_found",
                "Version not found",
            ))
        }
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return Err(ApiError::internal(
                "media.version_get_failed",
                "Failed to activate version",
            )
            .with_cause(&e)
            .with_context(json!({
                "operation": "media.activate_version",
                "media_id": media_id,
                "version_id": version_id
            })));
        }
    };

    if version.state != "ready" {
        return Err(ApiError::bad_request(
            "version.not_ready",
            "Version is not ready",
        ));
    }

    // Check if already current
    let media_row = match media::get_media(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return Err(ApiError::not_found(
                "media.not_found",
                "Media item not found",
            ))
        }
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
            Err(ApiError::internal(
                "media.set_current_version_failed",
                "Failed to activate version",
            )
            .with_cause(&e)
            .with_context(json!({
                "operation": "media.activate_version",
                "media_id": media_id,
                "version_id": version_id
            })))
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
        Ok(Some(_)) => {
            return Err(ApiError::bad_request(
                "version.wrong_media",
                "Version does not belong to this media",
            ))
        }
        Ok(None) => {
            return Err(ApiError::not_found(
                "version.not_found",
                "Version not found",
            ))
        }
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
        Ok(None) => {
            return Err(ApiError::not_found(
                "media.not_found",
                "Media item not found",
            ))
        }
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
