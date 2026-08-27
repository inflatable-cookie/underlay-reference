use super::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn with_env_var(key: &str, value: Option<&str>) -> Option<String> {
    let previous = std::env::var(key).ok();
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
    previous
}

fn restore_env_var(key: &str, previous: Option<String>) {
    match previous {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "acme-infra-config-tests-{}-{}",
        std::process::id(),
        nanos
    ))
}

fn write_default_toml(base_dir: &Path, content: &str) {
    let config_dir = base_dir.join("config");
    std::fs::create_dir_all(&config_dir).expect("failed to create config dir");
    std::fs::write(config_dir.join("default.toml"), content).expect("failed to write default.toml");
}

#[test]
fn migrated_behavior_keys_use_typed_config_not_env_overrides() {
    let _lock = ENV_LOCK.lock().unwrap();

    let test_dir = unique_temp_dir();
    write_default_toml(
        &test_dir,
        r#"
[email]
default_from = "typed@example.com"
app_name = "Typed App"
app_url = "https://typed.example.com"
support_email = "support@typed.example.com"
templates_dir = "typed/templates"

[auth]
jwt_access_token_lifetime_minutes = 77
jwt_refresh_token_lifetime_days = 88
jwt_issuer = "typed-issuer"
jwt_audience = "typed-audience"
jwt_leeway_seconds = 99
webauthn_rp_id = "typed-rp-id"
webauthn_rp_origin = "https://typed-rp.example.com"
webauthn_rp_name = "Typed RP"
argon2_memory_kb = 11111
argon2_iterations = 12
argon2_parallelism = 13
"#,
    );

    let original_cwd = std::env::current_dir().expect("failed to get cwd");
    std::env::set_current_dir(&test_dir).expect("failed to set cwd");

    let env_overrides = [
        ("EMAIL_DEFAULT_FROM", "env@example.com"),
        ("EMAIL_APP_NAME", "Env App"),
        ("EMAIL_APP_URL", "https://env.example.com"),
        ("EMAIL_SUPPORT", "support@env.example.com"),
        ("EMAIL_TEMPLATES_DIR", "env/templates"),
        ("AUTH_ACCESS_TOKEN_LIFETIME_MINUTES", "15"),
        ("AUTH_REFRESH_TOKEN_LIFETIME_DAYS", "30"),
        ("AUTH_JWT_ISSUER", "env-issuer"),
        ("AUTH_JWT_AUDIENCE", "env-audience"),
        ("AUTH_JWT_LEEWAY_SECONDS", "30"),
        ("WEBAUTHN_RP_ID", "env-rp-id"),
        ("WEBAUTHN_RP_ORIGIN", "https://env-rp.example.com"),
        ("WEBAUTHN_RP_NAME", "Env RP"),
        ("ARGON2_MEMORY_KB", "22222"),
        ("ARGON2_ITERATIONS", "21"),
        ("ARGON2_PARALLELISM", "22"),
    ];
    let mut previous = Vec::new();
    for (key, value) in env_overrides {
        previous.push((key, with_env_var(key, Some(value))));
    }

    let config = AppConfig::from_env().expect("valid config stack");
    assert_eq!(config.email.default_from, "typed@example.com");
    assert_eq!(config.email.app_name, "Typed App");
    assert_eq!(config.email.app_url, "https://typed.example.com");
    assert_eq!(config.email.support_email, "support@typed.example.com");
    assert_eq!(config.email.templates_dir, "typed/templates");
    assert_eq!(config.behavior.auth.jwt_access_token_lifetime_minutes, 77);
    assert_eq!(config.behavior.auth.jwt_refresh_token_lifetime_days, 88);
    assert_eq!(config.behavior.auth.jwt_issuer, "typed-issuer");
    assert_eq!(
        config.behavior.auth.jwt_audience.as_deref(),
        Some("typed-audience")
    );
    assert_eq!(config.behavior.auth.jwt_leeway_seconds, 99);
    assert_eq!(config.behavior.auth.webauthn_rp_id, "typed-rp-id");
    assert_eq!(
        config.behavior.auth.webauthn_rp_origin,
        "https://typed-rp.example.com"
    );
    assert_eq!(config.behavior.auth.webauthn_rp_name, "Typed RP");
    assert_eq!(config.behavior.auth.argon2_memory_kb, 11111);
    assert_eq!(config.behavior.auth.argon2_iterations, 12);
    assert_eq!(config.behavior.auth.argon2_parallelism, 13);

    for (key, value) in previous {
        restore_env_var(key, value);
    }
    std::env::set_current_dir(original_cwd).expect("failed to restore cwd");
    std::fs::remove_dir_all(test_dir).expect("failed to remove temp dir");
}

