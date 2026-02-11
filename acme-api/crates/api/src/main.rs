//! Acme HTTP API entrypoint.

use acme_db::infra::DbEmailStore;
use acme_db::{create_pool, run_dev_seeds, run_migrations};
use acme_infra::{create_email_manager, create_template_engine, log_effective_config, AppConfig};
use std::sync::Arc;
use tracing::info;
use underlay_blob::{BlobAdapter, LocalAdapter, LocalConfig, NoopAdapter};

// Routes and state from the library crate
use acme_api::config::AcmeConfig;
use acme_api::routes;
use acme_api::state::{AppState, DB_POOL};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config first (includes env vars and .env file)
    let app_config = AppConfig::from_env();

    // Initialize tracing with environment-appropriate format
    acme_infra::init_tracing(&app_config);
    log_effective_config(&app_config);

    let db_url = app_config
        .database
        .url
        .clone()
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL must be set"))?;

    let pool = create_pool(&db_url).await?;
    run_migrations(&pool).await?;

    // Run dev seeds in local/dev environments
    if app_config.env.is_development() {
        if let Err(err) = run_dev_seeds(&pool).await {
            tracing::error!(%err, "failed to run dev seed SQL");
        }
    }

    // Initialize auth service
    let local_auth = match acme_auth::AcmeLocalAuthService::from_env(pool.clone()) {
        Ok(service) => Arc::new(service),
        Err(err) => {
            tracing::error!(code = err.code(), message = %err.message(), "failed to configure local auth");
            tracing::error!("set AUTH_JWT_PRIVATE_KEY/AUTH_JWT_PUBLIC_KEY env vars");
            std::process::exit(1);
        }
    };

    let auth_provider: Arc<dyn underlay_auth::AuthProvider> =
        Arc::new(acme_auth::AcmeLocalAuthProvider::new(local_auth.clone()));

    // Configure auth cookies
    let cookie_secure = std::env::var("COOKIE_SECURE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(!app_config.env.is_development());

    if !cookie_secure && !app_config.env.is_development() {
        tracing::warn!(
            "COOKIE_SECURE=false in non-development environment; auth cookies may be sent over HTTP"
        );
    }

    // SameSite cookie policy for CSRF protection
    // Default to Strict in production, Lax in development
    let same_site = if std::env::var("COOKIE_SAMESITE_STRICT")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(!app_config.env.is_development())
    {
        underlay_http::cookies::SameSite::Strict
    } else {
        underlay_http::cookies::SameSite::Lax
    };

    let mut cookie_config = underlay_http::AuthCookieConfig::new()
        .with_secure(cookie_secure)
        .with_same_site(same_site);

    if let Ok(domain) = std::env::var("COOKIE_DOMAIN") {
        cookie_config = cookie_config.with_domain(domain);
    }

    // Allow customizing cookie prefix (e.g., "acme_" for "acme_refresh_token")
    if let Ok(prefix) = std::env::var("COOKIE_PREFIX") {
        cookie_config = cookie_config.with_cookie_prefix(prefix);
    }

    // Allow customizing refresh token max age (in seconds)
    if let Ok(max_age) = std::env::var("REFRESH_TOKEN_MAX_AGE")
        .and_then(|v| v.parse::<u64>().map_err(|_| std::env::VarError::NotPresent))
    {
        cookie_config = cookie_config.with_refresh_token_max_age(max_age);
    }

    let email_config = app_config.email.clone();

    // Create email manager with database-backed store for DevCapture
    let db_email_store = Arc::new(DbEmailStore::new(pool.clone()));
    let email_manager = Arc::new(
        create_email_manager(&email_config, Some(db_email_store))
            .map_err(|e| anyhow::anyhow!("failed to create email manager: {}", e))?,
    );

    // Load email templates from disk
    let email_templates = Arc::new(
        create_template_engine(&email_config)
            .map_err(|e| anyhow::anyhow!("failed to load email templates: {}", e))?,
    );

    // Initialize email TOTP service
    let email_totp = Arc::new(acme_auth::EmailTotpService::new(
        pool.clone(),
        email_manager.clone(),
        email_templates.clone(),
        email_config.clone(),
    ));

    // Initialize blob storage adapter
    // In local/dev, use LocalAdapter with filesystem storage
    // In production, this should be configured to use S3Adapter
    let (blob_adapter, local_adapter): (Arc<dyn BlobAdapter>, Option<Arc<LocalAdapter>>) =
        if app_config.env.is_development() {
            let local_config = LocalConfig::new(
                "./dev-uploads",
                format!(
                    "http://{}:{}/v1/dev-uploads",
                    app_config.http.public_host, app_config.http.port
                ),
            );

            let local = LocalAdapter::new(local_config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create local blob adapter: {}", e))?;

            let local = Arc::new(local);
            (local.clone(), Some(local))
        } else {
            // Production: use NoopAdapter as placeholder (configure S3 in production)
            tracing::warn!("Using NoopAdapter for blob storage - configure S3 for production");
            (Arc::new(NoopAdapter::new()), None)
        };

    // Create job repository for enqueueing jobs
    let job_repository = Some(Arc::new(underlay_jobs::JobRepository::new(pool.clone())));

    // Application config - use defaults, override as needed
    let config = AcmeConfig::default();

    // Trusted proxy configuration for secure IP extraction
    let trusted_proxy_config = acme_infra::TrustedProxyConfig::from_env();
    if trusted_proxy_config.trust_proxy_headers {
        tracing::info!(
            "Proxy headers enabled with {} trusted proxies",
            trusted_proxy_config.trusted_proxies.len()
        );
    } else {
        tracing::debug!("Proxy headers disabled - using direct connection IPs only");
    }

    let state = AppState {
        local_auth,
        auth_provider,
        cookie_config,
        email_manager,
        email_templates,
        email_totp,
        email_config,
        blob_adapter,
        job_repository,
        config,
        trusted_proxy_config,
    };

    // Set global DB pool for middleware
    if DB_POOL.set(pool.clone()).is_err() {
        tracing::warn!("DB pool already initialised for DB_POOL");
    }

    // Configure error logging middleware
    let error_logging_config = underlay_http::ErrorLoggingConfig::new(pool.clone())
        .with_source("acme-api")
        .with_client_errors(true)
        .with_server_errors(true);

    let app = routes::build_router()
        .with_state(state.clone())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            routes::csrf_protection_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            error_logging_config,
            underlay_http::error_logging_middleware,
        ))
        .layer(underlay_observability::trace_layer())
        .layer(underlay_observability::request_id_layer());

    // Add dev-uploads routes in development mode (for LocalAdapter uploads)
    let app = if let Some(local_adapter) = local_adapter {
        let dev_routes = underlay_blob::dev_server::DevBlobRoutes::new(local_adapter)
            .route_path("/v1/dev-uploads")
            .with_cors()
            .build();

        app.merge(dev_routes)
    } else {
        app
    };

    let addr = format!("{}:{}", app_config.http.bind_addr, app_config.http.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "api listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal as unix_signal, SignalKind};

        let mut sigint =
            unix_signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
        let mut sigterm =
            unix_signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = sigint.recv() => {
                tracing::info!("received SIGINT; starting graceful shutdown");
            }
            _ = sigterm.recv() => {
                tracing::info!("received SIGTERM; starting graceful shutdown");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        tracing::info!("received Ctrl+C; starting graceful shutdown");
    }
}
