use std::env;

// Re-export Environment from underlay-observability.
// This provides consistent environment handling across all Underlay apps.
pub use underlay_observability::Environment;

/// HTTP server configuration.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub bind_addr: String,
    pub port: u16,
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

/// Top-level application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub env: Environment,
    pub http: HttpConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub cors: CorsConfig,
    pub email: EmailConfig,
}

impl AppConfig {
    /// Load configuration from the environment, applying sensible defaults.
    pub fn from_env() -> Self {
        // Load variables from a local `.env` file if present.
        let _ = dotenvy::dotenv();

        // Environment
        let env_str = env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());
        let env = Environment::parse(&env_str);

        // Port
        let port = env::var("PORT")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok())
            .unwrap_or(3000);

        // Bind address
        let bind_addr = env::var("HOST").unwrap_or_else(|_| {
            let should_bind_publicly =
                !matches!(env, Environment::Local | Environment::Test) || env::var("PORT").is_ok();

            if should_bind_publicly {
                "0.0.0.0".to_string()
            } else {
                "localhost".to_string()
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

        AppConfig {
            env,
            http: HttpConfig { bind_addr, port },
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
        }
    }
}
