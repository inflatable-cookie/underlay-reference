//! Centralized authentication configuration.
//!
//! This module provides a single `AuthConfig` struct that consolidates all
//! authentication-related configuration values, replacing scattered magic numbers
//! throughout the codebase.
//!
//! # Usage
//!
//! The default configuration provides production-ready security settings:
//!
//! ```rust
//! use acme_auth::AuthConfig;
//!
//! // Use defaults
//! let config = AuthConfig::default();
//!
//! // Or customize via builder
//! let config = AuthConfig::builder()
//!     .login_rate_limit_per_hour(20)
//!     .max_totp_attempts(3)
//!     .build();
//! ```
//!
//! # Configuration Categories
//!
//! ## Rate Limiting
//! Controls how many requests are allowed per time period to prevent abuse:
//! - `login_rate_limit_per_hour` - Login attempts per email+IP (default: 10)
//! - `register_rate_limit_per_hour` - Registrations per IP (default: 5)
//! - `password_change_rate_limit_per_hour` - Password changes per user (default: 5)
//! - `passkey_register_rate_limit_per_hour` - Passkey registrations per user (default: 5)
//! - `passkey_login_rate_limit_per_hour` - Passkey logins per email (default: 10)
//!
//! ## Attempt Limits
//! Maximum attempts before state invalidation:
//! - `max_totp_attempts` - TOTP code attempts per login state (default: 5)
//! - `max_email_code_attempts` - Email code attempts per code (default: 5)
//! - `max_backup_code_attempts` - Backup code attempts per session (default: 5)
//!
//! ## Timeouts
//! How long various states remain valid:
//! - `totp_state_timeout` - 2FA login state validity (default: 5 minutes)
//! - `email_state_timeout` - Email verification state validity (default: 10 minutes)
//! - `verification_session_timeout` - Post-verification session validity (default: 5 minutes)
//!
//! ## Account Lockout
//! Protection against brute-force attacks:
//! - `max_failed_logins` - Failed attempts before lockout (default: 5)
//! - `lockout_duration` - How long account is locked (default: 15 minutes)
//!
//! # Security Considerations
//!
//! The default values are chosen to balance security with usability:
//! - Rate limits prevent automated attacks while allowing legitimate retries
//! - Short timeouts minimize attack windows for intercepted codes
//! - Lockout protects against password brute-forcing
//!
//! For high-security deployments, consider:
//! - Lower rate limits and attempt counts
//! - Shorter timeouts
//! - Longer lockout durations

use std::time::Duration;

use crate::redis_rate_limit::RateLimitBackendType;
use underlay_security_alerts::SecurityAlertConfig;

