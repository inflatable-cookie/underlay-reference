use super::*;

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
            let total = items.len();
            Ok(Json(json!({
                "data": items,
                "total": total,
                "has_more": false
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list usage: {}", e);
            Err(crate::db_errors::internal_with_diagnostics(
                "media.list_usage_failed",
                "Failed to list media usage",
                &e,
            )
            .with_context(json!({
                "operation": "media.list_usage",
                "media_id": media_id
            })))
        }
    }
}
