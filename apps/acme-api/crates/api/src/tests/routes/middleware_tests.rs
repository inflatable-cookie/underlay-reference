//! Direct-router proof for the CSRF seam.
//!
//! These build a real router with the real middleware layered over it, so the
//! assertions cover the policy as it is actually applied — not a reimplementation
//! of it. `CsrfState` is deliberately narrow enough to construct here without
//! standing up auth, email, or blob services.

use super::*;
use axum::body::Body;
use axum::http::{header, Request};
use axum::routing::post;
use axum::Router;
use tower::util::ServiceExt;

const PASSKEY_REGISTER_START: &str = "/v1/auth/passkeys/register/start";
const PASSKEY_REGISTER_FINISH: &str = "/v1/auth/passkeys/register/finish";
const PASSKEY_LOGIN_START: &str = "/v1/auth/passkeys/login/start";
const LOGIN: &str = "/v1/auth/login";
const ACCOUNT_PROFILE: &str = "/v1/account/profile";

async fn ok() -> &'static str {
    "ok"
}

fn cookie_config() -> AuthCookieConfig {
    AuthCookieConfig::new()
}

fn router(enabled: bool) -> Router {
    let csrf = CsrfState::new(enabled, cookie_config());

    Router::new()
        .route(PASSKEY_REGISTER_START, post(ok))
        .route(PASSKEY_REGISTER_FINISH, post(ok))
        .route(PASSKEY_LOGIN_START, post(ok))
        .route(LOGIN, post(ok))
        .route(ACCOUNT_PROFILE, post(ok))
        .layer(axum::middleware::from_fn_with_state(
            csrf,
            csrf_protection_middleware,
        ))
}

/// A browser request carrying a session refresh cookie, i.e. exactly the shape
/// CSRF exists to protect.
fn cookie_backed_request(
    path: &str,
    csrf_cookie: Option<&str>,
    csrf_header: Option<&str>,
) -> Request<Body> {
    let config = cookie_config();
    let mut cookies = format!("{}=refresh-token-value", config.refresh_token_name());
    if let Some(token) = csrf_cookie {
        cookies.push_str(&format!("; {}csrf_token={}", config.cookie_prefix(), token));
    }

    let mut builder = Request::builder()
        .method("POST")
        .uri(path)
        .header(header::COOKIE, cookies);

    if let Some(token) = csrf_header {
        builder = builder.header("x-csrf-token", token);
    }

    builder.body(Body::empty()).unwrap()
}

async fn status(enabled: bool, request: Request<Body>) -> StatusCode {
    router(enabled).oneshot(request).await.unwrap().status()
}

#[tokio::test]
async fn passkey_registration_is_not_a_bootstrap_exemption() {
    // Registration requires an authenticated user, so it is a cookie-backed
    // mutation. Login start/finish are the genuine bootstrap: no session exists
    // yet for an attacker to ride.
    assert!(!is_csrf_exempt_bootstrap_path(PASSKEY_REGISTER_START));
    assert!(!is_csrf_exempt_bootstrap_path(PASSKEY_REGISTER_FINISH));
    assert!(is_csrf_exempt_bootstrap_path(PASSKEY_LOGIN_START));
    assert!(is_csrf_exempt_bootstrap_path(LOGIN));
}

#[tokio::test]
async fn passkey_registration_start_rejects_a_cookie_mutation_without_a_token() {
    assert_eq!(
        status(
            true,
            cookie_backed_request(PASSKEY_REGISTER_START, None, None)
        )
        .await,
        StatusCode::FORBIDDEN,
    );
}

#[tokio::test]
async fn passkey_registration_finish_rejects_a_cookie_mutation_without_a_token() {
    assert_eq!(
        status(
            true,
            cookie_backed_request(PASSKEY_REGISTER_FINISH, None, None)
        )
        .await,
        StatusCode::FORBIDDEN,
    );
}

#[tokio::test]
async fn passkey_registration_rejects_a_mismatched_token() {
    assert_eq!(
        status(
            true,
            cookie_backed_request(
                PASSKEY_REGISTER_START,
                Some("cookie-token"),
                Some("header-token")
            ),
        )
        .await,
        StatusCode::FORBIDDEN,
    );
}

#[tokio::test]
async fn passkey_registration_rejects_a_header_without_a_cookie() {
    assert_eq!(
        status(
            true,
            cookie_backed_request(PASSKEY_REGISTER_START, None, Some("header-token")),
        )
        .await,
        StatusCode::FORBIDDEN,
    );
}

#[tokio::test]
async fn passkey_registration_accepts_a_matching_token() {
    assert_eq!(
        status(
            true,
            cookie_backed_request(PASSKEY_REGISTER_START, Some("matching"), Some("matching")),
        )
        .await,
        StatusCode::OK,
    );
}

#[tokio::test]
async fn other_authenticated_cookie_mutations_stay_protected() {
    assert_eq!(
        status(true, cookie_backed_request(ACCOUNT_PROFILE, None, None)).await,
        StatusCode::FORBIDDEN,
    );
}

#[tokio::test]
async fn unauthenticated_bootstrap_stays_exempt() {
    // Login is reachable before any session cookie exists; requiring a CSRF
    // token there would break the bootstrap it is named for.
    let request = Request::builder()
        .method("POST")
        .uri(LOGIN)
        .body(Body::empty())
        .unwrap();
    assert_eq!(status(true, request).await, StatusCode::OK);
}

#[tokio::test]
async fn bearer_only_clients_are_not_forced_through_csrf() {
    // No refresh cookie: a native/mobile bearer client cannot be CSRF'd.
    let request = Request::builder()
        .method("POST")
        .uri(PASSKEY_REGISTER_START)
        .header(header::AUTHORIZATION, "Bearer access-token")
        .body(Body::empty())
        .unwrap();
    assert_eq!(status(true, request).await, StatusCode::OK);
}

#[tokio::test]
async fn safe_methods_are_never_challenged() {
    let request = Request::builder()
        .method("GET")
        .uri(ACCOUNT_PROFILE)
        .header(
            header::COOKIE,
            cookie_config().refresh_token_name() + "=refresh-token-value",
        )
        .body(Body::empty())
        .unwrap();
    // GET is not registered on this route, so reaching 405 proves the
    // middleware passed the request through to the router untouched.
    assert_eq!(status(true, request).await, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn disabled_protection_lets_a_cookie_mutation_through() {
    // Only reachable in local/effigy/test: `acme_infra::resolve_csrf_protection`
    // rejects the disablement in every deployed environment, so this state
    // cannot be constructed by a deployed runtime.
    assert_eq!(
        status(
            false,
            cookie_backed_request(PASSKEY_REGISTER_START, None, None)
        )
        .await,
        StatusCode::OK,
    );
}
