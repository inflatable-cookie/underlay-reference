use serde_json::Value;
use underlay_http::ApiError;
use underlay_media::nightfire::{
    NightfireBlockMediaHandlerRegistry, NightfireBlockMediaUsageExtractor,
};
use underlay_media::sync::{sync_media_usages_for_record, MediaUsageSyncRepository};
use underlay_media::MediaUsageProvenanceKind;
use underlay_nightfire::{ensure_block_ids, NightfireValue};
use uuid::Uuid;

pub mod notes;

pub struct PreparedNightfireValue {
    value: NightfireValue,
    json: Value,
}

impl PreparedNightfireValue {
    pub fn value(&self) -> &NightfireValue {
        &self.value
    }

    pub fn json(&self) -> &Value {
        &self.json
    }
}

// ApiError is the canonical error type here; boxing it would force
// map_err at every `?` call site (matches underlay-http house style).
#[allow(clippy::result_large_err)]
pub fn prepare_nightfire_value(
    value: Option<NightfireValue>,
    operation: &'static str,
    error_code: &'static str,
    error_message: &'static str,
) -> Result<Option<PreparedNightfireValue>, ApiError> {
    value
        .map(|mut value| {
            ensure_block_ids(&mut value);
            serde_json::to_value(&value)
                .map(|json| PreparedNightfireValue { value, json })
                .map_err(|err| {
                    ApiError::internal(error_code, error_message).with_context(serde_json::json!({
                      "operation": operation,
                      "error": err.to_string()
                    }))
                })
        })
        .transpose()
}

// Parameter-heavy by design: the error-code/message pairs differ per call
// site (matches the house style in db/src/media).
#[allow(clippy::too_many_arguments)]
pub async fn sync_nightfire_block_media_usage<S, R>(
    repo: &S,
    used_by_type: &'static str,
    used_by_id: Uuid,
    owner_field: &'static str,
    value: Option<&NightfireValue>,
    provenance_kind: MediaUsageProvenanceKind,
    registry: R,
    sync_error_code: &'static str,
    sync_error_message: &'static str,
    clear_error_code: &'static str,
    clear_error_message: &'static str,
) -> Result<(), ApiError>
where
    S: MediaUsageSyncRepository,
    R: NightfireBlockMediaHandlerRegistry,
{
    if let Some(value) = value {
        let extractor = NightfireBlockMediaUsageExtractor::new(
            used_by_type,
            Some(used_by_id),
            owner_field,
            provenance_kind.clone(),
            registry,
        );

        extractor
            .extract_and_sync(repo, value)
            .await
            .map_err(|err| {
                ApiError::internal(sync_error_code, sync_error_message).with_context(
                    serde_json::json!({
                      "operation": format!("{used_by_type}.{owner_field}.sync"),
                      "used_by_type": used_by_type,
                      "used_by_id": used_by_id,
                      "owner_field": owner_field,
                      "error": err.to_string()
                    }),
                )
            })?;
    } else {
        sync_media_usages_for_record(repo, used_by_type, used_by_id, &[], &provenance_kind)
            .await
            .map_err(|err| {
                ApiError::internal(clear_error_code, clear_error_message).with_context(
                    serde_json::json!({
                      "operation": format!("{used_by_type}.{owner_field}.clear"),
                      "used_by_type": used_by_type,
                      "used_by_id": used_by_id,
                      "owner_field": owner_field,
                      "error": err.to_string()
                    }),
                )
            })?;
    }

    Ok(())
}
