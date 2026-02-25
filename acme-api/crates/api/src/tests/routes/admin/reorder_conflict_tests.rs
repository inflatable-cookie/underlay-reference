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
