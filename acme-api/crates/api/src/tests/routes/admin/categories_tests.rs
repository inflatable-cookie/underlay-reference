
use super::*;

#[test]
fn reorder_categories_conflict_contains_added_removed_ids() {
    let err = map_reorder_categories_result(
        2,
        categories::ReorderCategoriesResult {
            reordered_count: 0,
            missing_from_submission: vec![Uuid::new_v7().into_inner()],
            not_found: vec![Uuid::new_v7().into_inner()],
        },
    )
    .expect_err("expected conflict");

    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(err.err.code, "categories.reorder_conflict");
    assert!(err.context["added_ids"].is_array());
    assert!(err.context["removed_ids"].is_array());
}