/// Centralized configuration for all authentication operations.
///
/// All timeouts and limits can be customized via this struct. Sensible defaults
/// are provided via the `Default` implementation.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    // =========================================================================
    // Rate Limiting
    // =========================================================================
    /// Maximum login attempts per hour per email+IP combination.
    pub login_rate_limit_per_hour: u32,

    /// Maximum registration attempts per hour per IP.
    pub register_rate_limit_per_hour: u32,

    /// Maximum password change attempts per hour per user.
    pub password_change_rate_limit_per_hour: u32,

    /// Maximum passkey registration attempts per hour per user.
    pub passkey_register_rate_limit_per_hour: u32,

    /// Maximum passkey login attempts per hour per email.
    pub passkey_login_rate_limit_per_hour: u32,

    /// Maximum token refresh attempts per hour per fingerprint.
    /// Prevents brute force enumeration of valid refresh tokens.
    pub refresh_rate_limit_per_hour: u32,

    /// Type of rate limiter backend to use.
    /// Default: InMemory (single-instance)
    /// Set to Redis for distributed deployments.
    pub rate_limit_backend: RateLimitBackendType,

    /// Redis URL for rate limiting (only used when backend is Redis).
    /// Default: "redis://localhost:6379"
    pub redis_url: String,

    // =========================================================================
    // Attempt Limits
    // =========================================================================
    /// Maximum TOTP verification attempts per login state.
    pub max_totp_attempts: u32,

    /// Maximum email code verification attempts per code.
    pub max_email_code_attempts: u32,

    /// Maximum backup code verification attempts per session.
    pub max_backup_code_attempts: u32,

    // =========================================================================
    // Timeouts
    // =========================================================================
    /// TOTP login state timeout (how long a 2FA state remains valid).
    pub totp_state_timeout: Duration,

    /// Email login state timeout (how long an email verification state remains valid).
    pub email_state_timeout: Duration,

    /// Verification session timeout (how long a verification session is valid after code entry).
    pub verification_session_timeout: Duration,

    /// Rate limit cleanup interval (how often the rate limiter cleans up old entries).
    pub rate_limit_cleanup_interval: Duration,

    /// Absolute session timeout (maximum lifetime of a session regardless of activity).
    /// After this duration, the session is forcefully expired and the user must re-authenticate.
    /// Default: 30 days
    pub absolute_session_timeout: Duration,

    // =========================================================================
    // Rate Limit Responses
    // =========================================================================
    /// Retry-after value for short-term rate limits (e.g., wrong TOTP code).
    pub rate_limit_retry_after_short: Duration,

    /// Retry-after value for long-term rate limits (e.g., too many login attempts).
    pub rate_limit_retry_after_long: Duration,

    // =========================================================================
    // Account Lockout
    // =========================================================================
    /// Maximum failed login attempts before account lockout.
    pub max_failed_logins: u32,

    /// Duration of account lockout after too many failed attempts.
    pub lockout_duration: Duration,

    /// Login-attempt analysis window for security alerting.
    pub security_alert_window: Duration,

    /// Deduplication cooldown for repeated alerts of same type+IP.
    pub security_alert_cooldown: Duration,

    /// Failed attempts threshold for one IP within the alert window.
    pub security_alert_failed_attempts_threshold: u32,

    /// Distinct-user failure threshold for one IP within the alert window.
    pub security_alert_distinct_users_threshold: u32,

    /// Lockout threshold for one IP within the alert window.
    pub security_alert_lockouts_threshold: u32,

    // =========================================================================
    // Email TOTP
    // =========================================================================
    /// Email verification code expiry time.
    pub email_code_expiry: Duration,

    /// Maximum email codes that can be requested per hour per user.
    pub max_email_codes_per_hour: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            // Rate limiting
            login_rate_limit_per_hour: 10,
            register_rate_limit_per_hour: 5,
            password_change_rate_limit_per_hour: 5,
            passkey_register_rate_limit_per_hour: 5,
            passkey_login_rate_limit_per_hour: 10,
            refresh_rate_limit_per_hour: 60,
            rate_limit_backend: RateLimitBackendType::InMemory,
            redis_url: "redis://localhost:6379".to_string(),

            // Attempt limits
            max_totp_attempts: 5,
            max_email_code_attempts: 5,
            max_backup_code_attempts: 5,

            // Timeouts
            totp_state_timeout: Duration::from_secs(300), // 5 minutes
            email_state_timeout: Duration::from_secs(600), // 10 minutes
            verification_session_timeout: Duration::from_secs(300), // 5 minutes
            rate_limit_cleanup_interval: Duration::from_secs(300), // 5 minutes
            absolute_session_timeout: Duration::from_secs(30 * 24 * 60 * 60), // 30 days

            // Rate limit responses
            rate_limit_retry_after_short: Duration::from_secs(60), // 1 minute
            rate_limit_retry_after_long: Duration::from_secs(300), // 5 minutes

            // Account lockout
            max_failed_logins: 5,
            lockout_duration: Duration::from_secs(900), // 15 minutes
            security_alert_window: Duration::from_secs(600), // 10 minutes
            security_alert_cooldown: Duration::from_secs(1800), // 30 minutes
            security_alert_failed_attempts_threshold: 20,
            security_alert_distinct_users_threshold: 5,
            security_alert_lockouts_threshold: 3,

            // Email TOTP
            email_code_expiry: Duration::from_secs(600), // 10 minutes
            max_email_codes_per_hour: 5,
        }
    }
}

impl AuthConfig {
    /// Create a new AuthConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for customizing AuthConfig.
    pub fn builder() -> AuthConfigBuilder {
        AuthConfigBuilder::default()
    }

    /// Get TOTP state timeout in minutes (for chrono::Duration).
    pub fn totp_state_timeout_minutes(&self) -> i64 {
        self.totp_state_timeout.as_secs() as i64 / 60
    }

    /// Get email state timeout in minutes (for chrono::Duration).
    pub fn email_state_timeout_minutes(&self) -> i64 {
        self.email_state_timeout.as_secs() as i64 / 60
    }

    /// Get verification session timeout in minutes.
    pub fn verification_session_timeout_minutes(&self) -> i32 {
        (self.verification_session_timeout.as_secs() / 60) as i32
    }

    /// Get email code expiry in minutes.
    pub fn email_code_expiry_minutes(&self) -> i32 {
        (self.email_code_expiry.as_secs() / 60) as i32
    }

    /// Get rate limit retry-after (short) in seconds.
    pub fn retry_after_short_secs(&self) -> u64 {
        self.rate_limit_retry_after_short.as_secs()
    }

    /// Get rate limit retry-after (long) in seconds.
    pub fn retry_after_long_secs(&self) -> u64 {
        self.rate_limit_retry_after_long.as_secs()
    }

    /// Get lockout duration in seconds.
    pub fn lockout_duration_secs(&self) -> u64 {
        self.lockout_duration.as_secs()
    }

    pub fn security_alert_config(&self) -> SecurityAlertConfig {
        SecurityAlertConfig {
            window: chrono::Duration::from_std(self.security_alert_window)
                .unwrap_or_else(|_| chrono::Duration::minutes(10)),
            cooldown: chrono::Duration::from_std(self.security_alert_cooldown)
                .unwrap_or_else(|_| chrono::Duration::minutes(30)),
            failed_attempts_threshold: self.security_alert_failed_attempts_threshold as i64,
            distinct_users_threshold: self.security_alert_distinct_users_threshold as i64,
            lockouts_threshold: self.security_alert_lockouts_threshold as i64,
        }
    }
}

