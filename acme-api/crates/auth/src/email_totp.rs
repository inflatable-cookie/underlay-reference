//! Email TOTP verification service.
//!
//! Provides email-based one-time password verification as a fallback 2FA method.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use acme_core::Uuid;
use rand::Rng;
use thiserror::Error;
use tracing::{info, instrument, warn};
use underlay_auth_password::{Argon2Hasher, PasswordHasherExt, PasswordVerifierExt};
use underlay_email::{EmailContext, EmailManager, EmailTemplateEngine};

use acme_db::auth::{
    check_email_totp_rate_limit, create_email_totp_code, create_verification_session,
    get_email_totp_code_for_verification, increment_email_totp_attempts,
    increment_email_totp_send_count, mark_email_totp_code_used, EmailTotpPurpose,
    VerificationMethod, VerificationSessionRow,
};
use acme_db::DbPool;
use acme_infra::EmailConfig;

/// Errors that can occur during email TOTP operations.
#[derive(Debug, Error)]
pub enum EmailTotpError {
    #[error("rate limited: too many code requests")]
    RateLimited,

    #[error("code expired")]
    CodeExpired,

    #[error("invalid code")]
    InvalidCode,

    #[error("too many attempts")]
    TooManyAttempts,

    #[error("no active code found")]
    NoActiveCode,

    #[error("failed to send email: {0}")]
    EmailSendFailed(String),

    #[error("template error: {0}")]
    TemplateError(String),

    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Result type for email TOTP operations.
pub type EmailTotpResult<T> = Result<T, EmailTotpError>;

/// Configuration for the email TOTP service.
#[derive(Debug, Clone)]
pub struct EmailTotpConfig {
    /// Code expiry in minutes (default: 10)
    pub code_expiry_minutes: i32,
    /// Max codes per user per hour (default: 5)
    pub max_codes_per_hour: i32,
    /// Max verification attempts per code (default: 5)
    pub max_attempts: i32,
    /// Verification session expiry in minutes (default: 5)
    pub session_expiry_minutes: i32,
}

impl Default for EmailTotpConfig {
    fn default() -> Self {
        Self {
            code_expiry_minutes: 10,
            max_codes_per_hour: 5,
            max_attempts: 5,
            session_expiry_minutes: 5,
        }
    }
}

/// A verification session returned after successful code verification.
#[derive(Debug, Clone)]
pub struct VerificationSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub purpose: EmailTotpPurpose,
    pub method: VerificationMethod,
    pub verified_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<VerificationSessionRow> for VerificationSession {
    fn from(row: VerificationSessionRow) -> Self {
        Self {
            // Wrap raw uuid::Uuid into acme_core::Uuid
            session_id: Uuid(row.id),
            user_id: Uuid(row.user_id),
            purpose: EmailTotpPurpose::parse(&row.purpose).unwrap_or(EmailTotpPurpose::Login),
            method: VerificationMethod::parse(&row.method).unwrap_or(VerificationMethod::EmailTotp),
            verified_at: row.created_at,
            expires_at: row.expires_at,
        }
    }
}

/// Service for email-based TOTP verification.
pub struct EmailTotpService {
    pool: DbPool,
    email_manager: Arc<EmailManager>,
    email_templates: Arc<EmailTemplateEngine>,
    email_config: EmailConfig,
    hasher: Argon2Hasher,
    config: EmailTotpConfig,
}

impl EmailTotpService {
    /// Create a new email TOTP service.
    pub fn new(
        pool: DbPool,
        email_manager: Arc<EmailManager>,
        email_templates: Arc<EmailTemplateEngine>,
        email_config: EmailConfig,
    ) -> Self {
        Self {
            pool,
            email_manager,
            email_templates,
            email_config,
            hasher: Argon2Hasher::default(),
            config: EmailTotpConfig::default(),
        }
    }

