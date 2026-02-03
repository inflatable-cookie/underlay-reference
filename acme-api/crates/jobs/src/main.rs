//! Entry point for the Acme background job worker.

use std::time::Duration;

use acme_db::{create_pool, run_migrations};
use acme_infra::init_tracing;
use acme_jobs::{scheduled_task_definitions, JobRepository, JobRunner, JobRunnerConfig, PgJobNotifier, Scheduler, ScheduledTaskRepository};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_tracing();

    let db_url = match std::env::var("DATABASE_URL").or_else(|_| std::env::var("ACME_DATABASE_URL")) {
        Ok(url) => url,
        Err(_) => {
            error!("DATABASE_URL is not set; job worker cannot start");
            return;
        }
    };

    let pool = match create_pool(&db_url).await {
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

    // Create registry with all job handlers
    let registry = acme_jobs::create_registry(std::sync::Arc::new(pool.clone()));

    // Create scheduler and job runner.
    let job_repo = JobRepository::new(pool.clone());
    let runner = JobRunner::new(job_repo, registry).with_config(JobRunnerConfig {
        poll_interval: Duration::from_secs(30),
        ..Default::default()
    });

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
