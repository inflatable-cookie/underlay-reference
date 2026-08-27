//! Shared family router: auth and account.
//!
//! Shared does not mean public. These routes live here because more than one
//! client surface uses them — the front app, the admin app, and native
//! clients all authenticate and manage their own account through the same
//! endpoints. Most of them are authenticated.
//!
//! Access posture:
//! - unauthenticated bootstrap (register, login, password reset, passkey
//!   login) carries no session cookie yet and is CSRF-exempt by the
//!   route-family contract's bounded exemption
//! - everything else is an authenticated cookie-backed mutation or read, and
//!   mutations are CSRF-protected. Passkey *registration* belongs to this
//!   group, not to bootstrap
//! - all of it is a business surface and carries the declared
//!   `X-Api-Version` header

use axum::routing::{get, patch, post};
use axum::Router;

use super::{account, auth};
use crate::state::AppState;

pub fn build_shared_router() -> Router<AppState> {
    Router::new()
        // --- Unauthenticated bootstrap -----------------------------------
        .route("/v1/auth/register", post(auth::register))
        .route("/v1/auth/login", post(auth::login))
        .route("/v1/auth/login/start", post(auth::login_start))
        .route("/v1/auth/login/finish", post(auth::login_finish))
        .route("/v1/auth/csrf-token", get(auth::csrf_token))
        .route(
            "/v1/auth/password/reset/request",
            post(auth::password_reset_request),
        )
        .route(
            "/v1/auth/password/reset/verify",
            post(auth::password_reset_verify),
        )
        .route(
            "/v1/auth/password/reset/complete",
            post(auth::password_reset_complete),
        )
        .route(
            "/v1/auth/passkeys/login/start",
            post(auth::passkey_login_start),
        )
        .route(
            "/v1/auth/passkeys/login/finish",
            post(auth::passkey_login_finish),
        )
        .route(
            "/v1/auth/password/requirements",
            get(auth::password_requirements),
        )
        // --- Session lifecycle -------------------------------------------
        .route("/v1/auth/refresh", post(auth::refresh))
        .route("/v1/auth/logout", post(auth::logout))
        .route("/v1/auth/me", get(auth::me))
        .route("/v1/auth/sessions", get(auth::list_sessions))
        .route(
            "/v1/auth/sessions/{session_id}/revoke",
            post(auth::revoke_session),
        )
        // --- Authenticated credential maintenance -------------------------
        .route("/v1/auth/password/change", post(auth::change_password))
        .route(
            "/v1/auth/password/change-2fa",
            post(auth::change_password_with_verification),
        )
        .route("/v1/auth/totp/status", get(auth::totp_status))
        .route("/v1/auth/totp/setup", post(auth::totp_setup))
        .route("/v1/auth/totp/enable", post(auth::totp_enable))
        .route("/v1/auth/totp/disable", post(auth::totp_disable))
        .route("/v1/auth/totp/verify", post(auth::totp_verify))
        .route("/v1/auth/2fa-status", get(auth::two_factor_status))
        .route(
            "/v1/auth/email-totp/request",
            post(auth::email_totp_request),
        )
        .route("/v1/auth/email-totp/verify", post(auth::email_totp_verify))
        .route("/v1/auth/passkeys", get(auth::list_passkeys))
        .route(
            "/v1/auth/passkeys/{credential_id}",
            patch(auth::rename_passkey).delete(auth::delete_passkey),
        )
        .route(
            "/v1/auth/passkeys/register/start",
            post(auth::passkey_register_start),
        )
        .route(
            "/v1/auth/passkeys/register/finish",
            post(auth::passkey_register_finish),
        )
        .route(
            "/v1/auth/passkeys/verify/start",
            post(auth::passkey_verify_start),
        )
        .route(
            "/v1/auth/passkeys/verify/finish",
            post(auth::passkey_verify_finish),
        )
        // --- Account ------------------------------------------------------
        .route(
            "/v1/account/profile",
            get(account::get_profile).patch(account::update_profile),
        )
}
