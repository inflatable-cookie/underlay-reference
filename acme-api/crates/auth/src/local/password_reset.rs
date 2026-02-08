use super::*;
use super::helpers::map_user_row;

impl AcmeLocalAuthService {
    // ========================================================================
    // Password Reset (Forgot Password) Flow
    // ========================================================================

    /// Get user information for password reset by email.
    ///
    /// Returns the user ID and email if the user exists and is active.
    /// Returns None if user doesn't exist or is not active (prevents enumeration).
    pub async fn get_user_for_password_reset(
        &self,
        email: &str,
    ) -> AuthResult<Option<(Uuid, String)>> {
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
                crate::email_totp::EmailTotpError::TooManyAttempts => AuthError::BadRequest(
                    "Too many invalid attempts. Please request a new code.".into(),
                ),
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
}
