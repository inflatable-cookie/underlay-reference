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

use crate::config::AuthConfig;
#[allow(unused_imports)] // These will be used when email TOTP is integrated
use crate::email_totp::{EmailTotpService, VerificationSession};

use underlay_auth::{
    AuthError, AuthResult, Credential, CredentialMetadata, CredentialType, RoleSet, Session,
    SessionStatus, User, UserStatus,
};
use underlay_auth_jwt::{token_fingerprint, JwtConfig, JwtService};
#[allow(unused_imports)] // Prepared for future OAuth implementation
use underlay_auth_oauth::{
    GoogleOAuthAppService, GoogleOAuthService, OAuthCallbackRequest, OAuthLoginState, OAuthStart,
    OAuthTokenCipher, TokenSet,
};
use underlay_auth_password::{
    Argon2Hasher, PasswordHasherExt, PasswordStrengthAnalyzer, PasswordVerifierExt,
};
use underlay_auth_state::{AuthStateError, AuthStateStore};
use underlay_auth_totp::{TotpConfig, TotpService, TwoFactorVerified};
use underlay_auth_webauthn::{WebAuthnConfig, WebAuthnService};
use underlay_ratelimit::{InMemoryBackend, RateLimitBackend, RateLimitConfig};
use webauthn_rs_proto::{attest::RegisterPublicKeyCredential, auth::PublicKeyCredential};

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
#[serde(rename_all = "camelCase")]
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
    rate_limiter: InMemoryBackend,
    config: AuthConfig,
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
        let cfg = JwtConfig::from_env().map_err(AuthError::from)?;
        let jwt = JwtService::new(cfg).map_err(AuthError::from)?;

        let totp = TotpService::new(Some(TotpConfig {
            issuer: "Acme".to_string(),
            ..TotpConfig::default()
        }));

        // WebAuthn relying party configuration
        let webauthn_rp_id = std::env::var("WEBAUTHN_RP_ID")
            .unwrap_or_else(|_| "localhost".to_string());
        let webauthn_rp_origin = std::env::var("WEBAUTHN_RP_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:4174".to_string());
        let webauthn_rp_name = std::env::var("WEBAUTHN_RP_NAME")
            .unwrap_or_else(|_| "Acme".to_string());

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

        // Load auth configuration (uses defaults, can be extended for env var support)
        let config = AuthConfig::default();

        // Rate limiter with background cleanup
        let rate_limiter = InMemoryBackend::with_cleanup(config.rate_limit_cleanup_interval);

        // Argon2 password hashing parameters (configurable via environment)
        // Defaults: 64 MiB memory, 3 iterations, 4 parallelism
        let argon2_memory_kb: u32 = std::env::var("ARGON2_MEMORY_KB")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(65536);
        let argon2_iterations: u32 = std::env::var("ARGON2_ITERATIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);
        let argon2_parallelism: u32 = std::env::var("ARGON2_PARALLELISM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4);

        let password_hasher =
            Argon2Hasher::with_params(argon2_memory_kb, argon2_iterations, argon2_parallelism);

        Ok(Self {
            auth_state: AuthStateStore::new(pool.clone()),
            pool,
            jwt,
            password_hasher,
            password_analyzer: PasswordStrengthAnalyzer::new().with_min_length(12),
            totp,
            webauthn,
            webauthn_rp_id,
            webauthn_rp_origin,
            google_oauth,
            oauth_cipher,
            rate_limiter,
            config,
        })
    }

    /// Check rate limit for login attempts.
    async fn check_login_rate_limit(&self, email: &str, ip: Option<&str>) -> AuthResult<()> {
        let key = match ip {
            Some(ip) => format!("login:{}:{}", email.to_lowercase(), ip),
            None => format!("login:{}", email.to_lowercase()),
        };

        let rl_config = RateLimitConfig::per_hour(self.config.login_rate_limit_per_hour.into());

        let result = self
            .rate_limiter
            .check_and_increment(&key, &rl_config)
            .await
            .map_err(|e| AuthError::Internal(format!("Rate limit error: {}", e)))?;

        if result.is_denied() {
            return Err(AuthError::RateLimited {
                retry_after_seconds: result.retry_after_secs(),
            });
        }

        Ok(())
    }

    /// Check rate limit for registration attempts.
    async fn check_register_rate_limit(&self, ip: Option<&str>) -> AuthResult<()> {
        let key = match ip {
            Some(ip) => format!("register:{}", ip),
            None => return Ok(()), // Can't rate limit without IP
        };

        let rl_config = RateLimitConfig::per_hour(self.config.register_rate_limit_per_hour.into());

        let result = self
            .rate_limiter
            .check_and_increment(&key, &rl_config)
            .await
            .map_err(|e| AuthError::Internal(format!("Rate limit error: {}", e)))?;

        if result.is_denied() {
            return Err(AuthError::RateLimited {
                retry_after_seconds: result.retry_after_secs(),
            });
        }

        Ok(())
    }

    /// Check rate limit for password change attempts.
    async fn check_password_change_rate_limit(&self, user_id: Uuid) -> AuthResult<()> {
        let key = format!("password_change:{}", user_id);

        let rl_config = RateLimitConfig::per_hour(self.config.password_change_rate_limit_per_hour.into());

        let result = self
            .rate_limiter
            .check_and_increment(&key, &rl_config)
            .await
            .map_err(|e| AuthError::Internal(format!("Rate limit error: {}", e)))?;

        if result.is_denied() {
            return Err(AuthError::RateLimited {
                retry_after_seconds: result.retry_after_secs(),
            });
        }

        Ok(())
    }

    /// Check rate limit for passkey registration start.
    async fn check_passkey_register_rate_limit(&self, user_id: Uuid) -> AuthResult<()> {
        let key = format!("passkey_register:{}", user_id);

        let rl_config = RateLimitConfig::per_hour(self.config.passkey_register_rate_limit_per_hour.into());

        let result = self
            .rate_limiter
            .check_and_increment(&key, &rl_config)
            .await
            .map_err(|e| AuthError::Internal(format!("Rate limit error: {}", e)))?;

        if result.is_denied() {
            return Err(AuthError::RateLimited {
                retry_after_seconds: result.retry_after_secs(),
            });
        }

        Ok(())
    }

    /// Check rate limit for passkey login start.
    async fn check_passkey_login_rate_limit(&self, email: Option<&str>) -> AuthResult<()> {
        let key = match email {
            Some(email) => format!("passkey_login:{}", email.to_lowercase()),
            None => "passkey_login:discoverable".to_string(),
        };

        let rl_config = RateLimitConfig::per_hour(self.config.passkey_login_rate_limit_per_hour.into());

        let result = self
            .rate_limiter
            .check_and_increment(&key, &rl_config)
            .await
            .map_err(|e| AuthError::Internal(format!("Rate limit error: {}", e)))?;

        if result.is_denied() {
            return Err(AuthError::RateLimited {
                retry_after_seconds: result.retry_after_secs(),
            });
        }

        Ok(())
    }

    /// Validate password strength.
    fn validate_password(&self, password: &str) -> AuthResult<()> {
        if password.trim().is_empty() {
            return Err(AuthError::BadRequest("Password is required".into()));
        }

        self.password_analyzer.validate(password).map_err(|msg| {
            if msg.contains("common") {
                AuthError::BadRequest(
                    "This password is too common. Please choose a stronger password.".into(),
                )
            } else {
                AuthError::PasswordTooWeak
            }
        })?;

        Ok(())
    }

    /// Get password requirements configuration.
    pub fn password_requirements(&self) -> underlay_auth_password::PasswordRequirements {
        self.password_analyzer.requirements()
    }

    /// Check if a user is currently locked out.
    async fn check_lockout(&self, user_id: Uuid) -> AuthResult<Option<u64>> {
        let row = sqlx::query(
            r#"
            SELECT lockout_until
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error checking lockout".into()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let lockout_until: Option<DateTime<Utc>> = row.get("lockout_until");

        if let Some(until) = lockout_until {
            let now = Utc::now();
            if until > now {
                let remaining = (until - now).num_seconds().max(0) as u64;
                return Ok(Some(remaining));
            }
        }

        Ok(None)
    }

    /// Record a failed login attempt.
    async fn record_failed_login(
        &self,
        user_id: Uuid,
        ip: Option<&str>,
        reason: &str,
    ) -> AuthResult<Option<u64>> {
        // Log the attempt
        sqlx::query(
            r#"
            INSERT INTO auth.login_attempts (user_id, ip_address, success, failure_reason)
            VALUES ($1, $2::inet, FALSE, $3)
            "#,
        )
        .bind(user_id.into_inner())
        .bind(ip)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error logging login attempt".into()))?;

        // Increment failed count and check for lockout
        let row = sqlx::query(
            r#"
            UPDATE auth.users
            SET
                failed_login_count = failed_login_count + 1,
                lockout_until = CASE
                    WHEN failed_login_count + 1 >= $2
                    THEN NOW() + ($3 || ' seconds')::interval
                    ELSE lockout_until
                END
            WHERE id = $1
            RETURNING failed_login_count, lockout_until
            "#,
        )
        .bind(user_id.into_inner())
        .bind(self.config.max_failed_logins as i32)
        .bind(self.config.lockout_duration_secs().to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error recording failed login".into()))?;

        let lockout_until: Option<DateTime<Utc>> = row.get("lockout_until");

        if let Some(until) = lockout_until {
            let now = Utc::now();
            if until > now {
                let remaining = (until - now).num_seconds().max(0) as u64;
                return Ok(Some(remaining));
            }
        }

        Ok(None)
    }

    /// Reset failed login attempts after successful login.
    async fn reset_failed_logins(&self, user_id: Uuid, ip: Option<&str>) -> AuthResult<()> {
        // Log the successful attempt
        sqlx::query(
            r#"
            INSERT INTO auth.login_attempts (user_id, ip_address, success)
            VALUES ($1, $2::inet, TRUE)
            "#,
        )
        .bind(user_id.into_inner())
        .bind(ip)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error logging login attempt".into()))?;

        // Reset the counters
        sqlx::query(
            r#"
            UPDATE auth.users
            SET failed_login_count = 0, lockout_until = NULL
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error resetting failed logins".into()))?;

        Ok(())
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> AuthResult<AuthSession> {
        self.register_with_ip(email, password, display_name, None)
            .await
    }

    pub async fn register_with_ip(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
        ip: Option<&str>,
    ) -> AuthResult<AuthSession> {
        // Check rate limit before any database lookups
        self.check_register_rate_limit(ip).await?;

        if email.trim().is_empty() {
            return Err(AuthError::BadRequest("Email is required".into()));
        }
        if display_name.trim().is_empty() {
            return Err(AuthError::BadRequest("Display name is required".into()));
        }
        if password.is_empty() {
            return Err(AuthError::BadRequest("Password is required".into()));
        }

        let existing = self.find_user_by_email(email).await?;
        if existing.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        let user = self.create_user(email, display_name, "user").await?;
        self.set_password(user.id, password).await?;

        let roles = roles_for_user("user");
        let (tokens, session) = self.create_session(user.id, roles).await?;

        Ok(AuthSession {
            user,
            session,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    /// Verify user credentials (email + password) with rate limiting and lockout.
    async fn verify_user_credentials(
        &self,
        email: &str,
        password: &str,
        ip: Option<&str>,
    ) -> AuthResult<(User, Credential, String)> {
        // Check rate limit before any database lookups
        self.check_login_rate_limit(email, ip).await?;

        let Some(user) = self.find_user_by_email(email).await? else {
            return Err(AuthError::WrongCredentials);
        };

        if user.status != UserStatus::Active {
            return Err(match user.status {
                UserStatus::Suspended => AuthError::AccountSuspended,
                UserStatus::Deleted => AuthError::AccountDeleted,
                UserStatus::Active => AuthError::Unauthorized,
            });
        }

        // Check if account is locked out
        if let Some(retry_after) = self.check_lockout(user.id).await? {
            return Err(AuthError::RateLimited {
                retry_after_seconds: retry_after,
            });
        }

        let Some((credential, role)) = self.find_password_credential_and_role(user.id).await?
        else {
            return Err(AuthError::WrongCredentials);
        };

        let ok = verify_password_blocking(
            self.password_hasher.clone(),
            password.as_bytes().to_vec(),
            credential.secret_encrypted.clone(),
        )
        .await?;

        if !ok {
            // Record failed attempt and check for lockout
            if let Some(retry_after) = self
                .record_failed_login(user.id, ip, "wrong_password")
                .await?
            {
                return Err(AuthError::RateLimited {
                    retry_after_seconds: retry_after,
                });
            }
            return Err(AuthError::WrongCredentials);
        }

        // Password correct - reset failed login count
        self.reset_failed_logins(user.id, ip).await?;

        self.update_credential_last_used(credential.id).await?;

        Ok((user, credential, role))
    }

    pub async fn login_with_password(
        &self,
        email: &str,
        password: &str,
        code: Option<&str>,
    ) -> AuthResult<AuthSession> {
        self.login_with_password_and_ip(email, password, code, None)
            .await
    }

    pub async fn login_with_password_and_ip(
        &self,
        email: &str,
        password: &str,
        code: Option<&str>,
        ip: Option<&str>,
    ) -> AuthResult<AuthSession> {
        let (user, _credential, role) = self.verify_user_credentials(email, password, ip).await?;

        if let Some(totp) = self.find_totp_details(user.id).await? {
            let Some(code) = code else {
                return Err(AuthError::TwoFactorRequired);
            };
            self.verify_totp_second_factor(&totp, code).await?;
        }

        let roles = roles_for_user(&role);
        let (tokens, session) = self.create_session(user.id, roles).await?;

        Ok(AuthSession {
            user,
            session,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    pub async fn login_start_with_password(
        &self,
        email: &str,
        password: &str,
        client_fingerprint: &str,
    ) -> AuthResult<LoginStartOutcome> {
        self.login_start_with_password_and_ip(email, password, client_fingerprint, None)
            .await
    }

    pub async fn login_start_with_password_and_ip(
        &self,
        email: &str,
        password: &str,
        client_fingerprint: &str,
        ip: Option<&str>,
    ) -> AuthResult<LoginStartOutcome> {
        let (user, _credential, role) = self.verify_user_credentials(email, password, ip).await?;

        if self.find_totp_details(user.id).await?.is_some() {
            let state_id = self
                .create_public_auth_state(
                    "login_2fa",
                    serde_json::to_value(LoginTwoFactorState {
                        user_id: user.id.to_string(),
                        client_fingerprint: client_fingerprint.to_string(),
                        attempts: 0,
                    })
                    .map_err(|_| AuthError::Internal("Failed to encode auth state".into()))?,
                    Duration::minutes(self.config.totp_state_timeout_minutes()),
                )
                .await?;

            return Ok(LoginStartOutcome::TwoFactorRequired {
                login_state_id: state_id,
            });
        }

        let roles = roles_for_user(&role);
        let (tokens, session) = self.create_session(user.id, roles).await?;

        Ok(LoginStartOutcome::Complete {
            session: Box::new(AuthSession {
                user,
                session,
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            }),
            role,
        })
    }

    /// Login with password, with optional email fallback enforcement.
    pub async fn login_start_with_email_fallback(
        &self,
        email: &str,
        password: &str,
        client_fingerprint: &str,
        ip: Option<&str>,
        enforce_email_fallback: bool,
    ) -> AuthResult<LoginStartOutcome> {
        let (user, _credential, role) = self.verify_user_credentials(email, password, ip).await?;

        // Check for TOTP configuration
        if self.find_totp_details(user.id).await?.is_some() {
            let state_id = self
                .create_public_auth_state(
                    "login_2fa",
                    serde_json::to_value(LoginTwoFactorState {
                        user_id: user.id.to_string(),
                        client_fingerprint: client_fingerprint.to_string(),
                        attempts: 0,
                    })
                    .map_err(|_| AuthError::Internal("Failed to encode auth state".into()))?,
                    Duration::minutes(self.config.totp_state_timeout_minutes()),
                )
                .await?;

            return Ok(LoginStartOutcome::TwoFactorRequired {
                login_state_id: state_id,
            });
        }

        // No TOTP configured - check if email fallback is enforced
        if enforce_email_fallback {
            let state_id = self
                .create_public_auth_state(
                    "login_email",
                    serde_json::to_value(LoginTwoFactorState {
                        user_id: user.id.to_string(),
                        client_fingerprint: client_fingerprint.to_string(),
                        attempts: 0,
                    })
                    .map_err(|_| AuthError::Internal("Failed to encode auth state".into()))?,
                    Duration::minutes(self.config.email_state_timeout_minutes()),
                )
                .await?;

            return Ok(LoginStartOutcome::EmailVerificationRequired {
                login_state_id: state_id,
                user_id: user.id,
                email: user.email,
            });
        }

        // No 2FA required, complete login
        let roles = roles_for_user(&role);
        let (tokens, session) = self.create_session(user.id, roles).await?;

        Ok(LoginStartOutcome::Complete {
            session: Box::new(AuthSession {
                user,
                session,
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            }),
            role,
        })
    }

    /// Get user ID from an email login state.
    pub async fn get_email_login_state(
        &self,
        login_state_id: Uuid,
        client_fingerprint: &str,
    ) -> AuthResult<Uuid> {
        let state_value = self
            .load_public_auth_state(login_state_id, "login_email")
            .await?
            .ok_or(AuthError::BadRequest(
                "Invalid or expired login state".into(),
            ))?;

        let state: LoginTwoFactorState = serde_json::from_value(state_value)
            .map_err(|_| AuthError::BadRequest("Invalid login state".into()))?;

        if state.client_fingerprint != client_fingerprint {
            let _ = self.delete_auth_state(login_state_id).await;
            return Err(AuthError::BadRequest(
                "Invalid or expired login state".into(),
            ));
        }

        if state.attempts >= self.config.max_email_code_attempts {
            let _ = self.delete_auth_state(login_state_id).await;
            return Err(AuthError::RateLimited {
                retry_after_seconds: self.config.retry_after_long_secs(),
            });
        }

        Uuid::parse_str(&state.user_id)
            .map_err(|_| AuthError::BadRequest("Invalid login state".into()))
    }

    /// Increment attempt count for email login state.
    pub async fn increment_email_login_attempts(&self, login_state_id: Uuid) -> AuthResult<()> {
        let state_value = self
            .load_public_auth_state(login_state_id, "login_email")
            .await?
            .ok_or(AuthError::BadRequest(
                "Invalid or expired login state".into(),
            ))?;

        let mut state: LoginTwoFactorState = serde_json::from_value(state_value)
            .map_err(|_| AuthError::BadRequest("Invalid login state".into()))?;

        state.attempts += 1;
        let _ = self
            .update_public_auth_state(
                login_state_id,
                "login_email",
                serde_json::to_value(state)
                    .map_err(|_| AuthError::Internal("Failed to encode auth state".into()))?,
            )
            .await;
        Ok(())
    }

    /// Complete email login after code has been verified externally.
    pub async fn login_finish_email_verified(
        &self,
        login_state_id: Uuid,
        session_fingerprint: Option<SessionFingerprint>,
    ) -> AuthResult<(AuthSession, String)> {
        let state_value = self
            .load_public_auth_state(login_state_id, "login_email")
            .await?
            .ok_or(AuthError::BadRequest(
                "Invalid or expired login state".into(),
            ))?;

        let state: LoginTwoFactorState = serde_json::from_value(state_value)
            .map_err(|_| AuthError::BadRequest("Invalid login state".into()))?;

        let user_id = Uuid::parse_str(&state.user_id)
            .map_err(|_| AuthError::BadRequest("Invalid login state".into()))?;

        // Delete the login state
        let _ = self.delete_auth_state(login_state_id).await;

        // Find user and create session
        let user = self
            .find_user_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;
        let role = self.get_user_role(user_id).await?.unwrap_or_default();

        let roles = roles_for_user(&role);
        let (tokens, session) = self
            .create_session_with_fingerprint(user.id, roles, session_fingerprint)
            .await?;

        Ok((
            AuthSession {
                user,
                session,
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            },
            role,
        ))
    }

    pub async fn login_finish_with_totp(
        &self,
        login_state_id: Uuid,
        code: &str,
        client_fingerprint: &str,
        session_fingerprint: Option<SessionFingerprint>,
    ) -> AuthResult<(AuthSession, String)> {
        // Return TwoFactorNotSetUp if no TOTP state exists - this allows
        // the caller to fall back to email verification
        let state_value = match self
            .load_public_auth_state(login_state_id, "login_2fa")
            .await?
        {
            Some(v) => v,
            None => return Err(AuthError::TwoFactorNotSetUp),
        };

        let mut state: LoginTwoFactorState = serde_json::from_value(state_value)
            .map_err(|_| AuthError::BadRequest("Invalid login state".into()))?;

        if state.client_fingerprint != client_fingerprint {
            let _ = self.delete_auth_state(login_state_id).await;
            return Err(AuthError::BadRequest(
                "Invalid or expired login state".into(),
            ));
        }

        if state.attempts >= self.config.max_totp_attempts {
            let _ = self.delete_auth_state(login_state_id).await;
            return Err(AuthError::RateLimited {
                retry_after_seconds: self.config.retry_after_short_secs(),
            });
        }

        let user_id = Uuid::parse_str(&state.user_id)
            .map_err(|_| AuthError::BadRequest("Invalid login state".into()))?;

        let user = self
            .find_user_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        if user.status != UserStatus::Active {
            let _ = self.delete_auth_state(login_state_id).await;
            return Err(match user.status {
                UserStatus::Suspended => AuthError::AccountSuspended,
                UserStatus::Deleted => AuthError::AccountDeleted,
                UserStatus::Active => AuthError::Unauthorized,
            });
        }

        let Some(totp) = self.find_totp_details(user.id).await? else {
            let _ = self.delete_auth_state(login_state_id).await;
            return Err(AuthError::TwoFactorNotSetUp);
        };

        if let Err(err) = self.verify_totp_second_factor(&totp, code).await {
            state.attempts = state.attempts.saturating_add(1);
            let state_json = serde_json::to_value(&state)
                .map_err(|_| AuthError::Internal("Failed to encode auth state".into()))?;
            let _ = self
                .update_public_auth_state(login_state_id, "login_2fa", state_json)
                .await;

            if state.attempts >= self.config.max_totp_attempts {
                let _ = self.delete_auth_state(login_state_id).await;
                return Err(AuthError::RateLimited {
                    retry_after_seconds: self.config.retry_after_short_secs(),
                });
            }

            return Err(err);
        }

        // Success: state is single-use.
        self.delete_auth_state(login_state_id).await?;

        let role = self
            .get_user_role(user.id)
            .await?
            .unwrap_or_else(|| "user".to_string());

        let roles = roles_for_user(&role);
        let (tokens, session) = self
            .create_session_with_fingerprint(user.id, roles, session_fingerprint)
            .await?;

        Ok((
            AuthSession {
                user,
                session,
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            },
            role,
        ))
    }

    pub async fn refresh(&self, refresh_token: &str) -> AuthResult<AuthSession> {
        self.refresh_with_fingerprint(refresh_token, None).await
    }

    /// Refresh tokens with optional fingerprint validation.
    pub async fn refresh_with_fingerprint(
        &self,
        refresh_token: &str,
        current_fingerprint: Option<SessionFingerprint>,
    ) -> AuthResult<AuthSession> {
        let claims = self
            .jwt
            .verify_refresh_token(refresh_token)
            .map_err(AuthError::from)?;

        let mut session = self
            .get_session(claims.session_id)
            .await?
            .ok_or(AuthError::SessionRevoked)?;

        if !session.is_active {
            return Err(AuthError::SessionRevoked);
        }

        if session.refresh_token_fingerprint != token_fingerprint(refresh_token) {
            return Err(AuthError::TokenFingerprintMismatch);
        }

        if session.refresh_token_id != claims.common.token_id {
            return Err(AuthError::TokenInvalid);
        }

        let expected_version =
            i32::try_from(claims.version).map_err(|_| AuthError::TokenInvalid)?;
        if session.refresh_token_version != expected_version {
            return Err(AuthError::TokenInvalid);
        }

        // Validate session fingerprint if provided
        if let Some(ref current) = current_fingerprint {
            let stored = SessionFingerprint {
                ip_address: session.ip_address.clone(),
                user_agent: session.user_agent.clone(),
            };

            if let Some(mismatch) = stored.mismatch_description(current) {
                tracing::warn!(
                    session_id = %session.id,
                    user_id = %session.user_id,
                    mismatch = %mismatch,
                    "Session fingerprint mismatch on token refresh"
                );
            }
        }

        let roles = session.roles.clone();

        let (new_access_token, access_claims) = self
            .jwt
            .issue_access_token(session.user_id, session.id, roles.clone())
            .map_err(AuthError::from)?;

        let (new_refresh_token, refresh_claims) = self
            .jwt
            .issue_refresh_token(
                session.user_id,
                session.id,
                Some(session.refresh_token_id),
                claims.version + 1,
            )
            .map_err(AuthError::from)?;

        session.access_token_fingerprint = token_fingerprint(&new_access_token);
        session.refresh_token_fingerprint = token_fingerprint(&new_refresh_token);
        session.refresh_token_id = refresh_claims.common.token_id;
        session.refresh_token_version =
            i32::try_from(refresh_claims.version).map_err(|_| AuthError::TokenInvalid)?;
        session.access_token_expires_at = timestamp_to_datetime(access_claims.common.expires_at);
        session.refresh_token_expires_at = timestamp_to_datetime(refresh_claims.common.expires_at);
        session.updated_at = Utc::now();
        session.last_used_at = Utc::now();

        // Update IP/User-Agent if provided
        if let Some(fp) = current_fingerprint {
            if fp.ip_address.is_some() {
                session.ip_address = fp.ip_address;
            }
            if fp.user_agent.is_some() {
                session.user_agent = fp.user_agent;
            }
        }

        self.update_session(&session).await?;

        let user = self
            .find_user_by_id(session.user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(AuthSession {
            user,
            session: session.into_public(),
            access_token: new_access_token,
            refresh_token: new_refresh_token,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> AuthResult<()> {
        let claims = self
            .jwt
            .verify_refresh_token(refresh_token)
            .map_err(AuthError::from)?;
        self.revoke_session(claims.session_id, "logout").await
    }

    /// List all sessions for a given user.
    pub async fn list_sessions(&self, user_id: Uuid) -> AuthResult<Vec<Session>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, roles, is_active,
                   access_token_fingerprint, refresh_token_fingerprint,
                   refresh_token_id, refresh_token_version,
                   access_token_expires_at, refresh_token_expires_at,
                   created_at, updated_at, last_used_at,
                   ip_address, user_agent,
                   status, revocation_reason, revoked_at
            FROM auth.sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(rows
            .into_iter()
            .map(|r| map_session_row(r).into_public())
            .collect())
    }

    /// Revoke a specific session for a user.
    pub async fn revoke_session_for_user(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        reason: &str,
    ) -> AuthResult<()> {
        let session = self.get_session(session_id).await?;

        match session {
            Some(s) if s.user_id == user_id => self.revoke_session(session_id, reason).await,
            Some(_) => Err(AuthError::Forbidden),
            None => Err(AuthError::BadRequest("Session not found".into())),
        }
    }

    /// Revoke all active sessions for a user.
    pub async fn revoke_all_sessions_for_user(
        &self,
        user_id: Uuid,
        reason: &str,
    ) -> AuthResult<u64> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE auth.sessions
            SET is_active = FALSE,
                status = 'revoked',
                revocation_reason = $2,
                revoked_at = $3,
                updated_at = $3
            WHERE user_id = $1
              AND is_active = TRUE
              AND status = 'active'
            "#,
        )
        .bind(user_id.into_inner())
        .bind(reason)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(result.rows_affected())
    }

    pub async fn me(&self, user_id: Uuid) -> AuthResult<(User, String)> {
        let user = self
            .find_user_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        let role = self
            .get_user_role(user_id)
            .await?
            .unwrap_or_else(|| "user".to_string());
        Ok((user, role))
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> AuthResult<()> {
        self.check_password_change_rate_limit(user_id).await?;
        self.validate_password(new_password)?;

        let Some(credential) = self.find_password_credential(user_id).await? else {
            return Err(AuthError::WrongCredentials);
        };

        let ok = verify_password_blocking(
            self.password_hasher.clone(),
            current_password.as_bytes().to_vec(),
            credential.secret_encrypted.clone(),
        )
        .await?;

        if !ok {
            return Err(AuthError::WrongPassword);
        }

        let hash = hash_password_blocking(
            self.password_hasher.clone(),
            new_password.as_bytes().to_vec(),
        )
        .await?;

        let metadata = CredentialMetadata::Password {
            algorithm: "argon2id".to_string(),
            memory_kb: 65536,
            iterations: 3,
            parallelism: 4,
        };
        let metadata_json = serde_json::to_value(&metadata)
            .map_err(|_| AuthError::Internal("Failed to encode credential metadata".into()))?;

        sqlx::query(
            r#"
            UPDATE auth.credentials
            SET secret_encrypted = $2,
                metadata = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(credential.id.into_inner())
        .bind(&hash)
        .bind(metadata_json)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        self.revoke_all_sessions_for_user(user_id, "password_changed")
            .await?;

        Ok(())
    }

    pub async fn verify_access_principal(
        &self,
        access_token: &str,
    ) -> AuthResult<(Uuid, Vec<String>)> {
        let claims = self
            .jwt
            .verify_access_token(access_token)
            .map_err(AuthError::from)?;

        let session = self
            .get_session(claims.session_id)
            .await?
            .ok_or(AuthError::SessionRevoked)?;

        if !session.is_active {
            return Err(AuthError::SessionRevoked);
        }

        if session.access_token_fingerprint != token_fingerprint(access_token) {
            return Err(AuthError::TokenFingerprintMismatch);
        }

        Ok((claims.common.subject, claims.roles))
    }

    // =====================
    // Internal helpers
    // =====================

    async fn get_user_role(&self, user_id: Uuid) -> AuthResult<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT role
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(|r| r.get::<String, _>("role")))
    }

    async fn find_user_by_email(&self, email: &str) -> AuthResult<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(map_user_row))
    }

    async fn find_user_by_id(&self, user_id: Uuid) -> AuthResult<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(map_user_row))
    }

    async fn create_user(&self, email: &str, display_name: &str, role: &str) -> AuthResult<User> {
        let id = Uuid::new_v7();

        let row = sqlx::query(
            r#"
            INSERT INTO auth.users (id, email, display_name, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, display_name, status, created_at, updated_at
            "#,
        )
        .bind(id.into_inner())
        .bind(email)
        .bind(display_name)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(map_user_row(row))
    }

    async fn find_password_credential(&self, user_id: Uuid) -> AuthResult<Option<Credential>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, type as credential_type, secret_encrypted, metadata, verified,
                   created_at, updated_at, last_used_at
            FROM auth.credentials
            WHERE user_id = $1 AND type = 'password'
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(map_credential_row))
    }

    async fn find_password_credential_and_role(
        &self,
        user_id: Uuid,
    ) -> AuthResult<Option<(Credential, String)>> {
        let row = sqlx::query(
            r#"
            SELECT c.id, c.user_id, c.type as credential_type, c.secret_encrypted, c.metadata, c.verified,
                   c.created_at, c.updated_at, c.last_used_at,
                   u.role
            FROM auth.credentials c
            JOIN auth.users u ON u.id = c.user_id
            WHERE c.user_id = $1 AND c.type = 'password'
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let role: String = row.get("role");
        let credential = map_credential_row(row);
        Ok(Some((credential, role)))
    }

    async fn set_password(&self, user_id: Uuid, password: &str) -> AuthResult<Credential> {
        self.validate_password(password)?;

        let hash =
            hash_password_blocking(self.password_hasher.clone(), password.as_bytes().to_vec())
                .await?;

        let id = Uuid::new_v7();

        let metadata = CredentialMetadata::Password {
            algorithm: "argon2id".to_string(),
            memory_kb: 65536,
            iterations: 3,
            parallelism: 4,
        };

        let metadata_json = serde_json::to_value(&metadata)
            .map_err(|_| AuthError::Internal("Failed to encode credential metadata".into()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO auth.credentials (
                id, user_id, type, secret_encrypted, metadata, verified
            ) VALUES ($1, $2, 'password', $3, $4, TRUE)
            RETURNING id, user_id, type as credential_type, secret_encrypted, metadata, verified,
                      created_at, updated_at, last_used_at
            "#,
        )
        .bind(id.into_inner())
        .bind(user_id.into_inner())
        .bind(&hash)
        .bind(metadata_json)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(map_credential_row(row))
    }

    async fn update_credential_last_used(&self, credential_id: Uuid) -> AuthResult<()> {
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE auth.credentials
            SET last_used_at = $2,
                updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(credential_id.into_inner())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    async fn get_session(&self, session_id: Uuid) -> AuthResult<Option<DbSession>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, roles, is_active,
                   access_token_fingerprint, refresh_token_fingerprint,
                   refresh_token_id, refresh_token_version,
                   access_token_expires_at, refresh_token_expires_at,
                   created_at, updated_at, last_used_at,
                   ip_address, user_agent,
                   status, revocation_reason, revoked_at
            FROM auth.sessions
            WHERE id = $1
            "#,
        )
        .bind(session_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(map_session_row))
    }

    async fn update_session(&self, session: &DbSession) -> AuthResult<()> {
        sqlx::query(
            r#"
            UPDATE auth.sessions
            SET roles = $2,
                is_active = $3,
                access_token_fingerprint = $4,
                refresh_token_fingerprint = $5,
                refresh_token_id = $6,
                refresh_token_version = $7,
                access_token_expires_at = $8,
                refresh_token_expires_at = $9,
                updated_at = $10,
                last_used_at = $11,
                ip_address = $12,
                user_agent = $13,
                status = $14,
                revocation_reason = $15,
                revoked_at = $16
            WHERE id = $1
            "#,
        )
        .bind(session.id.into_inner())
        .bind(serde_json::to_value(&session.roles).unwrap_or_else(|_| serde_json::json!([])))
        .bind(session.is_active)
        .bind(&session.access_token_fingerprint)
        .bind(&session.refresh_token_fingerprint)
        .bind(session.refresh_token_id.into_inner())
        .bind(session.refresh_token_version)
        .bind(session.access_token_expires_at)
        .bind(session.refresh_token_expires_at)
        .bind(session.updated_at)
        .bind(session.last_used_at)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(session_status_db(&session.status))
        .bind(&session.revocation_reason)
        .bind(session.revoked_at)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    async fn create_session(
        &self,
        user_id: Uuid,
        roles: Vec<String>,
    ) -> AuthResult<(Tokens, Session)> {
        self.create_session_with_fingerprint(user_id, roles, None).await
    }

    async fn create_session_with_fingerprint(
        &self,
        user_id: Uuid,
        roles: Vec<String>,
        fingerprint: Option<SessionFingerprint>,
    ) -> AuthResult<(Tokens, Session)> {
        let session_id = Uuid::new_v7();
        let fingerprint = fingerprint.unwrap_or_default();

        let (access_token, access_claims) = self
            .jwt
            .issue_access_token(user_id, session_id, roles.clone())
            .map_err(AuthError::from)?;

        let (refresh_token, refresh_claims) = self
            .jwt
            .issue_refresh_token(user_id, session_id, None, 1)
            .map_err(AuthError::from)?;

        let access_token_fingerprint = token_fingerprint(&access_token);
        let refresh_token_fingerprint = token_fingerprint(&refresh_token);

        let access_token_expires_at = timestamp_to_datetime(access_claims.common.expires_at);
        let refresh_token_expires_at = timestamp_to_datetime(refresh_claims.common.expires_at);

        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO auth.sessions (
                id,
                user_id,
                roles,
                is_active,
                access_token_fingerprint,
                refresh_token_fingerprint,
                refresh_token_id,
                refresh_token_version,
                access_token_expires_at,
                refresh_token_expires_at,
                created_at,
                updated_at,
                last_used_at,
                ip_address,
                user_agent,
                status
            ) VALUES ($1,$2,$3,TRUE,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'active')
            "#,
        )
        .bind(session_id.into_inner())
        .bind(user_id.into_inner())
        .bind(serde_json::to_value(&roles).unwrap_or_else(|_| serde_json::json!([])))
        .bind(&access_token_fingerprint)
        .bind(&refresh_token_fingerprint)
        .bind(refresh_claims.common.token_id.into_inner())
        .bind(refresh_claims.version as i32)
        .bind(access_token_expires_at)
        .bind(refresh_token_expires_at)
        .bind(now)
        .bind(now)
        .bind(now)
        .bind(&fingerprint.ip_address)
        .bind(&fingerprint.user_agent)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let session = Session {
            id: session_id,
            user_id,
            access_token_fingerprint,
            refresh_token_fingerprint,
            access_token_expires_at,
            refresh_token_expires_at,
            created_at: now,
            last_used_at: now,
            ip_address: fingerprint.ip_address,
            user_agent: fingerprint.user_agent,
            status: SessionStatus::Active,
            revocation_reason: None,
            revoked_at: None,
        };

        Ok((
            Tokens {
                access_token,
                refresh_token,
            },
            session,
        ))
    }

    async fn revoke_session(&self, session_id: Uuid, reason: &str) -> AuthResult<()> {
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE auth.sessions
            SET is_active = FALSE,
                status = 'revoked',
                revocation_reason = $2,
                revoked_at = $3,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(session_id.into_inner())
        .bind(reason)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    async fn create_public_auth_state(
        &self,
        state_type: &str,
        state: serde_json::Value,
        ttl: Duration,
    ) -> AuthResult<Uuid> {
        self.auth_state
            .create(None, state_type, state, ttl)
            .await
            .map_err(map_auth_state_error)
    }

    async fn load_public_auth_state(
        &self,
        state_id: Uuid,
        state_type: &str,
    ) -> AuthResult<Option<serde_json::Value>> {
        self.auth_state
            .load_public(state_id, state_type)
            .await
            .map_err(map_auth_state_error)
    }

    async fn update_public_auth_state(
        &self,
        state_id: Uuid,
        state_type: &str,
        state: serde_json::Value,
    ) -> AuthResult<()> {
        self.auth_state
            .update_public(state_id, state_type, state)
            .await
            .map_err(map_auth_state_error)
    }

    async fn delete_auth_state(&self, state_id: Uuid) -> AuthResult<()> {
        self.auth_state
            .delete(state_id)
            .await
            .map_err(map_auth_state_error)
    }

    async fn create_user_auth_state(
        &self,
        user_id: Uuid,
        state_type: &str,
        state: serde_json::Value,
        ttl: Duration,
    ) -> AuthResult<Uuid> {
        self.auth_state
            .create(Some(user_id), state_type, state, ttl)
            .await
            .map_err(map_auth_state_error)
    }

    async fn consume_user_auth_state(
        &self,
        user_id: Uuid,
        state_id: Uuid,
        state_type: &str,
    ) -> AuthResult<Option<serde_json::Value>> {
        self.auth_state
            .consume_user(state_id, user_id, state_type)
            .await
            .map_err(map_auth_state_error)
    }

    async fn find_totp_details(&self, user_id: Uuid) -> AuthResult<Option<TotpDetails>> {
        let row = sqlx::query(
            r#"
            SELECT c.id AS credential_id,
                   c.secret_encrypted,
                   t.last_counter,
                   t.backup_code_hashes
            FROM auth.credentials c
            JOIN auth.totp_credential t ON t.credential_id = c.id
            WHERE c.user_id = $1 AND c.type = 'totp' AND c.verified = TRUE
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let credential_id: sqlx::types::Uuid = row.get("credential_id");
        let last_counter: i64 = row.get("last_counter");
        let backup_code_hashes_value: serde_json::Value = row.get("backup_code_hashes");

        let backup_code_hashes =
            serde_json::from_value::<Vec<String>>(backup_code_hashes_value).unwrap_or_default();

        Ok(Some(TotpDetails {
            credential_id: Uuid(credential_id),
            secret_base32: row.get("secret_encrypted"),
            last_counter: u64::try_from(last_counter).unwrap_or(0),
            backup_code_hashes,
        }))
    }

    async fn write_totp_details(&self, details: &TotpDetails) -> AuthResult<()> {
        sqlx::query(
            r#"
            UPDATE auth.totp_credential
            SET last_counter = $2,
                backup_code_hashes = $3
            WHERE credential_id = $1
            "#,
        )
        .bind(details.credential_id.into_inner())
        .bind(i64::try_from(details.last_counter).unwrap_or(0))
        .bind(
            serde_json::to_value(&details.backup_code_hashes)
                .unwrap_or_else(|_| serde_json::json!([])),
        )
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    async fn verify_totp_second_factor(
        &self,
        details: &TotpDetails,
        code: &str,
    ) -> AuthResult<TwoFactorVerified> {
        let code = code.trim();

        let verified = if code.contains('-') {
            let index = self
                .totp
                .verify_backup_code(code, &details.backup_code_hashes)?;
            TwoFactorVerified::BackupCode { index }
        } else {
            TwoFactorVerified::Totp(
                self.totp
                    .verify_totp_with_replay_protection(
                        &details.secret_base32,
                        code,
                        SystemTime::now(),
                        details.last_counter,
                    )
                    .map_err(AuthError::from)?,
            )
        };

        let mut updated = details.clone();

        match verified {
            TwoFactorVerified::Totp(v) => {
                updated.last_counter = v.counter;
            }
            TwoFactorVerified::BackupCode { index } => {
                if index < updated.backup_code_hashes.len() {
                    updated.backup_code_hashes.remove(index);
                }
            }
        }

        self.write_totp_details(&updated).await?;
        self.update_credential_last_used(updated.credential_id)
            .await?;

        Ok(verified)
    }

    // ========================================================================
    // TOTP Management
    // ========================================================================

    pub async fn totp_setup(&self, user_id: Uuid) -> AuthResult<TotpSetupResult> {
        let (user, _) = self.me(user_id).await?;
        let setup = self.totp.setup(&user.email, 10)?;

        let setup_state = TotpSetupState {
            secret_base32: setup.secret.base32,
            backup_code_hashes: setup.backup_code_hashes,
            metadata: setup.metadata,
        };

        let setup_id = self
            .create_user_auth_state(
                user_id,
                "totp_setup",
                serde_json::to_value(setup_state)
                    .map_err(|_| AuthError::Internal("Failed to encode auth state".into()))?,
                Duration::minutes(15),
            )
            .await?;

        Ok(TotpSetupResult {
            setup_id,
            otpauth_uri: setup.otpauth_uri,
            qr_svg: setup.qr_svg,
            backup_codes: setup.backup_codes,
        })
    }

    pub async fn totp_enable(&self, user_id: Uuid, setup_id: Uuid, code: &str) -> AuthResult<()> {
        if self.find_totp_details(user_id).await?.is_some() {
            return Err(AuthError::BadRequest("TOTP is already enabled".into()));
        }

        let state_value = self
            .consume_user_auth_state(user_id, setup_id, "totp_setup")
            .await?
            .ok_or(AuthError::BadRequest("Invalid or expired setup".into()))?;

        let state: TotpSetupState = serde_json::from_value(state_value)
            .map_err(|_| AuthError::BadRequest("Invalid setup state".into()))?;

        let verified = self
            .totp
            .verify_totp_with_replay_protection(&state.secret_base32, code, SystemTime::now(), 0)
            .map_err(AuthError::from)?;

        let credential_id = Uuid::new_v7();
        let now = Utc::now();

        let metadata_json = serde_json::to_value(&state.metadata)
            .map_err(|_| AuthError::Internal("Failed to encode credential metadata".into()))?;

        sqlx::query(
            r#"
            INSERT INTO auth.credentials (
                id, user_id, type, secret_encrypted, metadata, verified,
                created_at, updated_at, last_used_at
            ) VALUES ($1, $2, 'totp', $3, $4, TRUE, $5, $6, NULL)
            "#,
        )
        .bind(credential_id.into_inner())
        .bind(user_id.into_inner())
        .bind(&state.secret_base32)
        .bind(metadata_json)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        sqlx::query(
            r#"
            INSERT INTO auth.totp_credential (credential_id, last_counter, backup_code_hashes)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(credential_id.into_inner())
        .bind(i64::try_from(verified.counter).unwrap_or(0))
        .bind(
            serde_json::to_value(&state.backup_code_hashes)
                .unwrap_or_else(|_| serde_json::json!([])),
        )
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    pub async fn totp_disable(&self, user_id: Uuid) -> AuthResult<()> {
        sqlx::query(
            r#"
            DELETE FROM auth.credentials
            WHERE user_id = $1 AND type = 'totp'
            "#,
        )
        .bind(user_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    pub async fn totp_is_enabled(&self, user_id: Uuid) -> AuthResult<bool> {
        Ok(self.find_totp_details(user_id).await?.is_some())
    }

    /// Get comprehensive 2FA status for a user.
    pub async fn get_2fa_status(&self, user_id: Uuid) -> AuthResult<TwoFactorStatus> {
        // Check TOTP status with timestamp
        let totp_row = sqlx::query(
            r#"
            SELECT c.created_at
            FROM auth.credentials c
            JOIN auth.totp_credential t ON t.credential_id = c.id
            WHERE c.user_id = $1 AND c.type = 'totp' AND c.verified = TRUE
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let has_totp = totp_row.is_some();
        let totp_enabled_at = totp_row.map(|r| r.get::<DateTime<Utc>, _>("created_at"));

        // For now, passkey count is 0 (not implemented yet)
        let passkey_count = 0u32;

        Ok(TwoFactorStatus {
            has_totp_configured: has_totp,
            has_passkey_configured: passkey_count > 0,
            totp_enabled_at,
            passkey_count,
        })
    }

    /// Verify a TOTP code and create a verification session.
    pub async fn verify_totp_for_verification(
        &self,
        user_id: Uuid,
        code: &str,
        purpose: EmailTotpPurpose,
    ) -> AuthResult<VerificationSessionRow> {
        let totp = self
            .find_totp_details(user_id)
            .await?
            .ok_or(AuthError::TwoFactorNotSetUp)?;

        // Verify the TOTP code
        self.verify_totp_second_factor(&totp, code).await?;

        // Create a verification session (5-minute expiry)
        let session = create_verification_session(
            &self.pool,
            user_id.into_inner(),
            purpose,
            VerificationMethod::Totp,
            5, // 5 minutes
        )
        .await
        .map_err(|e| AuthError::Internal(format!("Failed to create verification session: {}", e)))?;

        Ok(session)
    }

    // ========================================================================
    // Password Reset (Forgot Password) Flow
    // ========================================================================

    /// Get user information for password reset by email.
    ///
    /// Returns the user ID and email if the user exists and is active.
    /// Returns None if user doesn't exist or is not active (prevents enumeration).
    pub async fn get_user_for_password_reset(&self, email: &str) -> AuthResult<Option<(Uuid, String)>> {
        // Use case-insensitive lookup for password reset
        let row = sqlx::query(
            r#"
            SELECT id, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE LOWER(email) = LOWER($1)
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let user = match row.map(map_user_row) {
            Some(user) => user,
            None => return Ok(None),
        };

        if user.status != UserStatus::Active {
            return Ok(None);
        }

        Ok(Some((user.id, user.email)))
    }

    /// Verify a password reset code and return a verification session ID.
    ///
    /// The verification session can be used to complete the password reset.
    /// Uses case-insensitive email lookup to match the request step.
    pub async fn verify_password_reset_code(
        &self,
        email: &str,
        code: &str,
        email_totp: &crate::email_totp::EmailTotpService,
    ) -> AuthResult<crate::email_totp::VerificationSession> {
        // Look up user by email (case-insensitive to match request step)
        let row = sqlx::query(
            r#"
            SELECT id, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE LOWER(email) = LOWER($1)
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let user = row
            .map(map_user_row)
            .ok_or(AuthError::BadRequest("Invalid or expired code".into()))?;

        if user.status != UserStatus::Active {
            return Err(AuthError::BadRequest("Invalid or expired code".into()));
        }

        // Verify the code using the email TOTP service
        let session = email_totp
            .verify_code(user.id, EmailTotpPurpose::PasswordReset, code)
            .await
            .map_err(|e| match e {
                crate::email_totp::EmailTotpError::InvalidCode
                | crate::email_totp::EmailTotpError::CodeExpired
                | crate::email_totp::EmailTotpError::NoActiveCode => {
                    AuthError::BadRequest("Invalid or expired code".into())
                }
                crate::email_totp::EmailTotpError::TooManyAttempts => {
                    AuthError::BadRequest("Too many invalid attempts. Please request a new code.".into())
                }
                _ => AuthError::Internal(format!("Verification failed: {}", e)),
            })?;

        Ok(session)
    }

    /// Complete a password reset using a verification session.
    ///
    /// This consumes the verification session, validates the new password,
    /// updates the user's password, and revokes all existing sessions.
    pub async fn complete_password_reset(
        &self,
        verification_session_id: Uuid,
        new_password: &str,
    ) -> AuthResult<()> {
        // Validate new password strength
        self.validate_password(new_password)?;

        // First, get the verification session to find the user_id
        // We need to look it up before consuming
        let session_row = sqlx::query(
            r#"
            SELECT user_id
            FROM auth.verification_sessions
            WHERE id = $1
              AND purpose = 'password_reset'
              AND used_at IS NULL
              AND expires_at > NOW()
            "#,
        )
        .bind(verification_session_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Internal(format!("DB error: {}", e)))?;

        let user_id: Uuid = match session_row {
            Some(row) => Uuid(row.get("user_id")),
            None => {
                return Err(AuthError::BadRequest(
                    "Invalid or expired reset token".into(),
                ));
            }
        };

        // Consume the verification session
        let consumed = consume_verification_session(
            &self.pool,
            verification_session_id.into_inner(),
            user_id.into_inner(),
            EmailTotpPurpose::PasswordReset,
        )
        .await
        .map_err(|e| AuthError::Internal(format!("DB error: {}", e)))?;

        if !consumed {
            return Err(AuthError::BadRequest(
                "Invalid or expired reset token".into(),
            ));
        }

        // Find the password credential
        let credential = match self.find_password_credential(user_id).await? {
            Some(cred) => cred,
            None => {
                // User has no password credential - create one
                self.set_password(user_id, new_password).await?;
                // Revoke all sessions
                self.revoke_all_sessions_for_user(user_id, "password_reset")
                    .await?;
                return Ok(());
            }
        };

        // Hash new password
        let hash = hash_password_blocking(
            self.password_hasher.clone(),
            new_password.as_bytes().to_vec(),
        )
        .await?;

        let metadata = CredentialMetadata::Password {
            algorithm: "argon2id".to_string(),
            memory_kb: 65536,
            iterations: 3,
            parallelism: 4,
        };
        let metadata_json = serde_json::to_value(&metadata)
            .map_err(|_| AuthError::Internal("Failed to encode credential metadata".into()))?;

        // Update the password
        sqlx::query(
            r#"
            UPDATE auth.credentials
            SET secret_encrypted = $2,
                metadata = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(credential.id.into_inner())
        .bind(&hash)
        .bind(metadata_json)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        // Revoke all sessions - user must re-authenticate with new password
        self.revoke_all_sessions_for_user(user_id, "password_reset")
            .await?;

        Ok(())
    }

    pub async fn change_password_with_verification(
        &self,
        user_id: Uuid,
        verification_session_id: Uuid,
        new_password: &str,
    ) -> AuthResult<()> {
        // Validate new password strength
        self.validate_password(new_password)?;

        // Consume the verification session (validates it and marks as used)
        let consumed = consume_verification_session(
            &self.pool,
            verification_session_id.into_inner(),
            user_id.into_inner(),
            EmailTotpPurpose::PasswordChange,
        )
        .await
        .map_err(|e| AuthError::Internal(format!("DB error: {}", e)))?;

        if !consumed {
            return Err(AuthError::BadRequest(
                "Invalid, expired, or already used verification session".into(),
            ));
        }

        // Find the password credential
        let Some(credential) = self.find_password_credential(user_id).await? else {
            return Err(AuthError::WrongCredentials);
        };

        // Hash new password
        let hash = hash_password_blocking(
            self.password_hasher.clone(),
            new_password.as_bytes().to_vec(),
        )
        .await?;

        let metadata = CredentialMetadata::Password {
            algorithm: "argon2id".to_string(),
            memory_kb: 65536,
            iterations: 3,
            parallelism: 4,
        };
        let metadata_json = serde_json::to_value(&metadata)
            .map_err(|_| AuthError::Internal("Failed to encode credential metadata".into()))?;

        // Update the password
        sqlx::query(
            r#"
            UPDATE auth.credentials
            SET secret_encrypted = $2,
                metadata = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(credential.id.into_inner())
        .bind(&hash)
        .bind(metadata_json)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        // Revoke all sessions - user must re-authenticate with new password
        self.revoke_all_sessions_for_user(user_id, "password_changed")
            .await?;

        Ok(())
    }

    // ========================================================================
    // Passkey (WebAuthn) Management
    // ========================================================================

    /// Find all passkey credentials for a user.
    async fn find_passkey_credentials(
        &self,
        user_id: Uuid,
    ) -> AuthResult<Vec<underlay_auth_webauthn::StoredPasskey>> {
        let rows = sqlx::query(
            r#"
            SELECT secret_encrypted
            FROM auth.credentials
            WHERE user_id = $1 AND type = 'passkey' AND verified = TRUE
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let mut passkeys = Vec::with_capacity(rows.len());
        for row in rows {
            let secret: String = row.get("secret_encrypted");
            let stored: underlay_auth_webauthn::StoredPasskey = serde_json::from_str(&secret)
                .map_err(|_| AuthError::Internal("Failed to decode passkey".into()))?;
            passkeys.push(stored);
        }

        Ok(passkeys)
    }

    /// Find a passkey by its credential ID (base64url encoded).
    async fn find_passkey_by_credential_id(
        &self,
        credential_id: &str,
    ) -> AuthResult<Option<(Uuid, Uuid, underlay_auth_webauthn::StoredPasskey)>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, secret_encrypted
            FROM auth.credentials
            WHERE type = 'passkey'
              AND verified = TRUE
              AND metadata->>'credentialId' = $1
            "#,
        )
        .bind(credential_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: sqlx::types::Uuid = row.get("id");
        let user_id: sqlx::types::Uuid = row.get("user_id");
        let secret: String = row.get("secret_encrypted");

        let stored: underlay_auth_webauthn::StoredPasskey = serde_json::from_str(&secret)
            .map_err(|_| AuthError::Internal("Failed to decode passkey".into()))?;

        Ok(Some((Uuid(user_id), Uuid(id), stored)))
    }

    /// List all passkeys for a user with their display names.
    pub async fn list_passkeys(&self, user_id: Uuid) -> AuthResult<Vec<PasskeyRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, type as credential_type, secret_encrypted, metadata, verified,
                   display_name, created_at, updated_at, last_used_at
            FROM auth.credentials
            WHERE user_id = $1 AND type = 'passkey'
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let display_name: Option<String> = row.get("display_name");
                PasskeyRecord {
                    credential: map_credential_row(row),
                    display_name,
                }
            })
            .collect())
    }

    /// Rename a passkey's display name.
    pub async fn rename_passkey(
        &self,
        user_id: Uuid,
        credential_id: Uuid,
        display_name: &str,
    ) -> AuthResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE auth.credentials
            SET display_name = $3, updated_at = NOW()
            WHERE id = $1 AND user_id = $2 AND type = 'passkey'
            "#,
        )
        .bind(credential_id.into_inner())
        .bind(user_id.into_inner())
        .bind(display_name)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::BadRequest("Passkey not found".into()));
        }

        Ok(())
    }

    /// Delete a passkey.
    pub async fn delete_passkey(&self, user_id: Uuid, credential_id: Uuid) -> AuthResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM auth.credentials
            WHERE id = $1 AND user_id = $2 AND type = 'passkey'
            "#,
        )
        .bind(credential_id.into_inner())
        .bind(user_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::BadRequest("Passkey not found".into()));
        }

        Ok(())
    }

    /// Start passkey registration.
    pub async fn passkey_register_start(
        &self,
        user_id: Uuid,
    ) -> AuthResult<underlay_auth_webauthn::StartPasskeyRegistrationResponse> {
        self.check_passkey_register_rate_limit(user_id).await?;

        let (user, _) = self.me(user_id).await?;

        let stored_passkeys = self.find_passkey_credentials(user_id).await?;
        let exclude = stored_passkeys
            .into_iter()
            .filter_map(|stored| {
                // CredentialId deserializes from a base64url JSON string
                serde_json::from_value::<underlay_auth_webauthn::CredentialId>(
                    serde_json::Value::String(stored.credential_id.clone())
                ).ok()
            })
            .collect::<Vec<_>>();

        let exclude = if exclude.is_empty() { None } else { Some(exclude) };

        let display_name = user
            .display_name
            .as_deref()
            .unwrap_or_else(|| user.email.split('@').next().unwrap_or("User"));

        let (options, state) = self.webauthn.start_passkey_registration(
            user_id,
            &user.email,
            display_name,
            exclude,
        )?;

        let encoded = underlay_auth_webauthn::WebAuthnService::encode_registration_state(&state)?;
        let state_id = self
            .create_user_auth_state(
                user_id,
                "passkey_registration",
                serde_json::Value::String(encoded),
                Duration::minutes(15),
            )
            .await?;

        Ok(underlay_auth_webauthn::StartPasskeyRegistrationResponse {
            options,
            state_id: state_id.to_string(),
        })
    }

    /// Finish passkey registration.
    pub async fn passkey_register_finish(
        &self,
        user_id: Uuid,
        state_id: Uuid,
        credential: RegisterPublicKeyCredential,
        display_name: Option<&str>,
    ) -> AuthResult<Credential> {
        self.check_passkey_register_rate_limit(user_id).await?;

        let value = self
            .consume_user_auth_state(user_id, state_id, "passkey_registration")
            .await?
            .ok_or_else(|| {
                AuthError::BadRequest("invalid or expired passkey registration".into())
            })?;

        let encoded = value
            .as_str()
            .ok_or_else(|| AuthError::BadRequest("invalid passkey registration state".into()))?;

        let state = underlay_auth_webauthn::WebAuthnService::decode_registration_state(encoded)?;
        let passkey = self.webauthn.finish_passkey_registration(&state, &credential)?;

        let stored_passkey = self.webauthn.stored_passkey_from_passkey(&passkey)?;
        let secret = serde_json::to_string(&stored_passkey)
            .map_err(|_| AuthError::Internal("failed to encode passkey".into()))?;
        let metadata = underlay_auth_webauthn::WebAuthnService::credential_metadata_from_stored_passkey(&stored_passkey);

        let credential_id = Uuid::new_v7();
        let now = Utc::now();

        let metadata_json = serde_json::to_value(&metadata)
            .map_err(|_| AuthError::Internal("Failed to encode credential metadata".into()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO auth.credentials (
                id, user_id, type, secret_encrypted, metadata, verified, display_name,
                created_at, updated_at, last_used_at
            ) VALUES ($1, $2, 'passkey', $3, $4, TRUE, $5, $6, $7, NULL)
            RETURNING id, user_id, type as credential_type, secret_encrypted, metadata, verified,
                      created_at, updated_at, last_used_at
            "#,
        )
        .bind(credential_id.into_inner())
        .bind(user_id.into_inner())
        .bind(&secret)
        .bind(metadata_json)
        .bind(display_name)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(map_credential_row(row))
    }

    /// Start passkey login.
    pub async fn passkey_login_start(
        &self,
        email: Option<&str>,
    ) -> AuthResult<underlay_auth_webauthn::StartPasskeyAuthenticationResponse> {
        self.check_passkey_login_rate_limit(email).await?;

        if let Some(email) = email {
            let Some(user) = self.find_user_by_email(email).await? else {
                return Err(AuthError::WrongCredentials);
            };

            let stored = self.find_passkey_credentials(user.id).await?;
            let mut allowed = Vec::with_capacity(stored.len());
            for pk in stored {
                allowed.push(self.webauthn.passkey_from_stored_passkey(&pk)?);
            }

            let (options, state) = self.webauthn.start_passkey_authentication(allowed)?;
            let encoded = underlay_auth_webauthn::WebAuthnService::encode_authentication_state(&state)?;

            let state_id = self
                .create_public_auth_state(
                    "passkey_authentication",
                    serde_json::Value::String(encoded),
                    Duration::minutes(15),
                )
                .await?;

            return Ok(underlay_auth_webauthn::StartPasskeyAuthenticationResponse {
                options,
                state_id: state_id.to_string(),
            });
        }

        let (options, state) = self.webauthn.start_discoverable_authentication()?;
        let encoded = underlay_auth_webauthn::WebAuthnService::encode_discoverable_authentication_state(&state)?;

        let state_id = self
            .create_public_auth_state(
                "passkey_discoverable_authentication",
                serde_json::Value::String(encoded),
                Duration::minutes(15),
            )
            .await?;

        Ok(underlay_auth_webauthn::StartPasskeyAuthenticationResponse {
            options,
            state_id: state_id.to_string(),
        })
    }

    /// Finish passkey login.
    pub async fn passkey_login_finish(
        &self,
        state_id: Uuid,
        credential: PublicKeyCredential,
        session_fingerprint: Option<SessionFingerprint>,
    ) -> AuthResult<AuthSession> {
        let (user_id, passkey_credential_id, stored, result) = if let Some(value) = self
            .consume_public_auth_state(state_id, "passkey_authentication")
            .await?
        {
            let encoded = value.as_str().ok_or_else(|| {
                AuthError::BadRequest("invalid passkey authentication state".into())
            })?;

            let state = underlay_auth_webauthn::WebAuthnService::decode_authentication_state(encoded)?;
            let result = self.webauthn.finish_passkey_authentication(&credential, &state)?;

            let credential_id =
                underlay_auth_webauthn::WebAuthnService::authentication_result_credential_id_base64url(&result)?;

            let (user_id, passkey_credential_id, stored) = self
                .find_passkey_by_credential_id(&credential_id)
                .await?
                .ok_or(AuthError::PassKeyCredentialNotFound)?;

            (user_id, passkey_credential_id, stored, result)
        } else {
            let value = self
                .consume_public_auth_state(state_id, "passkey_discoverable_authentication")
                .await?
                .ok_or_else(|| {
                    AuthError::BadRequest("invalid or expired passkey authentication".into())
                })?;

            let encoded = value.as_str().ok_or_else(|| {
                AuthError::BadRequest("invalid passkey authentication state".into())
            })?;

            let state = underlay_auth_webauthn::WebAuthnService::decode_discoverable_authentication_state(encoded)?;

            let (identified_user_id, cred_id) = self
                .webauthn
                .identify_discoverable_authentication(&credential)?;
            let credential_id = underlay_auth_webauthn::WebAuthnService::credential_id_to_base64url(&cred_id)?;

            let (user_id, passkey_credential_id, stored) = self
                .find_passkey_by_credential_id(&credential_id)
                .await?
                .ok_or(AuthError::PassKeyCredentialNotFound)?;

            if user_id != identified_user_id {
                return Err(AuthError::PassKeyAuthenticationFailed);
            }

            let passkey = self.webauthn.passkey_from_stored_passkey(&stored)?;
            let result = self.webauthn.finish_discoverable_authentication(
                &credential,
                &state,
                vec![passkey],
            )?;

            (user_id, passkey_credential_id, stored, result)
        };

        // Update the stored passkey with new counter
        let update_result = self.webauthn.update_stored_passkey_after_authentication(&stored, &result)?;
        if update_result.changed {
            let updated_secret = serde_json::to_string(&update_result.stored_passkey)
                .map_err(|_| AuthError::Internal("failed to encode passkey".into()))?;

            sqlx::query(
                r#"
                UPDATE auth.credentials
                SET secret_encrypted = $2, last_used_at = NOW(), updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(passkey_credential_id.into_inner())
            .bind(&updated_secret)
            .execute(&self.pool)
            .await
            .map_err(|_| AuthError::Internal("DB error".into()))?;
        } else {
            // Just update last_used_at
            sqlx::query(
                r#"
                UPDATE auth.credentials
                SET last_used_at = NOW(), updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(passkey_credential_id.into_inner())
            .execute(&self.pool)
            .await
            .map_err(|_| AuthError::Internal("DB error".into()))?;
        }

        // Get user and role
        let user = self
            .find_user_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        let role = self
            .get_user_role(user_id)
            .await?
            .unwrap_or_else(|| "user".to_string());

        let roles = roles_for_user(&role);
        let (tokens, session) = self
            .create_session_with_fingerprint(user.id, roles, session_fingerprint)
            .await?;

        Ok(AuthSession {
            user,
            session,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    /// Start passkey verification (for 2FA gates).
    pub async fn passkey_verify_start(
        &self,
        user_id: Uuid,
        purpose: EmailTotpPurpose,
    ) -> AuthResult<underlay_auth_webauthn::StartPasskeyAuthenticationResponse> {
        let stored = self.find_passkey_credentials(user_id).await?;
        if stored.is_empty() {
            return Err(AuthError::TwoFactorNotSetUp);
        }

        let mut allowed = Vec::with_capacity(stored.len());
        for pk in stored {
            allowed.push(self.webauthn.passkey_from_stored_passkey(&pk)?);
        }

        let (options, state) = self.webauthn.start_passkey_authentication(allowed)?;
        let encoded = underlay_auth_webauthn::WebAuthnService::encode_authentication_state(&state)?;

        let state_data = serde_json::json!({
            "encoded_state": encoded,
            "user_id": user_id.to_string(),
            "purpose": purpose.as_str()
        });

        let state_id = self
            .create_public_auth_state("passkey_verification", state_data, Duration::minutes(15))
            .await?;

        Ok(underlay_auth_webauthn::StartPasskeyAuthenticationResponse {
            options,
            state_id: state_id.to_string(),
        })
    }

    /// Finish passkey verification and create a verification session.
    pub async fn passkey_verify_finish(
        &self,
        user_id: Uuid,
        state_id: Uuid,
        credential: PublicKeyCredential,
    ) -> AuthResult<VerificationSessionRow> {
        let value = self
            .consume_public_auth_state(state_id, "passkey_verification")
            .await?
            .ok_or_else(|| {
                AuthError::BadRequest("invalid or expired passkey verification".into())
            })?;

        let state_data = value.as_object().ok_or_else(|| {
            AuthError::BadRequest("invalid passkey verification state".into())
        })?;

        let encoded = state_data
            .get("encoded_state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::BadRequest("invalid passkey verification state".into()))?;

        let stored_user_id = state_data
            .get("user_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AuthError::BadRequest("invalid passkey verification state".into()))?;

        let purpose_str = state_data
            .get("purpose")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::BadRequest("invalid passkey verification state".into()))?;

        let purpose = EmailTotpPurpose::parse(purpose_str)
            .ok_or_else(|| AuthError::BadRequest("invalid purpose".into()))?;

        if stored_user_id != user_id {
            return Err(AuthError::PassKeyAuthenticationFailed);
        }

        let state = underlay_auth_webauthn::WebAuthnService::decode_authentication_state(encoded)?;
        let result = self.webauthn.finish_passkey_authentication(&credential, &state)?;

        let credential_id =
            underlay_auth_webauthn::WebAuthnService::authentication_result_credential_id_base64url(&result)?;

        let (pk_user_id, passkey_credential_id, stored) = self
            .find_passkey_by_credential_id(&credential_id)
            .await?
            .ok_or(AuthError::PassKeyCredentialNotFound)?;

        if pk_user_id != user_id {
            return Err(AuthError::PassKeyAuthenticationFailed);
        }

        // Update the stored passkey with new counter
        let update_result = self.webauthn.update_stored_passkey_after_authentication(&stored, &result)?;
        if update_result.changed {
            let updated_secret = serde_json::to_string(&update_result.stored_passkey)
                .map_err(|_| AuthError::Internal("failed to encode passkey".into()))?;

            sqlx::query(
                r#"
                UPDATE auth.credentials
                SET secret_encrypted = $2, last_used_at = NOW(), updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(passkey_credential_id.into_inner())
            .bind(&updated_secret)
            .execute(&self.pool)
            .await
            .map_err(|_| AuthError::Internal("DB error".into()))?;
        }

        // Create verification session
        let session = create_verification_session(
            &self.pool,
            user_id.into_inner(),
            purpose,
            VerificationMethod::Passkey,
            5,
        )
        .await
        .map_err(|e| AuthError::Internal(format!("Failed to create verification session: {}", e)))?;

        Ok(session)
    }

    /// Consume a public auth state (for passkey flows).
    async fn consume_public_auth_state(
        &self,
        state_id: Uuid,
        state_type: &str,
    ) -> AuthResult<Option<serde_json::Value>> {
        self.auth_state
            .consume_public(state_id, state_type)
            .await
            .map_err(map_auth_state_error)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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

fn roles_for_user(role: &str) -> Vec<String> {
    match role {
        "superadmin" => vec!["superadmin".to_string()],
        "admin" => vec!["admin".to_string()],
        "support" => vec!["support".to_string()],
        "tester" => vec!["tester".to_string()],
        _ => vec!["user".to_string()],
    }
}

fn map_user_row(row: sqlx::postgres::PgRow) -> User {
    let id: sqlx::types::Uuid = row.get("id");
    let status: String = row.get("status");

    let status = match status.as_str() {
        "active" => UserStatus::Active,
        "suspended" => UserStatus::Suspended,
        "deleted" => UserStatus::Deleted,
        _ => UserStatus::Active,
    };

    let display_name: Option<String> = row.get("display_name");

    User {
        id: Uuid(id),
        email: row.get("email"),
        display_name,
        status,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn map_credential_row(row: sqlx::postgres::PgRow) -> Credential {
    let id: sqlx::types::Uuid = row.get("id");
    let user_id: sqlx::types::Uuid = row.get("user_id");
    let credential_type: String = row.get("credential_type");
    let metadata: serde_json::Value = row.get("metadata");

    let credential_type = match credential_type.as_str() {
        "password" => CredentialType::Password,
        "totp" => CredentialType::Totp,
        "passkey" => CredentialType::Passkey,
        "oauth_google" => CredentialType::OAuthGoogle,
        _ => CredentialType::Password,
    };

    let metadata = serde_json::from_value::<CredentialMetadata>(metadata).unwrap_or(
        CredentialMetadata::Password {
            algorithm: "argon2id".to_string(),
            memory_kb: 65536,
            iterations: 3,
            parallelism: 4,
        },
    );

    Credential {
        id: Uuid(id),
        user_id: Uuid(user_id),
        credential_type,
        secret_encrypted: row.get("secret_encrypted"),
        metadata,
        verified: row.get("verified"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_used_at: row.get("last_used_at"),
    }
}

fn map_session_row(row: sqlx::postgres::PgRow) -> DbSession {
    let id: sqlx::types::Uuid = row.get("id");
    let user_id: sqlx::types::Uuid = row.get("user_id");
    let roles_value: serde_json::Value = row.get("roles");
    let roles = serde_json::from_value::<Vec<String>>(roles_value).unwrap_or_default();

    let status: String = row.get("status");
    let status = match status.as_str() {
        "active" => SessionStatus::Active,
        "revoked" => SessionStatus::Revoked,
        "expired" => SessionStatus::Expired,
        _ => SessionStatus::Active,
    };

    DbSession {
        id: Uuid(id),
        user_id: Uuid(user_id),
        roles,
        is_active: row.get("is_active"),
        access_token_fingerprint: row.get("access_token_fingerprint"),
        refresh_token_fingerprint: row.get("refresh_token_fingerprint"),
        refresh_token_id: Uuid(row.get("refresh_token_id")),
        refresh_token_version: row.get("refresh_token_version"),
        access_token_expires_at: row.get("access_token_expires_at"),
        refresh_token_expires_at: row.get("refresh_token_expires_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_used_at: row.get("last_used_at"),
        ip_address: row.get("ip_address"),
        user_agent: row.get("user_agent"),
        status,
        revocation_reason: row.get("revocation_reason"),
        revoked_at: row.get("revoked_at"),
    }
}

fn timestamp_to_datetime(ts: u64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts as i64, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

fn session_status_db(status: &SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Revoked => "revoked",
        SessionStatus::Expired => "expired",
    }
}
