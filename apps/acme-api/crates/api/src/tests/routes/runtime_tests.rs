//! Runtime-family proof: OpenAPI exposure follows the environment, and runtime
//! endpoints are never asked for a business API version.

use super::*;
use crate::routes::middleware::{
    api_version_middleware, is_versioned_business_path, ApiVersionState,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use tower::util::ServiceExt;
use underlay_observability::Environment;

async fn business_handler() -> &'static str {
    "ok"
}

/// The version vocabulary as it ships in `config/default.toml`, resolved the
/// same way bootstrap resolves it.
fn version_state() -> ApiVersionState {
    ApiVersionState::from_behavior(&acme_infra::AppBehaviorConfig::default().api)
}

/// The runtime family layered under the real version middleware, plus one
/// business route to prove the exemption is scoped rather than global.
fn router(environment: Environment) -> Router {
    build_runtime_router_for_environment(environment)
        .route("/v1/projects", get(business_handler))
        .with_state(())
        .layer(axum::middleware::from_fn_with_state(
            version_state(),
            api_version_middleware,
        ))
}

async fn status(include_docs: bool, path: &str, version: Option<&str>) -> StatusCode {
    let environment = if include_docs {
        Environment::Dev
    } else {
        Environment::Prod
    };
    status_for_environment(environment, path, version).await
}

async fn status_for_environment(
    environment: Environment,
    path: &str,
    version: Option<&str>,
) -> StatusCode {
    let mut builder = Request::builder().uri(path);
    if let Some(version) = version {
        builder = builder.header("x-api-version", version);
    }
    router(environment)
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

#[test]
fn runtime_paths_are_not_a_versioned_business_surface() {
    for path in [
        "/v1/health",
        "/favicon.ico",
        "/api/openapi.json",
        "/api/docs",
        "/api/docs/index.html",
    ] {
        assert!(is_runtime_path(path), "{path} should be runtime-owned");
        assert!(
            !is_versioned_business_path(path),
            "{path} must not require the business version header"
        );
    }
}

#[test]
fn business_families_all_carry_the_declared_version_header() {
    // Once declared, the header applies across shared, front, and admin — not
    // to one pocket while the others drift.
    for path in [
        "/v1/auth/login",
        "/v1/account/profile",
        "/v1/projects",
        "/v1/admin/users",
    ] {
        assert!(!is_runtime_path(path), "{path} is a business surface");
        assert!(
            is_versioned_business_path(path),
            "{path} must carry the declared version header"
        );
    }
}

#[tokio::test]
async fn health_answers_without_a_version_header() {
    assert_eq!(status(true, "/v1/health", None).await, StatusCode::OK);
}

#[tokio::test]
async fn health_answers_even_with_an_unsupported_version_header() {
    // Platform infrastructure should not have to know the app's version
    // vocabulary to probe liveness.
    assert_eq!(
        status(true, "/v1/health", Some("1999-01-01")).await,
        StatusCode::OK,
    );
}

#[tokio::test]
async fn a_business_route_still_rejects_an_unsupported_version() {
    assert_eq!(
        status(true, "/v1/projects", Some("1999-01-01")).await,
        StatusCode::BAD_REQUEST,
    );
}

#[tokio::test]
async fn favicon_returns_no_content() {
    assert_eq!(
        status(true, "/favicon.ico", None).await,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn openapi_is_served_when_docs_are_included() {
    assert_eq!(
        status(true, "/api/openapi.json", None).await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn openapi_is_absent_when_docs_are_excluded() {
    // The production, staging, test, and fail-closed unknown environments do
    // not mount either documentation route.
    assert_eq!(
        status(false, "/api/openapi.json", None).await,
        StatusCode::NOT_FOUND,
    );
    assert_eq!(
        status(false, "/api/docs", None).await,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn openapi_is_absent_for_staging_test_and_unknown_environments() {
    for environment in [
        Environment::Staging,
        Environment::Test,
        Environment::parse("unknown"),
    ] {
        assert_eq!(
            status_for_environment(environment, "/api/openapi.json", None).await,
            StatusCode::NOT_FOUND,
        );
        assert_eq!(
            status_for_environment(environment, "/api/docs", None).await,
            StatusCode::NOT_FOUND,
        );
    }
}
