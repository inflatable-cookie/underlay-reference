use underlay_http::ApiError;

pub(super) fn reorder_conflict_error(
    code: &'static str,
    operation: &'static str,
    count: usize,
    added_ids: Vec<String>,
    removed_ids: Vec<String>,
    extra_context: serde_json::Value,
) -> ApiError {
    let mut context = serde_json::json!({
        "operation": operation,
        "count": count,
        "added_ids": added_ids,
        "removed_ids": removed_ids
    });

    if let (serde_json::Value::Object(base), serde_json::Value::Object(extra)) =
        (&mut context, extra_context)
    {
        for (key, value) in extra {
            base.insert(key, value);
        }
    }

    ApiError::conflict(code, "Items have changed since you started reordering.")
        .with_context(context)
}

#[cfg(test)]
#[path = "../../tests/routes/admin/reorder_conflict_tests.rs"]
mod tests;
