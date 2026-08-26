use super::*;

#[test]
fn reorder_tasks_conflict_contains_added_removed_ids_and_project_id() {
    let project_id = Uuid::new_v7().into_inner();
    let err = map_reorder_tasks_result(
        project_id,
        4,
        tasks::ReorderTasksResult {
            reordered_count: 0,
            missing_from_submission: vec![Uuid::new_v7().into_inner()],
            not_found: vec![Uuid::new_v7().into_inner()],
        },
    )
    .expect_err("expected conflict");

    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(err.err.code, "tasks.reorder_conflict");
    assert!(err.context["added_ids"].is_array());
    assert!(err.context["removed_ids"].is_array());
    assert_eq!(
        err.context["project_id"],
        serde_json::json!(project_id.to_string())
    );
}
