use std::{env, fs, path::Path};

use serde::Deserialize;

// Re-export Environment from underlay-observability.
// This provides consistent environment handling across all Underlay apps.
pub use underlay_observability::Environment;

/// HTTP server configuration.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Address to bind the server socket to (e.g., "127.0.0.1", "0.0.0.0").
    pub bind_addr: String,
    /// Port to listen on.
    pub port: u16,
    /// Public hostname for constructing URLs (e.g., "localhost", "api.example.com").
    /// Used for things like blob storage URLs that need to be accessible from clients.
    pub public_host: String,
}

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Postgres connection string. Optional for now so that
    /// local development can run without a database; once the
    /// DB-backed repositories are in place this will become required.
    pub url: Option<String>,
}

/// Logging configuration (level, sinks, sampling, etc.).
///
/// For now this only captures the log level; wiring of `tracing`
/// and sinks happens separately.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

/// CORS and cookie configuration for browser clients.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins for CORS requests.
    /// In production, these should be the specific frontend domains.
    /// Empty means allow any origin (for local dev).
    pub allowed_origins: Vec<String>,
    /// Cookie domain for auth cookies (e.g., ".acme.example.com").
    /// If None, cookies are scoped to the current host only.
    pub cookie_domain: Option<String>,
    /// Whether cookies should have the Secure flag.
    /// Should be true in production (HTTPS only).
    pub cookie_secure: bool,
}

/// Email adapter type selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmailAdapterType {
    /// No-op adapter (for testing, emails are discarded).
    #[default]
    Noop,
    /// Development capture adapter (saves to database, optionally forwards whitelisted).
    DevCapture,
    /// SMTP adapter using lettre.
    Smtp,
    /// AWS SES adapter.
    Ses,
}

impl EmailAdapterType {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "dev_capture" | "devcapture" | "capture" => EmailAdapterType::DevCapture,
            "smtp" => EmailAdapterType::Smtp,
            "ses" | "aws_ses" => EmailAdapterType::Ses,
            "noop" | "none" | "" => EmailAdapterType::Noop,
            _ => EmailAdapterType::Noop,
        }
    }
}

impl std::str::FromStr for EmailAdapterType {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(value))
    }
}

/// SMTP configuration for the SMTP email adapter.
#[derive(Debug, Clone)]
pub struct SmtpEmailConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    /// TLS mode: "required", "opportunistic", or "none"
    pub tls_mode: String,
}

/// AWS SES configuration for the SES email adapter.
#[derive(Debug, Clone)]
pub struct SesEmailConfig {
    pub region: String,
    pub configuration_set: Option<String>,
}

/// Development capture configuration.
#[derive(Debug, Clone)]
pub struct DevCaptureEmailConfig {
    /// Whitelist of email addresses that should also be delivered via fallback adapter.
    pub whitelist: Vec<String>,
    /// Fallback adapter type for whitelisted addresses.
    pub fallback_adapter: Option<EmailAdapterType>,
}

/// Email system configuration.
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// Which email adapter to use.
    pub adapter: EmailAdapterType,
    /// Default "from" address for all emails.
    pub default_from: String,
    /// Application name for email templates.
    pub app_name: String,
    /// Application URL for email templates.
    pub app_url: String,
    /// Support email address for email templates.
    pub support_email: String,
    /// Path to email templates directory.
    pub templates_dir: String,
    /// SMTP configuration (when adapter = smtp).
    pub smtp: Option<SmtpEmailConfig>,
    /// AWS SES configuration (when adapter = ses).
    pub ses: Option<SesEmailConfig>,
    /// Dev capture configuration (when adapter = dev_capture).
    pub dev_capture: Option<DevCaptureEmailConfig>,
}

#[derive(Debug, Clone)]
pub struct AppBehaviorConfig {
    pub auth: AuthBehaviorDefaults,
}

#[derive(Debug, Clone)]
pub struct AuthBehaviorDefaults {
    pub argon2_memory_kb: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
}