    /// Create a new email TOTP service with custom configuration.
    pub fn with_config(mut self, config: EmailTotpConfig) -> Self {
        self.config = config;
        self
    }

    /// Request a new email TOTP code.
    ///
    /// Generates a 6-digit code, stores its hash in the database, and sends
    /// the code via email. Returns an error if the user is rate limited.
    #[instrument(skip(self), fields(purpose = %purpose.as_str()))]
    pub async fn request_code(
        &self,
        user_id: Uuid,
        email: &str,
        purpose: EmailTotpPurpose,
    ) -> EmailTotpResult<DateTime<Utc>> {
        // Check rate limit and record attempt (convert to raw uuid for DB)
        let raw_user_id = user_id.into_inner();
        let rate_limit =
            check_email_totp_rate_limit(&self.pool, raw_user_id, self.config.max_codes_per_hour)
                .await?;

        if rate_limit.is_limited {
            warn!(
                user_id = %user_id,
                send_count = rate_limit.send_count,
                attempt_count = rate_limit.attempt_count,
                "Email TOTP rate limit exceeded"
            );
            return Err(EmailTotpError::RateLimited);
        }

        // Generate a 6-digit code
        let code = generate_code();

        // Hash the code for storage
        let code_hash = self
            .hasher
            .hash_password(code.as_bytes())
            .map_err(|e| EmailTotpError::DatabaseError(sqlx::Error::Protocol(e.to_string())))?;

        // Store the code
        let code_row = create_email_totp_code(
            &self.pool,
            raw_user_id,
            email,
            &code_hash,
            purpose,
            self.config.code_expiry_minutes,
        )
        .await?;

        // Send the email
        self.send_code_email(email, &code, purpose, self.config.code_expiry_minutes)
            .await?;

        // Increment send count after successful send
        increment_email_totp_send_count(&self.pool, raw_user_id).await?;

        info!(
            user_id = %user_id,
            purpose = %purpose.as_str(),
            expires_at = %code_row.expires_at,
            "Email TOTP code sent"
        );

        Ok(code_row.expires_at)
    }

    /// Internal code verification logic shared by verify_code and verify_code_for_login.
    ///
    /// Handles: fetching stored code, checking expiry, checking attempts,
    /// verifying the code, incrementing attempts on failure, and marking as used.
    async fn verify_code_internal(
        &self,
        user_id: Uuid,
        purpose: EmailTotpPurpose,
        code: &str,
    ) -> EmailTotpResult<()> {
        // Convert to raw uuid for DB operations
        let raw_user_id = user_id.into_inner();

        // Get the active code for this user/purpose
        let stored_code = get_email_totp_code_for_verification(&self.pool, raw_user_id, purpose)
            .await?
            .ok_or(EmailTotpError::NoActiveCode)?;

        // Check if expired
        if stored_code.expires_at < Utc::now() {
            return Err(EmailTotpError::CodeExpired);
        }

        // Check if too many attempts
        if stored_code.attempts >= stored_code.max_attempts {
            return Err(EmailTotpError::TooManyAttempts);
        }

        // Verify the code
        let is_valid = self
            .hasher
            .verify_password(code.as_bytes(), &stored_code.code_hash)
            .unwrap_or(false);

        if !is_valid {
            // Increment attempt count
            let new_attempts = increment_email_totp_attempts(&self.pool, stored_code.id).await?;

            warn!(
                user_id = %user_id,
                attempts = new_attempts,
                max_attempts = stored_code.max_attempts,
                purpose = %purpose.as_str(),
                "Invalid email TOTP code attempt"
            );

            return Err(EmailTotpError::InvalidCode);
        }

        // Mark code as used
        mark_email_totp_code_used(&self.pool, stored_code.id).await?;

        Ok(())
    }

