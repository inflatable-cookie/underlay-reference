//! Entry point for the Acme background job worker.

use std::sync::Arc;
use std::time::Duration;

use acme_db::{create_pool, run_migrations};
use acme_infra::{init_tracing, log_effective_config, AppConfig};
use acme_jobs::{
    scheduled_task_definitions, JobRepository, JobRunner, JobRunnerConfig, PgJobNotifier,
    PostgresJobRunnerExt, ScheduledTaskRepository, Scheduler,
};
use tracing::{error, info};
use underlay_blob::{BlobAdapter, NoopAdapter, S3Adapter, S3Config};
use underlay_media::renditions::RenditionConfig;

#[tokio::main]
async fn main() {
    // Load config first. A malformed layered config stack is fatal outside
    // local/effigy/test; the worker must not run on defaults nobody chose.
    let app_config = match AppConfig::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("failed to load configuration; job worker exiting: {err}");
            std::process::exit(1);
        }
    };
    init_tracing(&app_config);
    log_effective_config(&app_config);

    let db_url = match app_config.database.url.as_deref() {
        Some(url) => url,
        None => {
            error!("DATABASE_URL is not set; job worker cannot start");
            return;
        }
    };

    let pool = match create_pool(db_url).await {
        Ok(pool) => pool,
        Err(err) => {
            error!(%err, "failed to connect to database; job worker exiting");
            return;
        }
    };

    if let Err(err) = run_migrations(&pool).await {
        error!(%err, "failed to run DB migrations; job worker exiting");
        return;
    }

    info!("starting acme job worker");

    // Initialize blob adapter for media processing. Mirrors the API binary:
    // local/dev uses MinIO; production builds real S3 from ACME_S3_* and
    // refuses to boot with a data-dropping NoopAdapter unless explicitly
    // opted out.
    let blob_adapter: Arc<dyn BlobAdapter> = if app_config.env.is_development() {
        let s3_config = S3Config::minio_dev("acme-media", "https://s3.acme.test");
        match S3Adapter::new(s3_config).await {
            Ok(adapter) => {
                if let Err(err) = adapter.ensure_bucket_ready().await {
                    error!(%err, "failed to reconcile MinIO media bucket");
                }
                Arc::new(adapter)
            }
            Err(err) => {
                error!(%err, "failed to create MinIO blob adapter; using noop");
                Arc::new(NoopAdapter::new())
            }
        }
    } else if let Ok(bucket) = std::env::var("ACME_S3_BUCKET") {
        let region = std::env::var("ACME_S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let mut s3_config = S3Config::new(bucket.clone(), region);
        if let Ok(endpoint) = std::env::var("ACME_S3_ENDPOINT") {
            s3_config = s3_config.endpoint_url(endpoint);
        }
        if let Ok(public_url_base) = std::env::var("ACME_S3_PUBLIC_URL_BASE") {
            s3_config = s3_config.public_url_base(public_url_base);
        }
        if std::env::var("ACME_S3_PATH_STYLE").as_deref() == Ok("1") {
            s3_config = s3_config.path_style(true);
        }

        let adapter = match S3Adapter::new(s3_config).await {
            Ok(adapter) => adapter,
            Err(err) => {
                error!(%err, %bucket, "failed to initialize S3 blob adapter; job worker exiting");
                std::process::exit(1);
            }
        };
        info!(%bucket, "S3 blob adapter initialised");
        Arc::new(adapter)
    } else if std::env::var("ACME_ALLOW_NOOP_BLOB").as_deref() == Ok("1") {
        tracing::warn!(
            "ACME_ALLOW_NOOP_BLOB=1 set — media job outputs are discarded (NoopAdapter)"
        );
        Arc::new(NoopAdapter::new())
    } else {
        error!(
            "blob storage is not configured (set ACME_S3_BUCKET, or ACME_ALLOW_NOOP_BLOB=1 for a deliberately storage-less deployment); job worker exiting"
        );
        std::process::exit(1);
    };

    // Rendition config for generated media derivatives.
    let rendition_config = RenditionConfig::default().thumbnail_size(300);

    // Create registry with all job handlers
    let registry = acme_jobs::create_registry_with_media(
        Arc::new(pool.clone()),
        Some(blob_adapter),
        rendition_config,
    );

    // Create scheduler and job runner.
    let job_repo = JobRepository::new(pool.clone());
    let runner = JobRunner::new(job_repo, registry)
        .with_config(JobRunnerConfig::default().with_poll_interval(Duration::from_secs(30)));

    let scheduler_job_repo = JobRepository::new(pool.clone());
    let task_repo = ScheduledTaskRepository::new(pool.clone());
    let scheduler = Scheduler::new(scheduler_job_repo, task_repo);

    // Register scheduled task definitions on startup.
    // This upserts task definitions and disables any stale tasks.
    let task_definitions = scheduled_task_definitions();
    if let Err(err) = scheduler.register_tasks(&task_definitions).await {
        error!(%err, "failed to register scheduled tasks");
        return;
    }
    info!(count = task_definitions.len(), "registered scheduled tasks");

    let mut notifier = match PgJobNotifier::connect(&pool).await {
        Ok(n) => n,
        Err(err) => {
            error!(%err, "failed to create job notifier; job worker exiting");
            return;
        }
    };

    tokio::select! {
        result = runner.run_with_notifier(&mut notifier) => {
            if let Err(err) = result {
                error!(%err, "job runner failed");
            }
        }
        _ = run_scheduler(scheduler) => {
            info!("scheduler stopped");
        }
        _ = shutdown_signal() => {
            info!("shutdown signal received");
        }
    }

    info!("job worker shutting down");
}

async fn run_scheduler(scheduler: Scheduler) {
    loop {
        if let Err(err) = scheduler.tick().await {
            error!(%err, "scheduler tick failed");
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
