//! Admin family router: operator surfaces for the admin UI and tooling.
//!
//! Access posture: every route is gated by the `AdminUser` extractor, sits
//! under `/v1/admin/*`, CSRF-protects cookie-backed mutations, and carries the
//! declared `X-Api-Version` header.
//!
//! Operator and system surfaces (jobs, scheduled tasks, error logs, activity)
//! stay in this family. The route-family contract forbids parallel
//! `/v1/system/*` or `/v1/operator/*` roots for ordinary admin work.

use axum::routing::{delete, get, post, put};
use axum::Router;

use super::{
    activity, categories, dashboard, error_logs, jobs, media, projects, scheduled_tasks, tasks,
    users, validation,
};
use crate::state::AppState;

pub fn build_admin_router() -> Router<AppState> {
    Router::new()
        // Dashboard stats
        .route(
            "/v1/admin/dashboard/stats",
            get(dashboard::get_dashboard_stats),
        )
        // User management
        .route(
            "/v1/admin/users",
            get(users::list_users).post(users::create_user),
        )
        .route(
            "/v1/admin/users/{user_id}",
            get(users::get_user).put(users::update_user),
        )
        .route(
            "/v1/admin/users/{user_id}/role",
            put(users::update_user_role),
        )
        .route(
            "/v1/admin/users/{user_id}/suspend",
            post(users::suspend_user),
        )
        .route(
            "/v1/admin/users/{user_id}/unsuspend",
            post(users::unsuspend_user),
        )
        .route(
            "/v1/admin/users/{user_id}/activity",
            get(activity::list_activity_for_user),
        )
        .route(
            "/v1/admin/users/{user_id}/sessions",
            get(users::list_user_sessions),
        )
        .route(
            "/v1/admin/users/{user_id}/sessions/{session_id}/revoke",
            post(users::revoke_user_session),
        )
        // Activity/audit log
        .route("/v1/admin/activity", get(activity::list_activity))
        .route(
            "/v1/admin/activity/entity/{entity_type}/{entity_id}",
            get(activity::list_activity_for_entity),
        )
        // Validation endpoint (for async form validation)
        .route("/v1/admin/validate-field", post(validation::validate_field))
        // Category admin routes
        .route(
            "/v1/admin/categories",
            get(categories::list_categories).post(categories::create_category),
        )
        .route(
            "/v1/admin/categories/reorder",
            put(categories::reorder_categories),
        )
        .route(
            "/v1/admin/categories:batch-delete",
            post(categories::batch_delete_categories),
        )
        .route(
            "/v1/admin/categories/{category_id}",
            get(categories::get_category)
                .patch(categories::update_category)
                .delete(categories::soft_delete_category),
        )
        .route(
            "/v1/admin/categories/{category_id}/restore",
            post(categories::restore_category),
        )
        // Project admin routes
        .route(
            "/v1/admin/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/v1/admin/projects/reorder",
            put(projects::reorder_projects),
        )
        .route(
            "/v1/admin/projects:batch-delete",
            post(projects::batch_delete_projects),
        )
        .route(
            "/v1/admin/projects/{project_id}",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::soft_delete_project),
        )
        .route(
            "/v1/admin/projects/{project_id}/restore",
            post(projects::restore_project),
        )
        // Task admin routes (nested under projects)
        .route(
            "/v1/admin/projects/{project_id}/tasks",
            get(tasks::list_tasks).post(tasks::create_task),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/reorder",
            put(tasks::reorder_tasks),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/batch-delete",
            post(tasks::batch_delete_tasks),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/batch-update",
            post(tasks::batch_update_task_status),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/{task_id}",
            get(tasks::get_task)
                .patch(tasks::update_task)
                .delete(tasks::soft_delete_task),
        )
        .route(
            "/v1/admin/projects/{project_id}/tasks/{task_id}/labels",
            get(tasks::get_task_labels).put(tasks::set_task_labels),
        )
        // Label admin routes (nested under projects)
        .route(
            "/v1/admin/projects/{project_id}/labels",
            get(tasks::list_labels).post(tasks::create_label),
        )
        .route(
            "/v1/admin/projects/{project_id}/labels/{label_id}",
            get(tasks::get_label)
                .patch(tasks::update_label)
                .delete(tasks::soft_delete_label),
        )
        // ====================================================================
        // Media Library admin routes
        // ====================================================================
        .route(
            "/v1/admin/media/check-duplicate",
            post(media::check_duplicate),
        )
        .route(
            "/v1/admin/media:batch-delete",
            post(media::batch_delete_media),
        )
        .route(
            "/v1/admin/media",
            get(media::list_media).post(media::create_media),
        )
        .route("/v1/admin/media/trash", get(media::list_media_trash))
        .route(
            "/v1/admin/media/{media_id}",
            get(media::get_media)
                .put(media::update_media)
                .delete(media::purge_media),
        )
        .route(
            "/v1/admin/media/{media_id}/soft-delete",
            post(media::soft_delete_media),
        )
        .route(
            "/v1/admin/media/{media_id}/restore",
            post(media::restore_media),
        )
        .route(
            "/v1/admin/media/{media_id}/versions",
            get(media::list_versions),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/initiate-upload",
            post(media::initiate_upload),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/{version_id}/finalise-upload",
            post(media::finalise_upload),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/{version_id}/activate",
            post(media::activate_version),
        )
        .route(
            "/v1/admin/media/{media_id}/versions/{version_id}",
            delete(media::delete_version),
        )
        .route("/v1/admin/media/{media_id}/usage", get(media::list_usage))
        // ====================================================================
        // Background Jobs admin routes
        // ====================================================================
        .route("/v1/admin/jobs", get(jobs::list_jobs))
        .route("/v1/admin/jobs/stats", get(jobs::get_job_stats))
        .route("/v1/admin/jobs/{job_id}", get(jobs::get_job))
        .route("/v1/admin/jobs/{job_id}/cancel", post(jobs::cancel_job))
        .route("/v1/admin/jobs/{job_id}/retry", post(jobs::retry_job))
        // ====================================================================
        // Scheduled Tasks admin routes
        // ====================================================================
        .route(
            "/v1/admin/scheduled-tasks",
            get(scheduled_tasks::list_scheduled_tasks),
        )
        .route(
            "/v1/admin/scheduled-tasks/{task_id}",
            get(scheduled_tasks::get_scheduled_task),
        )
        .route(
            "/v1/admin/scheduled-tasks/{task_id}/toggle",
            post(scheduled_tasks::toggle_scheduled_task),
        )
        .route(
            "/v1/admin/scheduled-tasks/{task_id}/trigger",
            post(scheduled_tasks::trigger_scheduled_task),
        )
        // ====================================================================
        // Error Logs admin routes
        // ====================================================================
        .route(
            "/v1/admin/error-logs",
            get(error_logs::list_error_logs_handler),
        )
        .route(
            "/v1/admin/error-logs/stats",
            get(error_logs::get_error_log_stats),
        )
        .route(
            "/v1/admin/error-logs/{id}",
            get(error_logs::get_error_log_handler),
        )
}
