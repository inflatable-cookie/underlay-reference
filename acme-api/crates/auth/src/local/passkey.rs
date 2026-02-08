use super::*;
use super::helpers::{map_credential_row, roles_for_user};

impl AcmeLocalAuthService {
    // ========================================================================
    // Passkey (WebAuthn) Management
    // ========================================================================

    /// Find all passkey credentials for a user.
    pub(super) async fn find_passkey_credentials(
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
    pub(super) async fn find_passkey_by_credential_id(
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
                    serde_json::Value::String(stored.credential_id.clone()),
                )
                .ok()
            })
            .collect::<Vec<_>>();

        let exclude = if exclude.is_empty() {
            None
        } else {
            Some(exclude)
        };

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
        let passkey = self
            .webauthn
            .finish_passkey_registration(&state, &credential)?;

        let stored_passkey = self.webauthn.stored_passkey_from_passkey(&passkey)?;
        let secret = serde_json::to_string(&stored_passkey)
            .map_err(|_| AuthError::Internal("failed to encode passkey".into()))?;
        let metadata =
            underlay_auth_webauthn::WebAuthnService::credential_metadata_from_stored_passkey(
                &stored_passkey,
            );

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
            let encoded =
                underlay_auth_webauthn::WebAuthnService::encode_authentication_state(&state)?;

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
        let encoded =
            underlay_auth_webauthn::WebAuthnService::encode_discoverable_authentication_state(
                &state,
            )?;

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

            let state =
                underlay_auth_webauthn::WebAuthnService::decode_authentication_state(encoded)?;
            let result = self
                .webauthn
                .finish_passkey_authentication(&credential, &state)?;

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

            let state =
                underlay_auth_webauthn::WebAuthnService::decode_discoverable_authentication_state(
                    encoded,
                )?;

            let (identified_user_id, cred_id) = self
                .webauthn
                .identify_discoverable_authentication(&credential)?;
            let credential_id =
                underlay_auth_webauthn::WebAuthnService::credential_id_to_base64url(&cred_id)?;

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
        let update_result = self
            .webauthn
            .update_stored_passkey_after_authentication(&stored, &result)?;
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

        let state_data = value
            .as_object()
            .ok_or_else(|| AuthError::BadRequest("invalid passkey verification state".into()))?;

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
        let result = self
            .webauthn
            .finish_passkey_authentication(&credential, &state)?;

        let credential_id =
            underlay_auth_webauthn::WebAuthnService::authentication_result_credential_id_base64url(
                &result,
            )?;

        let (pk_user_id, passkey_credential_id, stored) = self
            .find_passkey_by_credential_id(&credential_id)
            .await?
            .ok_or(AuthError::PassKeyCredentialNotFound)?;

        if pk_user_id != user_id {
            return Err(AuthError::PassKeyAuthenticationFailed);
        }

        // Update the stored passkey with new counter
        let update_result = self
            .webauthn
            .update_stored_passkey_after_authentication(&stored, &result)?;
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
        .map_err(|e| {
            AuthError::Internal(format!("Failed to create verification session: {}", e))
        })?;

        Ok(session)
    }
}
