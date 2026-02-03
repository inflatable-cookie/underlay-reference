//! Auth-related database operations.
//!
//! Session cleanup and maintenance functions for the auth schema.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DbPool;
use tracing::{info, instrument};

/// Result of a session cleanup operation.
#[derive(Debug)]
pub struct SessionCleanupResult {
    /// Number of expired sessions deleted.
    pub expired_deleted: u64,
    /// Number of revoked sessions deleted (past retention period).
    pub revoked_deleted: u64,
}

/// Delete old sessions from the database.
///
/// This function removes:
/// - Sessions where `refresh_token_expires_at` is more than `expired_retention_days` old
/// - Revoked sessions where `revoked_at` is more than `revoked_retention_days` old
///
/// Recommended values:
/// - `expired_retention_days`: 30 (keep expired sessions for a month)
/// - `revoked_retention_days`: 90 (keep revoked sessions for audit trail)
#[instrument(skip(pool), fields(expired_retention_days, revoked_retention_days))]
pub async fn cleanup_expired_sessions(
    pool: &DbPool,
    expired_retention_days: i32,
    revoked_retention_days: i32,
) -> Result<SessionCleanupResult, sqlx::Error> {
    // Delete sessions that expired more than N days ago
    let expired_result = sqlx::query(
        r#"
        DELETE FROM auth.sessions
        WHERE refresh_token_expires_at < NOW() - make_interval(days => $1)
        "#,
    )
    .bind(expired_retention_days)
    .execute(pool)
    .await?;

    let expired_deleted = expired_result.rows_affected();

    // Delete revoked sessions older than N days (keeping recent ones for audit)
    let revoked_result = sqlx::query(
        r#"
        DELETE FROM auth.sessions
        WHERE status = 'revoked'
          AND revoked_at IS NOT NULL
          AND revoked_at < NOW() - make_interval(days => $1)
        "#,
    )
    .bind(revoked_retention_days)
    .execute(pool)
    .await?;

    let revoked_deleted = revoked_result.rows_affected();

    info!(
        expired_deleted,
        revoked_deleted, "Session cleanup completed"
    );

    Ok(SessionCleanupResult {
        expired_deleted,
        revoked_deleted,
    })
}

/// Delete all expired auth state entries (MFA setup, passkey registration, etc.).
///
/// These are short-lived state entries used for multi-step auth flows.
/// Typically expire within minutes, so cleanup can be aggressive.
#[instrument(skip(pool))]
pub async fn cleanup_expired_auth_state(pool: &DbPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM auth.auth_state
        WHERE expires_at < NOW()
        "#,
    )
    .execute(pool)
    .await?;

    let deleted = result.rows_affected();

    if deleted > 0 {
        info!(deleted, "Expired auth state entries cleaned up");
    }

    Ok(deleted)
}

/// Delete old login attempts (for storage hygiene).
///
/// Login attempts are useful for recent audit/rate-limiting but don't need
/// indefinite retention.
#[instrument(skip(pool), fields(retention_days))]
pub async fn cleanup_old_login_attempts(
    pool: &DbPool,
    retention_days: i32,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM auth.login_attempts
        WHERE attempted_at < NOW() - make_interval(days => $1)
        "#,
    )
    .bind(retention_days)
    .execute(pool)
    .await?;

    let deleted = result.rows_affected();

    if deleted > 0 {
        info!(deleted, "Old login attempts cleaned up");
    }

    Ok(deleted)
}

// ============================================================================
// Email TOTP Functions
// ============================================================================

/// Purpose of an email TOTP code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailTotpPurpose {
    Login,
    PasswordChange,
    SensitiveAction,
    PasswordReset,
}

impl EmailTotpPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailTotpPurpose::Login => "login",
            EmailTotpPurpose::PasswordChange => "password_change",
            EmailTotpPurpose::SensitiveAction => "sensitive_action",
            EmailTotpPurpose::PasswordReset => "password_reset",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "login" => Some(EmailTotpPurpose::Login),
            "password_change" => Some(EmailTotpPurpose::PasswordChange),
            "sensitive_action" => Some(EmailTotpPurpose::SensitiveAction),
            "password_reset" => Some(EmailTotpPurpose::PasswordReset),
            _ => None,
        }
    }
}

/// Row returned when creating an email TOTP code.
#[derive(Debug, FromRow)]
pub struct EmailTotpCodeRow {
    pub id: Uuid,
    pub expires_at: DateTime<Utc>,
}

