//! Route handlers for the Acme API.
//!
//! This module organizes routes into:
//! - `shared/` - Auth, health, account routes (used by all clients)
//! - `tasks` - User-facing project/task routes
//! - `admin/` - Admin-only routes with enhanced features

use axum::http::header::HeaderName;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use underlay_http::{cors_layer, CorsConfig};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;
use crate::state::AppState;

mod admin;
mod shared;
mod tasks;

/// Build the main API router with all routes configured.
pub fn build_router() -> Router<AppState> {
    let cors = build_cors_layer();

    Router::new()
        // OpenAPI / Swagger UI
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        // Health
        .route("/v1/health", get(shared::health::health))
        // Auth routes
        .route("/v1/auth/register", post(shared::auth::register))
        .route("/v1/auth/login", post(shared::auth::login))
        .route("/v1/auth/login/start", post(shared::auth::login_start))
        .route("/v1/auth/login/finish", post(shared::auth::login_finish))
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
            "/v1/auth/passkeys/:credential_id",
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
            "/v1/auth/sessions/:session_id/revoke",
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
            "/v1/projects/:project_id",
            get(tasks::get_project)
                .patch(tasks::update_project)
                .delete(tasks::delete_project),
        )
        // Task routes
        .route(
            "/v1/projects/:project_id/tasks",
            get(tasks::list_tasks).post(tasks::create_task),
        )
        .route(
            "/v1/projects/:project_id/tasks/:task_id",
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
        .route("/v1/admin/users", get(admin::users::list_users))
        .route("/v1/admin/users/:user_id", get(admin::users::get_user))
        .route(
            "/v1/admin/users/:user_id/role",
            put(admin::users::update_user_role),
        )
        .route(
            "/v1/admin/users/:user_id/suspend",
            post(admin::users::suspend_user),
        )
        .route(
            "/v1/admin/users/:user_id/unsuspend",
            post(admin::users::unsuspend_user),
        )
        .route(
            "/v1/admin/users/:user_id/activity",
            get(admin::activity::list_activity_for_user),
        )
        .route(
            "/v1/admin/users/:user_id/sessions",
            get(admin::users::list_user_sessions),
        )
        .route(
            "/v1/admin/users/:user_id/sessions/:session_id/revoke",
            post(admin::users::revoke_user_session),
        )
        // Activity/audit log
        .route("/v1/admin/activity", get(admin::activity::list_activity))
        .route(
            "/v1/admin/activity/entity/:entity_type/:entity_id",
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
            "/v1/admin/categories/:category_id",
            get(admin::categories::get_category)
                .patch(admin::categories::update_category)
                .delete(admin::categories::soft_delete_category),
        )
        .route(
            "/v1/admin/categories/:category_id/restore",
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
            "/v1/admin/projects/:project_id",
            get(admin::projects::get_project)
                .patch(admin::projects::update_project)
                .delete(admin::projects::soft_delete_project),
        )
        .route(
            "/v1/admin/projects/:project_id/restore",
            post(admin::projects::restore_project),
        )
        // Task admin routes (nested under projects)
        .route(
            "/v1/admin/projects/:project_id/tasks",
            get(admin::tasks::list_tasks).post(admin::tasks::create_task),
        )
        .route(
            "/v1/admin/projects/:project_id/tasks/reorder",
            put(admin::tasks::reorder_tasks),
        )
        .route(
            "/v1/admin/projects/:project_id/tasks/batch-delete",
            post(admin::tasks::batch_delete_tasks),
        )
        .route(
            "/v1/admin/projects/:project_id/tasks/batch-update",
            post(admin::tasks::batch_update_task_status),
        )
        .route(
            "/v1/admin/projects/:project_id/tasks/:task_id",
            get(admin::tasks::get_task)
                .patch(admin::tasks::update_task)
                .delete(admin::tasks::soft_delete_task),
        )
        .route(
            "/v1/admin/projects/:project_id/tasks/:task_id/labels",
            get(admin::tasks::get_task_labels).put(admin::tasks::set_task_labels),
        )
        // Label admin routes (nested under projects)
        .route(
            "/v1/admin/projects/:project_id/labels",
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
        .route(
            "/v1/admin/media/paginated",
            get(admin::media::list_media_paginated),
        )
        .route("/v1/admin/media/trash", get(admin::media::list_media_trash))
        .route(
            "/v1/admin/media/:media_id",
            get(admin::media::get_media)
                .put(admin::media::update_media)
                .delete(admin::media::purge_media),
        )
        .route(
            "/v1/admin/media/:media_id/soft-delete",
            post(admin::media::soft_delete_media),
        )
        .route(
            "/v1/admin/media/:media_id/restore",
            post(admin::media::restore_media),
        )
        .route(
            "/v1/admin/media/:media_id/versions",
            get(admin::media::list_versions),
        )
        .route(
            "/v1/admin/media/:media_id/versions/initiate-upload",
            post(admin::media::initiate_upload),
        )
        .route(
            "/v1/admin/media/:media_id/versions/:version_id/finalise-upload",
            post(admin::media::finalise_upload),
        )
        .route(
            "/v1/admin/media/:media_id/versions/:version_id/activate",
            post(admin::media::activate_version),
        )
        .route(
            "/v1/admin/media/:media_id/versions/:version_id",
            delete(admin::media::delete_version),
        )
        .route(
            "/v1/admin/media/:media_id/usage",
            get(admin::media::list_usage),
        )
        // ====================================================================
        // Background Jobs admin routes
        // ====================================================================
        .route("/v1/admin/jobs", get(admin::jobs::list_jobs))
        .route("/v1/admin/jobs/stats", get(admin::jobs::get_job_stats))
        .route("/v1/admin/jobs/:job_id", get(admin::jobs::get_job))
        .route(
            "/v1/admin/jobs/:job_id/cancel",
            post(admin::jobs::cancel_job),
        )
        .route("/v1/admin/jobs/:job_id/retry", post(admin::jobs::retry_job))
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
            "/v1/admin/error-logs/:id",
            get(admin::error_logs::get_error_log_handler),
        )
        .layer(cors)
}

fn build_cors_layer() -> tower_http::cors::CorsLayer {
    // Underlay CORS policy (matches guide patterns):
    // - Use `CORS_ORIGINS` in production.
    // - In local/dev, if `CORS_ORIGINS` is unset, mirror the request origin.
    // - Allow credentials so cookie-based auth can be enabled without reworking CORS.

    let env = std::env::var("ENVIRONMENT")
        .or_else(|_| std::env::var("ACME_ENV"))
        .unwrap_or_else(|_| "local".to_string());

    let origins = parse_cors_origins();

    // If no explicit origins are set and we're in local/dev, mirror request origin.
    let mirror_origin = origins.is_empty() && (env == "local" || env == "dev");

    // NOTE: the browser will preflight if we send `X-Api-Version`.
    // Add it explicitly to allowed headers.
    let mut config = CorsConfig::default()
        .with_header(HeaderName::from_static("x-api-version"))
        .with_credentials(true);

    if mirror_origin {
        config = config.with_mirror_origin();
    } else if !origins.is_empty() {
        config.allowed_origins = origins;
        config.allow_any_origin = false;
    }

    cors_layer(config)
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
