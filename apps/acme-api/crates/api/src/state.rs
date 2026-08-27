//! Application state and authentication extractors for the Acme API.

use acme_auth::{user_principal_from_underlay, EmailTotpService, UserPrincipal, UserRole};
use acme_db::DbPool;
use acme_infra::EmailConfig;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use once_cell::sync::OnceCell;
use serde_json::json;
use std::sync::Arc;
use underlay_auth::Authenticated;
use underlay_blob::BlobAdapter;
use underlay_email::{EmailManager, EmailTemplateEngine};
use underlay_http::AuthCookieConfig;
use underlay_jobs_postgres::JobRepository;

use crate::config::AcmeConfig;

/// Global database pool for use in middleware (e.g., error logging).
pub static DB_POOL: OnceCell<DbPool> = OnceCell::new();

/// Core application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    pub local_auth: Arc<acme_auth::AcmeLocalAuthService>,
    pub auth_provider: Arc<dyn underlay_auth::AuthProvider>,
    pub cookie_config: AuthCookieConfig,
    pub email_manager: Arc<EmailManager>,
    pub email_templates: Arc<EmailTemplateEngine>,
    pub email_totp: Arc<EmailTotpService>,
    pub email_config: EmailConfig,
    pub blob_adapter: Arc<dyn BlobAdapter>,
    pub job_repository: Option<Arc<JobRepository>>,
    /// Application configuration.
    pub config: AcmeConfig,
}

impl underlay_auth::HasAuthProvider for AppState {
    fn auth_provider(&self) -> &dyn underlay_auth::AuthProvider {
        self.auth_provider.as_ref()
    }
}

/// Minimal auth extractor for Acme.
///
/// This is wired through Underlay's `underlay-auth` boundary so that Acme can
/// swap providers without rewriting API handlers.
///
/// Behaviour:
/// - A valid `Authorization: Bearer <access token>` is required.
pub struct AuthenticatedUser(pub UserPrincipal);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync + underlay_auth::HasAuthProvider,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Authenticated(principal) = Authenticated::from_request_parts(parts, state)
            .await
            .map_err(|rej| rej.into_response())?;

        Ok(AuthenticatedUser(user_principal_from_underlay(principal)))
    }
}

/// Admin-only extractor for Acme.
///
/// Validates that the user is both authenticated AND has the Admin role.
/// Returns 403 Forbidden if the user lacks admin privileges.
///
/// Use this instead of `AuthenticatedUser` + manual `has_role(UserRole::Admin)` checks
/// in admin route handlers.
pub struct AdminUser(pub UserPrincipal);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync + underlay_auth::HasAuthProvider,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthenticatedUser(user) = AuthenticatedUser::from_request_parts(parts, state).await?;

        if !user.has_role(UserRole::Admin) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "ok": false,
                    "error": {
                        "code": "auth.forbidden",
                        "message": "Admin access required."
                    }
                })),
            )
                .into_response());
        }

        Ok(AdminUser(user))
    }
}
