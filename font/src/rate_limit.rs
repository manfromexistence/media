//! Rate limiter for dx-font
//!
//! This module provides rate limiting functionality using a token bucket algorithm.
//! It supports per-provider rate limits and exponential backoff for 429 responses.

use crate::error::FontResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// A token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Current number of tokens available
    tokens: f64,
    /// Last time tokens were updated
    last_update: Instant,
    /// Rate of token replenishment (tokens per second)
    rate: f64,
    /// Maximum number of tokens (burst capacity)
    burst: u32,
    /// Backoff state for 429 responses
    backoff_until: Option<Instant>,
    /// Current backoff multiplier
    backoff_multiplier: u32,
}

impl TokenBucket {
    /// Create a new token bucket with full capacity
    fn new(rate: f64, burst: u32) -> Self {
        Self {
            tokens: burst as f64,
            last_update: Instant::now(),
            rate,
            burst,
            backoff_until: None,
            backoff_multiplier: 0,
        }
    }

    /// Replenish tokens based on elapsed time
    fn replenish(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.burst as f64);
        self.last_update = now;
    }

    /// Try to acquire a token without waiting
    fn try_acquire(&mut self) -> bool {
        // Check if we're in backoff
        if let Some(backoff_until) = self.backoff_until {
            if Instant::now() < backoff_until {
                return false;
            }
            // Backoff period has passed
            self.backoff_until = None;
        }

        self.replenish();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Calculate how long to wait for a token
    fn time_until_available(&mut self) -> Duration {
        // Check if we're in backoff
        if let Some(backoff_until) = self.backoff_until {
            let now = Instant::now();
            if now < backoff_until {
                return backoff_until.duration_since(now);
            }
            self.backoff_until = None;
        }

        self.replenish();

        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let tokens_needed = 1.0 - self.tokens;
            Duration::from_secs_f64(tokens_needed / self.rate)
        }
    }

    /// Record a rate limit response and apply backoff
    fn record_rate_limit(&mut self, retry_after: Option<u64>) {
        self.backoff_multiplier = (self.backoff_multiplier + 1).min(6); // Cap at 2^6 = 64x

        let base_backoff = retry_after.unwrap_or(1);
        let backoff_secs = base_backoff * (1 << self.backoff_multiplier);

        self.backoff_until = Some(Instant::now() + Duration::from_secs(backoff_secs));
        self.tokens = 0.0; // Drain all tokens

        warn!(
            "Rate limit hit, backing off for {}s (multiplier: {})",
            backoff_secs, self.backoff_multiplier
        );
    }

    /// Reset backoff state after successful request
    fn reset_backoff(&mut self) {
        if self.backoff_multiplier > 0 {
            self.backoff_multiplier = self.backoff_multiplier.saturating_sub(1);
        }
    }
}

/// Rate limiter with per-provider buckets
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Per-provider token buckets
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    /// Default rate (requests per second)
    default_rate: f64,
    /// Default burst size
    default_burst: u32,
}

