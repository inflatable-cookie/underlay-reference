//! Repository implementations for email TOTP using Acme database.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use acme_db::auth::{
    check_email_totp_rate_limit, consume_verification_session, create_email_totp_code,
    create_verification_session, get_email_totp_code_for_verification,
    get_verification_session, increment_email_totp_attempts, increment_email_totp_send_count,
    mark_email_totp_code_used, EmailTotpPurpose, VerificationMethod,
};
use acme_db::DbPool;
use underlay_auth_email_totp::{
    EmailTotpCodeRepository, EmailTotpError, EmailTotpResult, RateLimitStatus, StoredCode,
    VerificationSession, VerificationSessionRepository,
};

/// Acme database implementation of email TOTP code repository.
#[derive(Clone)]
pub struct AcmeEmailTotpCodeRepository {
    pool: DbPool,
}

impl AcmeEmailTotpCodeRepository {
    /// Create a new repository with the given database pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Parse a purpose string to EmailTotpPurpose.
fn parse_purpose(purpose: &str) -> EmailTotpPurpose {
    EmailTotpPurpose::parse(purpose).unwrap_or(EmailTotpPurpose::SensitiveAction)
}

#[async_trait]
impl EmailTotpCodeRepository for AcmeEmailTotpCodeRepository {
    async fn check_rate_limit(
        &self,
        user_id: &str,
        _purpose: &str,
        max_per_hour: i32,
    ) -> EmailTotpResult<RateLimitStatus> {
        let user_uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid user_id: {}", e)))?;

        let result = check_email_totp_rate_limit(&self.pool, user_uuid, max_per_hour)
            .await
            .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(RateLimitStatus {
            send_count: result.send_count,
            attempt_count: result.attempt_count,
            is_limited: result.is_limited,
        })
    }

    async fn increment_send_count(&self, user_id: &str, _purpose: &str) -> EmailTotpResult<()> {
        let user_uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid user_id: {}", e)))?;

        increment_email_totp_send_count(&self.pool, user_uuid)
            .await
            .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn store_code(
        &self,
        user_id: &str,
        email: &str,
        code_hash: &str,
        purpose: &str,
        expires_at: DateTime<Utc>,
        _max_attempts: i32,
    ) -> EmailTotpResult<String> {
        let user_uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid user_id: {}", e)))?;

        let purpose_enum = parse_purpose(purpose);

        // Calculate expiry minutes from now to expires_at
        let now = Utc::now();
        let duration = expires_at.signed_duration_since(now);
        let expiry_minutes = duration.num_minutes().max(1) as i32;

        let row = create_email_totp_code(
            &self.pool,
            user_uuid,
            email,
            code_hash,
            purpose_enum,
            expiry_minutes,
        )
        .await
        .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(row.id.to_string())
    }

    async fn get_active_code(
        &self,
        user_id: &str,
        purpose: &str,
    ) -> EmailTotpResult<Option<StoredCode>> {
        let user_uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid user_id: {}", e)))?;

        let purpose_enum = parse_purpose(purpose);

        let maybe_code = get_email_totp_code_for_verification(&self.pool, user_uuid, purpose_enum)
            .await
            .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(maybe_code.map(|c| StoredCode {
            id: c.id.to_string(),
            code_hash: c.code_hash,
            expires_at: c.expires_at,
            attempts: c.attempts,
            max_attempts: c.max_attempts,
        }))
    }

    async fn increment_attempts(&self, code_id: &str) -> EmailTotpResult<i32> {
        let code_uuid = uuid::Uuid::parse_str(code_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid code_id: {}", e)))?;

        let attempts = increment_email_totp_attempts(&self.pool, code_uuid)
            .await
            .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(attempts)
    }

    async fn mark_code_used(&self, code_id: &str) -> EmailTotpResult<()> {
        let code_uuid = uuid::Uuid::parse_str(code_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid code_id: {}", e)))?;

        mark_email_totp_code_used(&self.pool, code_uuid)
            .await
            .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(())
    }
}

/// Acme database implementation of verification session repository.
#[derive(Clone)]
pub struct AcmeVerificationSessionRepository {
    pool: DbPool,
}

impl AcmeVerificationSessionRepository {
    /// Create a new repository with the given database pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VerificationSessionRepository for AcmeVerificationSessionRepository {
    async fn create_session(
        &self,
        user_id: &str,
        purpose: &str,
        _method: &str,
        expires_at: DateTime<Utc>,
    ) -> EmailTotpResult<VerificationSession> {
        let user_uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid user_id: {}", e)))?;

        let purpose_enum = parse_purpose(purpose);

        // Calculate expiry minutes from now to expires_at
        let now = Utc::now();
        let duration = expires_at.signed_duration_since(now);
        let expiry_minutes = duration.num_minutes().max(1) as i32;

        let row = create_verification_session(
            &self.pool,
            user_uuid,
            purpose_enum,
            VerificationMethod::EmailTotp,
            expiry_minutes,
        )
        .await
        .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(VerificationSession {
            id: row.id.to_string(),
            user_id: row.user_id.to_string(),
            purpose: row.purpose,
            method: row.method,
            created_at: row.created_at,
            expires_at: row.expires_at,
        })
    }

    async fn consume_session(
        &self,
        session_id: &str,
        user_id: &str,
        purpose: &str,
    ) -> EmailTotpResult<VerificationSession> {
        let session_uuid = uuid::Uuid::parse_str(session_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid session_id: {}", e)))?;
        let user_uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid user_id: {}", e)))?;

        let purpose_enum = parse_purpose(purpose);

        let consumed = consume_verification_session(&self.pool, session_uuid, user_uuid, purpose_enum)
            .await
            .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        if !consumed {
            return Err(EmailTotpError::SessionNotFound);
        }

        // Get the session details (it's now marked as used, but we can still read it)
        // For simplicity, we construct a minimal session since the important thing is
        // that it was successfully consumed
        Ok(VerificationSession {
            id: session_id.to_string(),
            user_id: user_id.to_string(),
            purpose: purpose.to_string(),
            method: "email_totp".to_string(),
            created_at: Utc::now(), // Approximate
            expires_at: Utc::now(), // Approximate
        })
    }

    async fn get_session(
        &self,
        session_id: &str,
        user_id: &str,
        purpose: &str,
    ) -> EmailTotpResult<Option<VerificationSession>> {
        let session_uuid = uuid::Uuid::parse_str(session_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid session_id: {}", e)))?;
        let user_uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|e| EmailTotpError::Storage(format!("invalid user_id: {}", e)))?;

        let purpose_enum = parse_purpose(purpose);

        let maybe_row = get_verification_session(&self.pool, session_uuid, user_uuid, purpose_enum)
            .await
            .map_err(|e| EmailTotpError::Storage(e.to_string()))?;

        Ok(maybe_row.map(|row| VerificationSession {
            id: row.id.to_string(),
            user_id: row.user_id.to_string(),
            purpose: row.purpose,
            method: row.method,
            created_at: row.created_at,
            expires_at: row.expires_at,
        }))
    }
}
