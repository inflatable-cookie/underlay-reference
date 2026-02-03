use dotenvy::dotenv;
use tracing::{error, info};

use acme_db::{run_dev_seeds, run_migrations};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Prefer standard Underlay env vars; accept ACME_* as legacy fallbacks.
    if std::env::var("DATABASE_URL").is_err() {
        if let Ok(v) = std::env::var("ACME_DATABASE_URL") {
            std::env::set_var("DATABASE_URL", v);
        }
    }

    let pool = match underlay_devtools::migrate_from_env_with("DATABASE_URL", |pool| {
        Box::pin(run_migrations(pool))
    })
    .await
    {
        Ok(pool) => pool,
        Err(err) => {
            error!(%err, "failed to run DB migrations");
            eprintln!("Failed to run DB migrations: {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = run_dev_seeds(&pool).await {
        error!(%err, "failed to run dev seeds");
        eprintln!("Failed to run dev seeds: {err}");
        std::process::exit(1);
    }

    info!("dev database migrations + seeds complete");
    println!("Dev database migrations + seeds complete.");
}
