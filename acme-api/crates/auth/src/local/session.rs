use super::*;

impl AcmeLocalAuthService {
    pub async fn refresh(&self, refresh_token: &str) -> AuthResult<AuthSession> {
        self.refresh_with_fingerprint(refresh_token, None).await
    }

    /// Refresh tokens with optional fingerprint validation. Delegates to the
    /// canonical underlay-auth-session state machine.
    pub async fn refresh_with_fingerprint(
        &self,
        refresh_token: &str,
        current_fingerprint: Option<SessionFingerprint>,
    ) -> AuthResult<AuthSession> {
        let outcome = self
            .sessions
            .refresh(refresh_token, current_fingerprint.map(to_canonical_fingerprint))
            .await?;

        let user = self
            .find_user_by_id(outcome.session.user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(AuthSession {
            user,
            session: record_into_public(outcome.session),
            access_token: outcome.tokens.access_token,
            refresh_token: outcome.tokens.refresh_token,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> AuthResult<()> {
        self.sessions.logout(refresh_token).await
    }

    /// List all sessions for a given user.
    pub async fn list_sessions(&self, user_id: Uuid) -> AuthResult<Vec<Session>> {
        Ok(self
            .sessions
            .list_sessions_for_user(user_id)
            .await?
            .into_iter()
            .map(record_into_public)
            .collect())
    }

    /// Revoke a specific session for a user.
    pub async fn revoke_session_for_user(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        reason: &str,
    ) -> AuthResult<()> {
        self.sessions
            .revoke_session_for_user(user_id, session_id, reason)
            .await
    }

    /// Revoke all active sessions for a user.
    pub async fn revoke_all_sessions_for_user(
        &self,
        user_id: Uuid,
        reason: &str,
    ) -> AuthResult<u64> {
        self.sessions
            .revoke_all_sessions_for_user(user_id, reason)
            .await
    }

    pub async fn verify_access_principal(
        &self,
        access_token: &str,
    ) -> AuthResult<(Uuid, Vec<String>)> {
        self.sessions.verify_access_principal(access_token).await
    }

    // =====================
    // Internal session helpers
    // =====================

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
        let (tokens, record) = self
            .sessions
            .create_session(user_id, roles, fingerprint.map(to_canonical_fingerprint))
            .await?;
        Ok((
            Tokens {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            },
            record_into_public(record),
        ))
    }
}

/// Convert the public-facing fingerprint type into the canonical one.
fn to_canonical_fingerprint(fp: SessionFingerprint) -> underlay_auth_session::SessionFingerprint {
    underlay_auth_session::SessionFingerprint::new(fp.ip_address, fp.user_agent)
}

/// Map a canonical SessionRecord onto the public underlay Session shape.
fn record_into_public(record: underlay_auth_session::SessionRecord) -> Session {
    Session {
        id: record.id,
        user_id: record.user_id,
        access_token_fingerprint: record.access_token_fingerprint,
        refresh_token_fingerprint: record.refresh_token_fingerprint,
        access_token_expires_at: record.access_token_expires_at,
        refresh_token_expires_at: record.refresh_token_expires_at,
        created_at: record.created_at,
        last_used_at: record.last_used_at,
        ip_address: record.ip_address,
        user_agent: record.user_agent,
        status: if record.is_active {
            SessionStatus::Active
        } else {
            SessionStatus::Revoked
        },
        revocation_reason: record.revoked_reason,
        revoked_at: record.revoked_at,
    }
}
