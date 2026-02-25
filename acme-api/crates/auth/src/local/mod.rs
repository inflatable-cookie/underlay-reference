use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::time::SystemTime;

use acme_core::Uuid;
use acme_db::auth::{
    consume_verification_session, create_verification_session, EmailTotpPurpose,
    VerificationMethod, VerificationSessionRow,
};
use acme_infra::AppBehaviorConfig;

use crate::config::AuthConfig;
#[allow(unused_imports)] // These will be used when email TOTP is integrated
use crate::email_totp::{EmailTotpService, VerificationSession};

use underlay_auth::state::{AuthStateError, AuthStateStore};
use underlay_auth::{
    AuthError, AuthResult, Credential, CredentialMetadata, CredentialType, RoleSet, Session,
    SessionStatus, User, UserStatus,
};
use underlay_auth_jwt::{token_fingerprint, JwtBehaviorDefaults, JwtConfig, JwtService};
#[allow(unused_imports)] // Prepared for future OAuth implementation
use underlay_auth_oauth::{
    GoogleOAuthAppService, GoogleOAuthService, OAuthCallbackRequest, OAuthLoginState, OAuthStart,
    OAuthTokenCipher, TokenSet,
};
use underlay_auth_password::{
    Argon2Hasher, PasswordHasherExt, PasswordStrengthAnalyzer, PasswordVerifierExt,
};
use underlay_auth_totp::{TotpConfig, TotpService, TwoFactorVerified};
use underlay_auth_webauthn::{WebAuthnConfig, WebAuthnService};
use underlay_ratelimit::{InMemoryBackend, RateLimitBackend, RateLimitConfig};

use crate::rate_limiter::DynamicRateLimiter;
use crate::redis_rate_limit::RedisRateLimitBackend;
use webauthn_rs_proto::{attest::RegisterPublicKeyCredential, auth::PublicKeyCredential};

mod auth_state;
mod helpers;
mod lockout;
mod login;
mod passkey;
mod password;
mod password_reset;
mod rate_limit;
mod session;
mod totp;
mod user;

async fn verify_password_blocking(
    hasher: Argon2Hasher,
    password: Vec<u8>,
    secret_encrypted: String,
) -> AuthResult<bool> {
    tokio::task::spawn_blocking(move || {
        hasher
            .verify_password(&password, &secret_encrypted)
            .map_err(|_| AuthError::Internal("Failed to verify password".into()))
    })
    .await
    .map_err(|_| AuthError::Internal("Failed to verify password".into()))?
}

async fn hash_password_blocking(hasher: Argon2Hasher, password: Vec<u8>) -> AuthResult<String> {
    tokio::task::spawn_blocking(move || {
        hasher
            .hash_password(&password)
            .map_err(|_| AuthError::Internal("Failed to hash password".into()))
    })
    .await
    .map_err(|_| AuthError::Internal("Failed to hash password".into()))?
}

fn map_auth_state_error(err: AuthStateError) -> AuthError {
    match err {
        AuthStateError::InvalidOrExpired => {
            AuthError::BadRequest("Invalid or expired auth state".into())
        }
        AuthStateError::Encode(_) | AuthStateError::Decode(_) => {
            AuthError::Internal("Failed to encode auth state".into())
        }
        AuthStateError::Db(_) => AuthError::Internal("DB error".into()),
    }
}

fn is_local_or_dev_environment() -> bool {
    let environment = std::env::var("ENVIRONMENT")
        .or_else(|_| std::env::var("ACME_ENV"))
        .unwrap_or_else(|_| "local".to_string())
        .to_ascii_lowercase();

    matches!(
        environment.as_str(),
        "local" | "dev" | "development" | "test"
    )
}

