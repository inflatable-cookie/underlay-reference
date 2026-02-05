use dotenvy::dotenv;
use std::io::ErrorKind;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Prefer standard Underlay env vars; accept ACME_* as legacy fallbacks.
    if std::env::var("DATABASE_URL").is_err() {
        if let Ok(v) = std::env::var("ACME_DATABASE_URL") {
            std::env::set_var("DATABASE_URL", v);
        }
    }

    if let Err(err) =
        underlay_devtools::reset_from_env("DATABASE_URL", acme_db::DEV_RESET_SCHEMAS, true, true)
            .await
    {
        eprintln!("Failed to reset schemas: {err}");
        std::process::exit(1);
    }

    let blob_dir =
        std::env::var("BLOB_STORAGE_DIR").unwrap_or_else(|_| "./dev-uploads".to_string());
    if let Err(err) = std::fs::remove_dir_all(&blob_dir) {
        if err.kind() != ErrorKind::NotFound {
            eprintln!("Warning: failed to remove dev uploads at {blob_dir}: {err}");
        }
    }
    if let Err(err) = std::fs::create_dir_all(&blob_dir) {
        eprintln!("Warning: failed to recreate dev uploads at {blob_dir}: {err}");
    }

    println!(
        "Dev database schema dropped. Run `cargo run -p acme-db --bin migrate_dev_db` or `bun db:migrate` to apply migrations and seeds."
    );
}