    /// Verify an email TOTP code.
    ///
    /// Returns a verification session on success. The session can be used
    /// to authorize sensitive actions like password changes.
    #[instrument(skip(self, code), fields(purpose = %purpose.as_str()))]
    pub async fn verify_code(
        &self,
        user_id: Uuid,
        purpose: EmailTotpPurpose,
        code: &str,
    ) -> EmailTotpResult<VerificationSession> {
        self.verify_code_internal(user_id, purpose, code).await?;

        // Create verification session
        let session = create_verification_session(
            &self.pool,
            user_id.into_inner(),
            purpose,
            VerificationMethod::EmailTotp,
            self.config.session_expiry_minutes,
        )
        .await?;

        info!(
            user_id = %user_id,
            session_id = %session.id,
            purpose = %purpose.as_str(),
            "Email TOTP verification successful"
        );

        Ok(session.into())
    }

    /// Verify an email TOTP code for login (without creating a verification session).
    ///
    /// This is used during the login flow when email fallback is enforced.
    /// Unlike `verify_code`, this does not create a verification session.
    #[instrument(skip(self, code))]
    pub async fn verify_code_for_login(&self, user_id: Uuid, code: &str) -> EmailTotpResult<()> {
        self.verify_code_internal(user_id, EmailTotpPurpose::Login, code)
            .await?;

        info!(
            user_id = %user_id,
            "Email TOTP verification for login successful"
        );

        Ok(())
    }

    /// Send the TOTP code email.
    async fn send_code_email(
        &self,
        to_email: &str,
        code: &str,
        purpose: EmailTotpPurpose,
        expiry_minutes: i32,
    ) -> EmailTotpResult<()> {
        // Create email context
        let mut context = EmailContext::with_app_info(
            &self.email_config.app_name,
            &self.email_config.app_url,
            &self.email_config.support_email,
        );
        context.set("code", code);
        context.set("expiry_minutes", expiry_minutes);
        context.set("purpose", purpose.as_str());

        // Render the template
        let html_body = self
            .email_templates
            .render("auth/email_totp.html", &context)
            .map_err(|e| EmailTotpError::TemplateError(e.to_string()))?;

        // Build subject based on purpose
        let subject = match purpose {
            EmailTotpPurpose::Login => format!("Your {} login code", self.email_config.app_name),
            EmailTotpPurpose::PasswordChange => {
                format!("Your {} password change code", self.email_config.app_name)
            }
            EmailTotpPurpose::SensitiveAction => {
                format!("Your {} verification code", self.email_config.app_name)
            }
            EmailTotpPurpose::PasswordReset => {
                format!("Reset your {} password", self.email_config.app_name)
            }
        };

        // Build plain text body
        let text_body = format!(
            "Your verification code is: {}\n\nThis code expires in {} minutes.\n\nIf you didn't request this code, you can safely ignore this email.",
            code, expiry_minutes
        );

        // Send the email
        let to = underlay_email::EmailAddress::new(to_email)
            .map_err(|e| EmailTotpError::EmailSendFailed(e.to_string()))?;

        self.email_manager
            .send_email(to, &subject, &text_body, Some(html_body))
            .await
            .map_err(|e| EmailTotpError::EmailSendFailed(e.to_string()))?;

        Ok(())
    }
}

/// Generate a secure 6-digit code.
fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(0..1_000_000);
    format!("{:06}", code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_format() {
        for _ in 0..100 {
            let code = generate_code();
            assert_eq!(code.len(), 6);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_generate_code_randomness() {
        // Generate multiple codes and ensure they're not all the same
        let codes: Vec<String> = (0..10).map(|_| generate_code()).collect();
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        // With 10 codes from 1M possibilities, we should have high diversity
        assert!(unique.len() > 5);
    }

    #[test]
    fn test_email_totp_config_defaults() {
        let config = EmailTotpConfig::default();
        assert_eq!(config.code_expiry_minutes, 10);
        assert_eq!(config.max_codes_per_hour, 5);
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.session_expiry_minutes, 5);
    }
}
