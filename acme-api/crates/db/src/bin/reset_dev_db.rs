use dotenvy::dotenv;

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

    println!(
        "Dev database schema dropped. Run `cargo run -p acme-db --bin migrate_dev_db` or `bun db:migrate` to apply migrations and seeds."
    );
}
