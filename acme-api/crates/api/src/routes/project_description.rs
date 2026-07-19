use underlay_http::ApiError;
use underlay_media::MediaUsageProvenanceKind;
use underlay_nightfire::NightfireValue;
use uuid::Uuid;

use crate::nightfire::{
    notes::build_notes_media_registry, prepare_nightfire_value, sync_nightfire_block_media_usage,
    PreparedNightfireValue,
};
use acme_db::media;

// ApiError is the canonical error type here; boxing it would force
// map_err at every `?` call site (matches underlay-http house style).
#[allow(clippy::result_large_err)]
pub(crate) fn prepare_project_description(
    description: Option<NightfireValue>,
) -> Result<Option<PreparedNightfireValue>, ApiError> {
    prepare_nightfire_value(
        description,
        "projects.description.serialize",
        "projects.description_serialize_failed",
        "Failed to serialize project description",
    )
}

pub(crate) async fn sync_project_description_media_usage(
    pool: &acme_db::DbPool,
    project_id: Uuid,
    description: Option<&NightfireValue>,
) -> Result<(), ApiError> {
    let repo = media::AcmeMediaUsageSyncRepo::new(pool);
    sync_nightfire_block_media_usage(
        &repo,
        "project",
        project_id,
        "description",
        description,
        MediaUsageProvenanceKind::ContentSync,
        build_notes_media_registry(),
        "projects.description_media_sync_failed",
        "Failed to sync project description media usage",
        "projects.description_media_clear_failed",
        "Failed to clear project description media usage",
    )
    .await
}