impl Default for AppBehaviorConfig {
    fn default() -> Self {
        Self {
            auth: AuthBehaviorDefaults {
                argon2_memory_kb: 131072,
                argon2_iterations: 4,
                argon2_parallelism: 4,
            },
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileBehaviorConfig {
    auth: Option<FileAuthBehaviorDefaults>,
}

#[derive(Debug, Default, Deserialize)]
struct FileAuthBehaviorDefaults {
    argon2_memory_kb: Option<u32>,
    argon2_iterations: Option<u32>,
    argon2_parallelism: Option<u32>,
}

impl AppBehaviorConfig {
    pub fn load() -> Self {
        let mut behavior = Self::default();
        let config_path = Path::new("config/default.toml");

        let raw = match fs::read_to_string(config_path) {
            Ok(contents) => contents,
            Err(_) => return behavior,
        };

        let parsed: FileBehaviorConfig = match toml::from_str(&raw) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!(
                    "warning: failed to parse {}: {}. Falling back to built-in behavior defaults.",
                    config_path.display(),
                    err
                );
                return behavior;
            }
        };

        if let Some(auth) = parsed.auth {
            if let Some(v) = auth.argon2_memory_kb {
                behavior.auth.argon2_memory_kb = v;
            }
            if let Some(v) = auth.argon2_iterations {
                behavior.auth.argon2_iterations = v;
            }
            if let Some(v) = auth.argon2_parallelism {
                behavior.auth.argon2_parallelism = v;
            }
        }

        behavior
    }
}

fn env_behavior_override(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(value) => {
            eprintln!(
                "warning: {key} is deprecated for app-behavior config; move it to config/default.toml"
            );
            Some(value)
        }
        Err(_) => None,
    }
}

/// Top-level application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub env: Environment,
    pub http: HttpConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub cors: CorsConfig,
    pub email: EmailConfig,
    pub behavior: AppBehaviorConfig,
}

impl AppConfig {
    /// Load configuration from the environment, applying sensible defaults.
    pub fn from_env() -> Self {
        // Load variables from a local `.env` file if present.
        let _ = dotenvy::dotenv();

        let mut behavior = AppBehaviorConfig::load();

        // Environment
        let env_str = env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());
        let env = Environment::parse(&env_str);

