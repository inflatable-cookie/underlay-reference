//! Test database setup and management.
//!
//! Provides utilities for creating isolated test database connections
//! and managing test data lifecycle.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::sync::OnceLock;

/// Shared multi-thread runtime for tests that exercise code reading a
/// process-global pool (e.g. handlers using `DB_POOL`).
///
/// A pool stored in a process-global can only be bound to one runtime; if
/// each `#[tokio::test]` spins its own runtime, the pool dies with the first
/// test's runtime and later tests fail with "Tokio context is being
/// shutdown". Such tests must use `#[test]` +
/// `shared_runtime().block_on(...)` so they all share this runtime.
pub fn shared_runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("build shared test runtime")
    })
}

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
    TestDb::new(create_test_pool().await)
}

/// Create a new test database pool.
///
/// Each call creates a fresh pool bound to the calling test's runtime.
/// A process-global pool cannot be shared here: every `#[tokio::test]`
/// runs its own runtime, and a pool created on the first test's runtime
/// breaks with "Tokio context is being shutdown" / pool timeouts once that
/// runtime exits. Connection count is kept small so parallel tests stay
/// within Postgres limits.
async fn create_test_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("TEST_DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set for tests");

    PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
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
