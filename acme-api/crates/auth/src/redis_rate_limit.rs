//! Redis-backed rate limiting backend for distributed deployments.
//!
//! This backend uses Redis to store rate limit counters, enabling rate limiting
//! to work correctly across multiple API server instances.

use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use std::time::{SystemTime, UNIX_EPOCH};
use underlay_ratelimit::{
    RateLimitBackend, RateLimitConfig, RateLimitError, RateLimitResult, Result,
};

/// Redis-backed rate limiter backend.
#[derive(Debug, Clone)]
pub struct RedisRateLimitBackend {
    client: Client,
}

impl RedisRateLimitBackend {
    /// Create a new Redis rate limiter backend.
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| RateLimitError::Backend(format!("Failed to connect to Redis: {}", e)))?;
        Ok(Self { client })
    }

    /// Create a new Redis rate limiter backend from environment.
    pub fn from_env() -> Result<Self> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        Self::new(&redis_url)
    }

    /// Get the current timestamp in seconds since UNIX epoch.
    fn current_timestamp_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    /// Build the Redis key for a rate limit counter.
    fn build_key(base_key: &str, window_start: u64) -> String {
        format!("ratelimit:{}:{}", base_key, window_start)
    }
}

#[async_trait]
impl RateLimitBackend for RedisRateLimitBackend {
    /// Check if a request is allowed under the rate limit.
    async fn check(&self, key: &str, config: &RateLimitConfig) -> Result<RateLimitResult> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimitError::Backend(format!("Redis connection error: {}", e)))?;

        let now = Self::current_timestamp_secs();
        let window_size_secs = config.window.as_secs();
        let window_start = now / window_size_secs * window_size_secs;
        let redis_key = Self::build_key(key, window_start);

        let count: Option<u64> = conn
            .get(&redis_key)
            .await
            .map_err(|e| RateLimitError::Backend(format!("Redis GET error: {}", e)))?;
        let count = count.unwrap_or(0);

        let max_requests = config.max_requests;

        if count < max_requests {
            let remaining = max_requests - count;
            Ok(RateLimitResult::allowed(remaining, count))
        } else {
            let reset_after = std::time::Duration::from_secs(
                (window_start + window_size_secs).saturating_sub(now),
            );
            Ok(RateLimitResult::denied(count, reset_after))
        }
    }

    /// Increment the counter for a key and return the new count.
    async fn increment(&self, key: &str, config: &RateLimitConfig) -> Result<u64> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimitError::Backend(format!("Redis connection error: {}", e)))?;

        let now = Self::current_timestamp_secs();
        let window_size_secs = config.window.as_secs();
        let window_start = now / window_size_secs * window_size_secs;
        let redis_key = Self::build_key(key, window_start);

        let (count,): (u64,) = redis::pipe()
            .atomic()
            .incr(&redis_key, 1)
            .expire(&redis_key, window_size_secs as i64 + 1)
            .ignore()
            .query_async(&mut conn)
            .await
            .map_err(|e| RateLimitError::Backend(format!("Redis INCR error: {}", e)))?;

        Ok(count)
    }

    /// Reset the counter for a key.
    async fn reset(&self, key: &str) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimitError::Backend(format!("Redis connection error: {}", e)))?;

        let now = Self::current_timestamp_secs();
        let keys_to_delete: Vec<String> = (0..2)
            .map(|i| {
                let window_start = (now.saturating_sub(i * 3600)) / 3600 * 3600;
                Self::build_key(key, window_start)
            })
            .collect();

        let _: () = conn
            .del(&keys_to_delete)
            .await
            .map_err(|e| RateLimitError::Backend(format!("Redis DEL error: {}", e)))?;

        Ok(())
    }
}

/// Rate limiter backend type selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitBackendType {
    /// In-memory backend (default, single-instance only).
    InMemory,
    /// Redis backend (distributed, multi-instance).
    Redis,
}

impl Default for RateLimitBackendType {
    fn default() -> Self {
        Self::InMemory
    }
}

impl std::str::FromStr for RateLimitBackendType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "memory" | "inmemory" | "in-memory" => Ok(Self::InMemory),
            "redis" => Ok(Self::Redis),
            _ => Err(format!(
                "Invalid rate limit backend type: {}. Use 'memory' or 'redis'.",
                s
            )),
        }
    }
}

#[cfg(test)]
mod tests {
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
}
