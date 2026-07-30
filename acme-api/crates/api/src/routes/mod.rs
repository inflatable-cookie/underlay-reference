//! Route handlers for the Acme API.
//!
//! This module organizes routes into:
//! - `shared/` - Auth, health, account routes (used by all clients)
//! - `tasks` - User-facing project/task routes
//! - `admin/` - Admin-only routes with enhanced features

use axum::extract::State;
use axum::http::{header::HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use std::sync::OnceLock;
use underlay_http::{cors_layer_for_env, ApiError, CorsConfig};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;
use crate::state::AppState;

mod admin;
mod project_description;
pub mod shared;
mod tasks;

/// Build the main API router with all routes configured.
pub fn build_router() -> Router<AppState> {
    build_router_with_options(true)
}

/// Build the main API router, optionally exposing Swagger UI / OpenAPI JSON.
/// Production deployments should pass `include_docs = false`.
pub fn build_router_with_options(include_docs: bool) -> Router<AppState> {
    let cors = build_cors_layer();

    let router = Router::new();
    let router = if include_docs {
        router.merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
    } else {
        router
    };

    let router = router
        // Favicon (return 204 to stop browser 404s)
        .route("/favicon.ico", get(|| async { StatusCode::NO_CONTENT }))
        // Health
        .route("/v1/health", get(shared::health::health))
        // Auth routes
        .route("/v1/auth/register", post(shared::auth::register))
        .route("/v1/auth/login", post(shared::auth::login))
        .route("/v1/auth/login/start", post(shared::auth::login_start))
        .route("/v1/auth/login/finish", post(shared::auth::login_finish))
        .route("/v1/auth/csrf-token", get(shared::auth::csrf_token))
        .route("/v1/auth/refresh", post(shared::auth::refresh))
        .route("/v1/auth/logout", post(shared::auth::logout))
        .route("/v1/auth/me", get(shared::auth::me))
        .route(
            "/v1/auth/password/change",
            post(shared::auth::change_password),
        )
        .route(
            "/v1/auth/password/requirements",
            get(shared::auth::password_requirements),
        )
        .route(
            "/v1/auth/password/change-2fa",
            post(shared::auth::change_password_with_verification),
        )
        // Password reset (forgot password) routes
        .route(
            "/v1/auth/password/reset/request",
            post(shared::auth::password_reset_request),
        )
        .route(
            "/v1/auth/password/reset/verify",
            post(shared::auth::password_reset_verify),
        )
        .route(
            "/v1/auth/password/reset/complete",
            post(shared::auth::password_reset_complete),
        )
        // TOTP routes
        .route("/v1/auth/totp/status", get(shared::auth::totp_status))
        .route("/v1/auth/totp/setup", post(shared::auth::totp_setup))
        .route("/v1/auth/totp/enable", post(shared::auth::totp_enable))
        .route("/v1/auth/totp/disable", post(shared::auth::totp_disable))
        .route("/v1/auth/totp/verify", post(shared::auth::totp_verify))
        // 2FA status route
        .route("/v1/auth/2fa-status", get(shared::auth::two_factor_status))
        // Email TOTP routes
        .route(
            "/v1/auth/email-totp/request",
            post(shared::auth::email_totp_request),
        )
        .route(
            "/v1/auth/email-totp/verify",
            post(shared::auth::email_totp_verify),
        )
        // Passkey routes
        .route("/v1/auth/passkeys", get(shared::auth::list_passkeys))
        .route(
            "/v1/auth/passkeys/{credential_id}",
            patch(shared::auth::rename_passkey).delete(shared::auth::delete_passkey),
        )
        .route(
            "/v1/auth/passkeys/register/start",
            post(shared::auth::passkey_register_start),
        )
        .route(
            "/v1/auth/passkeys/register/finish",
            post(shared::auth::passkey_register_finish),
        )
        .route(
            "/v1/auth/passkeys/login/start",
            post(shared::auth::passkey_login_start),
        )
        .route(
            "/v1/auth/passkeys/login/finish",
            post(shared::auth::passkey_login_finish),
        )
        .route(
            "/v1/auth/passkeys/verify/start",
            post(shared::auth::passkey_verify_start),
        )
        .route(
            "/v1/auth/passkeys/verify/finish",
            post(shared::auth::passkey_verify_finish),
        )
        // Session routes
        .route("/v1/auth/sessions", get(shared::auth::list_sessions))
        .route(
            "/v1/auth/sessions/{session_id}/revoke",
            post(shared::auth::revoke_session),
        )
        // Account routes
        .route(
            "/v1/account/profile",
            get(shared::account::get_profile).patch(shared::account::update_profile),
        )
        // ====================================================================
        // User-facing routes (authenticated users)
        // ====================================================================
        // Project routes
        .route(
            "/v1/projects",
            get(tasks::list_projects).post(tasks::create_project),
        )
        .route(
            "/v1/projects/{project_id}",
            get(tasks::get_project)
                .patch(tasks::update_project)
                .delete(tasks::delete_project),
        )
        // Task routes
        .route(
            "/v1/projects/{project_id}/tasks",
            get(tasks::list_tasks).post(tasks::create_task),
        )
        .route(
            "/v1/projects/{project_id}/tasks/{task_id}",
            patch(tasks::update_task).delete(tasks::delete_task),
        )
        // ====================================================================
        // Admin routes (require Admin role)
        // ====================================================================
        // Dashboard stats
        .route(
            "/v1/admin/dashboard/stats",
            get(admin::dashboard::get_dashboard_stats),
        )
        // User management
        .route(
            "/v1/admin/users",
            get(admin::users::list_users).post(admin::users::create_user),
        )
        .route(
            "/v1/admin/users/{user_id}",
            get(admin::users::get_user).put(admin::users::update_user),
        )
        .route(
            "/v1/admin/users/{user_id}/role",
            put(admin::users::update_user_role),
        )
        .route(
            "/v1/admin/users/{user_id}/suspend",
            post(admin::users::suspend_user),
        )
        .route(
            "/v1/admin/users/{user_id}/unsuspend",
            post(admin::users::unsuspend_user),
        )
        .route(
            "/v1/admin/users/{user_id}/activity",
            get(admin::activity::list_activity_for_user),
        )
        .route(
            "/v1/admin/users/{user_id}/sessions",
            get(admin::users::list_user_sessions),
        )
        .route(
            "/v1/admin/users/{user_id}/sessions/{session_id}/revoke",
            post(admin::users::revoke_user_session),
        )
        // Activity/audit log
        .route("/v1/admin/activity", get(admin::activity::list_activity))
        .route(
            "/v1/admin/activity/entity/{entity_type}/{entity_id}",
            get(admin::activity::list_activity_for_entity),
        )
        // Validation endpoint (for async form validation)
        .route(
            "/v1/admin/validate-field",
            post(admin::validation::validate_field),
        )
        // Category admin routes
        .route(
            "/v1/admin/categories",
            get(admin::categories::list_categories).post(admin::categories::create_category),
        )
        .route(
            "/v1/admin/categories/reorder",
            put(admin::categories::reorder_categories),
        )
        .route(
            "/v1/admin/categories:batch-delete",
            post(admin::categories::batch_delete_categories),
        )
        .route(
            "/v1/admin/categories/{category_id}",
            get(admin::categories::get_category)
                .patch(admin::categories::update_category)
                .delete(admin::categories::soft_delete_category),
        )
        .route(
            "/v1/admin/categories/{category_id}/restore",
            post(admin::categories::restore_category),
        )
        // Project admin routes
        .route(
            "/v1/admin/projects",
            get(admin::projects::list_projects).post(admin::projects::create_project),
        )
        .route(
            "/v1/admin/projects/reorder",
            put(admin::projects::reorder_projects),
        )
        .route(
            "/v1/admin/projects:batch-delete",
            post(admin::projects::batch_delete_projects),
        )
        .route(
            "/v1/admin/projects/{project_id}",
            get(admin::projects::get_project)
                .patch(admin::projects::update_project)
                .delete(admin::projects::soft_delete_project),
        )
        .route(
            "/v1/admin/projects/{project_id}/restore",
            post(admin::projects::restore_project),
        )
        // Task admin routes (nested under projects)
        .route(
            "/v1/admin/projects/{project_id}/tasks",
            get(admin::tasks::list_tasks).post(admin::tasks::create_task),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/reorder",
            put(admin::tasks::reorder_tasks),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/batch-delete",
            post(admin::tasks::batch_delete_tasks),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/batch-update",
            post(admin::tasks::batch_update_task_status),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/{task_id}",
            get(admin::tasks::get_task)
                .patch(admin::tasks::update_task)
                .delete(admin::tasks::soft_delete_task),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/{task_id}/labels",
            get(admin::tasks::get_task_labels).put(admin::tasks::set_task_labels),
        )
        // Label admin routes (nested under projects)
        .route(
            "/v1/admin/projects/{project_id}/labels",
            get(admin::tasks::list_labels).post(admin::tasks::create_label),
        )
        // ====================================================================
        // Media Library admin routes
        // ====================================================================
        .route(
            "/v1/admin/media/check-duplicate",
            post(admin::media::check_duplicate),
        )
        .route(
            "/v1/admin/media:batch-delete",
            post(admin::media::batch_delete_media),
        )
        .route(
            "/v1/admin/media",
            get(admin::media::list_media).post(admin::media::create_media),
        )
        .route("/v1/admin/media/trash", get(admin::media::list_media_trash))
        .route(
            "/v1/admin/media/{media_id}",
            get(admin::media::get_media)
                .put(admin::media::update_media)
                .delete(admin::media::purge_media),
        )
        .route(
            "/v1/admin/media/{media_id}/soft-delete",
            post(admin::media::soft_delete_media),
        )
        .route(
            "/v1/admin/media/{media_id}/restore",
            post(admin::media::restore_media),
        )
        .route(
            "/v1/admin/media/{media_id}/versions",
            get(admin::media::list_versions),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/initiate-upload",
            post(admin::media::initiate_upload),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/{version_id}/finalise-upload",
            post(admin::media::finalise_upload),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/{version_id}/activate",
            post(admin::media::activate_version),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/{version_id}",
            delete(admin::media::delete_version),
        )
        .route(
            "/v1/admin/media/{media_id}/usage",
            get(admin::media::list_usage),
        )
        // ====================================================================
        // Background Jobs admin routes
        // ====================================================================
        .route("/v1/admin/jobs", get(admin::jobs::list_jobs))
        .route("/v1/admin/jobs/stats", get(admin::jobs::get_job_stats))
        .route("/v1/admin/jobs/{job_id}", get(admin::jobs::get_job))
        .route(
            "/v1/admin/jobs/{job_id}/cancel",
            post(admin::jobs::cancel_job),
        )
        .route(
            "/v1/admin/jobs/{job_id}/retry",
            post(admin::jobs::retry_job),
        )
        // ====================================================================
        // Scheduled Tasks admin routes
        // ====================================================================
        .route(
            "/v1/admin/scheduled-tasks",
            get(admin::scheduled_tasks::list_scheduled_tasks),
        )
        .route(
            "/v1/admin/scheduled-tasks/{task_id}",
            get(admin::scheduled_tasks::get_scheduled_task),
        )
        .route(
            "/v1/admin/scheduled-tasks/{task_id}/toggle",
            post(admin::scheduled_tasks::toggle_scheduled_task),
        )
        .route(
            "/v1/admin/scheduled-tasks/{task_id}/trigger",
            post(admin::scheduled_tasks::trigger_scheduled_task),
        )
        // ====================================================================
        // Error Logs admin routes
        // ====================================================================
        .route(
            "/v1/admin/error-logs",
            get(admin::error_logs::list_error_logs_handler),
        )
        .route(
            "/v1/admin/error-logs/stats",
            get(admin::error_logs::get_error_log_stats),
        )
        .route(
            "/v1/admin/error-logs/{id}",
            get(admin::error_logs::get_error_log_handler),
        )
        .layer(cors);

    #[cfg(debug_assertions)]
    let router = router.route("/v1/dev/error-smoke", post(shared::health::error_smoke));

    router
}

fn build_cors_layer() -> tower_http::cors::CorsLayer {
    // Underlay CORS policy (matches guide patterns):
    // - Use `CORS_ORIGINS` in production.
    // - In local/dev, if `CORS_ORIGINS` is unset, mirror the request origin.
    // - Allow credentials so cookie-based auth can be enabled without reworking CORS.

    let env = std::env::var("ENVIRONMENT")
        .or_else(|_| std::env::var("ACME_ENV"))
        .unwrap_or_else(|_| "prod".to_string());

    let origins = parse_cors_origins();

    // If no explicit origins are set and we're in local/dev, mirror request origin.
    let mirror_origin = origins.is_empty() && (env == "local" || env == "dev");

    // NOTE: the browser will preflight if we send `X-Api-Version`.
    // Add it explicitly to allowed headers.
    let mut config = CorsConfig::default()
        .with_header(HeaderName::from_static("x-api-version"))
        .with_header(HeaderName::from_static("x-auth-token-mode"))
        .with_header(HeaderName::from_static("x-csrf-token"))
        .with_credentials(true);

    if mirror_origin {
        config = config.with_mirror_origin();
    } else if !origins.is_empty() {
        config = config.with_origin_values(origins);
    }

    // Underlay now gates mirror-origin + credentials to Local/Test; build with
    // the resolved environment so that combination is only allowed in
    // local/dev and a misconfigured prod fails fast.
    cors_layer_for_env(config, underlay_env(&env))
}

fn underlay_env(env: &str) -> underlay_observability::Environment {
    use underlay_observability::Environment;
    match env.to_ascii_lowercase().as_str() {
        "local" => Environment::Local,
        "dev" | "development" => Environment::Dev,
        "staging" | "stage" => Environment::Staging,
        "prod" | "production" => Environment::Prod,
        "test" => Environment::Test,
        // Fail closed: unknown values must not enable permissive local CORS.
        _ => Environment::Prod,
    }
}

fn parse_cors_origins() -> Vec<axum::http::HeaderValue> {
    let raw = std::env::var("CORS_ORIGINS").unwrap_or_default();
    if raw.trim().is_empty() {
        return vec![];
    }

    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| axum::http::HeaderValue::from_str(s).ok())
        .collect()
}

