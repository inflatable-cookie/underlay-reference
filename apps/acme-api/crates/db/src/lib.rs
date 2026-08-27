//! Database wiring and helpers.
//!
//! This crate owns the Postgres connection pool and migration helpers.

pub mod account;
pub mod activity;
pub mod auth;
pub mod categories;
pub mod media;
pub mod stats;
pub mod tasks;
pub mod users;

use underlay_db::DbConfig;

/// Convenience type alias for the shared database pool.
pub type DbPool = underlay_db::DbPool;

/// Resolve `DATABASE_URL` for local migration binaries.
///
/// Process env wins. Otherwise use the same app config stack the Effigy
/// runtime loads (`ENVIRONMENT=effigy` -> `config/effigy.toml`).
pub fn ensure_database_url() {
    if std::env::var("DATABASE_URL").is_ok() {
        return;
    }
    if let Ok(url) = std::env::var("ACME_DATABASE_URL") {
        std::env::set_var("DATABASE_URL", url);
        return;
    }
    match acme_infra::AppBehaviorConfig::load() {
        Ok(behavior) => {
            if let Some(url) = behavior.database_url {
                std::env::set_var("DATABASE_URL", url);
            }
        }
        // Migration binaries are operator tooling: report the malformed stack
        // and let the caller fail on the missing URL rather than masking it.
        Err(err) => eprintln!("warning: could not resolve DATABASE_URL from config: {err}"),
    }
}

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
