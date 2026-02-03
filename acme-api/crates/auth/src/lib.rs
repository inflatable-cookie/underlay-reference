//! Auth boundary types and traits.
//!
//! This crate defines the canonical authentication/authorisation boundary
//! for Acme.

mod config;
mod email_totp;
mod email_totp_repository;
mod email_totp_sender;
mod local;
mod principal;
mod provider;
mod underlay;

pub use config::{AuthConfig, AuthConfigBuilder};
pub use email_totp::{
    EmailTotpConfig, EmailTotpError, EmailTotpResult, EmailTotpService, VerificationSession,
};
pub use email_totp_repository::{AcmeEmailTotpCodeRepository, AcmeVerificationSessionRepository};
pub use email_totp_sender::AcmeEmailTotpSender;
pub use local::{
    AcmeLocalAuthProvider, AcmeLocalAuthService, AuthSession, GoogleOAuthStartResult,
    LoginStartOutcome, PasskeyRecord, SessionFingerprint, TotpSetupResult,
};
pub use principal::{UserId, UserPrincipal, UserRole};
pub use provider::{AuthError, AuthProvider};
pub use underlay::user_principal_from_underlay;
