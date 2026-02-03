//! Database wiring and helpers.
//!
//! This crate owns the Postgres connection pool and migration helpers.

pub mod account;
pub mod auth;
pub mod categories;
pub mod infra;
pub mod media;
pub mod stats;
pub mod tasks;
pub mod users;

use underlay_db::DbConfig;

/// Convenience type alias for the shared database pool.
pub type DbPool = underlay_db::DbPool;

// Main schema migrations live at `api/migrations` (repo root).
// This path is relative to `crates/db/src`.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

/// Schemas to drop when resetting the dev database.
///
/// This list is app-owned (schema names are part of the app DB contract).
pub const DEV_RESET_SCHEMAS: &[&str] = &["public", "auth", "account", "platform", "media", "acme"];

/// Create a new Postgres connection pool from a URL.
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    underlay_db::create_pool(&DbConfig::new(database_url)).await
}

/// Run all pending SQL migrations against the given pool.
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    underlay_db::run_migrations(pool, &MIGRATOR).await
}

/// Run dev-only seed SQL (for local/dev environments).
///
/// Notes:
/// - Seeds are loaded from disk at runtime so they can be modified without recompiling.
/// - Do not rely on `SET search_path`; always fully qualify schema names.
/// - Avoid procedural SQL blocks with semicolons (e.g. `DO $$ ... $$`).
pub async fn run_dev_seeds(pool: &DbPool) -> Result<(), sqlx::Error> {
    let dev_seed_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations_dev");

    underlay_db::run_sql_dir(pool, dev_seed_dir).await
}
