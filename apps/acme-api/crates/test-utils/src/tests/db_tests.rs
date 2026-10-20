use super::*;

#[tokio::test]
#[ignore = "requires disposable Postgres; run effigy dev:test"]
async fn test_db_connects_successfully() {
    // Skip if no DATABASE_URL is set
    let db = setup_test_db().await;
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(db.pool())
        .await
        .expect("Query should succeed");
    assert_eq!(result.0, 1);
}