#[derive(Debug, Clone)]
pub enum LoginStartOutcome {
    Complete {
        session: Box<AuthSession>,
        role: String,
    },
    TwoFactorRequired {
        login_state_id: Uuid,
    },
    /// Email verification required (when no TOTP/passkey configured and email fallback enforced).
    /// The route handler should call email_totp.request_code() with the provided user info.
    EmailVerificationRequired {
        login_state_id: Uuid,
        user_id: Uuid,
        email: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct LoginTwoFactorState {
    user_id: String,
    client_fingerprint: String,
    attempts: u32,
}

/// Client fingerprint for session tracking and validation.
///
/// Used to detect suspicious token refresh attempts from different
/// clients than the session was created with.
#[derive(Debug, Clone, Default)]
pub struct SessionFingerprint {
    /// Client IP address (from X-Forwarded-For or X-Real-IP).
    pub ip_address: Option<String>,
    /// User-Agent header value.
    pub user_agent: Option<String>,
}

impl SessionFingerprint {
    /// Create a new fingerprint with the given values.
    pub fn new(ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self {
            ip_address,
            user_agent,
        }
    }

    /// Check if fingerprint matches another (for refresh validation).
    ///
    /// Returns true if fingerprints are compatible:
    /// - If either has no IP, IP check is skipped
    /// - If either has no User-Agent, UA check is skipped
    /// - Both must match if both are present
    pub fn matches(&self, other: &SessionFingerprint) -> bool {
        // IP check - skip if either is None
        let ip_matches = match (&self.ip_address, &other.ip_address) {
            (Some(a), Some(b)) => a == b,
            _ => true, // Skip check if either is missing
        };

        // User-Agent check - skip if either is None
        let ua_matches = match (&self.user_agent, &other.user_agent) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        };

        ip_matches && ua_matches
    }

    /// Check if there's a significant mismatch worth logging.
    ///
    /// Returns a description of what changed, or None if compatible.
    pub fn mismatch_description(&self, other: &SessionFingerprint) -> Option<String> {
        let mut mismatches = Vec::new();

        if let (Some(a), Some(b)) = (&self.ip_address, &other.ip_address) {
            if a != b {
                mismatches.push(format!("IP changed from {} to {}", a, b));
            }
        }

        if let (Some(a), Some(b)) = (&self.user_agent, &other.user_agent) {
            if a != b {
                mismatches.push("User-Agent changed".to_string());
            }
        }

        if mismatches.is_empty() {
            None
        } else {
            Some(mismatches.join(", "))
        }
    }
}

pub struct AcmeLocalAuthService {
    pool: sqlx::PgPool,
    auth_state: AuthStateStore,
    jwt: JwtService,
    password_hasher: Argon2Hasher,
    password_analyzer: PasswordStrengthAnalyzer,
    totp: TotpService,
    webauthn: WebAuthnService,
    webauthn_rp_id: String,
    webauthn_rp_origin: String,
    #[allow(dead_code)] // Prepared for future OAuth implementation
    google_oauth: Option<GoogleOAuthAppService>,
    #[allow(dead_code)] // Prepared for future OAuth implementation
    oauth_cipher: Option<OAuthTokenCipher>,
    rate_limiter: DynamicRateLimiter,
    config: AuthConfig,
    encryption: Option<acme_infra::EncryptionService>,
    argon2_memory_kb: u32,
    argon2_iterations: u32,
    argon2_parallelism: u32,
}

impl AcmeLocalAuthService {
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    pub fn webauthn_rp_id(&self) -> &str {
        &self.webauthn_rp_id
    }

    pub fn webauthn_rp_origin(&self) -> &str {
        &self.webauthn_rp_origin
    }

