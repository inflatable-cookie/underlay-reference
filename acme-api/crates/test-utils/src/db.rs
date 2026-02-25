//! Test database setup and management.
//!
//! Provides utilities for creating isolated test database connections
//! and managing test data lifecycle.

use sqlx::PgPool;
use std::env;
use std::sync::OnceLock;

/// Global test database pool.
///
/// Uses a static pool to avoid creating multiple connections during
/// parallel test execution. The pool is lazily initialized on first use.
static TEST_POOL: OnceLock<PgPool> = OnceLock::new();

/// A wrapper around a database pool for testing.
///
/// Provides a clean interface for test database operations.
#[derive(Clone)]
pub struct TestDb {
    pool: PgPool,
}

impl TestDb {
    /// Create a new TestDb from an existing pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get the underlying pool reference.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get a clone of the pool.
    pub fn pool_clone(&self) -> PgPool {
        self.pool.clone()
    }
}

impl AsRef<PgPool> for TestDb {
    fn as_ref(&self) -> &PgPool {
        &self.pool
    }
}

/// Set up the test database connection pool.
///
/// This function:
/// 1. Reads DATABASE_URL from environment
/// 2. Creates a connection pool optimized for testing
/// 3. Returns a shared pool instance for test reuse
///
/// # Panics
///
/// Panics if DATABASE_URL is not set or connection fails.
///
/// # Example
///
/// ```ignore
/// use acme_test_utils::setup_test_db;
///
/// #[tokio::test]
/// async fn test_database_operation() {
///     let db = setup_test_db().await;
///     // Use db.pool() for queries
/// }
/// ```
pub async fn setup_test_db() -> TestDb {
    // Try to get existing pool or create new one
    let pool = TEST_POOL
        .get_or_init(|| {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async { create_test_pool().await })
            })
        })
        .clone();

    TestDb::new(pool)
}

/// Create a new test database pool.
async fn create_test_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("TEST_DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set for tests");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Run a database transaction that rolls back after the test.
///
/// This is useful for tests that modify data but should not
/// persist changes between tests.
///
/// # Example
///
/// ```ignore
/// use acme_test_utils::db::run_in_transaction;
///
/// #[tokio::test]
/// async fn test_with_rollback() {
///     let db = setup_test_db().await;
///
///     run_in_transaction(db.pool(), |tx| async move {
///         // All changes here will be rolled back
///         sqlx::query("INSERT INTO ...")
///             .execute(&mut *tx)
///             .await?;
///         Ok(())
///     }).await;
/// }
/// ```
pub async fn run_in_transaction<F, Fut, T>(pool: &PgPool, f: F) -> Result<T, sqlx::Error>
where
    F: FnOnce(sqlx::Transaction<'_, sqlx::Postgres>) -> Fut,
    Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
{
    let tx = pool.begin().await?;
    let result = f(tx).await;
    // Transaction is automatically rolled back when dropped without commit
    result
}

#[cfg(test)]
#[path = "tests/db_tests.rs"]
mod tests;