#[test]
fn legacy_behavior_key_collection_detects_set_env_keys() {
    let _lock = ENV_LOCK.lock().unwrap();

    let prev_a = with_env_var("EMAIL_DEFAULT_FROM", Some("legacy@example.com"));
    let prev_b = with_env_var("AUTH_JWT_ISSUER", Some("legacy-issuer"));
    let prev_c = with_env_var("ARGON2_ITERATIONS", None);

    let keys = collect_set_legacy_behavior_env_keys();
    assert!(
        keys.iter().any(|(k, _)| *k == "EMAIL_DEFAULT_FROM"),
        "expected EMAIL_DEFAULT_FROM to be detected"
    );
    assert!(
        keys.iter().any(|(k, _)| *k == "AUTH_JWT_ISSUER"),
        "expected AUTH_JWT_ISSUER to be detected"
    );
    assert!(
        !keys.iter().any(|(k, _)| *k == "ARGON2_ITERATIONS"),
        "expected ARGON2_ITERATIONS to be absent when unset"
    );

    restore_env_var("EMAIL_DEFAULT_FROM", prev_a);
    restore_env_var("AUTH_JWT_ISSUER", prev_b);
    restore_env_var("ARGON2_ITERATIONS", prev_c);
}

// ============================================================================
// Settled startup-failure policy (g09.047)
// ============================================================================
//
// `local`, `effigy`, and `test` are the bounded non-deployed set. Everything
// else — including `dev`, and including any unrecognised name, which
// `Environment::parse` resolves to production — fails closed.

#[test]
fn only_local_effigy_and_test_may_warn_and_continue() {
    for environment in [Environment::Local, Environment::Effigy, Environment::Test] {
        assert_eq!(
            startup_posture(environment),
            StartupPosture::WarnAndContinue,
            "{environment:?} is a bounded non-deployed environment and may warn"
        );
    }
}

#[test]
fn deployed_environments_are_fatal() {
    for environment in [Environment::Dev, Environment::Staging, Environment::Prod] {
        assert_eq!(
            startup_posture(environment),
            StartupPosture::Fatal,
            "{environment:?} is deployed and must fail closed"
        );
    }
}

#[test]
fn dev_is_deployed_not_a_developer_environment() {
    // Regression guard: `is_development()` returns true for Dev, so deriving
    // the boundary from it would let a deployed `dev` runtime boot on code
    // defaults with insecure cookies.
    assert!(Environment::Dev.is_development());
    assert!(startup_posture(Environment::Dev).is_fatal());
}

#[test]
fn unrecognised_environment_name_fails_closed() {
    let parsed = Environment::parse("staging-2");
    assert_eq!(parsed, Environment::Prod);
    assert!(startup_posture(parsed).is_fatal());
}

#[test]
fn insecure_cookies_are_fatal_only_outside_the_non_deployed_set() {
    for environment in [Environment::Local, Environment::Effigy, Environment::Test] {
        assert!(
            enforce_cookie_secure(environment, false).is_ok(),
            "{environment:?} may serve a plaintext loopback stack"
        );
    }

    for environment in [Environment::Dev, Environment::Staging, Environment::Prod] {
        let err = enforce_cookie_secure(environment, false)
            .expect_err("COOKIE_SECURE=false must be fatal in a deployed environment");
        assert!(matches!(err, ConfigError::InsecureAuthCookies { .. }));
    }
}

#[test]
fn secure_cookies_are_always_accepted() {
    for environment in [
        Environment::Local,
        Environment::Effigy,
        Environment::Test,
        Environment::Dev,
        Environment::Staging,
        Environment::Prod,
    ] {
        assert!(enforce_cookie_secure(environment, true).is_ok());
    }
}

#[test]
fn csrf_disablement_is_rejected_by_every_deployed_environment() {
    for environment in [Environment::Local, Environment::Effigy, Environment::Test] {
        assert!(
            !resolve_csrf_protection(environment, false).expect("allowed in the non-deployed set"),
            "{environment:?} may disable CSRF"
        );
    }

    for environment in [Environment::Dev, Environment::Staging, Environment::Prod] {
        let err = resolve_csrf_protection(environment, false)
            .expect_err("CSRF disablement must be rejected in a deployed environment");
        assert!(matches!(err, ConfigError::CsrfDisabled { .. }));
    }
}

#[test]
fn csrf_stays_enabled_when_not_disabled() {
    for environment in [Environment::Local, Environment::Prod] {
        assert!(resolve_csrf_protection(environment, true).expect("enabled is always valid"));
    }
}

