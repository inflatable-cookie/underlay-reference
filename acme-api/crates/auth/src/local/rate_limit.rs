use super::*;

impl AcmeLocalAuthService {
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
    pub(super) async fn check_register_rate_limit(&self, ip: Option<&str>) -> AuthResult<()> {
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
    pub(super) async fn check_password_change_rate_limit(
        &self,
        user_id: Uuid,
    ) -> AuthResult<()> {
        let key = format!("password_change:{}", user_id);

        let rl_config =
            RateLimitConfig::per_hour(self.config.password_change_rate_limit_per_hour.into());

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
    pub(super) async fn check_passkey_register_rate_limit(
        &self,
        user_id: Uuid,
    ) -> AuthResult<()> {
        let key = format!("passkey_register:{}", user_id);

        let rl_config =
            RateLimitConfig::per_hour(self.config.passkey_register_rate_limit_per_hour.into());

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
    pub(super) async fn check_passkey_login_rate_limit(
        &self,
        email: Option<&str>,
    ) -> AuthResult<()> {
        let key = match email {
            Some(email) => format!("passkey_login:{}", email.to_lowercase()),
            None => "passkey_login:discoverable".to_string(),
        };

        let rl_config =
            RateLimitConfig::per_hour(self.config.passkey_login_rate_limit_per_hour.into());

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
}
