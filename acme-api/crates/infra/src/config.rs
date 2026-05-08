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
    /// Public URL scheme for constructing externally reachable URLs.
    pub public_scheme: String,
    /// Public hostname for constructing URLs (e.g., "localhost", "api.example.com").
    /// Used for things like blob storage URLs that need to be accessible from clients.
    pub public_host: String,
    /// Optional externally visible port for public URLs. When omitted, standard scheme ports are assumed.
    pub public_port: Option<u16>,
}

impl HttpConfig {
    pub fn public_origin(&self) -> String {
        match self.public_port {
            Some(port)
                if !(self.public_scheme == "https" && port == 443)
                    && !(self.public_scheme == "http" && port == 80) =>
            {
                format!("{}://{}:{}", self.public_scheme, self.public_host, port)
            }
            _ => format!("{}://{}", self.public_scheme, self.public_host),
        }
    }
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
    /// SMTP adapter using lettre.
    Smtp,
    /// AWS SES adapter.
    Ses,
}

impl EmailAdapterType {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            // Legacy local-dev alias. Mailpit now replaces DB-backed email capture.
            "dev_capture" | "devcapture" | "capture" => EmailAdapterType::Smtp,
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
}

#[derive(Debug, Clone)]
pub struct AppBehaviorConfig {
    pub email: EmailBehaviorDefaults,
    pub auth: AuthBehaviorDefaults,
}

#[derive(Debug, Clone)]
pub struct EmailBehaviorDefaults {
    pub default_from: String,
    pub app_name: String,
    pub app_url: String,
    pub support_email: String,
    pub templates_dir: String,
}

#[derive(Debug, Clone)]
pub struct AuthBehaviorDefaults {
    pub jwt_access_token_lifetime_minutes: i64,
    pub jwt_refresh_token_lifetime_days: i64,
    pub jwt_issuer: String,
    pub jwt_audience: Option<String>,
    pub jwt_leeway_seconds: u64,
    pub totp_issuer: String,
    pub password_min_length: usize,
    pub login_rate_limit_per_hour: u32,
    pub register_rate_limit_per_hour: u32,
    pub password_change_rate_limit_per_hour: u32,
    pub passkey_register_rate_limit_per_hour: u32,
    pub passkey_login_rate_limit_per_hour: u32,
    pub refresh_rate_limit_per_hour: u32,
    pub max_totp_attempts: u32,
    pub max_email_code_attempts: u32,
    pub max_backup_code_attempts: u32,
    pub totp_state_timeout_secs: u64,
    pub email_state_timeout_secs: u64,
    pub verification_session_timeout_secs: u64,
    pub rate_limit_cleanup_interval_secs: u64,
    pub absolute_session_timeout_days: u64,
    pub rate_limit_retry_after_short_secs: u64,
    pub rate_limit_retry_after_long_secs: u64,
    pub max_failed_logins: u32,
    pub lockout_duration_secs: u64,
    pub security_alert_window_secs: u64,
    pub security_alert_cooldown_secs: u64,
    pub security_alert_failed_attempts_threshold: u32,
    pub security_alert_distinct_users_threshold: u32,
    pub security_alert_lockouts_threshold: u32,
    pub email_code_expiry_secs: u64,
    pub max_email_codes_per_hour: u32,
    pub webauthn_rp_id: String,
    pub webauthn_rp_origin: String,
    pub webauthn_rp_name: String,
    pub argon2_memory_kb: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
}