/// Builder for AuthConfig with fluent API.
#[derive(Debug, Clone, Default)]
pub struct AuthConfigBuilder {
    config: AuthConfig,
}

impl AuthConfigBuilder {
    /// Set the login rate limit per hour.
    pub fn login_rate_limit_per_hour(mut self, value: u32) -> Self {
        self.config.login_rate_limit_per_hour = value;
        self
    }

    /// Set the register rate limit per hour.
    pub fn register_rate_limit_per_hour(mut self, value: u32) -> Self {
        self.config.register_rate_limit_per_hour = value;
        self
    }

    /// Set the password change rate limit per hour.
    pub fn password_change_rate_limit_per_hour(mut self, value: u32) -> Self {
        self.config.password_change_rate_limit_per_hour = value;
        self
    }

    /// Set the passkey registration rate limit per hour.
    pub fn passkey_register_rate_limit_per_hour(mut self, value: u32) -> Self {
        self.config.passkey_register_rate_limit_per_hour = value;
        self
    }

    /// Set the passkey login rate limit per hour.
    pub fn passkey_login_rate_limit_per_hour(mut self, value: u32) -> Self {
        self.config.passkey_login_rate_limit_per_hour = value;
        self
    }

    /// Set the token refresh rate limit per hour.
    pub fn refresh_rate_limit_per_hour(mut self, value: u32) -> Self {
        self.config.refresh_rate_limit_per_hour = value;
        self
    }

    /// Set the rate limit backend type.
    pub fn rate_limit_backend(mut self, backend: RateLimitBackendType) -> Self {
        self.config.rate_limit_backend = backend;
        self
    }

    /// Set the Redis URL for rate limiting.
    pub fn redis_url(mut self, url: impl Into<String>) -> Self {
        self.config.redis_url = url.into();
        self
    }

    /// Set the maximum TOTP attempts.
    pub fn max_totp_attempts(mut self, value: u32) -> Self {
        self.config.max_totp_attempts = value;
        self
    }

    /// Set the maximum email code attempts.
    pub fn max_email_code_attempts(mut self, value: u32) -> Self {
        self.config.max_email_code_attempts = value;
        self
    }

    /// Set the TOTP state timeout.
    pub fn totp_state_timeout(mut self, duration: Duration) -> Self {
        self.config.totp_state_timeout = duration;
        self
    }

    /// Set the email state timeout.
    pub fn email_state_timeout(mut self, duration: Duration) -> Self {
        self.config.email_state_timeout = duration;
        self
    }

    /// Set the maximum failed logins before lockout.
    pub fn max_failed_logins(mut self, value: u32) -> Self {
        self.config.max_failed_logins = value;
        self
    }

    /// Set the lockout duration.
    pub fn lockout_duration(mut self, duration: Duration) -> Self {
        self.config.lockout_duration = duration;
        self
    }

    /// Build the AuthConfig.
    pub fn build(self) -> AuthConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = AuthConfig::default();

        assert_eq!(config.login_rate_limit_per_hour, 10);
        assert_eq!(config.register_rate_limit_per_hour, 5);
        assert_eq!(config.max_totp_attempts, 5);
        assert_eq!(config.max_failed_logins, 5);
        assert_eq!(config.totp_state_timeout, Duration::from_secs(300));
        assert_eq!(config.lockout_duration, Duration::from_secs(900));
        assert_eq!(config.security_alert_window, Duration::from_secs(600));
        assert_eq!(config.security_alert_cooldown, Duration::from_secs(1800));
        assert_eq!(config.security_alert_failed_attempts_threshold, 20);
        assert_eq!(config.security_alert_distinct_users_threshold, 5);
        assert_eq!(config.security_alert_lockouts_threshold, 3);
    }

    #[test]
    fn test_builder() {
        let config = AuthConfig::builder()
            .login_rate_limit_per_hour(20)
            .max_totp_attempts(3)
            .lockout_duration(Duration::from_secs(1800))
            .build();

        assert_eq!(config.login_rate_limit_per_hour, 20);
        assert_eq!(config.max_totp_attempts, 3);
        assert_eq!(config.lockout_duration, Duration::from_secs(1800));
        // Other values should be defaults
        assert_eq!(config.register_rate_limit_per_hour, 5);
    }

    #[test]
    fn test_helper_methods() {
        let config = AuthConfig::default();

        assert_eq!(config.totp_state_timeout_minutes(), 5);
        assert_eq!(config.email_state_timeout_minutes(), 10);
        assert_eq!(config.verification_session_timeout_minutes(), 5);
        assert_eq!(config.email_code_expiry_minutes(), 10);
        assert_eq!(config.retry_after_short_secs(), 60);
        assert_eq!(config.retry_after_long_secs(), 300);
        assert_eq!(config.lockout_duration_secs(), 900);
    }
}
