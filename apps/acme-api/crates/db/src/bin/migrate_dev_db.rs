use tracing::{error, info};

use acme_db::{run_dev_seeds, run_migrations};

enum Mode {
    Schema,
    Overlay,
}

fn load_database_url() {
    if std::env::var("DATABASE_URL").is_err() {
        if let Ok(v) = std::env::var("ACME_DATABASE_URL") {
            std::env::set_var("DATABASE_URL", v);
        }
    }
}

fn parse_mode() -> Mode {
    match std::env::args().nth(1).as_deref() {
        Some("schema") => Mode::Schema,
        Some("overlay") => Mode::Overlay,
        Some(other) => {
            eprintln!("Usage: migrate_dev_db schema|overlay (got {other})");
            std::process::exit(2);
        }
        None => {
            eprintln!("Usage: migrate_dev_db schema|overlay");
            std::process::exit(2);
        }
    }
}

fn fail(err: impl std::fmt::Display, context: &str) -> ! {
    error!("{context}: {err}");
    eprintln!("{context}: {err}");
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    load_database_url();
    match parse_mode() {
        Mode::Schema => apply_schema().await,
        Mode::Overlay => apply_overlay().await,
    }
}

async fn apply_schema() {
    match underlay_devtools::migrate_from_env_with("DATABASE_URL", |pool| {
        Box::pin(run_migrations(pool))
    })
    .await
    {
        Ok(_) => {
            info!("dev database schema migrations complete");
            println!("Dev database schema migrations complete.");
        }
        Err(err) => fail(err, "failed to run DB migrations"),
    }
}

async fn apply_overlay() {
    let database_url = match underlay_devtools::require_env("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => fail(err, "failed to read DATABASE_URL"),
    };
    let pool = match underlay_devtools::connect(&database_url).await {
        Ok(pool) => pool,
        Err(err) => fail(err, "failed to connect for dev overlay"),
    };
    if let Err(err) = run_dev_seeds(&pool).await {
        fail(err, "failed to run dev seeds");
    }
    info!("dev overlay complete");
    println!("Dev overlay complete.");
}
