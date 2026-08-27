//! Cross-tab CSRF issuance proof.
//!
//! The GET handler inspects the incoming cookie before choosing a token. These
//! drive that same issuance decision over HTTP and then the real CSRF
//! middleware, so a handler that always mints would fail the two-tab case.

use super::*;
use crate::routes::{csrf_protection_middleware, CsrfState};
use axum::body::{to_bytes, Body};
use axum::http::Request;
use axum::routing::{get, post};
use axum::Router;
use tower::util::ServiceExt;
use underlay_http::AuthCookieConfig;

async fn ok() -> &'static str {
    "ok"
}

fn cookie_config() -> AuthCookieConfig {
    AuthCookieConfig::new()
}

fn issue_router() -> Router {
    Router::new()
        .route("/v1/auth/csrf-token", get(csrf_token))
        .with_state(cookie_config())
}

fn mutate_router() -> Router {
    Router::new()
        .route("/v1/auth/refresh", post(ok))
        .layer(axum::middleware::from_fn_with_state(
            CsrfState::new(true, cookie_config()),
            csrf_protection_middleware,
        ))
}

fn cookie_header(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .expect("set-cookie pair")
        .trim()
        .to_string()
}

fn cookie_attributes(set_cookie: &str) -> Vec<&str> {
    set_cookie
        .split(';')
        .skip(1)
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

async fn json_token(response: axum::response::Response) -> (String, String, Vec<String>) {
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("csrf set-cookie")
        .to_str()
        .unwrap()
        .to_string();
    let attributes = cookie_attributes(&set_cookie)
        .into_iter()
        .map(str::to_string)
        .collect();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    let token = value["data"]["csrf_token"]
        .as_str()
        .expect("csrf_token")
        .to_string();
    assert_eq!(
        cookie_header(&set_cookie),
        format!("{}={}", csrf_cookie_name(&cookie_config()), token),
        "body and emitted cookie must carry the same token"
    );
    (token, set_cookie, attributes)
}

async fn issue(issue: Router, cookie: Option<&str>) -> axum::response::Response {
    let mut builder = Request::builder().uri("/v1/auth/csrf-token");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    issue
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn an_absent_cookie_mints_a_new_token() {
    let response = issue(issue_router(), None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (token, set_cookie, attributes) = json_token(response).await;
    assert!(!token.is_empty(), "minted token must be non-empty");
    assert_eq!(
        attributes,
        vec![
            "SameSite=Lax".to_string(),
            "Path=/".to_string(),
            format!("Max-Age={}", cookie_config().refresh_token_max_age()),
            "Secure".to_string(),
        ],
        "cookie attributes must stay on the existing issuance contract"
    );
    assert!(set_cookie.starts_with(&format!("{}={}", csrf_cookie_name(&cookie_config()), token)));
}

#[tokio::test]
async fn an_empty_cookie_mints_a_new_token() {
    let empty = format!("{}=", csrf_cookie_name(&cookie_config()));
    let response = issue(issue_router(), Some(&empty)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (token, _, _) = json_token(response).await;
    assert!(!token.is_empty(), "empty cookie must mint, not echo blank");
}

#[tokio::test]
async fn a_non_empty_cookie_is_returned_unchanged() {
    let existing = "existing-csrf-token";
    let cookie = format!("{}={}", csrf_cookie_name(&cookie_config()), existing);
    let response = issue(issue_router(), Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (token, _, attributes) = json_token(response).await;
    assert_eq!(token, existing);
    assert_eq!(
        attributes,
        vec![
            "SameSite=Lax".to_string(),
            "Path=/".to_string(),
            format!("Max-Age={}", cookie_config().refresh_token_max_age()),
            "Secure".to_string(),
        ],
    );
}

#[tokio::test]
async fn a_second_tab_does_not_rotate_the_csrf_cookie_out_from_under_the_first() {
    let issue = issue_router();
    let mutate = mutate_router();
    let config = cookie_config();

    let tab_a_issue = issue
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/auth/csrf-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tab_a_issue.status(), StatusCode::OK);
    let (token_a, set_cookie, _) = json_token(tab_a_issue).await;
    let browser_cookie = cookie_header(&set_cookie);

    // Tab B is another client on the same origin: it sends the cookie tab A
    // just set, and must receive that same token rather than minting a new
    // one that would overwrite the browser-wide cookie.
    let tab_b_issue = issue
        .oneshot(
            Request::builder()
                .uri("/v1/auth/csrf-token")
                .header(header::COOKIE, &browser_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tab_b_issue.status(), StatusCode::OK);
    let (token_b, tab_b_set_cookie, _) = json_token(tab_b_issue).await;
    assert_eq!(token_a, token_b, "second tab rotated the CSRF cookie");
    assert_eq!(
        cookie_header(&tab_b_set_cookie),
        browser_cookie,
        "second tab must not emit a different cookie value"
    );

    // Tab A still holds header token_a. After tab B's fetch, the cookie
    // must still match that header or every mutation 403s.
    let tab_a_refresh = mutate
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/refresh")
                .header(
                    header::COOKIE,
                    format!(
                        "{}=refresh-token-value; {}",
                        config.refresh_token_name(),
                        browser_cookie
                    ),
                )
                .header("x-csrf-token", &token_a)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tab_a_refresh.status(), StatusCode::OK);
}
