use super::*;

impl AcmeLocalAuthService {
    pub(super) async fn find_totp_details(
        &self,
        user_id: Uuid,
    ) -> AuthResult<Option<TotpDetails>> {
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

    pub(super) async fn write_totp_details(&self, details: &TotpDetails) -> AuthResult<()> {
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

    pub(super) async fn verify_totp_second_factor(
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
        .map_err(|e| {
            AuthError::Internal(format!("Failed to create verification session: {}", e))
        })?;

        Ok(session)
    }
}