/// Create a new email TOTP code.
///
/// Returns the code ID and expiry time. The actual code should be generated
/// by the caller and passed as a hash.
#[instrument(skip(pool, code_hash))]
pub async fn create_email_totp_code(
    pool: &DbPool,
    user_id: Uuid,
    email: &str,
    code_hash: &str,
    purpose: EmailTotpPurpose,
    expires_in_minutes: i32,
) -> Result<EmailTotpCodeRow, sqlx::Error> {
    // First, invalidate any existing unused codes for this user/purpose
    sqlx::query(
        r#"
        UPDATE auth.email_totp_codes
        SET used_at = NOW()
        WHERE user_id = $1
          AND purpose = $2
          AND used_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(purpose.as_str())
    .execute(pool)
    .await?;

    // Create the new code
    let row = sqlx::query_as::<_, EmailTotpCodeRow>(
        r#"
        INSERT INTO auth.email_totp_codes (user_id, email, code_hash, purpose, expires_at)
        VALUES ($1, $2, $3, $4, NOW() + make_interval(mins => $5))
        RETURNING id, expires_at
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(code_hash)
    .bind(purpose.as_str())
    .bind(expires_in_minutes)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Result of attempting to get a code for verification.
#[derive(Debug, FromRow)]
pub struct EmailTotpCodeForVerification {
    pub id: Uuid,
    pub code_hash: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub expires_at: DateTime<Utc>,
}

/// Get an email TOTP code for verification.
///
/// Returns None if no valid code exists for this user/purpose.
#[instrument(skip(pool))]
pub async fn get_email_totp_code_for_verification(
    pool: &DbPool,
    user_id: Uuid,
    purpose: EmailTotpPurpose,
) -> Result<Option<EmailTotpCodeForVerification>, sqlx::Error> {
    let row = sqlx::query_as::<_, EmailTotpCodeForVerification>(
        r#"
        SELECT id, code_hash, attempts, max_attempts, expires_at
        FROM auth.email_totp_codes
        WHERE user_id = $1
          AND purpose = $2
          AND used_at IS NULL
          AND expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(purpose.as_str())
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Increment the attempt count for a code.
///
/// Returns the new attempt count.
#[instrument(skip(pool))]
pub async fn increment_email_totp_attempts(
    pool: &DbPool,
    code_id: Uuid,
) -> Result<i32, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i32>(
        r#"
        UPDATE auth.email_totp_codes
        SET attempts = attempts + 1
        WHERE id = $1
        RETURNING attempts
        "#,
    )
    .bind(code_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Mark an email TOTP code as used.
#[instrument(skip(pool))]
pub async fn mark_email_totp_code_used(pool: &DbPool, code_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE auth.email_totp_codes
        SET used_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(code_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Rate limit result.
#[derive(Debug)]
pub struct RateLimitResult {
    /// Current send count (emails actually sent this hour).
    pub send_count: i32,
    /// Current attempt count (all attempts including blocked).
    pub attempt_count: i32,
    /// Whether the rate limit has been exceeded.
    pub is_limited: bool,
}

/// Check the email TOTP rate limit and record an attempt.
///
/// This increments attempt_count but NOT send_count. Call `increment_email_totp_send_count`
/// after successfully sending the email to increment send_count.
///
/// Returns the current counts and whether the user is rate limited.
/// Max 5 codes per hour by default.
#[instrument(skip(pool))]
pub async fn check_email_totp_rate_limit(
    pool: &DbPool,
    user_id: Uuid,
    max_per_hour: i32,
) -> Result<RateLimitResult, sqlx::Error> {
    // Upsert the rate limit record, incrementing attempt_count only
    let row = sqlx::query_as::<_, (i32, i32)>(
        r#"
        INSERT INTO auth.email_totp_rate_limits (user_id, hour_bucket, send_count, attempt_count)
        VALUES ($1, date_trunc('hour', NOW()), 0, 1)
        ON CONFLICT (user_id, hour_bucket)
        DO UPDATE SET attempt_count = auth.email_totp_rate_limits.attempt_count + 1
        RETURNING send_count, attempt_count
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(RateLimitResult {
        send_count: row.0,
        attempt_count: row.1,
        is_limited: row.0 >= max_per_hour,
    })
}

/// Increment the send count after successfully sending an email.
///
/// Call this after the email has been sent successfully.
#[instrument(skip(pool))]
pub async fn increment_email_totp_send_count(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<i32, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i32>(
        r#"
        UPDATE auth.email_totp_rate_limits
        SET send_count = send_count + 1
        WHERE user_id = $1 AND hour_bucket = date_trunc('hour', NOW())
        RETURNING send_count
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// Clean up expired email TOTP codes and old rate limit records.
#[instrument(skip(pool))]
pub async fn cleanup_expired_email_totp(pool: &DbPool) -> Result<u64, sqlx::Error> {
    // Delete expired codes (keep for 1 day after expiry for debugging)
    let codes_result = sqlx::query(
        r#"
        DELETE FROM auth.email_totp_codes
        WHERE expires_at < NOW() - INTERVAL '1 day'
        "#,
    )
    .execute(pool)
    .await?;

    // Delete old rate limit records (older than 2 hours)
    let limits_result = sqlx::query(
        r#"
        DELETE FROM auth.email_totp_rate_limits
        WHERE hour_bucket < NOW() - INTERVAL '2 hours'
        "#,
    )
    .execute(pool)
    .await?;

    let total = codes_result.rows_affected() + limits_result.rows_affected();

    if total > 0 {
        info!(
            codes_deleted = codes_result.rows_affected(),
            limits_deleted = limits_result.rows_affected(),
            "Email TOTP cleanup completed"
        );
    }

    Ok(total)
}

// ============================================================================
// Verification Session Functions
// ============================================================================

/// Method used for verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMethod {
    Totp,
    Passkey,
    EmailTotp,
}

impl VerificationMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationMethod::Totp => "totp",
            VerificationMethod::Passkey => "passkey",
            VerificationMethod::EmailTotp => "email_totp",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "totp" => Some(VerificationMethod::Totp),
            "passkey" => Some(VerificationMethod::Passkey),
            "email_totp" => Some(VerificationMethod::EmailTotp),
            _ => None,
        }
    }
}

/// A verification session row.
#[derive(Debug, FromRow)]
pub struct VerificationSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub purpose: String,
    pub method: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

/// Create a verification session after successful 2FA.
///
/// Returns the session ID and expiry time.
#[instrument(skip(pool))]
pub async fn create_verification_session(
    pool: &DbPool,
    user_id: Uuid,
    purpose: EmailTotpPurpose,
    method: VerificationMethod,
    expires_in_minutes: i32,
) -> Result<VerificationSessionRow, sqlx::Error> {
    let row = sqlx::query_as::<_, VerificationSessionRow>(
        r#"
        INSERT INTO auth.verification_sessions (user_id, purpose, method, expires_at)
        VALUES ($1, $2, $3, NOW() + make_interval(mins => $4))
        RETURNING id, user_id, purpose, method, created_at, expires_at, used_at
        "#,
    )
    .bind(user_id)
    .bind(purpose.as_str())
    .bind(method.as_str())
    .bind(expires_in_minutes)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Get a verification session if it's valid (not used, not expired).
#[instrument(skip(pool))]
pub async fn get_verification_session(
    pool: &DbPool,
    session_id: Uuid,
    user_id: Uuid,
    purpose: EmailTotpPurpose,
) -> Result<Option<VerificationSessionRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, VerificationSessionRow>(
        r#"
        SELECT id, user_id, purpose, method, created_at, expires_at, used_at
        FROM auth.verification_sessions
        WHERE id = $1
          AND user_id = $2
          AND purpose = $3
          AND used_at IS NULL
          AND expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(purpose.as_str())
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Consume (mark as used) a verification session.
///
/// Returns true if the session was found and consumed, false if not found or already used.
#[instrument(skip(pool))]
pub async fn consume_verification_session(
    pool: &DbPool,
    session_id: Uuid,
    user_id: Uuid,
    purpose: EmailTotpPurpose,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE auth.verification_sessions
        SET used_at = NOW()
        WHERE id = $1
          AND user_id = $2
          AND purpose = $3
          AND used_at IS NULL
          AND expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(purpose.as_str())
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Clean up expired verification sessions.
#[instrument(skip(pool))]
pub async fn cleanup_expired_verification_sessions(pool: &DbPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM auth.verification_sessions
        WHERE expires_at < NOW() - INTERVAL '1 day'
        "#,
    )
    .execute(pool)
    .await?;

    let deleted = result.rows_affected();

    if deleted > 0 {
        info!(deleted, "Expired verification sessions cleaned up");
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a test database.
    // These are placeholder tests to verify the module compiles.

    #[test]
    fn cleanup_result_debug() {
        let result = SessionCleanupResult {
            expired_deleted: 10,
            revoked_deleted: 5,
        };
        assert!(format!("{:?}", result).contains("expired_deleted: 10"));
    }

    #[test]
    fn email_totp_purpose_conversion() {
        assert_eq!(EmailTotpPurpose::Login.as_str(), "login");
        assert_eq!(
            EmailTotpPurpose::parse("password_change"),
            Some(EmailTotpPurpose::PasswordChange)
        );
        assert_eq!(
            EmailTotpPurpose::parse("password_reset"),
            Some(EmailTotpPurpose::PasswordReset)
        );
        assert_eq!(EmailTotpPurpose::parse("invalid"), None);
    }

    #[test]
    fn verification_method_conversion() {
        assert_eq!(VerificationMethod::EmailTotp.as_str(), "email_totp");
        assert_eq!(
            VerificationMethod::parse("passkey"),
            Some(VerificationMethod::Passkey)
        );
        assert_eq!(VerificationMethod::parse("invalid"), None);
    }
}
