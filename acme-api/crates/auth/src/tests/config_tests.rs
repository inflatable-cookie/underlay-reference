
use super::*;

#[test]
fn test_default_values() {
    let config = AuthConfig::default();

    assert_eq!(config.login_rate_limit_per_hour, 10);
    assert_eq!(config.register_rate_limit_per_hour, 5);
    assert_eq!(config.max_totp_attempts, 5);
    assert_eq!(config.max_failed_logins, 5);
    assert_eq!(config.totp_state_timeout, Duration::from_secs(300));
    assert_eq!(config.lockout_duration, Duration::from_secs(900));
    assert_eq!(config.security_alert_window, Duration::from_secs(600));
    assert_eq!(config.security_alert_cooldown, Duration::from_secs(1800));
    assert_eq!(config.security_alert_failed_attempts_threshold, 20);
    assert_eq!(config.security_alert_distinct_users_threshold, 5);
    assert_eq!(config.security_alert_lockouts_threshold, 3);
}

#[test]
fn test_builder() {
    let config = AuthConfig::builder()
        .login_rate_limit_per_hour(20)
        .max_totp_attempts(3)
        .lockout_duration(Duration::from_secs(1800))
        .build();

    assert_eq!(config.login_rate_limit_per_hour, 20);
    assert_eq!(config.max_totp_attempts, 3);
    assert_eq!(config.lockout_duration, Duration::from_secs(1800));
    // Other values should be defaults
    assert_eq!(config.register_rate_limit_per_hour, 5);
}

#[test]
fn test_helper_methods() {
    let config = AuthConfig::default();

    assert_eq!(config.totp_state_timeout_minutes(), 5);
    assert_eq!(config.email_state_timeout_minutes(), 10);
    assert_eq!(config.verification_session_timeout_minutes(), 5);
    assert_eq!(config.email_code_expiry_minutes(), 10);
    assert_eq!(config.retry_after_short_secs(), 60);
    assert_eq!(config.retry_after_long_secs(), 300);
    assert_eq!(config.lockout_duration_secs(), 900);
}
