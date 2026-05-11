
use super::*;

#[tokio::test]
async fn test_db_connects_successfully() {
    // Skip if no DATABASE_URL is set
    if env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }

    let db = setup_test_db().await;
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(db.pool())
        .await
        .expect("Query should succeed");
    assert_eq!(result.0, 1);
}
