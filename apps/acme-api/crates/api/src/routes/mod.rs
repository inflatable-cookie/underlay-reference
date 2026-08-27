//! HTTP route assembly for the Acme API.
//!
//! One root builder merges four explicit families, each owning its own module
//! so route placement is visible in the source layout rather than inferred
//! from path strings:
//!
//! - [`runtime`] — health, favicon, OpenAPI/Swagger. Operational, unauthenticated,
//!   exempt from the business version header.
//! - [`shared`] — auth and account. Used by more than one client surface.
//! - [`front`] — authenticated product-user project and task routes.
//! - [`admin`] — `AdminUser`-gated operator routes under `/v1/admin/*`.
//!
//! Public paths are unchanged by this layout. A family is expressed by which
//! builder registers a route, not by rewriting its URL to match a file name.
//!
//! Cross-cutting policy (version header, CSRF) lives in [`middleware`] and is
//! layered above the merged router by `main.rs`, so it applies uniformly
//! rather than per family.
//!
//! `project_description` is shared handler support used by both the front and
//! admin families. It registers no routes and so belongs to no family.

use axum::Router;
use underlay_observability::Environment;

use crate::state::AppState;

mod admin;
mod front;
pub mod middleware;
mod project_description;
pub mod runtime;
pub mod shared;

pub use middleware::{
    api_version_middleware, csrf_protection_middleware, is_versioned_business_path, CsrfState,
};

/// Build the API router with OpenAPI exposed.
///
/// Convenience for tests and tooling. `main.rs` uses
/// [`build_router_with_options`] so exposure follows the environment.
pub fn build_router() -> Router<AppState> {
    build_router_with_options(true)
}

/// Build the API router, optionally exposing Swagger UI and the OpenAPI JSON.
///
/// Pass `include_docs = false` outside development.
pub fn build_router_with_options(include_docs: bool) -> Router<AppState> {
    // Underlay CORS policy:
    // - `CORS_ORIGINS` supplies the explicit allowlist in deployed environments
    // - in local dev an empty list mirrors the request origin
    // - credentials are allowed so cookie auth works without reworking CORS
    let cors = underlay_http::admin_cors_layer_from_env(Environment::resolve(
        "ENVIRONMENT",
        Some("ACME_ENV"),
    ));

    Router::new()
        .merge(runtime::build_runtime_router(include_docs))
        .merge(shared::build_shared_router())
        .merge(front::build_front_router())
        .merge(admin::build_admin_router())
        .layer(cors)
}
