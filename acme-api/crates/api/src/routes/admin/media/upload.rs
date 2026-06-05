use super::*;

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

    // Request upload URL from blob adapter
    let upload_request =
        UploadRequest::from_object_key(object_key, &req.content_type, req.content_length);
    let upload_plan = match state.blob_adapter.initiate_upload(upload_request).await {
        Ok(plan) => plan,
        Err(e) => {
            tracing::error!("Failed to initiate upload: {}", e);
            // Mark version as failed
            let _ = media::fail_media_version(pool, version_id).await;
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

    // Finalise with blob adapter to get actual metadata
    let stored = match state
        .blob_adapter
        .finalise_upload_object_key(&object_key)
        .await
    {
        Ok(s) => s,
        Err(e) => {
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

    // Check file size limit
    if stored.size > state.config.media.max_file_size_bytes {
        tracing::warn!(
            "Upload rejected: file size {} exceeds limit {}",
            stored.size,
            state.config.media.max_file_size_bytes
        );
        // Clean up: delete the uploaded blob and fail the version
        let _ = state.blob_adapter.delete_object_key(&object_key).await;
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
    let file_bytes = match state.blob_adapter.get_object_bytes(&object_key).await {
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
                let _ = state.blob_adapter.delete_object_key(&object_key).await;
                let _ = media::fail_media_version(pool, version_id).await;
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "media.content_type_mismatch",
                    format!(
                        "File content does not match declared type. Detected: {}, Declared: {}",
                        detected_mime, declared_mime
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
