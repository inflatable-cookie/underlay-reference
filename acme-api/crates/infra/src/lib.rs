//! Cross-cutting infrastructure (config, logging).

mod config;
pub mod email;

pub use config::{
    AppConfig, CorsConfig, DatabaseConfig, DevCaptureEmailConfig, EmailAdapterType, EmailConfig,
    Environment, HttpConfig, LoggingConfig, SesEmailConfig, SmtpEmailConfig,
};
pub use email::{create_email_context, create_email_manager, create_template_engine, EmailInitError};

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}
