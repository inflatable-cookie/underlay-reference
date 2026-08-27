//! Acme HTTP API entrypoint.

use acme_db::{create_pool, run_dev_seeds, run_migrations};
use acme_infra::{create_email_manager, create_template_engine, log_effective_config, AppConfig};
use std::sync::Arc;
use tracing::info;
use underlay_blob::{BlobAdapter, NoopAdapter, S3Adapter, S3Config};

// Routes and state from the library crate
use acme_api::config::AcmeConfig;
use acme_api::routes;
use acme_api::state::{AppState, DB_POOL};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config first. A malformed layered config stack is fatal outside
    // local/effigy/test rather than silently degrading to code defaults.
    let app_config = AppConfig::from_env()?;

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

    // Run dev seeds only in local dev/test environments against local
    // databases. Dev seeds contain well-known credentials and must never
    // reach a deployed instance.
    let seed_allowed = app_config.env.is_local_dev() && underlay_db::is_local_database_url(&db_url);

    if seed_allowed {
        if let Err(err) = run_dev_seeds(&pool).await {
            tracing::error!(%err, "failed to run dev seed SQL");
        }
    }

    // Initialize auth service from the already-resolved config authority. It
    // must not re-resolve ENVIRONMENT or reload the config stack: a second
    // load could disagree with the posture this binary already enforced.
    let local_auth = match acme_auth::AcmeLocalAuthService::from_config(pool.clone(), &app_config) {
        Ok(service) => Arc::new(service),
        Err(err) => {
            tracing::error!(code = err.code(), message = %err.message(), "failed to configure local auth");
            tracing::error!("set AUTH_JWT_PRIVATE_KEY/AUTH_JWT_PUBLIC_KEY env vars");
            std::process::exit(1);
        }
    };

    let auth_provider: Arc<dyn underlay_auth::AuthProvider> =
        Arc::new(acme_auth::AcmeLocalAuthProvider::new(local_auth.clone()));

    // Configure auth cookies. Insecure cookies are fatal outside the bounded
    // non-deployed set: a deployed runtime must never put a session cookie on
    // the wire in plaintext.
    let cookie_secure = app_config.cors.cookie_secure;
    acme_infra::enforce_cookie_secure(app_config.env, cookie_secure)?;

    // Resolve cookie-backed browser mutation protection once, here. The
    // middleware consumes the decision; it never re-reads the environment per
    // request, so a later env change cannot weaken a running deployment.
    let csrf_protection_enabled = acme_infra::resolve_csrf_protection(
        app_config.env,
        acme_infra::csrf_protection_requested(),
    )?;

    // Cookie shape comes from typed config. These are behavior knobs, not
    // deployment wiring, so they carry committed defaults rather than living
    // in env; `AppConfig::from_env` warns if the migrated legacy keys are set.
    let cookies = &app_config.behavior.cors;
    let same_site = if cookies.cookie_same_site_strict {
        underlay_http::cookies::SameSite::Strict
    } else {
        underlay_http::cookies::SameSite::Lax
    };

    let mut cookie_config = underlay_http::AuthCookieConfig::new()
        .with_secure(cookie_secure)
        .with_same_site(same_site)
        .with_refresh_token_max_age(cookies.refresh_token_max_age_secs);

    if let Some(domain) = app_config.cors.cookie_domain.clone() {
        cookie_config = cookie_config.try_with_domain(domain)?;
    }

    if !cookies.cookie_prefix.is_empty() {
        cookie_config = cookie_config.try_with_cookie_prefix(cookies.cookie_prefix.clone())?;
    }

    let email_config = app_config.email.clone();

    let email_manager = Arc::new(
        create_email_manager(&email_config)
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

    // Initialize blob storage adapter.
    // Local/dev uses the shared MinIO-backed S3 shape; production builds a
    // real S3 adapter from ACME_S3_* env vars (credentials come from the AWS
    // default chain: env vars, shared config, or an instance/task role).
    let blob_adapter: Arc<dyn BlobAdapter> = if app_config.env.is_development() {
        let s3_config = S3Config::minio_dev("acme-media", "https://s3.acme.test");
        match S3Adapter::new(s3_config).await {
            Ok(adapter) => {
                if let Err(err) = adapter.ensure_bucket_ready().await {
                    tracing::warn!(%err, "Failed to reconcile MinIO media bucket");
                }
                Arc::new(adapter)
            }
            Err(e) => {
                tracing::warn!(%e, "Failed to initialize MinIO blob adapter, falling back to noop");
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

        // Production storage must not fall back to noop: a broken adapter is
        // a boot failure, not silent data loss.
        let adapter = S3Adapter::new(s3_config).await.map_err(|e| {
            anyhow::anyhow!("failed to initialize S3 blob adapter for bucket {bucket}: {e}")
        })?;
        if let Err(err) = adapter.ensure_bucket_ready().await {
            return Err(anyhow::anyhow!(
                "S3 media bucket {bucket} is not usable: {err}"
            ));
        }
        info!(%bucket, "S3 blob adapter initialised");
        Arc::new(adapter)
    } else if std::env::var("ACME_ALLOW_NOOP_BLOB").as_deref() == Ok("1") {
        // Explicit opt-in for a deliberately storage-less deployment.
        tracing::warn!("ACME_ALLOW_NOOP_BLOB=1 set — media uploads are discarded (NoopAdapter)");
        Arc::new(NoopAdapter::new())
    } else {
        // Fail closed: silently accepting-then-dropping prod uploads with a
        // NoopAdapter is data loss. Either configure real storage or opt out
        // explicitly.
        return Err(anyhow::anyhow!(
            "No production blob storage configured. Set ACME_S3_BUCKET (plus \
             optional ACME_S3_REGION / ACME_S3_ENDPOINT / ACME_S3_PUBLIC_URL_BASE / \
             ACME_S3_PATH_STYLE=1) to wire S3, or set ACME_ALLOW_NOOP_BLOB=1 to run \
             without media storage on purpose."
        ));
    };

    // Create job repository for enqueueing jobs
    let job_repository = Some(Arc::new(underlay_jobs_postgres::JobRepository::new(
        pool.clone(),
    )));

    // Application config - use defaults, override as needed
    let config = AcmeConfig::default();

    // Declare the proxy topology that resolves the client IP. This is the one
    // trust boundary: `RequestContext` honours a forwarding header only
    // because this says a trusted proxy sets it. Unset or unrecognised
    // (`TRUSTED_PROXY`) trusts nothing and uses the socket peer, so a
    // misconfiguration cannot start trusting client-supplied headers.
    let trusted_proxy = underlay_http::TrustedProxyConfig::from_env();
    tracing::info!(?trusted_proxy, "client IP trust boundary resolved");

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

    let app = routes::build_router_with_options(app_config.env.is_development())
        .with_state(state.clone())
        .layer(axum::Extension(trusted_proxy))
        .layer(axum::middleware::from_fn_with_state(
            routes::ApiVersionState::from_behavior(&app_config.behavior.api),
            routes::api_version_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            routes::CsrfState::new(csrf_protection_enabled, state.cookie_config.clone()),
            routes::csrf_protection_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            error_logging_config,
            underlay_http::error_logging_middleware,
        ))
        .layer(underlay_observability::trace_layer())
        .layer(underlay_observability::request_id_layer());

    let addr = format!("{}:{}", app_config.http.bind_addr, app_config.http.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "api listening");

    // Serve with connection info so underlay's IP resolution has a socket-peer
    // fallback when no trusted forwarding header is present.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
