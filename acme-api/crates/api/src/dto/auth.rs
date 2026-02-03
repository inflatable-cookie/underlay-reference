//! Authentication-related DTOs for the Acme API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ============================================================================
// Request DTOs
// ============================================================================

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    #[validate(length(max = 254, message = "Email too long"))]
    pub email: String,
    #[validate(length(
        min = 1,
        max = 128,
        message = "Password must be between 1 and 128 characters"
    ))]
    pub password: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    #[validate(length(max = 254, message = "Email too long"))]
    pub email: String,
    #[validate(length(min = 1, max = 128, message = "Password required"))]
    pub password: String,
    #[validate(length(max = 10, message = "Code too long"))]
    pub code: Option<String>,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginStartRequest {
    #[validate(email(message = "Invalid email address"))]
    #[validate(length(max = 254, message = "Email too long"))]
    pub email: String,
    #[validate(length(min = 1, max = 128, message = "Password required"))]
    pub password: String,
    /// When true, if user has no 2FA configured, email verification will be required.
    /// Used by admin login to enforce 2FA even when user hasn't set up TOTP/passkey.
    #[serde(default)]
    pub enforce_email_fallback: bool,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginFinishRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid login state"))]
    pub login_state_id: String,
    #[validate(length(min = 1, max = 20, message = "Invalid code"))]
    pub code: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginEmailFallbackRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid login state"))]
    pub login_state_id: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginEmailFallbackResponse {
    /// New login state ID for email verification
    pub login_state_id: String,
    /// Masked email address (for display)
    pub email: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginEmailResendRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid login state"))]
    pub login_state_id: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    /// Refresh token. Can be omitted if refresh token is provided via httpOnly cookie.
    #[serde(default)]
    #[validate(length(max = 2048, message = "Invalid refresh token"))]
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    /// Refresh token. Can be omitted if refresh token is provided via httpOnly cookie.
    #[serde(default)]
    #[validate(length(max = 2048, message = "Invalid refresh token"))]
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, max = 128, message = "Current password required"))]
    pub current_password: String,
    #[validate(length(min = 1, max = 128, message = "New password required"))]
    pub new_password: String,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TotpEnableRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid setup ID"))]
    pub setup_id: String,
    #[validate(length(min = 1, max = 10, message = "Invalid code"))]
    pub code: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyRenameRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: String,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyLoginStartRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: Option<String>,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyRegisterFinishRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid state ID"))]
    pub state_id: String,
    pub credential: webauthn_rs_proto::attest::RegisterPublicKeyCredential,
    #[validate(length(max = 100, message = "Display name too long"))]
    pub display_name: Option<String>,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyLoginFinishRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid state ID"))]
    pub state_id: String,
    pub credential: webauthn_rs_proto::auth::PublicKeyCredential,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GoogleOAuthCallbackRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid state ID"))]
    pub state_id: String,
    #[validate(length(min = 1, max = 2048, message = "Invalid code"))]
    pub code: String,
    #[validate(length(min = 1, max = 2048, message = "Invalid state"))]
    pub state: String,
}

