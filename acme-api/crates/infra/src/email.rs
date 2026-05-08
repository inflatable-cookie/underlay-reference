//! Email system factory functions.
//!
//! Creates EmailManager and EmailTemplateEngine based on EmailConfig.

use std::path::Path;
use std::sync::Arc;

use underlay_email::{
    EmailAdapter, EmailAddress, EmailContext, EmailManager, EmailTemplateEngine, NoopAdapter,
    SmtpAdapter, SmtpConfig, TlsMode,
};

use crate::config::{EmailAdapterType, EmailConfig};

/// Error type for email system initialization.
#[derive(Debug)]
pub struct EmailInitError(pub String);

impl std::fmt::Display for EmailInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "email initialization error: {}", self.0)
    }
}

impl std::error::Error for EmailInitError {}

/// Create an EmailManager based on the provided configuration.
///
/// # Arguments
///
/// * `config` - Email configuration from AppConfig
/// # Returns
///
/// Returns an EmailManager configured with the appropriate adapter.
pub fn create_email_manager(config: &EmailConfig) -> Result<EmailManager, EmailInitError> {
    let default_from = EmailAddress::new(&config.default_from)
        .map_err(|e| EmailInitError(format!("invalid default_from address: {e}")))?;

    let adapter: Arc<dyn EmailAdapter> = match config.adapter {
        EmailAdapterType::Noop => Arc::new(NoopAdapter::new()),

        EmailAdapterType::Smtp => {
            let smtp_config = config.smtp.as_ref().ok_or_else(|| {
                EmailInitError("smtp adapter requires smtp config".to_string())
            })?;

            let tls_mode = match smtp_config.tls_mode.to_ascii_lowercase().as_str() {
                "required" => TlsMode::Required,
                "opportunistic" => TlsMode::Opportunistic,
                "none" => TlsMode::None,
                value => {
                    return Err(EmailInitError(format!(
                        "invalid SMTP_TLS value `{value}`; expected required, opportunistic, or none"
                    )));
                }
            };

            let smtp_config = SmtpConfig {
                host: smtp_config.host.clone(),
                port: smtp_config.port,
                username: smtp_config.username.clone(),
                password: smtp_config.password.clone(),
                tls_mode,
            };

            let smtp_adapter = SmtpAdapter::new(&smtp_config)
                .map_err(|e| EmailInitError(format!("failed to create SMTP adapter: {e}")))?;

            Arc::new(smtp_adapter)
        }

        EmailAdapterType::Ses => {
            return Err(EmailInitError(
                "SES adapter requires ses feature and async initialization".to_string(),
            ));
        }
    };

    Ok(EmailManager::new(adapter, default_from))
}

/// Create an EmailTemplateEngine from configuration.
///
/// # Arguments
///
/// * `config` - Email configuration from AppConfig
///
/// # Returns
///
/// Returns an EmailTemplateEngine loaded with templates from the configured directory.
pub fn create_template_engine(config: &EmailConfig) -> Result<EmailTemplateEngine, EmailInitError> {
    let templates_path = Path::new(&config.templates_dir);

    EmailTemplateEngine::new(templates_path)
        .map_err(|e| EmailInitError(format!("failed to load email templates: {e}")))
}

/// Create an EmailContext pre-populated with app-level variables.
///
/// # Arguments
///
/// * `config` - Email configuration from AppConfig
///
/// # Returns
///
/// Returns an EmailContext with app_name, app_url, support_email, and current_year set.
pub fn create_email_context(config: &EmailConfig) -> EmailContext {
    EmailContext::with_app_info(&config.app_name, &config.app_url, &config.support_email)
}