#[test]
fn csrf_protection_defaults_to_enabled_when_unset() {
    let _lock = ENV_LOCK.lock().unwrap();

    let previous = with_env_var("CSRF_PROTECTION", None);
    assert!(csrf_protection_requested(), "unset must mean protected");

    with_env_var("CSRF_PROTECTION", Some("nonsense"));
    assert!(
        !csrf_protection_requested(),
        "an explicit non-truthy value is a disablement request, resolved by policy"
    );

    with_env_var("CSRF_PROTECTION", Some(" TRUE "));
    assert!(
        csrf_protection_requested(),
        "value is trimmed and case-folded"
    );

    restore_env_var("CSRF_PROTECTION", previous);
}

#[test]
fn malformed_config_stack_is_fatal_in_a_deployed_environment() {
    let _lock = ENV_LOCK.lock().unwrap();

    let test_dir = unique_temp_dir();
    write_default_toml(&test_dir, "this is not = valid toml [[[");

    let original_cwd = std::env::current_dir().expect("failed to read cwd");
    std::env::set_current_dir(&test_dir).expect("failed to enter temp dir");
    let previous_env = with_env_var("ENVIRONMENT", Some("production"));
    let previous_legacy = with_env_var("ACME_ENV", None);

    let result = AppBehaviorConfig::load();

    restore_env_var("ACME_ENV", previous_legacy);
    restore_env_var("ENVIRONMENT", previous_env);
    std::env::set_current_dir(original_cwd).expect("failed to restore cwd");
    std::fs::remove_dir_all(test_dir).expect("failed to remove temp dir");

    let err = result.expect_err("a malformed stack must not degrade to defaults when deployed");
    assert!(matches!(err, ConfigError::ConfigStack { .. }));
}

#[test]
fn malformed_config_stack_falls_back_to_defaults_in_local() {
    let _lock = ENV_LOCK.lock().unwrap();

    let test_dir = unique_temp_dir();
    write_default_toml(&test_dir, "this is not = valid toml [[[");

    let original_cwd = std::env::current_dir().expect("failed to read cwd");
    std::env::set_current_dir(&test_dir).expect("failed to enter temp dir");
    let previous_env = with_env_var("ENVIRONMENT", Some("local"));
    let previous_legacy = with_env_var("ACME_ENV", None);

    let result = AppBehaviorConfig::load();

    restore_env_var("ACME_ENV", previous_legacy);
    restore_env_var("ENVIRONMENT", previous_env);
    std::env::set_current_dir(original_cwd).expect("failed to restore cwd");
    std::fs::remove_dir_all(test_dir).expect("failed to remove temp dir");

    let behavior = result.expect("local dev keeps booting on committed defaults");
    assert_eq!(
        behavior.runtime.port,
        AppBehaviorConfig::default().runtime.port
    );
}

// ============================================================================
// Bootstrap-only env boundary (g09.047 review)
// ============================================================================
//
// Behavior knobs live in typed config with committed defaults. The migrated
// env keys are read only to warn, never to configure, and are absent from
// config/env-manifest.txt.

#[test]
fn migrated_behavior_keys_are_declared_legacy() {
    for key in [
        "SESSION_MAX_ABSOLUTE_DAYS",
        "SUPPORTED_API_VERSIONS",
        "DEFAULT_API_VERSION",
        "COOKIE_PREFIX",
        "COOKIE_SAMESITE_STRICT",
        "REFRESH_TOKEN_MAX_AGE",
    ] {
        assert!(
            LEGACY_BEHAVIOR_ENV_KEYS.iter().any(|(k, _)| *k == key),
            "{key} moved to typed config and must warn when still set"
        );
    }
}

#[test]
fn api_version_defaults_are_self_consistent() {
    let behavior = AppBehaviorConfig::default();
    assert!(
        !behavior.api.supported_versions.is_empty(),
        "an empty supported set would reject every business request"
    );
    assert!(
        behavior
            .api
            .supported_versions
            .contains(&behavior.api.default_version),
        "the default version must be one the server accepts"
    );
}

#[test]
fn cookie_defaults_are_the_safe_ones() {
    let behavior = AppBehaviorConfig::default();
    assert!(
        behavior.cors.cookie_same_site_strict,
        "SameSite=Strict is the default; Lax is an explicit local opt-out"
    );
    assert!(behavior.cors.refresh_token_max_age_secs > 0);
}