fn supported_api_versions() -> &'static Vec<String> {
    static SUPPORTED: OnceLock<Vec<String>> = OnceLock::new();
    SUPPORTED.get_or_init(|| {
        let configured = std::env::var("SUPPORTED_API_VERSIONS").unwrap_or_default();
        let parsed: Vec<String> = configured
            .split(',')
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToString::to_string)
            .collect();

        if parsed.is_empty() {
            vec!["2025-01-01".to_string()]
        } else {
            parsed
        }
    })
}

fn default_api_version() -> &'static String {
    static DEFAULT: OnceLock<String> = OnceLock::new();
    DEFAULT.get_or_init(|| {
        let supported = supported_api_versions();
        let fallback = supported
            .first()
            .cloned()
            .unwrap_or_else(|| "2025-01-01".to_string());

        match std::env::var("DEFAULT_API_VERSION") {
            Ok(version) if supported.contains(&version) => version,
            Ok(version) => {
                tracing::warn!(
                    default_api_version = %version,
                    supported_versions = ?supported,
                    "DEFAULT_API_VERSION is not in SUPPORTED_API_VERSIONS; falling back"
                );
                fallback
            }
            Err(_) => fallback,
        }
    })
}

pub async fn api_version_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
    if !req.uri().path().starts_with("/v1/") {
        return next.run(req).await;
    }

    let supported = supported_api_versions();
    let requested = req
        .headers()
        .get("x-api-version")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(default_api_version().as_str())
        .to_string();

    if !supported.iter().any(|v| v == &requested) {
        return ApiError::bad_request(
            "api.unsupported_version",
            "Unsupported API version. Set X-Api-Version to a supported version.",
        )
        .with_context(serde_json::json!({
            "requested_version": requested,
            "supported_versions": supported,
            "default_version": default_api_version(),
        }))
        .into_response();
    }

    let mut response = next.run(req).await;
    if let Ok(version_header) = HeaderValue::from_str(&requested) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-api-version"), version_header);
    }
    response
}

