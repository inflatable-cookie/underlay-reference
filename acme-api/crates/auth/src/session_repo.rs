//! Persistence adapters wiring underlay-auth-session to acme's schema.

use async_trait::async_trait;
use sqlx::Row;
use underlay_auth::AuthResult;
use underlay_auth_session::{
    AccountProvider, AccountState, AccountStatus, SessionRecord, SessionRepository,
};
use underlay_core::Uuid;

use crate::local::helpers::roles_for_user;

fn map_session_record(row: &sqlx::postgres::PgRow) -> SessionRecord {
    SessionRecord {
        id: Uuid(row.get("id")),
        user_id: Uuid(row.get("user_id")),
        roles: row
            .get::<serde_json::Value, _>("roles")
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        is_active: row.get("is_active"),
        access_token_fingerprint: row.get("access_token_fingerprint"),
        refresh_token_fingerprint: row.get("refresh_token_fingerprint"),
        refresh_token_id: Uuid(row.get("refresh_token_id")),
        refresh_token_version: row.get("refresh_token_version"),
        access_token_expires_at: row.get("access_token_expires_at"),
        refresh_token_expires_at: row.get("refresh_token_expires_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_used_at: row.get("last_used_at"),
        ip_address: row.get("ip_address"),
        user_agent: row.get("user_agent"),
        revoked_reason: row.get("revocation_reason"),
        revoked_at: row.get("revoked_at"),
    }
}

const SESSION_COLUMNS: &str = r#"
    id, user_id, roles, is_active,
    access_token_fingerprint, refresh_token_fingerprint,
    refresh_token_id, refresh_token_version,
    access_token_expires_at, refresh_token_expires_at,
    created_at, updated_at, last_used_at,
    ip_address, user_agent,
    status, revocation_reason, revoked_at
"#;

/// `auth.sessions`-backed repository for the canonical session service.
#[derive(Debug, Clone)]
pub struct AcmeSessionRepo {
    pool: sqlx::PgPool,
}

impl AcmeSessionRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for AcmeSessionRepo {
    async fn get_session(&self, session_id: Uuid) -> AuthResult<Option<SessionRecord>> {
        let row = sqlx::query(&format!(
            "SELECT {SESSION_COLUMNS} FROM auth.sessions WHERE id = $1"
        ))
        .bind(session_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| underlay_auth::AuthError::Internal("DB error".into()))?;

        Ok(row.as_ref().map(map_session_record))
    }

    async fn insert_session(&self, session: &SessionRecord) -> AuthResult<()> {
        sqlx::query(
            r#"
            INSERT INTO auth.sessions (
                id, user_id, roles, is_active,
                access_token_fingerprint, refresh_token_fingerprint,
                refresh_token_id, refresh_token_version,
                access_token_expires_at, refresh_token_expires_at,
                created_at, updated_at, last_used_at,
                ip_address, user_agent, status
            ) VALUES ($1,$2,$3,TRUE,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'active')
            "#,
        )
        .bind(session.id.into_inner())
        .bind(session.user_id.into_inner())
        .bind(serde_json::to_value(&session.roles).unwrap_or_else(|_| serde_json::json!([])))
        .bind(&session.access_token_fingerprint)
        .bind(&session.refresh_token_fingerprint)
        .bind(session.refresh_token_id.into_inner())
        .bind(session.refresh_token_version)
        .bind(session.access_token_expires_at)
        .bind(session.refresh_token_expires_at)
        .bind(session.created_at)
        .bind(session.updated_at)
        .bind(session.last_used_at)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .execute(&self.pool)
        .await
        .map_err(|_| underlay_auth::AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    async fn rotate_session_if_current(
        &self,
        session: &SessionRecord,
        expected_refresh_token_id: Uuid,
        expected_refresh_token_version: i32,
    ) -> AuthResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE auth.sessions
            SET roles = $2,
                is_active = TRUE,
                access_token_fingerprint = $3,
                refresh_token_fingerprint = $4,
                refresh_token_id = $5,
                refresh_token_version = $6,
                access_token_expires_at = $7,
                refresh_token_expires_at = $8,
                updated_at = $9,
                last_used_at = $10,
                ip_address = $11,
                user_agent = $12
            WHERE id = $1
              AND is_active = TRUE
              AND refresh_token_id = $13
              AND refresh_token_version = $14
            "#,
        )
        .bind(session.id.into_inner())
        .bind(serde_json::to_value(&session.roles).unwrap_or_else(|_| serde_json::json!([])))
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
        .bind(expected_refresh_token_id.into_inner())
        .bind(expected_refresh_token_version)
        .execute(&self.pool)
        .await
        .map_err(|_| underlay_auth::AuthError::Internal("DB error".into()))?;

        Ok(result.rows_affected() == 1)
    }

    async fn revoke_session(&self, session_id: Uuid, reason: &str) -> AuthResult<()> {
        let now = chrono::Utc::now();
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
        .map_err(|_| underlay_auth::AuthError::Internal("DB error".into()))?;

        Ok(())
    }

    async fn list_sessions_for_user(&self, user_id: Uuid) -> AuthResult<Vec<SessionRecord>> {
        let rows = sqlx::query(&format!(
            "SELECT {SESSION_COLUMNS} FROM auth.sessions WHERE user_id = $1 ORDER BY created_at DESC"
        ))
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(|_| underlay_auth::AuthError::Internal("DB error".into()))?;

        Ok(rows.iter().map(map_session_record).collect())
    }

    async fn revoke_all_sessions_for_user(&self, user_id: Uuid, reason: &str) -> AuthResult<u64> {
        let now = chrono::Utc::now();
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
        .map_err(|_| underlay_auth::AuthError::Internal("DB error".into()))?;

        Ok(result.rows_affected())
    }
}

/// Account provider over `auth.users`: fail-closed status mapping plus
/// the cp role-string mapping.
#[derive(Debug, Clone)]
pub struct AcmeAccountProvider {
    pool: sqlx::PgPool,
}

impl AcmeAccountProvider {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountProvider for AcmeAccountProvider {
    async fn account_state(&self, user_id: Uuid) -> AuthResult<Option<AccountState>> {
        let row = sqlx::query("SELECT status, role FROM auth.users WHERE id = $1")
            .bind(user_id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| underlay_auth::AuthError::Internal("DB error".into()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let status = match row.get::<String, _>("status").as_str() {
            "active" => AccountStatus::Active,
            "deleted" => AccountStatus::Deleted,
            // Fail closed: unknown statuses must not authenticate.
            _ => AccountStatus::Suspended,
        };

        let roles = roles_for_user(row.get::<String, _>("role").as_str());

        Ok(Some(AccountState { status, roles }))
    }
}
