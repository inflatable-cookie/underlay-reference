use serde_json::Value;
use underlay_nightfire::{NightfireMediaLocator, NightfireValue};
use uuid::Uuid;

use crate::DbPool;

use super::TaskRow;

pub fn resolve_task_notes_locator_value(
    notes: &Value,
    locator_kind: &str,
    locator_key: &str,
) -> Option<Value> {
    match locator_kind {
        "block_id" => {
            let value: NightfireValue = serde_json::from_value(notes.clone()).ok()?;
            let locator = NightfireMediaLocator::parse(locator_key).ok()?;
            locator.resolve_in_value(&value).cloned()
        }
        "path" => notes.pointer(locator_key).cloned(),
        _ => None,
    }
}

pub fn resolve_task_notes_locator_for_row(
    task: &TaskRow,
    locator_kind: &str,
    locator_key: &str,
) -> Option<Value> {
    let notes = task.notes.as_ref()?;
    resolve_task_notes_locator_value(notes, locator_kind, locator_key)
}

pub async fn resolve_task_notes_locator(
    pool: &DbPool,
    task_id: Uuid,
    locator_kind: &str,
    locator_key: &str,
) -> Result<Option<Value>, sqlx::Error> {
    let Some(task) = super::get_task(pool, task_id).await? else {
        return Ok(None);
    };

    Ok(resolve_task_notes_locator_for_row(
        &task,
        locator_kind,
        locator_key,
    ))
}