// ============================================================================
// Response DTOs
// ============================================================================

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthUserDto {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthSessionDto {
    pub user: AuthUserDto,
    pub session_id: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginStartResponse {
    pub requires_two_factor: bool,
    /// When true, the required verification is via email code (not TOTP/passkey).
    /// This happens when `enforceEmailFallback` was requested and user has no 2FA configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_email_verification: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_state_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<AuthSessionDto>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TotpStatusResponse {
    pub enabled: bool,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorStatusResponse {
    pub has_totp_configured: bool,
    pub has_passkey_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp_enabled_at: Option<DateTime<Utc>>,
    pub passkey_count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub id: String,
    pub user_id: String,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
    pub revocation_reason: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<underlay_auth::Session> for SessionDto {
    fn from(s: underlay_auth::Session) -> Self {
        SessionDto {
            id: s.id.to_string(),
            user_id: s.user_id.to_string(),
            access_token_expires_at: s.access_token_expires_at,
            refresh_token_expires_at: s.refresh_token_expires_at,
            created_at: s.created_at,
            last_used_at: s.last_used_at,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            status: match s.status {
                underlay_auth::SessionStatus::Active => "active",
                underlay_auth::SessionStatus::Expired => "expired",
                underlay_auth::SessionStatus::Revoked => "revoked",
            }
            .to_string(),
            revocation_reason: s.revocation_reason,
            revoked_at: s.revoked_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyStartRegistrationDto {
    pub options: webauthn_rs_proto::attest::CreationChallengeResponse,
    pub state_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyCredentialDto {
    pub id: String,
    pub display_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyLoginStartDto {
    pub options: webauthn_rs_proto::auth::RequestChallengeResponse,
    pub state_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleOAuthStartDto {
    pub authorization_url: String,
    pub state_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleOAuthStatusDto {
    pub connected: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleOAuthTokenDto {
    pub access_token: String,
    pub expires_in_seconds: Option<u64>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
}

// ============================================================================
// Helper functions
// ============================================================================

pub fn auth_session_dto_from_session(
    session: acme_auth::AuthSession,
    role: String,
) -> AuthSessionDto {
    // Use display_name if present, otherwise fall back to email username
    let display_name = session
        .user
        .display_name
        .unwrap_or_else(|| derive_display_name_from_email(&session.user.email));

    AuthSessionDto {
        user: AuthUserDto {
            user_id: session.user.id.to_string(),
            email: session.user.email,
            display_name,
            roles: roles_for_user(&role),
        },
        session_id: session.session.id.to_string(),
        access_token: session.access_token,
        refresh_token: session.refresh_token,
    }
}

/// Derive a display name from an email address.
fn derive_display_name_from_email(email: &str) -> String {
    email.split('@').next().unwrap_or("User").to_string()
}

pub fn roles_for_user(role: &str) -> Vec<String> {
    match role {
        "superadmin" => vec!["superadmin".to_string()],
        "admin" => vec!["admin".to_string()],
        "support" => vec!["support".to_string()],
        "tester" => vec!["tester".to_string()],
        _ => vec!["user".to_string()],
    }
}

// ============================================================================
// Email TOTP DTOs
// ============================================================================

/// Purpose for email TOTP verification.
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EmailTotpPurposeDto {
    Login,
    PasswordChange,
    SensitiveAction,
}

impl EmailTotpPurposeDto {
    pub fn to_db_purpose(self) -> acme_db::auth::EmailTotpPurpose {
        match self {
            EmailTotpPurposeDto::Login => acme_db::auth::EmailTotpPurpose::Login,
            EmailTotpPurposeDto::PasswordChange => acme_db::auth::EmailTotpPurpose::PasswordChange,
            EmailTotpPurposeDto::SensitiveAction => {
                acme_db::auth::EmailTotpPurpose::SensitiveAction
            }
        }
    }
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EmailTotpRequestRequest {
    /// The purpose for the verification code.
    pub purpose: EmailTotpPurposeDto,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EmailTotpVerifyRequest {
    /// The 6-digit verification code sent via email.
    #[validate(length(min = 6, max = 6, message = "Code must be 6 digits"))]
    pub code: String,
    /// The purpose this code was requested for.
    pub purpose: EmailTotpPurposeDto,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmailTotpRequestResponse {
    /// When the code expires.
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmailTotpVerifyResponse {
    /// Verification session ID (can be used to authorize sensitive actions).
    pub verification_session_id: String,
    /// When the verification session expires.
    pub expires_at: DateTime<Utc>,
}

// ============================================================================
// TOTP Verification DTOs (for 2FA gates)
// ============================================================================

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TotpVerifyRequest {
    /// The 6-digit TOTP code from authenticator app.
    #[validate(length(min = 1, max = 10, message = "Invalid code"))]
    pub code: String,
    /// The purpose this verification is for.
    pub purpose: EmailTotpPurposeDto,
}

/// Response from TOTP verification. Same structure as email TOTP response.
pub type TotpVerifyResponse = EmailTotpVerifyResponse;

// ============================================================================
// Passkey Verification DTOs (for 2FA gates)
// ============================================================================

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyVerifyStartRequest {
    /// The purpose this verification is for.
    pub purpose: EmailTotpPurposeDto,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyVerifyFinishRequest {
    #[validate(length(min = 1, max = 100, message = "Invalid state ID"))]
    pub state_id: String,
    pub credential: webauthn_rs_proto::auth::PublicKeyCredential,
}

/// Response from passkey verification. Same structure as email TOTP response.
pub type PasskeyVerifyResponse = EmailTotpVerifyResponse;

// ============================================================================
// Password Change with Verification Session DTOs
// ============================================================================

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordWithVerificationRequest {
    /// The verification session ID obtained from 2FA verification.
    #[validate(length(min = 1, max = 100, message = "Invalid verification session ID"))]
    pub verification_session_id: String,
    /// The new password.
    #[validate(length(min = 1, max = 128, message = "New password required"))]
    pub new_password: String,
}

// ============================================================================
// Password Reset (Forgot Password) DTOs
// ============================================================================

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetRequestRequest {
    /// The email address of the account to reset.
    #[validate(email(message = "Invalid email address"))]
    #[validate(length(max = 254, message = "Email too long"))]
    pub email: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetVerifyRequest {
    /// The email address of the account.
    #[validate(email(message = "Invalid email address"))]
    #[validate(length(max = 254, message = "Email too long"))]
    pub email: String,
    /// The 6-digit verification code sent via email.
    #[validate(length(min = 6, max = 6, message = "Code must be 6 digits"))]
    pub code: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetVerifyResponse {
    /// The reset token (verification session ID) to use when setting the new password.
    pub reset_token: String,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetCompleteRequest {
    /// The reset token obtained from the verify step.
    #[validate(length(min = 1, max = 100, message = "Invalid reset token"))]
    pub reset_token: String,
    /// The new password.
    #[validate(length(min = 1, max = 128, message = "New password required"))]
    pub new_password: String,
}
