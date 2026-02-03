//! Email system factory functions.
//!
//! Creates EmailManager and EmailTemplateEngine based on EmailConfig.

use std::path::Path;
use std::sync::Arc;

use underlay_email::{
    DevCaptureAdapter, DevCaptureConfig, EmailAdapter, EmailAddress, EmailContext, EmailManager,
    EmailStore, EmailTemplateEngine, NoopAdapter,
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
/// * `email_store` - For dev_capture adapter, the store to save captured emails
///
/// # Returns
///
/// Returns an EmailManager configured with the appropriate adapter.
pub fn create_email_manager<S>(
    config: &EmailConfig,
    email_store: Option<Arc<S>>,
) -> Result<EmailManager, EmailInitError>
where
    S: EmailStore + 'static,
{
    let default_from = EmailAddress::new(&config.default_from)
        .map_err(|e| EmailInitError(format!("invalid default_from address: {e}")))?;

    let adapter: Arc<dyn EmailAdapter> = match config.adapter {
        EmailAdapterType::Noop => Arc::new(NoopAdapter::new()),

        EmailAdapterType::DevCapture => {
            let store = email_store.ok_or_else(|| {
                EmailInitError("dev_capture adapter requires an email store".to_string())
            })?;

            let dev_config = config.dev_capture.as_ref().ok_or_else(|| {
                EmailInitError("dev_capture adapter requires dev_capture config".to_string())
            })?;

            // Build the underlay DevCaptureConfig
            let capture_config = DevCaptureConfig {
                whitelist: dev_config.whitelist.clone(),
                use_fallback: dev_config.fallback_adapter.is_some(),
            };

            // For now, we don't support fallback adapters in the factory
            let fallback: Option<Arc<dyn EmailAdapter>> = None;

            Arc::new(DevCaptureAdapter::new(store, capture_config, fallback))
        }

        EmailAdapterType::Smtp => {
            return Err(EmailInitError(
                "SMTP adapter requires smtp feature and create_smtp_email_manager".to_string(),
            ));
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
