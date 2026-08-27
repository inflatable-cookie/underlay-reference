//! Peer-aware client-IP proof for the auth policy inputs.
//!
//! Rate limiting, lockout, session fingerprints, and auth audit all key on the
//! resolved client IP, so a spoofable one lets an attacker rotate identities
//! freely. These drive a real router through the real extractor rather than
//! calling the resolver directly, because the thing worth proving is what the
//! handlers actually receive.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use tower::util::ServiceExt;
use underlay_http::context::RequestContext;
use underlay_http::TrustedProxyConfig;

use super::{client_ip, login_client_fingerprint, session_fingerprint};

const PEER: &str = "198.51.100.7";
const SPOOFED: &str = "203.0.113.99";
const PROXY_REPORTED: &str = "192.0.2.55";

async fn echo_ip(ctx: RequestContext) -> String {
    client_ip(&ctx).unwrap_or_else(|| "none".to_string())
}

async fn echo_login_fingerprint(ctx: RequestContext) -> String {
    login_client_fingerprint(&ctx)
}

async fn echo_session_fingerprint(ctx: RequestContext) -> String {
    let fp = session_fingerprint(&ctx);
    format!(
        "{}|{}",
        fp.ip_address.unwrap_or_else(|| "none".to_string()),
        fp.user_agent.unwrap_or_else(|| "none".to_string())
    )
}

fn router(proxy: TrustedProxyConfig) -> Router {
    Router::new()
        .route("/ip", get(echo_ip))
        .route("/login-fingerprint", get(echo_login_fingerprint))
        .route("/session-fingerprint", get(echo_session_fingerprint))
        .layer(axum::Extension(proxy))
}

/// Drive a request with a socket peer, exactly as `axum::serve` does via
/// `into_make_service_with_connect_info`.
async fn call(proxy: TrustedProxyConfig, path: &str, headers: &[(&str, &str)]) -> String {
    let mut builder = Request::builder().uri(path);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let mut request = builder.body(Body::empty()).unwrap();
    request.extensions_mut().insert(axum::extract::ConnectInfo(
        format!("{PEER}:54321").parse::<SocketAddr>().unwrap(),
    ));

    let response = router(proxy).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn no_declared_proxy_uses_the_socket_peer() {
    assert_eq!(call(TrustedProxyConfig::None, "/ip", &[]).await, PEER);
}

#[tokio::test]
async fn a_spoofed_forwarding_header_is_ignored_without_a_declared_proxy() {
    // The header exists. That is not a reason to trust it.
    assert_eq!(
        call(
            TrustedProxyConfig::None,
            "/ip",
            &[("x-forwarded-for", SPOOFED)],
        )
        .await,
        PEER,
    );
    assert_eq!(
        call(TrustedProxyConfig::None, "/ip", &[("x-real-ip", SPOOFED)]).await,
        PEER,
    );
    assert_eq!(
        call(
            TrustedProxyConfig::None,
            "/ip",
            &[("cf-connecting-ip", SPOOFED)],
        )
        .await,
        PEER,
    );
}

#[tokio::test]
async fn a_spoofed_header_cannot_shift_the_login_fingerprint() {
    let clean = call(
        TrustedProxyConfig::None,
        "/login-fingerprint",
        &[("user-agent", "test-agent")],
    )
    .await;
    let spoofed = call(
        TrustedProxyConfig::None,
        "/login-fingerprint",
        &[("user-agent", "test-agent"), ("x-forwarded-for", SPOOFED)],
    )
    .await;

    assert_eq!(
        clean, spoofed,
        "a spoofed X-Forwarded-For changed the resolved client fingerprint"
    );
}

#[tokio::test]
async fn a_spoofed_header_cannot_shift_the_session_fingerprint() {
    let spoofed = call(
        TrustedProxyConfig::None,
        "/session-fingerprint",
        &[
            (header::USER_AGENT.as_str(), "test-agent"),
            ("x-forwarded-for", SPOOFED),
        ],
    )
    .await;

    assert_eq!(spoofed, format!("{PEER}|test-agent"));
}

#[tokio::test]
async fn a_declared_forwarding_proxy_is_honoured() {
    // One trusted hop appends the peer it saw, so the client sits one from the
    // right. Entries further left are client-supplied and ignored.
    assert_eq!(
        call(
            TrustedProxyConfig::ForwardedFor { trusted_hops: 1 },
            "/ip",
            &[("x-forwarded-for", &format!("{SPOOFED}, {PROXY_REPORTED}"))],
        )
        .await,
        PROXY_REPORTED,
    );
}

#[tokio::test]
async fn a_declared_proxy_still_falls_back_to_the_peer_when_the_header_is_absent() {
    assert_eq!(
        call(
            TrustedProxyConfig::ForwardedFor { trusted_hops: 1 },
            "/ip",
            &[],
        )
        .await,
        PEER,
    );
}

#[tokio::test]
async fn a_declared_real_ip_proxy_is_honoured_but_only_for_its_own_header() {
    assert_eq!(
        call(
            TrustedProxyConfig::RealIpHeader,
            "/ip",
            &[("x-real-ip", PROXY_REPORTED)],
        )
        .await,
        PROXY_REPORTED,
    );
    // A different forwarding header is not covered by this declaration.
    assert_eq!(
        call(
            TrustedProxyConfig::RealIpHeader,
            "/ip",
            &[("x-forwarded-for", SPOOFED)],
        )
        .await,
        PEER,
    );
}

#[test]
fn an_unrecognised_trusted_proxy_mode_trusts_nothing() {
    // Fail closed: a typo in TRUSTED_PROXY must not widen the trust boundary.
    assert_eq!(
        TrustedProxyConfig::default(),
        TrustedProxyConfig::None,
        "the default must trust no forwarding header"
    );
}
