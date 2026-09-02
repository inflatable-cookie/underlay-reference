use super::*;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use underlay_blob::{
    BlobAdapter, BlobAdapterPromotionExt, BlobAdapterUploadExt, BlobError, BlobObjectKey,
    BlobUploadConfig, StoredObject, VerifiedPromotionResult,
};

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
        BlobError::DestinationExists(key) => Some(ApiError::new(
            StatusCode::CONFLICT,
            "media.destination_exists",
            format!("Published object already exists: {key}"),
        )),
        BlobError::Unsupported(detail) => Some(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "media.source_unsupported",
            format!("Upload source cannot be published: {detail}"),
        )),
        BlobError::NotFound(_) => Some(ApiError::bad_request(
            "media.staging_not_found",
            "Uploaded object was not found",
        )),
        BlobError::InvalidKey(detail) => Some(ApiError::bad_request(
            "media.invalid_object_key",
            detail.clone(),
        )),
        _ => None,
    }
}

/// Distinct published key derived from the client upload (staging) key.
fn published_object_key(staging: &BlobObjectKey) -> Result<BlobObjectKey, String> {
    let staging_key = staging.as_str();
    let (parent, name) = staging_key
        .rsplit_once('/')
        .ok_or_else(|| "staging object key has no parent path".to_string())?;
    BlobObjectKey::parse(format!("{parent}/published/{name}")).map_err(|e| e.to_string())
}

#[async_trait]
trait ReadyCurrentStore: Send + Sync {
    async fn activate_ready_current(
        &self,
        media_id: Uuid,
        version_id: Uuid,
        promoted: &VerifiedPromotionResult,
    ) -> Result<acme_db::media::MediaVersionRow, sqlx::Error>;
}

struct PoolStore<'a>(&'a acme_db::DbPool);

#[async_trait]
impl ReadyCurrentStore for PoolStore<'_> {
    async fn activate_ready_current(
        &self,
        media_id: Uuid,
        version_id: Uuid,
        promoted: &VerifiedPromotionResult,
    ) -> Result<acme_db::media::MediaVersionRow, sqlx::Error> {
        media::activate_ready_current(
            self.0,
            version_id,
            media_id,
            promoted.object.size as i64,
            &promoted.object.content_type,
            &promoted.sha256,
            &promoted.object.provider,
            &promoted.object.bucket,
            &promoted.object.key,
        )
        .await
    }
}

/// Promote staging to a distinct destination, or accept a destination that
/// already holds exactly the captured staging bytes (convergent retry).
async fn promote_or_converge(
    blob: &dyn BlobAdapter,
    staging: &BlobObjectKey,
    destination: &BlobObjectKey,
    declared_content_type: &str,
    config: &BlobUploadConfig,
) -> Result<VerifiedPromotionResult, BlobError> {
    match blob
        .promote_verified(staging, destination, declared_content_type, config)
        .await
    {
        Ok(result) => Ok(result),
        Err(BlobError::DestinationExists(_)) => {
            converge_existing_destination(blob, staging, destination, declared_content_type, config)
                .await
        }
        Err(err) => Err(err),
    }
}

async fn converge_existing_destination(
    blob: &dyn BlobAdapter,
    staging: &BlobObjectKey,
    destination: &BlobObjectKey,
    declared_content_type: &str,
    config: &BlobUploadConfig,
) -> Result<VerifiedPromotionResult, BlobError> {
    let max_bytes = config.max_file_size_bytes_limit();
    let staging_bytes = blob.get_bytes_bounded(staging.as_str(), max_bytes).await?;
    let destination_bytes = blob
        .get_bytes_bounded(destination.as_str(), max_bytes)
        .await?;
    if staging_bytes != destination_bytes {
        return Err(BlobError::DestinationExists(
            destination.as_str().to_string(),
        ));
    }
    if !config.is_content_type_allowed(declared_content_type) {
        return Err(BlobError::InvalidContentType(
            declared_content_type.to_string(),
        ));
    }

    Ok(VerifiedPromotionResult {
        object: StoredObject::new(
            blob.name(),
            blob.bucket(),
            destination.as_str(),
            destination_bytes.len() as u64,
            declared_content_type,
        ),
        sha256: hex::encode(Sha256::digest(&destination_bytes)),
    })
}

