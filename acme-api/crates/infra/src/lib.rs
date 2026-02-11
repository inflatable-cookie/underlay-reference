//! Cross-cutting infrastructure (config, logging).

mod config;
pub mod email;
pub mod encryption;
pub mod network;

pub use config::{
    AppBehaviorConfig, AppConfig, AuthBehaviorDefaults, CorsConfig, DatabaseConfig,
    DevCaptureEmailConfig, EmailAdapterType, EmailConfig, Environment, HttpConfig, LoggingConfig,
    SesEmailConfig, SmtpEmailConfig,
};
pub use email::{
    create_email_context, create_email_manager, create_template_engine, EmailInitError,
};
pub use encryption::{generate_encryption_key, EncryptionError, EncryptionService};
pub use network::{extract_client_ip, TrustedProxyConfig};

/// Initialise structured logging and tracing subscribers.
///
/// - In `Local` / `Dev`, logs are emitted as pretty text for readability.
/// - In `Staging` / `Prod` / `Test`, logs are emitted as JSON.
/// - `RUST_LOG` can override the default level from config.
///
/// This is a thin wrapper around `underlay_observability::init_tracing_for_env`.
pub fn init_tracing(config: &AppConfig) {
    underlay_observability::init_tracing_for_env(config.env, &config.logging.level);
}

/// Initialise tracing with defaults (for backwards compatibility).
///
/// Uses Local environment with "info" level. Prefer `init_tracing(&config)`.
pub fn init_tracing_default() {
    underlay_observability::init_tracing_for_env(Environment::Local, "info");
}
