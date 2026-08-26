#[tokio::main]
async fn main() {
    acme_db::ensure_database_url();

    if let Err(err) =
        underlay_devtools::reset_from_env("DATABASE_URL", acme_db::DEV_RESET_SCHEMAS, true, true)
            .await
    {
        eprintln!("Failed to reset schemas: {err}");
        std::process::exit(1);
    }

    println!("Dev database schema dropped. Replay with `effigy acme-api/migration:reset`.");
}
