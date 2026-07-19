use super::helpers::roles_for_user;
use super::*;

impl AcmeLocalAuthService {
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

    /// Burn one KDF computation to equalize the timing of account-miss paths
    /// against a real password verify. The result is intentionally discarded.
    async fn dummy_verify(&self) {
        let _ = hash_password_blocking(
            self.password_hasher.clone(),
            b"acme-login-timing-equalizer".to_vec(),
        )
        .await;
    }

    /// Verify user credentials (email + password) with rate limiting and lockout.
    ///
    /// Deliberately hand-rolled rather than adopting the foundation
    /// `PasswordAuthService` (g01.008 decision): this path additionally records
    /// per-IP login attempts and feeds the security-alert pipeline
    /// (`record_failed_login` / `record_locked_login_attempt`), which the
    /// foundation's `PasswordAuthRepository` seam cannot express (its
    /// `record_failed_login` does not receive the client IP). It preserves the
    /// same contract-030 posture the service ships: per-email+IP rate limits,
    /// lockout with failure accounting, and the `dummy_verify` timing
    /// equalizer on every miss path.
    pub(super) async fn verify_user_credentials(
        &self,
        email: &str,
        password: &str,
        ip: Option<&str>,
    ) -> AuthResult<(User, Credential, String)> {
        // Check rate limit before any database lookups
        self.check_login_rate_limit(email, ip).await?;

        let Some(user) = self.find_user_by_email(email).await? else {
            // Burn one KDF pass so an unknown email costs the same as a real
            // verify (mirrors underlay's PasswordAuthService::dummy_verify).
            // Without this, the fast miss path is an account-existence oracle.
            self.dummy_verify().await;
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
            if let Err(err) = self.record_locked_login_attempt(user.id, ip).await {
                tracing::warn!(
                    error = ?err,
                    user_id = %user.id,
                    "failed to record locked login attempt"
                );
            }
            return Err(AuthError::RateLimited {
                retry_after_seconds: retry_after,
            });
        }

        let Some((credential, role)) = self.find_password_credential_and_role(user.id).await?
        else {
            // Same timing equalizer: a user with no password credential must
            // not resolve faster than a real password verify.
            self.dummy_verify().await;
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
            self.verify_totp_second_factor(user.id, &totp, code).await?;
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

        if let Err(err) = self.verify_totp_second_factor(user.id, &totp, code).await {
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
}
