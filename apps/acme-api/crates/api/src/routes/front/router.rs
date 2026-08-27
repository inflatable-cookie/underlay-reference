//! Front family router: authenticated product-user project and task routes.
//!
//! Access posture: `AuthenticatedUser`, no role gate, CSRF on cookie-backed
//! mutations, and the declared `X-Api-Version` header like every other
//! business family.

use axum::routing::{get, patch};
use axum::Router;

use super::tasks;
use crate::state::AppState;

pub fn build_front_router() -> Router<AppState> {
    Router::new()
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
        .route(
            "/v1/projects/{project_id}/tasks",
            get(tasks::list_tasks).post(tasks::create_task),
        )
        .route(
            "/v1/projects/{project_id}/tasks/{task_id}",
            patch(tasks::update_task).delete(tasks::delete_task),
        )
}