        // Port
        let port = env::var("PORT")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok())
            .unwrap_or(3000);

        // Bind address (must be a valid IP for socket binding)
        let bind_addr = env::var("HOST").unwrap_or_else(|_| {
            let should_bind_publicly =
                !matches!(env, Environment::Local | Environment::Test) || env::var("PORT").is_ok();

            if should_bind_publicly {
                "0.0.0.0".to_string()
            } else {
                "127.0.0.1".to_string()
            }
        });

        // Public hostname for URLs (defaults to localhost for local/dev/test)
        let public_host = env::var("PUBLIC_HOST").unwrap_or_else(|_| {
            if matches!(
                env,
                Environment::Local | Environment::Dev | Environment::Test
            ) {
                "localhost".to_string()
            } else {
                bind_addr.clone()
            }
        });

        // Log level (RUST_LOG is also commonly used but handled by tracing directly)
        let logging_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        // Database
        let database_url = env::var("DATABASE_URL").ok();

        // CORS configuration
        let allowed_origins: Vec<String> = env::var("CORS_ORIGINS")
            .ok()
            .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
            .unwrap_or_default();

        // Optional domain for cookies
        let cookie_domain = env::var("COOKIE_DOMAIN").ok();

        // Cookie secure flag: true in production, false in local/dev unless explicitly set
        let cookie_secure = env::var("COOKIE_SECURE")
            .ok()
            .map(|s| s.to_lowercase() == "true" || s == "1")
            .unwrap_or_else(|| !matches!(env, Environment::Local | Environment::Dev));

        // Email configuration
        let email_adapter_str = env::var("EMAIL_ADAPTER").unwrap_or_else(|_| "noop".to_string());
        let email_adapter = EmailAdapterType::parse(&email_adapter_str);

        // Email branding
        let default_from = env::var("EMAIL_DEFAULT_FROM")
            .unwrap_or_else(|_| "noreply@acme.example.com".to_string());
        let app_name = env::var("EMAIL_APP_NAME").unwrap_or_else(|_| "Acme".to_string());
        let app_url =
            env::var("EMAIL_APP_URL").unwrap_or_else(|_| "https://acme.example.com".to_string());
        let support_email =
            env::var("EMAIL_SUPPORT").unwrap_or_else(|_| "support@acme.example.com".to_string());
        let templates_dir =
            env::var("EMAIL_TEMPLATES_DIR").unwrap_or_else(|_| "templates/emails".to_string());

        // SMTP config (when adapter=smtp)
        let smtp_config = if email_adapter == EmailAdapterType::Smtp {
            Some(SmtpEmailConfig {
                host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
                port: env::var("SMTP_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(587),
                username: env::var("SMTP_USERNAME").ok(),
                password: env::var("SMTP_PASSWORD").ok(),
                tls_mode: env::var("SMTP_TLS").unwrap_or_else(|_| "opportunistic".to_string()),
            })
        } else {
            None
        };

        // SES config (when adapter=ses)
        let ses_config = if email_adapter == EmailAdapterType::Ses {
            Some(SesEmailConfig {
                region: env::var("SES_REGION").unwrap_or_else(|_| "eu-west-1".to_string()),
                configuration_set: env::var("SES_CONFIGURATION_SET").ok(),
            })
        } else {
            None
        };

        // Dev capture config (when adapter=dev_capture)
        let dev_capture_config = if email_adapter == EmailAdapterType::DevCapture {
            let whitelist: Vec<String> = env::var("EMAIL_WHITELIST")
                .ok()
                .map(|s| s.split(',').map(|e| e.trim().to_string()).collect())
                .unwrap_or_default();

            let fallback_adapter = env::var("EMAIL_FALLBACK_ADAPTER")
                .ok()
                .map(|s| EmailAdapterType::parse(&s))
                .filter(|a| *a != EmailAdapterType::Noop && *a != EmailAdapterType::DevCapture);

            Some(DevCaptureEmailConfig {
                whitelist,
                fallback_adapter,
            })
        } else {
            None
        };

        let email = EmailConfig {
            adapter: email_adapter,
            default_from,
            app_name,
            app_url,
            support_email,
            templates_dir,
            smtp: smtp_config,
            ses: ses_config,
            dev_capture: dev_capture_config,
        };

        let argon2_memory_kb: u32 = env_behavior_override("ARGON2_MEMORY_KB")
            .and_then(|v| v.parse().ok())
            .unwrap_or(behavior.auth.argon2_memory_kb);
        let argon2_iterations: u32 = env_behavior_override("ARGON2_ITERATIONS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(behavior.auth.argon2_iterations);
        let argon2_parallelism: u32 = env_behavior_override("ARGON2_PARALLELISM")
            .and_then(|v| v.parse().ok())
            .unwrap_or(behavior.auth.argon2_parallelism);

        behavior.auth.argon2_memory_kb = argon2_memory_kb;
        behavior.auth.argon2_iterations = argon2_iterations;
        behavior.auth.argon2_parallelism = argon2_parallelism;

        AppConfig {
            env,
            http: HttpConfig {
                bind_addr,
                port,
                public_host,
            },
            database: DatabaseConfig { url: database_url },
            logging: LoggingConfig {
                level: logging_level,
            },
            cors: CorsConfig {
                allowed_origins,
                cookie_domain,
                cookie_secure,
            },
            email,
            behavior,
        }
    }
}

/// Emit a redacted startup snapshot for effective configuration diagnostics.
pub fn log_effective_config(config: &AppConfig) {
    tracing::info!(
        env = ?config.env,
        bind_addr = %config.http.bind_addr,
        port = config.http.port,
        public_host = %config.http.public_host,
        log_level = %config.logging.level,
        db_configured = config.database.url.is_some(),
        cors_origins = config.cors.allowed_origins.len(),
        cookie_secure = config.cors.cookie_secure,
        email_adapter = ?config.email.adapter,
        email_templates_dir = %config.email.templates_dir,
        argon2_memory_kb = config.behavior.auth.argon2_memory_kb,
        argon2_iterations = config.behavior.auth.argon2_iterations,
        argon2_parallelism = config.behavior.auth.argon2_parallelism,
        "effective configuration loaded"
    );
}