fn blob_finalise_error(
    err: BlobError,
    state: &AppState,
    media_id: Uuid,
    version_id: Uuid,
) -> ApiError {
    if let Some(rejection) = blob_rejection_to_api_error(&err, state) {
        return rejection;
    }
    tracing::error!("Failed to promote upload: {}", err);
    crate::db_errors::internal_with_diagnostics(
        "media.upload_finalise_failed",
        "Failed to finalise upload",
        &err,
    )
    .with_context(json!({
        "operation": "media.finalise_upload",
        "media_id": media_id,
        "version_id": version_id
    }))
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
    let pool = state.local_auth.pool();
    finalise_upload_with(&state, PoolStore(pool), media_id, version_id, req).await
}

async fn finalise_upload_with<S: ReadyCurrentStore>(
    state: &AppState,
    store: S,
    media_id: Uuid,
    version_id: Uuid,
    req: FinaliseUploadRequest,
) -> Result<Response, ApiError> {
    if let Err(e) = req.validate() {
        return Err(ApiError::bad_request("validation.failed", e.to_string()));
    }

    let pool = state.local_auth.pool();

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

    if version.state == "ready" && media_row.current_version_id == Some(version_id) {
        return finalise_success_response(pool, media_id, version).await;
    }

    if version.state != "uploading" {
        return Err(ApiError::bad_request(
            "version.not_uploading",
            "Version is not in uploading state",
        ));
    }

    // Staging key matches initiate. The published destination is distinct so
    // exclusive create cannot overwrite the client upload object.
    let filename = media_row.original_filename.clone().unwrap_or_else(|| {
        let ext = underlay_media::storage::mime_to_extension(&req.content_type);
        format!("file.{}", ext)
    });
    let staging_key = version_object_key(media_id, version_id, &filename)
        .map_err(|e| ApiError::bad_request("media.invalid_object_key", e.to_string()))?;
    let destination_key = published_object_key(&staging_key)
        .map_err(|e| ApiError::bad_request("media.invalid_object_key", e))?;

    // Client sha256/size/provider/bucket/final key are not authority. Digest,
    // size, MIME, provider, bucket, and destination key come from the
    // captured promotion result.
    let promoted = match promote_or_converge(
        state.blob_adapter.as_ref(),
        &staging_key,
        &destination_key,
        &req.content_type,
        &state.config.media,
    )
    .await
    {
        Ok(result) => result,
        Err(err) => return Err(blob_finalise_error(err, state, media_id, version_id)),
    };

    let finalised_version = match store
        .activate_ready_current(media_id, version_id, &promoted)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to activate ready/current: {}", e);
            return Err(crate::db_errors::internal_with_diagnostics(
                "media.version_finalise_failed",
                "Failed to finalise upload",
                &e,
            )
            .with_context(json!({
                "operation": "media.finalise_upload",
                "media_id": media_id,
                "version_id": version_id,
                "staging_key": staging_key.as_str(),
                "destination_key": destination_key.as_str()
            })));
        }
    };

    if promoted.object.content_type.starts_with("image/")
        && promoted.object.content_type != "image/svg+xml"
    {
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

    finalise_success_response(pool, media_id, finalised_version).await
}

async fn finalise_success_response(
    pool: &acme_db::DbPool,
    media_id: Uuid,
    finalised_version: acme_db::media::MediaVersionRow,
) -> Result<Response, ApiError> {
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
                "media_id": media_id
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

#[cfg(test)]
#[path = "../../../tests/routes/admin/media_finalise_promotion_tests.rs"]
mod finalise_promotion_tests;