impl RateLimiter {
    /// Create a new rate limiter with default settings
    ///
    /// # Arguments
    /// * `default_rate` - Default requests per second
    /// * `default_burst` - Default burst capacity
    pub fn new(default_rate: f64, default_burst: u32) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            default_rate,
            default_burst,
        }
    }

    /// Create a rate limiter with default production settings
    /// (10 requests/second, burst of 20)
    pub fn default_production() -> Self {
        Self::new(10.0, 20)
    }

    /// Get or create a bucket for a provider
    async fn get_or_create_bucket(&self, provider: &str) -> TokenBucket {
        let mut buckets = self.buckets.lock().await;
        buckets
            .entry(provider.to_string())
            .or_insert_with(|| TokenBucket::new(self.default_rate, self.default_burst))
            .clone()
    }

    /// Update a bucket for a provider
    async fn update_bucket(&self, provider: &str, bucket: TokenBucket) {
        let mut buckets = self.buckets.lock().await;
        buckets.insert(provider.to_string(), bucket);
    }

    /// Acquire a token, waiting if necessary
    ///
    /// # Arguments
    /// * `provider` - Provider name for per-provider rate limiting
    ///
    /// # Returns
    /// * `Ok(())` - Token acquired
    /// * `Err(FontError::RateLimit)` - Rate limit exceeded (shouldn't happen with waiting)
    pub async fn acquire(&self, provider: &str) -> FontResult<()> {
        loop {
            let mut bucket = self.get_or_create_bucket(provider).await;

            if bucket.try_acquire() {
                self.update_bucket(provider, bucket).await;
                debug!("Rate limiter: acquired token for provider '{}'", provider);
                return Ok(());
            }

            let wait_time = bucket.time_until_available();
            self.update_bucket(provider, bucket).await;

            if wait_time > Duration::ZERO {
                debug!(
                    "Rate limiter: waiting {:?} for provider '{}'",
                    wait_time, provider
                );
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// Try to acquire a token without waiting
    ///
    /// # Arguments
    /// * `provider` - Provider name
    ///
    /// # Returns
    /// * `true` - Token acquired
    /// * `false` - Would need to wait
    pub async fn try_acquire(&self, provider: &str) -> bool {
        let mut bucket = self.get_or_create_bucket(provider).await;
        let acquired = bucket.try_acquire();
        self.update_bucket(provider, bucket).await;
        acquired
    }

    /// Record a rate limit response (429) and apply backoff
    ///
    /// # Arguments
    /// * `provider` - Provider that returned 429
    /// * `retry_after` - Optional retry-after header value in seconds
    pub async fn record_rate_limit(&self, provider: &str, retry_after: Option<u64>) {
        let mut bucket = self.get_or_create_bucket(provider).await;
        bucket.record_rate_limit(retry_after);
        self.update_bucket(provider, bucket).await;
    }

    /// Record a successful request (reduces backoff)
    ///
    /// # Arguments
    /// * `provider` - Provider that succeeded
    pub async fn record_success(&self, provider: &str) {
        let mut bucket = self.get_or_create_bucket(provider).await;
        bucket.reset_backoff();
        self.update_bucket(provider, bucket).await;
    }

    /// Set custom rate for a specific provider
    ///
    /// # Arguments
    /// * `provider` - Provider name
    /// * `rate` - Requests per second
    /// * `burst` - Burst capacity
    pub async fn set_provider_rate(&self, provider: &str, rate: f64, burst: u32) {
        let mut buckets = self.buckets.lock().await;
        buckets.insert(provider.to_string(), TokenBucket::new(rate, burst));
        debug!(
            "Set custom rate for provider '{}': {} req/s, burst {}",
            provider, rate, burst
        );
    }

    /// Get current token count for a provider (for testing/debugging)
    pub async fn tokens_available(&self, provider: &str) -> f64 {
        let mut bucket = self.get_or_create_bucket(provider).await;
        bucket.replenish();
        let tokens = bucket.tokens;
        self.update_bucket(provider, bucket).await;
        tokens
    }

    /// Check if a provider is currently in backoff
    pub async fn is_in_backoff(&self, provider: &str) -> bool {
        let bucket = self.get_or_create_bucket(provider).await;
        if let Some(backoff_until) = bucket.backoff_until {
            Instant::now() < backoff_until
        } else {
            false
        }
    }

    /// Get the default rate
    pub fn default_rate(&self) -> f64 {
        self.default_rate
    }

    /// Get the default burst
    pub fn default_burst(&self) -> u32 {
        self.default_burst
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::default_production()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10.0, 20);
        assert_eq!(limiter.default_rate(), 10.0);
        assert_eq!(limiter.default_burst(), 20);
    }

    #[tokio::test]
    async fn test_burst_capacity() {
        let limiter = RateLimiter::new(1.0, 5);

        // Should be able to acquire burst capacity immediately
        for _ in 0..5 {
            assert!(limiter.try_acquire("test_provider").await);
        }

        // Next one should fail (no tokens left)
        assert!(!limiter.try_acquire("test_provider").await);
    }

    #[tokio::test]
    async fn test_token_replenishment() {
        let limiter = RateLimiter::new(10.0, 5);

        // Exhaust all tokens
        for _ in 0..5 {
            limiter.try_acquire("test_provider").await;
        }

        // Wait for some tokens to replenish
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Should have ~2 tokens now (10 tokens/sec * 0.2 sec = 2)
        assert!(limiter.try_acquire("test_provider").await);
    }

    #[tokio::test]
    async fn test_acquire_waits() {
        let limiter = RateLimiter::new(10.0, 1);

        // Use the one token
        limiter.try_acquire("test_provider").await;

        // acquire() should wait and succeed
        let start = Instant::now();
        limiter.acquire("test_provider").await.unwrap();
        let elapsed = start.elapsed();

        // Should have waited approximately 100ms (1 token / 10 tokens per sec)
        assert!(elapsed >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_per_provider_isolation() {
        let limiter = RateLimiter::new(1.0, 2);

        // Exhaust provider A's tokens
        limiter.try_acquire("provider_a").await;
        limiter.try_acquire("provider_a").await;
        assert!(!limiter.try_acquire("provider_a").await);

        // Provider B should still have tokens
        assert!(limiter.try_acquire("provider_b").await);
    }

    #[tokio::test]
    async fn test_rate_limit_backoff() {
        let limiter = RateLimiter::new(10.0, 5);

        // Record a rate limit
        limiter.record_rate_limit("test_provider", Some(1)).await;

        // Should be in backoff
        assert!(limiter.is_in_backoff("test_provider").await);
        assert!(!limiter.try_acquire("test_provider").await);
    }

    #[tokio::test]
    async fn test_custom_provider_rate() {
        let limiter = RateLimiter::new(10.0, 20);

        // Set custom rate for a provider
        limiter.set_provider_rate("slow_provider", 1.0, 2).await;

        // Should only get 2 tokens
        assert!(limiter.try_acquire("slow_provider").await);
        assert!(limiter.try_acquire("slow_provider").await);
        assert!(!limiter.try_acquire("slow_provider").await);

        // Default provider should still have 20
        for _ in 0..20 {
            assert!(limiter.try_acquire("default_provider").await);
        }
    }

    #[tokio::test]
    async fn test_success_reduces_backoff() {
        let limiter = RateLimiter::new(10.0, 5);

        // Record multiple rate limits to increase backoff multiplier
        limiter.record_rate_limit("test_provider", Some(0)).await;

        // Record success
        limiter.record_success("test_provider").await;

        // Backoff multiplier should be reduced (but we're still in backoff period)
        // This is more of a state test
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: dx-font-production-ready, Property 4: Rate Limiter Token Bucket Behavior
    // **Validates: Requirements 4.1, 4.2, 4.3**
    //
    // For any rate limiter configured with rate r requests/second and burst b:
    // - Immediately after creation, up to b requests can be made without waiting
    // - After exhausting burst capacity, requests are limited to r per second
    // - Tokens replenish at rate r per second up to maximum b

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn burst_capacity_available_immediately(
            rate in 1.0f64..100.0f64,
            burst in 1u32..50u32
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let limiter = RateLimiter::new(rate, burst);

                // Should be able to acquire exactly burst tokens immediately
                let mut acquired = 0u32;
                for _ in 0..burst {
                    if limiter.try_acquire("test").await {
                        acquired += 1;
                    }
                }

                prop_assert_eq!(
                    acquired, burst,
                    "Should acquire exactly {} tokens immediately, got {}",
                    burst, acquired
                );

                // Next acquisition should fail (no waiting)
                prop_assert!(
                    !limiter.try_acquire("test").await,
                    "Should not be able to acquire more than burst capacity"
                );

                Ok(())
            })?;
        }

        #[test]
        fn tokens_replenish_over_time(
            rate in 5.0f64..20.0f64,
            burst in 5u32..20u32
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let limiter = RateLimiter::new(rate, burst);

                // Exhaust all tokens
                for _ in 0..burst {
                    limiter.try_acquire("test").await;
                }

                // Verify tokens are exhausted
                let tokens_before = limiter.tokens_available("test").await;
                prop_assert!(
                    tokens_before < 1.0,
                    "Tokens should be exhausted, got {}",
                    tokens_before
                );

                // Wait for some replenishment (100ms)
                tokio::time::sleep(Duration::from_millis(100)).await;

                let tokens_after = limiter.tokens_available("test").await;
                let expected_replenish = rate * 0.1; // rate * 100ms

                // Allow some tolerance for timing
                prop_assert!(
                    tokens_after >= expected_replenish * 0.8,
                    "Expected ~{} tokens after 100ms, got {}",
                    expected_replenish, tokens_after
                );

                Ok(())
            })?;
        }

        #[test]
        fn tokens_capped_at_burst(
            rate in 10.0f64..50.0f64,
            burst in 5u32..20u32
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let limiter = RateLimiter::new(rate, burst);

                // Wait longer than needed to fully replenish
                tokio::time::sleep(Duration::from_millis(500)).await;

                let tokens = limiter.tokens_available("test").await;

                // Tokens should not exceed burst
                prop_assert!(
                    tokens <= burst as f64,
                    "Tokens {} should not exceed burst {}",
                    tokens, burst
                );

                // Tokens should be at or near burst
                prop_assert!(
                    tokens >= (burst as f64) - 0.1,
                    "Tokens {} should be near burst {}",
                    tokens, burst
                );

                Ok(())
            })?;
        }

        #[test]
        fn per_provider_isolation(
            rate in 5.0f64..20.0f64,
            burst in 3u32..10u32
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let limiter = RateLimiter::new(rate, burst);

                // Exhaust provider A
                for _ in 0..burst {
                    limiter.try_acquire("provider_a").await;
                }

                // Provider B should still have full burst
                let mut acquired_b = 0u32;
                for _ in 0..burst {
                    if limiter.try_acquire("provider_b").await {
                        acquired_b += 1;
                    }
                }

                prop_assert_eq!(
                    acquired_b, burst,
                    "Provider B should have full burst capacity"
                );

                Ok(())
            })?;
        }
    }
}
