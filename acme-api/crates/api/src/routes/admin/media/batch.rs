use super::*;

/// Request for batch delete operation.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
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
