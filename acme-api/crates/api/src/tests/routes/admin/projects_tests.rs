    use super::*;

    #[test]
    fn reorder_projects_conflict_contains_added_removed_ids() {
        let err = map_reorder_projects_result(
            3,
            tasks::ReorderProjectsResult {
                reordered_count: 0,
                missing_from_submission: vec![Uuid::new_v7().into_inner()],
                not_found: vec![Uuid::new_v7().into_inner()],
            },
        )
        .expect_err("expected conflict");

        assert_eq!(err.status, StatusCode::CONFLICT);
        assert_eq!(err.err.code, "projects.reorder_conflict");
        assert!(err.context["added_ids"].is_array());
        assert!(err.context["removed_ids"].is_array());
    }