#[test]
fn typed_api_and_cookie_config_override_the_defaults() {
    let _lock = ENV_LOCK.lock().unwrap();

    let test_dir = unique_temp_dir();
    write_default_toml(
        &test_dir,
        r#"
[api]
supported_versions = ["2025-01-01", "2026-01-01"]
default_version = "2026-01-01"

[cors]
cookie_prefix = "acme_"
cookie_same_site_strict = false
refresh_token_max_age_secs = 900
"#,
    );

    let original_cwd = std::env::current_dir().expect("failed to read cwd");
    std::env::set_current_dir(&test_dir).expect("failed to enter temp dir");
    let previous_env = with_env_var("ENVIRONMENT", Some("local"));
    let previous_legacy = with_env_var("ACME_ENV", None);

    let result = AppBehaviorConfig::load();

    restore_env_var("ACME_ENV", previous_legacy);
    restore_env_var("ENVIRONMENT", previous_env);
    std::env::set_current_dir(original_cwd).expect("failed to restore cwd");
    std::fs::remove_dir_all(test_dir).expect("failed to remove temp dir");

    let behavior = result.expect("valid config stack");
    assert_eq!(
        behavior.api.supported_versions,
        vec!["2025-01-01".to_string(), "2026-01-01".to_string()]
    );
    assert_eq!(behavior.api.default_version, "2026-01-01");
    assert_eq!(behavior.cors.cookie_prefix, "acme_");
    assert!(!behavior.cors.cookie_same_site_strict);
    assert_eq!(behavior.cors.refresh_token_max_age_secs, 900);
}

#[test]
fn a_default_version_outside_the_supported_set_falls_back() {
    let _lock = ENV_LOCK.lock().unwrap();

    let test_dir = unique_temp_dir();
    write_default_toml(
        &test_dir,
        r#"
[api]
supported_versions = ["2025-01-01"]
default_version = "2030-01-01"
"#,
    );

    let original_cwd = std::env::current_dir().expect("failed to read cwd");
    std::env::set_current_dir(&test_dir).expect("failed to enter temp dir");
    let previous_env = with_env_var("ENVIRONMENT", Some("local"));
    let previous_legacy = with_env_var("ACME_ENV", None);

    let result = AppBehaviorConfig::load();

    restore_env_var("ACME_ENV", previous_legacy);
    restore_env_var("ENVIRONMENT", previous_env);
    std::env::set_current_dir(original_cwd).expect("failed to restore cwd");
    std::fs::remove_dir_all(test_dir).expect("failed to remove temp dir");

    let behavior = result.expect("valid config stack");
    assert_eq!(
        behavior.api.default_version, "2025-01-01",
        "an unserviceable default must fall back rather than reject every request"
    );
}

#[test]
fn migrated_env_keys_no_longer_configure_anything() {
    let _lock = ENV_LOCK.lock().unwrap();

    let test_dir = unique_temp_dir();
    write_default_toml(&test_dir, "[api]\nsupported_versions = [\"2025-01-01\"]\n");

    let original_cwd = std::env::current_dir().expect("failed to read cwd");
    std::env::set_current_dir(&test_dir).expect("failed to enter temp dir");
    let previous = [
        ("ENVIRONMENT", with_env_var("ENVIRONMENT", Some("local"))),
        ("ACME_ENV", with_env_var("ACME_ENV", None)),
        (
            "SUPPORTED_API_VERSIONS",
            with_env_var("SUPPORTED_API_VERSIONS", Some("1999-01-01")),
        ),
        (
            "DEFAULT_API_VERSION",
            with_env_var("DEFAULT_API_VERSION", Some("1999-01-01")),
        ),
        (
            "COOKIE_PREFIX",
            with_env_var("COOKIE_PREFIX", Some("evil_")),
        ),
        (
            "SESSION_MAX_ABSOLUTE_DAYS",
            with_env_var("SESSION_MAX_ABSOLUTE_DAYS", Some("9999")),
        ),
    ];

    let result = AppBehaviorConfig::load();

    for (key, value) in previous {
        restore_env_var(key, value);
    }
    std::env::set_current_dir(original_cwd).expect("failed to restore cwd");
    std::fs::remove_dir_all(test_dir).expect("failed to remove temp dir");

    let behavior = result.expect("valid config stack");
    let defaults = AppBehaviorConfig::default();
    assert_eq!(
        behavior.api.supported_versions,
        vec!["2025-01-01".to_string()]
    );
    assert_eq!(behavior.api.default_version, "2025-01-01");
    assert_eq!(behavior.cors.cookie_prefix, defaults.cors.cookie_prefix);
    assert_eq!(
        behavior.auth.absolute_session_timeout_days,
        defaults.auth.absolute_session_timeout_days,
    );
}