pub async fn csrf_protection_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let csrf_protection_enabled = std::env::var("CSRF_PROTECTION")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

    if !csrf_protection_enabled {
        return next.run(req).await;
    }

    let method = req.method();
    if !matches!(
        method,
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
    ) {
        return next.run(req).await;
    }

    // Skip CSRF for authentication endpoints that don't have cookies yet
    let path = req.uri().path();
    let is_auth_endpoint_without_cookie = matches!(
        path,
        "/v1/auth/register"
            | "/v1/auth/login"
            | "/v1/auth/login/start"
            | "/v1/auth/login/finish"
            | "/v1/auth/csrf-token"
            | "/v1/auth/password/reset/request"
            | "/v1/auth/password/reset/verify"
            | "/v1/auth/password/reset/complete"
            | "/v1/auth/passkeys/login/start"
            | "/v1/auth/passkeys/login/finish"
            | "/v1/auth/passkeys/register/start"
            | "/v1/auth/passkeys/register/finish"
    );

    if is_auth_endpoint_without_cookie {
        return next.run(req).await;
    }

    // Only enforce for browser-style cookie flows.
    // Mobile/native clients can keep using bearer tokens without CSRF.
    let headers = req.headers();
    let has_refresh_cookie =
        underlay_http::extract_refresh_token(headers, &state.cookie_config).is_some();
    if !has_refresh_cookie {
        return next.run(req).await;
    }

    let cookie_token = shared::auth::extract_csrf_token(headers, &state.cookie_config);
    let header_token = headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if cookie_token.is_none() || header_token.is_none() || cookie_token != header_token {
        return underlay_http::ApiError::new(
            StatusCode::FORBIDDEN,
            "auth.csrf.invalid",
            "CSRF validation failed",
        )
        .into_response();
    }

    next.run(req).await
}
