//! Dynamic rate limiter backend wrapper.
//!
//! Provides a unified interface for both in-memory and Redis backends.

use async_trait::async_trait;
use underlay_ratelimit::{RateLimitBackend, RateLimitConfig, RateLimitResult, Result};

use crate::redis_rate_limit::RedisRateLimitBackend;

/// A boxed rate limiter backend that can be either in-memory or Redis.
#[derive(Debug, Clone)]
pub enum DynamicRateLimiter {
    /// In-memory backend.
    InMemory(underlay_ratelimit::InMemoryBackend),
    /// Redis backend.
    Redis(RedisRateLimitBackend),
}

impl DynamicRateLimiter {
    /// Create a new in-memory rate limiter.
    pub fn in_memory(backend: underlay_ratelimit::InMemoryBackend) -> Self {
        Self::InMemory(backend)
    }

    /// Create a new Redis rate limiter.
    pub fn redis(backend: RedisRateLimitBackend) -> Self {
        Self::Redis(backend)
    }
}

#[async_trait]
impl RateLimitBackend for DynamicRateLimiter {
    async fn check(&self,
        key: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult> {
        match self {
            Self::InMemory(backend) => backend.check(key, config).await,
            Self::Redis(backend) => backend.check(key, config).await,
        }
    }

    async fn increment(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> Result<u64> {
        match self {
            Self::InMemory(backend) => backend.increment(key, config).await,
            Self::Redis(backend) => backend.increment(key, config).await,
        }
    }

    async fn reset(&self, key: &str) -> Result<()> {
        match self {
            Self::InMemory(backend) => backend.reset(key).await,
            Self::Redis(backend) => backend.reset(key).await,
        }
    }
}
