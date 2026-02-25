    use super::*;
    use std::path::PathBuf;
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
        std::fs::write(config_dir.join("default.toml"), content)
            .expect("failed to write default.toml");
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

        let config = AppConfig::from_env();
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
