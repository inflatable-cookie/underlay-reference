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
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn builds_conflict_with_context_keys() {
        let err = reorder_conflict_error(
            "projects.reorder_conflict",
            "projects.reorder",
            4,
            vec!["a".to_string()],
            vec!["b".to_string()],
            serde_json::json!({}),
        );

        assert_eq!(err.status, StatusCode::CONFLICT);
        assert_eq!(err.err.code, "projects.reorder_conflict");
        assert_eq!(
            err.err.message,
            "Items have changed since you started reordering."
        );
        assert_eq!(err.context["operation"], "projects.reorder");
        assert_eq!(err.context["count"], 4);
        assert_eq!(err.context["added_ids"], serde_json::json!(["a"]));
        assert_eq!(err.context["removed_ids"], serde_json::json!(["b"]));
    }

    #[test]
    fn merges_extra_context() {
        let err = reorder_conflict_error(
            "tasks.reorder_conflict",
            "tasks.reorder",
            3,
            vec![],
            vec![],
            serde_json::json!({ "project_id": "p-1" }),
        );

        assert_eq!(err.context["project_id"], "p-1");
    }
}