    pub fn from_env(pool: sqlx::PgPool) -> AuthResult<Self> {
        let behavior = AppBehaviorConfig::load();
        let private_key_b64 = std::env::var("AUTH_JWT_PRIVATE_KEY")
            .map_err(|_| AuthError::Internal("AUTH_JWT_PRIVATE_KEY not set".to_string()))?;
        let public_key_b64 = std::env::var("AUTH_JWT_PUBLIC_KEY")
            .map_err(|_| AuthError::Internal("AUTH_JWT_PUBLIC_KEY not set".to_string()))?;
        let cfg = JwtConfig::from_values(
            private_key_b64,
            public_key_b64,
            JwtBehaviorDefaults {
                access_token_lifetime_minutes: behavior.auth.jwt_access_token_lifetime_minutes,
                refresh_token_lifetime_days: behavior.auth.jwt_refresh_token_lifetime_days,
                issuer: behavior.auth.jwt_issuer.clone(),
                audience: behavior.auth.jwt_audience.clone(),
                leeway_seconds: behavior.auth.jwt_leeway_seconds,
            },
        );
        let jwt = JwtService::new(cfg).map_err(AuthError::from)?;

        let rate_limit_backend = std::env::var("RATE_LIMIT_BACKEND")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_default();
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        // Absolute session timeout (default comes from layered config)
        let absolute_session_timeout_days: u64 = std::env::var("SESSION_MAX_ABSOLUTE_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(behavior.auth.absolute_session_timeout_days);
        let absolute_session_timeout =
            std::time::Duration::from_secs(absolute_session_timeout_days * 24 * 60 * 60);

        let config = AuthConfig {
            login_rate_limit_per_hour: behavior.auth.login_rate_limit_per_hour,
            register_rate_limit_per_hour: behavior.auth.register_rate_limit_per_hour,
            password_change_rate_limit_per_hour: behavior.auth.password_change_rate_limit_per_hour,
            passkey_register_rate_limit_per_hour: behavior
                .auth
                .passkey_register_rate_limit_per_hour,
            passkey_login_rate_limit_per_hour: behavior.auth.passkey_login_rate_limit_per_hour,
            refresh_rate_limit_per_hour: behavior.auth.refresh_rate_limit_per_hour,
            rate_limit_backend,
            redis_url,
            max_totp_attempts: behavior.auth.max_totp_attempts,
            max_email_code_attempts: behavior.auth.max_email_code_attempts,
            max_backup_code_attempts: behavior.auth.max_backup_code_attempts,
            totp_state_timeout: std::time::Duration::from_secs(
                behavior.auth.totp_state_timeout_secs,
            ),
            email_state_timeout: std::time::Duration::from_secs(
                behavior.auth.email_state_timeout_secs,
            ),
            verification_session_timeout: std::time::Duration::from_secs(
                behavior.auth.verification_session_timeout_secs,
            ),
            rate_limit_cleanup_interval: std::time::Duration::from_secs(
                behavior.auth.rate_limit_cleanup_interval_secs,
            ),
            absolute_session_timeout,
            rate_limit_retry_after_short: std::time::Duration::from_secs(
                behavior.auth.rate_limit_retry_after_short_secs,
            ),
            rate_limit_retry_after_long: std::time::Duration::from_secs(
                behavior.auth.rate_limit_retry_after_long_secs,
            ),
            max_failed_logins: behavior.auth.max_failed_logins,
            lockout_duration: std::time::Duration::from_secs(behavior.auth.lockout_duration_secs),
            security_alert_window: std::time::Duration::from_secs(
                behavior.auth.security_alert_window_secs,
            ),
            security_alert_cooldown: std::time::Duration::from_secs(
                behavior.auth.security_alert_cooldown_secs,
            ),
            security_alert_failed_attempts_threshold: behavior
                .auth
                .security_alert_failed_attempts_threshold,
            security_alert_distinct_users_threshold: behavior
                .auth
                .security_alert_distinct_users_threshold,
            security_alert_lockouts_threshold: behavior.auth.security_alert_lockouts_threshold,
            email_code_expiry: std::time::Duration::from_secs(behavior.auth.email_code_expiry_secs),
            max_email_codes_per_hour: behavior.auth.max_email_codes_per_hour,
        };

        let totp = TotpService::new(Some(TotpConfig {
            issuer: behavior.auth.totp_issuer.clone(),
            ..TotpConfig::default()
        }));

        // WebAuthn relying party configuration
        let webauthn_rp_id = behavior.auth.webauthn_rp_id.clone();
        let webauthn_rp_origin = behavior.auth.webauthn_rp_origin.clone();
        let webauthn_rp_name = behavior.auth.webauthn_rp_name.clone();

        let webauthn = WebAuthnService::new(WebAuthnConfig {
            rp_id: webauthn_rp_id.clone(),
            rp_origin: webauthn_rp_origin.clone(),
            rp_name: webauthn_rp_name,
        })?;

        let google_oauth = match GoogleOAuthService::from_env() {
            Ok(provider) => Some(GoogleOAuthAppService::new(provider)),
            Err(_) => None,
        };

        let oauth_cipher = OAuthTokenCipher::from_env_optional().ok().flatten();

        // Rate limiter with background cleanup
        let rate_limiter = match config.rate_limit_backend {
            crate::redis_rate_limit::RateLimitBackendType::InMemory => {
                DynamicRateLimiter::in_memory(InMemoryBackend::with_cleanup(
                    config.rate_limit_cleanup_interval,
                ))
            }
            crate::redis_rate_limit::RateLimitBackendType::Redis => {
                let redis_backend = RedisRateLimitBackend::new(&config.redis_url).map_err(|e| {
                    AuthError::Internal(format!(
                        "Failed to connect to Redis for rate limiting: {}",
                        e
                    ))
                })?;
                DynamicRateLimiter::redis(redis_backend)
            }
        };

        // Argon2 password hashing parameters (configurable via environment)
        // Defaults: 128 MiB memory, 4 iterations, 4 parallelism
        let argon2_memory_kb: u32 = behavior.auth.argon2_memory_kb;
        let argon2_iterations: u32 = behavior.auth.argon2_iterations;
        let argon2_parallelism: u32 = behavior.auth.argon2_parallelism;

        let password_hasher =
            Argon2Hasher::with_params(argon2_memory_kb, argon2_iterations, argon2_parallelism);

        // Initialize encryption service for TOTP secrets.
        let encryption = acme_infra::EncryptionService::from_env();
        if encryption.is_none() {
            if is_local_or_dev_environment() {
                tracing::warn!(
                    "ENCRYPTION_KEY not set - TOTP secrets will be stored in plaintext in local/dev"
                );
            } else {
                return Err(AuthError::Internal(
                    "ENCRYPTION_KEY must be set outside local/dev/test environments".into(),
                ));
            }
        }

        Ok(Self {
            auth_state: AuthStateStore::new(pool.clone()),
            pool,
            jwt,
            password_hasher,
            password_analyzer: PasswordStrengthAnalyzer::new()
                .with_min_length(behavior.auth.password_min_length),
            totp,
            webauthn,
            webauthn_rp_id,
            webauthn_rp_origin,
            google_oauth,
            oauth_cipher,
            rate_limiter,
            config,
            encryption,
            argon2_memory_kb,
            argon2_iterations,
            argon2_parallelism,
        })
    }
}

// ========================================================================
// Public types
// ========================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TotpSetupResult {
    pub setup_id: Uuid,
    pub otpauth_uri: String,
    pub qr_svg: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TotpSetupState {
    secret_base32: String,
    backup_code_hashes: Vec<String>,
    metadata: CredentialMetadata,
}

#[derive(Debug, Clone)]
pub struct TwoFactorStatus {
    pub has_totp_configured: bool,
    pub has_passkey_configured: bool,
    pub totp_enabled_at: Option<DateTime<Utc>>,
    pub passkey_count: u32,
}

#[derive(Debug, Clone)]
struct TotpDetails {
    credential_id: Uuid,
    secret_base32: String,
    last_counter: u64,
    backup_code_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GoogleOAuthStartResult {
    pub authorization_url: String,
    pub state_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct PasskeyRecord {
    pub credential: Credential,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub user: User,
    pub session: Session,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
struct Tokens {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Clone)]
struct DbSession {
    id: Uuid,
    user_id: Uuid,
    roles: Vec<String>,
    is_active: bool,

    access_token_fingerprint: String,
    refresh_token_fingerprint: String,

    refresh_token_id: Uuid,
    refresh_token_version: i32,

    access_token_expires_at: DateTime<Utc>,
    refresh_token_expires_at: DateTime<Utc>,

    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,

    ip_address: Option<String>,
    user_agent: Option<String>,

    status: SessionStatus,
    revocation_reason: Option<String>,
    revoked_at: Option<DateTime<Utc>>,
}

impl DbSession {
    fn into_public(self) -> Session {
        Session {
            id: self.id,
            user_id: self.user_id,
            access_token_fingerprint: self.access_token_fingerprint,
            refresh_token_fingerprint: self.refresh_token_fingerprint,
            access_token_expires_at: self.access_token_expires_at,
            refresh_token_expires_at: self.refresh_token_expires_at,
            created_at: self.created_at,
            last_used_at: self.last_used_at,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            status: self.status,
            revocation_reason: self.revocation_reason,
            revoked_at: self.revoked_at,
        }
    }
}

// ========================================================================
// Auth Provider trait implementation
// ========================================================================

#[derive(Clone)]
pub struct AcmeLocalAuthProvider {
    service: std::sync::Arc<AcmeLocalAuthService>,
}

impl AcmeLocalAuthProvider {
    pub fn new(service: std::sync::Arc<AcmeLocalAuthService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl underlay_auth::AuthProvider for AcmeLocalAuthProvider {
    async fn authenticate_bearer(
        &self,
        bearer_token: &str,
    ) -> AuthResult<underlay_auth::Principal> {
        let (user_id, roles) = self.service.verify_access_principal(bearer_token).await?;
        Ok(underlay_auth::Principal {
            user_id,
            roles: RoleSet::new(roles),
        })
    }
}
