use super::helpers::{map_session_row, timestamp_to_datetime};
use super::*;

impl AcmeLocalAuthService {
    pub async fn refresh(&self, refresh_token: &str) -> AuthResult<AuthSession> {
        self.refresh_with_fingerprint(refresh_token, None).await
    }

    /// Refresh tokens with optional fingerprint validation.
    ///
    /// Mirrors the foundation `SessionManager::refresh_session` posture
    /// (underlay contract 030, RFC 6819 / OAuth 2.0 Security BCP):
    /// - Reuse of a superseded refresh token (stale token fingerprint or
    ///   mismatched token id/version) revokes the whole session family.
    /// - The legitimate concurrent-refresh race is settled by an atomic CAS
    ///   (`rotate_session_if_current`); the loser is rejected without
    ///   revocation and retries with the freshly issued token.
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

        // Reuse detection: a token whose fingerprint or id/version no longer
        // matches the stored rotation state is a superseded token being
        // replayed. Revoke the entire session family.
        if session.refresh_token_fingerprint != token_fingerprint(refresh_token) {
            tracing::warn!(
                session_id = %session.id,
                user_id = %session.user_id,
                "Superseded refresh token replayed (fingerprint mismatch), revoking session family"
            );
            self.revoke_session(session.id, "refresh_reuse_detected")
                .await?;
            return Err(AuthError::TokenFingerprintMismatch);
        }

        let expected_version =
            i32::try_from(claims.version).map_err(|_| AuthError::TokenInvalid)?;
        if session.refresh_token_id != claims.common.token_id
            || session.refresh_token_version != expected_version
        {
            tracing::warn!(
                session_id = %session.id,
                user_id = %session.user_id,
                "Superseded refresh token replayed (id/version mismatch), revoking session family"
            );
            self.revoke_session(session.id, "refresh_reuse_detected")
                .await?;
            return Err(AuthError::TokenInvalid);
        }

        // Check absolute session timeout
        let session_age = Utc::now() - session.created_at;
        if session_age
            > chrono::Duration::from_std(self.config.absolute_session_timeout)
                .unwrap_or(chrono::Duration::days(30))
        {
            // Session has exceeded absolute lifetime - revoke it
            tracing::info!(
                session_id = %session.id,
                user_id = %session.user_id,
                session_age_days = session_age.num_days(),
                "Session exceeded absolute timeout, revoking"
            );
            self.revoke_session(session.id, "absolute_timeout").await?;
            return Err(AuthError::SessionExpired);
        }

        // Validate the client fingerprint (IP / User-Agent) if provided.
        // Strict mode rejects the refresh; the session is left intact because
        // a fingerprint change can be a legitimate network/browser change and
        // the token itself is still current.
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
                    strict = self.config.refresh_fingerprint_strict,
                    "Session fingerprint mismatch on token refresh"
                );

                if self.config.refresh_fingerprint_strict {
                    return Err(AuthError::TokenFingerprintMismatch);
                }
            }
        }

        // Snapshot the rotation state the CAS below must still observe.
        let expected_fingerprint = session.refresh_token_fingerprint.clone();
        let expected_token_id = session.refresh_token_id;

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

        // Atomic rotate-if-current: the WHERE clause re-checks the rotation
        // state so two concurrent refreshes with the same (valid) token cannot
        // both rotate. The loser of the race is rejected without revoking the
        // family - it lost to a legitimate rotation, not a replay.
        let rotated = self
            .rotate_session_if_current(
                &session,
                &expected_fingerprint,
                expected_token_id,
                expected_version,
            )
            .await?;

        if !rotated {
            tracing::info!(
                session_id = %session.id,
                user_id = %session.user_id,
                "Lost concurrent refresh race, rejecting without revocation"
            );
            return Err(AuthError::TokenInvalid);
        }

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
    // Internal session helpers
    // =====================

    pub(super) async fn get_session(&self, session_id: Uuid) -> AuthResult<Option<DbSession>> {
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

    /// Persist a refresh rotation only if the stored rotation state still
    /// matches what this request observed (mirrors the foundation
    /// `SessionStore::rotate_session_if_current` contract). The compare and
    /// the write are one atomic statement; returns `false` when a concurrent
    /// refresh rotated first.
    pub(super) async fn rotate_session_if_current(
        &self,
        session: &DbSession,
        expected_refresh_token_fingerprint: &str,
        expected_refresh_token_id: Uuid,
        expected_refresh_token_version: i32,
    ) -> AuthResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE auth.sessions
            SET access_token_fingerprint = $2,
                refresh_token_fingerprint = $3,
                refresh_token_id = $4,
                refresh_token_version = $5,
                access_token_expires_at = $6,
                refresh_token_expires_at = $7,
                updated_at = $8,
                last_used_at = $9,
                ip_address = $10,
                user_agent = $11
            WHERE id = $1
              AND is_active = TRUE
              AND status = 'active'
              AND refresh_token_fingerprint = $12
              AND refresh_token_id = $13
              AND refresh_token_version = $14
            "#,
        )
        .bind(session.id.into_inner())
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
        .bind(expected_refresh_token_fingerprint)
        .bind(expected_refresh_token_id.into_inner())
        .bind(expected_refresh_token_version)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(result.rows_affected() > 0)
    }

    pub(super) async fn create_session(
        &self,
        user_id: Uuid,
        roles: Vec<String>,
    ) -> AuthResult<(Tokens, Session)> {
        self.create_session_with_fingerprint(user_id, roles, None)
            .await
    }

    pub(super) async fn create_session_with_fingerprint(
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

    pub(super) async fn revoke_session(&self, session_id: Uuid, reason: &str) -> AuthResult<()> {
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
}
