use crate::UserPrincipal;
use acme_core::AppError;

/// Error type returned by auth providers.
#[derive(Debug)]
pub struct AuthError {
    pub code: &'static str,
    pub message: String,
}

impl AuthError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        AuthError {
            code,
            message: message.into(),
        }
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        AppError::new(err.code, err.message)
    }
}

/// Trait describing how to validate tokens or sessions at the auth boundary.
///
/// Concrete implementations (local auth, external IdP, legacy provider, etc.)
/// live in separate modules or crates and are selected at startup.
pub trait AuthProvider: Send + Sync {
    /// Validate an access token (e.g. Bearer token from `Authorization` header).
    fn verify_access_token(&self, token: &str) -> Result<UserPrincipal, AuthError>;

    /// Validate a session cookie value, if the deployment uses cookie-based auth.
    fn verify_session(&self, session: &str) -> Result<UserPrincipal, AuthError>;
}
