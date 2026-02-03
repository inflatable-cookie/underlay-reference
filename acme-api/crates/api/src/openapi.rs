//! OpenAPI specification module.
//!
//! Generates OpenAPI documentation for the Acme API.
//!
//! The OpenAPI spec is built incrementally. Not all handlers have full annotations
//! yet, but the spec documents the available endpoints and their schemas.

use utoipa::OpenApi;

use crate::dto::account::{UpdateProfileRequest, UserProfileDto};
use crate::dto::auth::{
    AuthSessionDto, AuthUserDto, ChangePasswordRequest, ChangePasswordWithVerificationRequest,
    EmailTotpPurposeDto, EmailTotpRequestRequest, EmailTotpRequestResponse, EmailTotpVerifyRequest,
    EmailTotpVerifyResponse, LoginFinishRequest, LoginRequest, LoginStartRequest,
    LoginStartResponse, PasskeyRenameRequest, PasskeyVerifyStartRequest,
    PasswordResetCompleteRequest, PasswordResetRequestRequest, PasswordResetVerifyRequest,
    PasswordResetVerifyResponse, RefreshRequest, RegisterRequest, TotpStatusResponse,
    TotpVerifyRequest, TwoFactorStatusResponse,
};

/// OpenAPI documentation for the Acme API.
///
/// Use with `utoipa_swagger_ui::SwaggerUi` to serve the interactive documentation.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Acme API",
        version = "1.0.0",
        description = "Underlay Reference Implementation API - A comprehensive example of building modern APIs with the Underlay framework.",
        contact(name = "Acme Team", email = "support@acme.example.com"),
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
    components(schemas(
        // Auth DTOs
        RegisterRequest,
        LoginRequest,
        LoginStartRequest,
        LoginFinishRequest,
        RefreshRequest,
        ChangePasswordRequest,
        PasskeyRenameRequest,
        AuthUserDto,
        AuthSessionDto,
        LoginStartResponse,
        TotpStatusResponse,
        TwoFactorStatusResponse,
        EmailTotpPurposeDto,
        EmailTotpRequestRequest,
        EmailTotpRequestResponse,
        EmailTotpVerifyRequest,
        EmailTotpVerifyResponse,
        TotpVerifyRequest,
        PasskeyVerifyStartRequest,
        ChangePasswordWithVerificationRequest,
        PasswordResetRequestRequest,
        PasswordResetVerifyRequest,
        PasswordResetVerifyResponse,
        PasswordResetCompleteRequest,
        // Account DTOs
        UserProfileDto,
        UpdateProfileRequest,
    )),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Authentication", description = "User authentication including login, registration, and session management"),
        (name = "Two-Factor Auth", description = "TOTP, passkey, and email verification for 2FA"),
        (name = "Account", description = "User profile and preferences management"),
        (name = "Projects", description = "User project management"),
        (name = "Admin - Dashboard", description = "Admin dashboard statistics"),
        (name = "Admin - Users", description = "User management and session control"),
        (name = "Admin - Activity", description = "Activity and audit log"),
        (name = "Admin - Categories", description = "Content category management"),
        (name = "Admin - Projects", description = "Admin project management"),
        (name = "Admin - Tasks", description = "Admin task management"),
        (name = "Admin - Media", description = "Media library management"),
        (name = "Admin - Jobs", description = "Background job monitoring"),
    ),
    servers(
        (url = "/", description = "Local server")
    )
)]
pub struct ApiDoc;
