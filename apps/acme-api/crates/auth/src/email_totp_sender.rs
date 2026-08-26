//! Email sender implementation for email TOTP using Acme email infrastructure.

use std::sync::Arc;

use acme_infra::EmailConfig;
use async_trait::async_trait;
use underlay_auth_email_totp::{EmailTotpError, EmailTotpResult, EmailTotpSender};
use underlay_email::{EmailAddress, EmailContext, EmailManager, EmailTemplateEngine};

/// Acme implementation of email TOTP sender.
pub struct AcmeEmailTotpSender {
    email_manager: Arc<EmailManager>,
    email_templates: Arc<EmailTemplateEngine>,
    email_config: EmailConfig,
}

impl AcmeEmailTotpSender {
    /// Create a new email sender with the Acme email infrastructure.
    pub fn new(
        email_manager: Arc<EmailManager>,
        email_templates: Arc<EmailTemplateEngine>,
        email_config: EmailConfig,
    ) -> Self {
        Self {
            email_manager,
            email_templates,
            email_config,
        }
    }
}

#[async_trait]
impl EmailTotpSender for AcmeEmailTotpSender {
    async fn send_code(
        &self,
        to_email: &str,
        code: &str,
        purpose: &str,
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
        context.set("purpose", purpose);

        // Render the template
        let html_body = self
            .email_templates
            .render("auth/email_totp.html", &context)
            .map_err(|e| EmailTotpError::EmailSendFailed(format!("template error: {}", e)))?;

        // Build subject based on purpose
        let subject = match purpose {
            "login" => format!("Your {} login code", self.email_config.app_name),
            "password_change" => {
                format!("Your {} password change code", self.email_config.app_name)
            }
            "password_reset" => format!("Reset your {} password", self.email_config.app_name),
            _ => format!("Your {} verification code", self.email_config.app_name),
        };

        // Build plain text body
        let text_body = format!(
            "Your verification code is: {}\n\nThis code expires in {} minutes.\n\nIf you didn't request this code, you can safely ignore this email.",
            code, expiry_minutes
        );

        // Send the email
        let to = EmailAddress::new(to_email)
            .map_err(|e| EmailTotpError::EmailSendFailed(format!("invalid email: {}", e)))?;

        self.email_manager
            .send_email(to, &subject, &text_body, Some(html_body))
            .await
            .map_err(|e| EmailTotpError::EmailSendFailed(e.to_string()))?;

        Ok(())
    }
}
