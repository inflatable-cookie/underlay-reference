use super::*;
use super::helpers::{map_session_row, session_status_db, timestamp_to_datetime};

impl AcmeLocalAuthService {
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

    pub(super) async fn update_session(&self, session: &DbSession) -> AuthResult<()> {
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

    pub(super) async fn revoke_session(
        &self,
        session_id: Uuid,
        reason: &str,
    ) -> AuthResult<()> {
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
