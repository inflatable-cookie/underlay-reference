use super::*;

impl AcmeLocalAuthService {
    /// Consume one slot of an hourly rate limit and translate the outcome.
    ///
    /// Every auth rate limit is the same decision — take a slot, and if the
    /// bucket is full report how long to wait — differing only in the key and
    /// the configured hourly ceiling. Keeping that in one place means a new
    /// limit cannot accidentally ship a different denial shape or forget to
    /// increment.
    async fn enforce_hourly_rate_limit(&self, key: &str, per_hour: u32) -> AuthResult<()> {
        let rl_config = RateLimitConfig::per_hour(per_hour.into());

        let result = self
            .rate_limiter
            .check_and_increment(key, &rl_config)
            .await
            .map_err(|e| AuthError::Internal(format!("Rate limit error: {}", e)))?;

        if result.is_denied() {
            return Err(AuthError::RateLimited {
                retry_after_seconds: result.retry_after_secs(),
            });
        }

        Ok(())
    }

    /// Check rate limit for login attempts.
    pub(super) async fn check_login_rate_limit(
        &self,
        email: &str,
        ip: Option<&str>,
    ) -> AuthResult<()> {
        let key = match ip {
            Some(ip) => format!("login:{}:{}", email.to_lowercase(), ip),
            None => format!("login:{}", email.to_lowercase()),
        };

        self.enforce_hourly_rate_limit(&key, self.config.login_rate_limit_per_hour)
            .await
    }

    /// Check rate limit for registration attempts.
    pub(super) async fn check_register_rate_limit(&self, ip: Option<&str>) -> AuthResult<()> {
        let Some(ip) = ip else {
            return Ok(()); // Can't rate limit without IP
        };

        self.enforce_hourly_rate_limit(
            &format!("register:{}", ip),
            self.config.register_rate_limit_per_hour,
        )
        .await
    }

    /// Check rate limit for password change attempts.
    pub(super) async fn check_password_change_rate_limit(&self, user_id: Uuid) -> AuthResult<()> {
        self.enforce_hourly_rate_limit(
            &format!("password_change:{}", user_id),
            self.config.password_change_rate_limit_per_hour,
        )
        .await
    }

    /// Check rate limit for passkey registration start.
    pub(super) async fn check_passkey_register_rate_limit(&self, user_id: Uuid) -> AuthResult<()> {
        self.enforce_hourly_rate_limit(
            &format!("passkey_register:{}", user_id),
            self.config.passkey_register_rate_limit_per_hour,
        )
        .await
    }

    /// Check rate limit for passkey login start.
    pub(super) async fn check_passkey_login_rate_limit(
        &self,
        email: Option<&str>,
    ) -> AuthResult<()> {
        let key = match email {
            Some(email) => format!("passkey_login:{}", email.to_lowercase()),
            None => "passkey_login:discoverable".to_string(),
        };

        self.enforce_hourly_rate_limit(&key, self.config.passkey_login_rate_limit_per_hour)
            .await
    }

    /// Check rate limit for token refresh attempts.
    ///
    /// Rate limits by fingerprint (IP + User-Agent hash) to prevent
    /// brute force enumeration of valid refresh tokens.
    pub async fn check_refresh_rate_limit(
        &self,
        fingerprint: &crate::SessionFingerprint,
    ) -> AuthResult<()> {
        // Create a key based on fingerprint components
        let ip_part = fingerprint.ip_address.as_deref().unwrap_or("unknown");
        let ua_part = fingerprint
            .user_agent
            .as_deref()
            .map(|ua| {
                // Short hash of the user agent to keep the key bounded. Format
                // to the full 16 hex digits before truncating: `{:x}` does not
                // pad, so a small hash would otherwise be shorter than the
                // slice and panic.
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                ua.hash(&mut hasher);
                format!("{:016x}", hasher.finish())[..8].to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());

        self.enforce_hourly_rate_limit(
            &format!("refresh:{}:{}", ip_part, ua_part),
            self.config.refresh_rate_limit_per_hour,
        )
        .await
    }
}
