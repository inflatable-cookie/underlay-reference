
use super::*;

#[test]
fn test_backend_type_parsing() {
    assert_eq!(
        "memory".parse::<RateLimitBackendType>().unwrap(),
        RateLimitBackendType::InMemory
    );
    assert_eq!(
        "redis".parse::<RateLimitBackendType>().unwrap(),
        RateLimitBackendType::Redis
    );
    assert!("invalid".parse::<RateLimitBackendType>().is_err());
}

#[test]
fn test_build_key() {
    let key = RedisRateLimitBackend::build_key("login:user@example.com", 1234567890);
    assert_eq!(key, "ratelimit:login:user@example.com:1234567890");
}
