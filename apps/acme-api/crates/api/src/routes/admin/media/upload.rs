use super::*;
use underlay_blob::{BlobAdapterUploadExt, BlobError};

fn file_too_large_error(size: u64, state: &AppState) -> ApiError {
    ApiError::new(
        StatusCode::PAYLOAD_TOO_LARGE,
        "media.file_too_large",
        format!(
            "File size ({:.1} MB) exceeds maximum allowed size ({})",
            size as f64 / (1024.0 * 1024.0),
            state.config.media.max_file_size_display()
        ),
    )
}

fn content_type_not_allowed_error(content_type: &str) -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "media.content_type_not_allowed",
        format!("Content type '{content_type}' is not allowed"),
    )
}

/// Map a rejection from the foundation's validated upload helpers to an API
/// error, or `None` when the failure is an internal/storage error.
fn blob_rejection_to_api_error(err: &BlobError, state: &AppState) -> Option<ApiError> {
    match err {
        BlobError::TooLarge(size, _) => Some(file_too_large_error(*size, state)),
        BlobError::InvalidContentType(detail) => Some(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "media.content_type_mismatch",
            format!("Upload rejected: {detail}"),
        )),
        _ => None,
    }
}

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

    // Pre-check the declared size and content type against the foundation
    // upload policy before creating any rows. `initiate_upload_validated`
    // re-enforces both before signing the upload URL.
    if !state.config.media.is_size_allowed(req.content_length) {
        return Err(file_too_large_error(req.content_length, &state));
    }
    if !state
        .config
        .media
        .is_content_type_allowed(&req.content_type)
    {
        return Err(content_type_not_allowed_error(&req.content_type));
    }

    let pool = state.local_auth.pool();

    // Verify media exists
    let media_row = match media::get_media_admin(pool, media_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return Err(ApiError::not_found(
                "media.not_found",
                "Media item not found",
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.get_failed",
                "Failed to initiate upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.initiate_upload",
                "media_id": media_id
            })));
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
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.version_create_failed",
                "Failed to initiate upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.initiate_upload",
                "media_id": media_id,
                "version_id": version_id
            })));
        }
    };

    // Generate object key using standardized storage pattern
    // Use original filename if available, otherwise generate from content type
    let filename = media_row.original_filename.clone().unwrap_or_else(|| {
        let ext = underlay_media::storage::mime_to_extension(&req.content_type);
        format!("file.{}", ext)
    });
    let object_key = version_object_key(media_id, version_id, &filename)
        .map_err(|e| ApiError::bad_request("media.invalid_object_key", e.to_string()))?;

    // Request upload URL from the blob adapter through the foundation's
    // validated helper (size cap + MIME allowlist enforced before signing).
    let upload_request =
        UploadRequest::from_object_key(object_key, &req.content_type, req.content_length);
    let upload_plan = match state
        .blob_adapter
        .initiate_upload_validated(upload_request, &state.config.media)
        .await
    {
        Ok(plan) => plan,
        Err(e) => {
            // Mark version as failed
            let _ = media::fail_media_version(pool, version_id).await;

            if let Some(rejection) = blob_rejection_to_api_error(&e, &state) {
                return Err(rejection);
            }

            tracing::error!("Failed to initiate upload: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.upload_initiate_failed",
                "Failed to initiate upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.initiate_upload",
                "media_id": media_id,
                "version_id": version_id
            })));
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
        Ok(Some(_)) => {
            return Err(ApiError::bad_request(
                "version.wrong_media",
                "Version does not belong to this media",
            ));
        }
        Ok(None) => {
            return Err(ApiError::not_found(
                "version.not_found",
                "Version not found",
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.version_get_failed",
                "Failed to finalise upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.finalise_upload",
                "media_id": media_id,
                "version_id": version_id
            })));
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
        Ok(None) => {
            return Err(ApiError::not_found(
                "media.not_found",
                "Media item not found",
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get media: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.get_failed",
                "Failed to finalise upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.finalise_upload",
                "media_id": media_id,
                "version_id": version_id
            })));
        }
    };

    // Generate object key (same as in initiate) using standardized storage pattern
    // Use original filename if available, otherwise generate from content type
    let filename = media_row.original_filename.clone().unwrap_or_else(|| {
        let ext = underlay_media::storage::mime_to_extension(&req.content_type);
        format!("file.{}", ext)
    });
    let object_key = version_object_key(media_id, version_id, &filename)
        .map_err(|e| ApiError::bad_request("media.invalid_object_key", e.to_string()))?;

    // Finalise through the foundation's verified helper: it enforces the
    // size cap, the MIME allowlist, and magic-byte verification of the
    // stored bytes against the declared content type in one place, then pins
    // the content type to the validated declared value.
    let stored = match state
        .blob_adapter
        .finalise_upload_verified(object_key.as_str(), &req.content_type, &state.config.media)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            if let Some(rejection) = blob_rejection_to_api_error(&e, &state) {
                // Policy rejection: clean up the stored object and fail the
                // version, then surface the 4xx.
                let _ = state.blob_adapter.delete_object_key(&object_key).await;
                let _ = media::fail_media_version(pool, version_id).await;
                return Err(rejection);
            }

            tracing::error!("Failed to finalise upload: {}", e);
            let _ = media::fail_media_version(pool, version_id).await;
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.upload_finalise_failed",
                "Failed to finalise upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.finalise_upload",
                "media_id": media_id,
                "version_id": version_id
            })));
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
        object_key.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to finalise version: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.version_finalise_failed",
                "Failed to finalise upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.finalise_upload",
                "media_id": media_id,
                "version_id": version_id
            })));
        }
    };

    // Set as current version
    if let Err(e) = media::set_current_version(pool, media_id, version_id).await {
        tracing::error!("Failed to set current version: {}", e);
        return Err(crate::db_errors::internal_with_diagnostics(
            "media.set_current_version_failed",
            "Failed to finalise upload",
            &e,
        )
        .with_context(json!({
            "operation": "media.finalise_upload",
            "media_id": media_id,
            "version_id": version_id
        })));
    }

    // Enqueue thumbnail generation job for images
    if stored.content_type.starts_with("image/") && stored.content_type != "image/svg+xml" {
        if let Some(job_repo) = &state.job_repository {
            let payload = json!({
                "media_id": media_id,
                "version_id": version_id
            });
            let config = JobConfig::new().with_max_attempts(3);
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
        Ok(None) => {
            return Err(ApiError::not_found(
                "media.not_found",
                "Media item not found",
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get updated media: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.get_failed",
                "Failed to finalise upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.finalise_upload",
                "media_id": media_id,
                "version_id": version_id
            })));
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