impl Default for AppBehaviorConfig {
    fn default() -> Self {
        Self {
            email: EmailBehaviorDefaults {
                default_from: "noreply@acme.example.com".to_string(),
                app_name: "Acme".to_string(),
                app_url: "https://acme.example.com".to_string(),
                support_email: "support@acme.example.com".to_string(),
                templates_dir: "templates/emails".to_string(),
            },
            auth: AuthBehaviorDefaults {
                jwt_access_token_lifetime_minutes: 15,
                jwt_refresh_token_lifetime_days: 30,
                jwt_issuer: "acme".to_string(),
                jwt_audience: Some("acme-api".to_string()),
                jwt_leeway_seconds: 30,
                totp_issuer: "Acme".to_string(),
                password_min_length: 12,
                login_rate_limit_per_hour: 10,
                register_rate_limit_per_hour: 5,
                password_change_rate_limit_per_hour: 5,
                passkey_register_rate_limit_per_hour: 5,
                passkey_login_rate_limit_per_hour: 10,
                refresh_rate_limit_per_hour: 60,
                max_totp_attempts: 5,
                max_email_code_attempts: 5,
                max_backup_code_attempts: 5,
                totp_state_timeout_secs: 300,
                email_state_timeout_secs: 600,
                verification_session_timeout_secs: 300,
                rate_limit_cleanup_interval_secs: 300,
                absolute_session_timeout_days: 30,
                rate_limit_retry_after_short_secs: 60,
                rate_limit_retry_after_long_secs: 300,
                max_failed_logins: 5,
                lockout_duration_secs: 900,
                security_alert_window_secs: 600,
                security_alert_cooldown_secs: 1800,
                security_alert_failed_attempts_threshold: 20,
                security_alert_distinct_users_threshold: 5,
                security_alert_lockouts_threshold: 3,
                email_code_expiry_secs: 600,
                max_email_codes_per_hour: 5,
                webauthn_rp_id: "localhost".to_string(),
                webauthn_rp_origin: "http://localhost:4174".to_string(),
                webauthn_rp_name: "Acme".to_string(),
                argon2_memory_kb: 131072,
                argon2_iterations: 4,
                argon2_parallelism: 4,
            },
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileBehaviorConfig {
    email: Option<FileEmailBehaviorDefaults>,
    auth: Option<FileAuthBehaviorDefaults>,
}

#[derive(Debug, Default, Deserialize)]
struct FileEmailBehaviorDefaults {
    default_from: Option<String>,
    app_name: Option<String>,
    app_url: Option<String>,
    support_email: Option<String>,
    templates_dir: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FileAuthBehaviorDefaults {
    jwt_access_token_lifetime_minutes: Option<i64>,
    jwt_refresh_token_lifetime_days: Option<i64>,
    jwt_issuer: Option<String>,
    jwt_audience: Option<String>,
    jwt_leeway_seconds: Option<u64>,
    totp_issuer: Option<String>,
    password_min_length: Option<usize>,
    login_rate_limit_per_hour: Option<u32>,
    register_rate_limit_per_hour: Option<u32>,
    password_change_rate_limit_per_hour: Option<u32>,
    passkey_register_rate_limit_per_hour: Option<u32>,
    passkey_login_rate_limit_per_hour: Option<u32>,
    refresh_rate_limit_per_hour: Option<u32>,
    max_totp_attempts: Option<u32>,
    max_email_code_attempts: Option<u32>,
    max_backup_code_attempts: Option<u32>,
    totp_state_timeout_secs: Option<u64>,
    email_state_timeout_secs: Option<u64>,
    verification_session_timeout_secs: Option<u64>,
    rate_limit_cleanup_interval_secs: Option<u64>,
    absolute_session_timeout_days: Option<u64>,
    rate_limit_retry_after_short_secs: Option<u64>,
    rate_limit_retry_after_long_secs: Option<u64>,
    max_failed_logins: Option<u32>,
    lockout_duration_secs: Option<u64>,
    security_alert_window_secs: Option<u64>,
    security_alert_cooldown_secs: Option<u64>,
    security_alert_failed_attempts_threshold: Option<u32>,
    security_alert_distinct_users_threshold: Option<u32>,
    security_alert_lockouts_threshold: Option<u32>,
    email_code_expiry_secs: Option<u64>,
    max_email_codes_per_hour: Option<u32>,
    webauthn_rp_id: Option<String>,
    webauthn_rp_origin: Option<String>,
    webauthn_rp_name: Option<String>,
    argon2_memory_kb: Option<u32>,
    argon2_iterations: Option<u32>,
    argon2_parallelism: Option<u32>,
}

impl AppBehaviorConfig {
    pub fn load() -> Self {
        let mut behavior = Self::default();
        Self::merge_file(&mut behavior, Path::new("config/default.toml"));
        Self::merge_file(&mut behavior, Path::new("config/local.toml"));

        behavior
    }

    fn merge_file(behavior: &mut Self, config_path: &Path) {
        let raw = match fs::read_to_string(config_path) {
            Ok(contents) => contents,
            Err(_) => return,
        };

        let parsed: FileBehaviorConfig = match toml::from_str(&raw) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!(
                    "warning: failed to parse {}: {}. Skipping this config layer.",
                    config_path.display(),
                    err
                );
                return;
            }
        };

        if let Some(email) = parsed.email {
            if let Some(v) = email.default_from {
                behavior.email.default_from = v;
            }
            if let Some(v) = email.app_name {
                behavior.email.app_name = v;
            }
            if let Some(v) = email.app_url {
                behavior.email.app_url = v;
            }
            if let Some(v) = email.support_email {
                behavior.email.support_email = v;
            }
            if let Some(v) = email.templates_dir {
                behavior.email.templates_dir = v;
            }
        }

        if let Some(auth) = parsed.auth {
            if let Some(v) = auth.jwt_access_token_lifetime_minutes {
                behavior.auth.jwt_access_token_lifetime_minutes = v;
            }
            if let Some(v) = auth.jwt_refresh_token_lifetime_days {
                behavior.auth.jwt_refresh_token_lifetime_days = v;
            }
            if let Some(v) = auth.jwt_issuer {
                behavior.auth.jwt_issuer = v;
            }
            if let Some(v) = auth.jwt_audience {
                behavior.auth.jwt_audience = normalize_optional_string(Some(v));
            }
            if let Some(v) = auth.jwt_leeway_seconds {
                behavior.auth.jwt_leeway_seconds = v;
            }
            if let Some(v) = auth.totp_issuer {
                behavior.auth.totp_issuer = v;
            }
            if let Some(v) = auth.password_min_length {
                behavior.auth.password_min_length = v;
            }
            if let Some(v) = auth.login_rate_limit_per_hour {
                behavior.auth.login_rate_limit_per_hour = v;
            }
            if let Some(v) = auth.register_rate_limit_per_hour {
                behavior.auth.register_rate_limit_per_hour = v;
            }
            if let Some(v) = auth.password_change_rate_limit_per_hour {
                behavior.auth.password_change_rate_limit_per_hour = v;
            }
            if let Some(v) = auth.passkey_register_rate_limit_per_hour {
                behavior.auth.passkey_register_rate_limit_per_hour = v;
            }
            if let Some(v) = auth.passkey_login_rate_limit_per_hour {
                behavior.auth.passkey_login_rate_limit_per_hour = v;
            }
            if let Some(v) = auth.refresh_rate_limit_per_hour {
                behavior.auth.refresh_rate_limit_per_hour = v;
            }
            if let Some(v) = auth.max_totp_attempts {
                behavior.auth.max_totp_attempts = v;
            }
            if let Some(v) = auth.max_email_code_attempts {
                behavior.auth.max_email_code_attempts = v;
            }
            if let Some(v) = auth.max_backup_code_attempts {
                behavior.auth.max_backup_code_attempts = v;
            }
            if let Some(v) = auth.totp_state_timeout_secs {
                behavior.auth.totp_state_timeout_secs = v;
            }
            if let Some(v) = auth.email_state_timeout_secs {
                behavior.auth.email_state_timeout_secs = v;
            }
            if let Some(v) = auth.verification_session_timeout_secs {
                behavior.auth.verification_session_timeout_secs = v;
            }
            if let Some(v) = auth.rate_limit_cleanup_interval_secs {
                behavior.auth.rate_limit_cleanup_interval_secs = v;
            }
            if let Some(v) = auth.absolute_session_timeout_days {
                behavior.auth.absolute_session_timeout_days = v;
            }
            if let Some(v) = auth.rate_limit_retry_after_short_secs {
                behavior.auth.rate_limit_retry_after_short_secs = v;
            }
            if let Some(v) = auth.rate_limit_retry_after_long_secs {
                behavior.auth.rate_limit_retry_after_long_secs = v;
            }
            if let Some(v) = auth.max_failed_logins {
                behavior.auth.max_failed_logins = v;
            }
            if let Some(v) = auth.lockout_duration_secs {
                behavior.auth.lockout_duration_secs = v;
            }
            if let Some(v) = auth.security_alert_window_secs {
                behavior.auth.security_alert_window_secs = v;
            }
            if let Some(v) = auth.security_alert_cooldown_secs {
                behavior.auth.security_alert_cooldown_secs = v;
            }
            if let Some(v) = auth.security_alert_failed_attempts_threshold {
                behavior.auth.security_alert_failed_attempts_threshold = v;
            }
            if let Some(v) = auth.security_alert_distinct_users_threshold {
                behavior.auth.security_alert_distinct_users_threshold = v;
            }
            if let Some(v) = auth.security_alert_lockouts_threshold {
                behavior.auth.security_alert_lockouts_threshold = v;
            }
            if let Some(v) = auth.email_code_expiry_secs {
                behavior.auth.email_code_expiry_secs = v;
            }
            if let Some(v) = auth.max_email_codes_per_hour {
                behavior.auth.max_email_codes_per_hour = v;
            }
            if let Some(v) = auth.webauthn_rp_id {
                behavior.auth.webauthn_rp_id = v;
            }
            if let Some(v) = auth.webauthn_rp_origin {
                behavior.auth.webauthn_rp_origin = v;
            }
            if let Some(v) = auth.webauthn_rp_name {
                behavior.auth.webauthn_rp_name = v;
            }
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
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
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

const LEGACY_BEHAVIOR_ENV_KEYS: [(&str, &str); 16] = [
    ("EMAIL_DEFAULT_FROM", "behavior.email.default_from"),
    ("EMAIL_APP_NAME", "behavior.email.app_name"),
    ("EMAIL_APP_URL", "behavior.email.app_url"),
    ("EMAIL_SUPPORT", "behavior.email.support_email"),
    ("EMAIL_TEMPLATES_DIR", "behavior.email.templates_dir"),
    (
        "AUTH_ACCESS_TOKEN_LIFETIME_MINUTES",
        "behavior.auth.jwt_access_token_lifetime_minutes",
    ),
    (
        "AUTH_REFRESH_TOKEN_LIFETIME_DAYS",
        "behavior.auth.jwt_refresh_token_lifetime_days",
    ),
    ("AUTH_JWT_ISSUER", "behavior.auth.jwt_issuer"),
    ("AUTH_JWT_AUDIENCE", "behavior.auth.jwt_audience"),
    ("AUTH_JWT_LEEWAY_SECONDS", "behavior.auth.jwt_leeway_seconds"),
    ("WEBAUTHN_RP_ID", "behavior.auth.webauthn_rp_id"),
    ("WEBAUTHN_RP_ORIGIN", "behavior.auth.webauthn_rp_origin"),
    ("WEBAUTHN_RP_NAME", "behavior.auth.webauthn_rp_name"),
    ("ARGON2_MEMORY_KB", "behavior.auth.argon2_memory_kb"),
    ("ARGON2_ITERATIONS", "behavior.auth.argon2_iterations"),
    ("ARGON2_PARALLELISM", "behavior.auth.argon2_parallelism"),
];

fn collect_set_legacy_behavior_env_keys() -> Vec<(&'static str, &'static str)> {
    LEGACY_BEHAVIOR_ENV_KEYS
        .iter()
        .copied()
        .filter(|(key, _)| {
            env::var(key)
                .ok()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        })
        .collect()
}

fn warn_legacy_behavior_env_keys() {
    for (legacy_key, replacement_field) in collect_set_legacy_behavior_env_keys() {
        tracing::warn!(
            legacy_key,
            replacement_field,
            "legacy behavior env key is set but ignored; use typed config field instead"
        );
    }
}

impl AppConfig {
    /// Load configuration from the environment, applying sensible defaults.
    pub fn from_env() -> Self {
        // Load variables from a local `.env` file if present.
        let _ = dotenvy::dotenv();
        warn_legacy_behavior_env_keys();

        let behavior = AppBehaviorConfig::load();

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
        let public_scheme = env::var("PUBLIC_SCHEME").unwrap_or_else(|_| {
            if public_host == "localhost"
                || public_host == "127.0.0.1"
                || public_host == "0.0.0.0"
            {
                "http".to_string()
            } else {
                "https".to_string()
            }
        });
        let public_port = env::var("PUBLIC_PORT")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok());

        // Log level (RUST_LOG is also commonly used but handled by tracing directly)
        let logging_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        // Database
        let database_url = env::var("DATABASE_URL")
            .or_else(|_| env::var("ACME_DATABASE_URL"))
            .ok();

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

        // Email branding from typed behavior config.
        let default_from = behavior.email.default_from.clone();
        let app_name = behavior.email.app_name.clone();
        let app_url = behavior.email.app_url.clone();
        let support_email = behavior.email.support_email.clone();
        let templates_dir = behavior.email.templates_dir.clone();

        // SMTP config (when adapter=smtp)
        let smtp_config = if email_adapter == EmailAdapterType::Smtp {
            Some(SmtpEmailConfig {
                host: env::var("SMTP_HOST").unwrap_or_else(|_| {
                    if matches!(env, Environment::Local | Environment::Dev | Environment::Test) {
                        "smtp.acme.test".to_string()
                    } else {
                        "localhost".to_string()
                    }
                }),
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

        let email = EmailConfig {
            adapter: email_adapter,
            default_from,
            app_name,
            app_url,
            support_email,
            templates_dir,
            smtp: smtp_config,
            ses: ses_config,
        };

        AppConfig {
            env,
            http: HttpConfig {
                bind_addr,
                port,
                public_scheme,
                public_host,
                public_port,
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

#[cfg(test)]
#[path = "tests/config_tests.rs"]
mod tests;
