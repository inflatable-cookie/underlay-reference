use super::*;

/// List one bounded page of usages for a media item.
///
/// GET /v1/admin/media/:media_id/usage
pub async fn list_usage(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
    Query(params): Query<PagePaginationParams>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();
    let mut params = params.clamped();
    params.page = params.page.max(1);
    params.limit = params.limit.max(1);

    match media::list_media_usages(pool, media_id, params.limit_i64(), params.offset_i64()).await {
        Ok((rows, total)) => {
            let items: Vec<MediaUsageDto> = rows.into_iter().map(Into::into).collect();
            Ok(Json(params.wrap_page_list(items, total as u64)).into_response())
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
