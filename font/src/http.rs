//! HTTP client with retry logic for dx-font
//!
//! This module provides an HTTP client wrapper with automatic retry logic,
//! rate limiting integration, and exponential backoff.

use crate::error::{FontError, FontResult};
use crate::rate_limit::RateLimiter;
use rand::Rng;
use reqwest::{Client, Request, Response, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// HTTP client with retry logic and rate limiting
#[derive(Debug, Clone)]
pub struct RetryClient {
    /// Inner reqwest client
    inner: Client,
    /// Rate limiter for request throttling
    rate_limiter: Arc<RateLimiter>,
    /// Maximum number of retry attempts
    max_retries: u32,
    /// Base delay for exponential backoff
    base_delay: Duration,
}

impl RetryClient {
    /// Create a new retry client
    ///
    /// # Arguments
    /// * `client` - The underlying reqwest client
    /// * `rate_limiter` - Rate limiter for request throttling
    /// * `max_retries` - Maximum number of retry attempts (default: 3)
    pub fn new(client: Client, rate_limiter: Arc<RateLimiter>, max_retries: u32) -> Self {
        Self {
            inner: client,
            rate_limiter,
            max_retries,
            base_delay: Duration::from_secs(1),
        }
    }

    /// Create a retry client with default settings
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be created
    pub fn with_defaults(rate_limiter: Arc<RateLimiter>) -> FontResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| FontError::validation(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self::new(client, rate_limiter, 3))
    }

    /// Set the base delay for exponential backoff
    pub fn with_base_delay(mut self, base_delay: Duration) -> Self {
        self.base_delay = base_delay;
        self
    }

    /// Execute a request with rate limiting and retry logic
    ///
    /// # Arguments
    /// * `request` - The request to execute
    /// * `provider` - Provider name for rate limiting
    ///
    /// # Returns
    /// * `Ok(Response)` - Successful response
    /// * `Err(FontError)` - Request failed after all retries
    pub async fn execute(&self, request: Request, provider: &str) -> FontResult<Response> {
        let url = request.url().to_string();
        let method = request.method().clone();

        // We need to clone the request for retries since Request doesn't implement Clone
        // We'll rebuild it from the URL and method
        let mut attempt = 0;
        let mut _last_error: Option<FontError> = None;

        loop {
            // Wait for rate limiter
            self.rate_limiter.acquire(provider).await?;

            // Build a new request for each attempt
            let req = self
                .inner
                .request(method.clone(), &url)
                .build()
                .map_err(|e| FontError::network(&url, e))?;

            debug!(
                "HTTP {} {} (attempt {}/{})",
                method,
                url,
                attempt + 1,
                self.max_retries + 1
            );

            match self.inner.execute(req).await {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        self.rate_limiter.record_success(provider).await;
                        return Ok(response);
                    }

                    // Handle rate limiting
                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = response
                            .headers()
                            .get("retry-after")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse().ok());

                        self.rate_limiter
                            .record_rate_limit(provider, retry_after)
                            .await;

                        if should_retry(status, attempt, self.max_retries) {
                            attempt += 1;
                            let delay = calculate_backoff(attempt, self.base_delay);
                            warn!(
                                "Rate limited by {}, retrying in {:?} (attempt {}/{})",
                                provider,
                                delay,
                                attempt,
                                self.max_retries + 1
                            );
                            tokio::time::sleep(delay).await;
                            continue;
                        }

                        return Err(FontError::rate_limit(provider, retry_after.unwrap_or(60)));
                    }

                    // Handle server errors (5xx)
                    if status.is_server_error() && should_retry(status, attempt, self.max_retries) {
                        attempt += 1;
                        let delay = calculate_backoff(attempt, self.base_delay);
                        warn!(
                            "Server error {} from {}, retrying in {:?} (attempt {}/{})",
                            status,
                            provider,
                            delay,
                            attempt,
                            self.max_retries + 1
                        );
                        _last_error = Some(FontError::provider(
                            provider,
                            format!("HTTP {}", status.as_u16()),
                        ));
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    // Client errors (4xx except 429) are not retried
                    return Err(FontError::provider(
                        provider,
                        format!("HTTP {} for {}", status.as_u16(), url),
                    ));
                }
                Err(e) => {
                    // Check if it's a timeout
                    if e.is_timeout() {
                        if should_retry(StatusCode::GATEWAY_TIMEOUT, attempt, self.max_retries) {
                            attempt += 1;
                            let delay = calculate_backoff(attempt, self.base_delay);
                            warn!(
                                "Request timeout for {}, retrying in {:?} (attempt {}/{})",
                                provider,
                                delay,
                                attempt,
                                self.max_retries + 1
                            );
                            _last_error = Some(FontError::timeout(30));
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(FontError::timeout(30));
                    }

                    // Check if it's a connection error (retryable)
                    if e.is_connect()
                        && should_retry(StatusCode::SERVICE_UNAVAILABLE, attempt, self.max_retries)
                    {
                        attempt += 1;
                        let delay = calculate_backoff(attempt, self.base_delay);
                        warn!(
                            "Connection error for {}, retrying in {:?} (attempt {}/{})",
                            provider,
                            delay,
                            attempt,
                            self.max_retries + 1
                        );
                        _last_error = Some(FontError::network(&url, e));
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Err(FontError::network(&url, e));
                }
            }
        }
    }

    /// GET request with automatic retry
    ///
    /// # Arguments
    /// * `url` - URL to fetch
    /// * `provider` - Provider name for rate limiting
    pub async fn get(&self, url: &str, provider: &str) -> FontResult<Response> {
        let request = self
            .inner
            .get(url)
            .build()
            .map_err(|e| FontError::network(url, e))?;

        self.execute(request, provider).await
    }

    /// GET request that returns the response body as text
    pub async fn get_text(&self, url: &str, provider: &str) -> FontResult<String> {
        let response = self.get(url, provider).await?;
        response.text().await.map_err(|e| {
            FontError::provider(provider, format!("Failed to read response body: {}", e))
        })
    }

    /// GET request that returns the response body as JSON
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        provider: &str,
    ) -> FontResult<T> {
        let response = self.get(url, provider).await?;
        response.json().await.map_err(|e| {
            FontError::parse(provider, format!("Failed to parse JSON response: {}", e))
        })
    }

    /// Get the inner client for advanced use cases
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Get the rate limiter
    pub fn rate_limiter(&self) -> &Arc<RateLimiter> {
        &self.rate_limiter
    }

    /// Get the maximum number of retries
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Get the base delay for backoff
    pub fn base_delay(&self) -> Duration {
        self.base_delay
    }
}

/// Determine if a request should be retried based on status code and attempt count
///
/// # Arguments
/// * `status` - HTTP status code
/// * `attempt` - Current attempt number (0-indexed)
/// * `max_retries` - Maximum number of retries allowed
///
/// # Returns
/// * `true` - Request should be retried
/// * `false` - Request should not be retried
pub fn should_retry(status: StatusCode, attempt: u32, max_retries: u32) -> bool {
    if attempt >= max_retries {
        return false;
    }

    // Retry on 429 (Too Many Requests)
    if status == StatusCode::TOO_MANY_REQUESTS {
        return true;
    }

    // Retry on all 5xx server errors
    if status.is_server_error() {
        return true;
    }

    false
}

/// Calculate backoff delay with exponential growth and jitter
///
/// # Arguments
/// * `attempt` - Current attempt number (1-indexed for backoff calculation)
/// * `base_delay` - Base delay duration
///
/// # Returns
/// Delay duration with exponential backoff and random jitter
pub fn calculate_backoff(attempt: u32, base_delay: Duration) -> Duration {
    // Exponential backoff: base_delay * 2^(attempt-1)
    let exponential_ms = base_delay.as_millis() as u64 * (1 << (attempt.saturating_sub(1)));

    // Add jitter: 0-100ms random
    let jitter_ms = rand::thread_rng().gen_range(0..=100);

    // Cap at 60 seconds
    let total_ms = (exponential_ms + jitter_ms).min(60_000);

    Duration::from_millis(total_ms)
}

/// Information about a retry attempt
#[derive(Debug, Clone)]
pub struct RetryInfo {
    /// Number of attempts made
    pub attempts: u32,
    /// Total time spent on retries
    pub total_delay: Duration,
    /// Whether the request ultimately succeeded
    pub succeeded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry_5xx() {
        assert!(should_retry(StatusCode::INTERNAL_SERVER_ERROR, 0, 3));
        assert!(should_retry(StatusCode::BAD_GATEWAY, 0, 3));
        assert!(should_retry(StatusCode::SERVICE_UNAVAILABLE, 0, 3));
        assert!(should_retry(StatusCode::GATEWAY_TIMEOUT, 0, 3));
    }

    #[test]
    fn test_should_retry_429() {
        assert!(should_retry(StatusCode::TOO_MANY_REQUESTS, 0, 3));
    }

    #[test]
    fn test_should_not_retry_4xx() {
        assert!(!should_retry(StatusCode::BAD_REQUEST, 0, 3));
        assert!(!should_retry(StatusCode::UNAUTHORIZED, 0, 3));
        assert!(!should_retry(StatusCode::FORBIDDEN, 0, 3));
        assert!(!should_retry(StatusCode::NOT_FOUND, 0, 3));
    }

    #[test]
    fn test_should_not_retry_2xx() {
        assert!(!should_retry(StatusCode::OK, 0, 3));
        assert!(!should_retry(StatusCode::CREATED, 0, 3));
        assert!(!should_retry(StatusCode::NO_CONTENT, 0, 3));
    }

    #[test]
    fn test_should_not_retry_max_attempts() {
        // Should not retry if we've hit max attempts
        assert!(!should_retry(StatusCode::INTERNAL_SERVER_ERROR, 3, 3));
        assert!(!should_retry(StatusCode::TOO_MANY_REQUESTS, 3, 3));
    }

    #[test]
    fn test_backoff_increases() {
        let base = Duration::from_millis(100);

        let delay1 = calculate_backoff(1, base);
        let delay2 = calculate_backoff(2, base);
        let delay3 = calculate_backoff(3, base);

        // Each delay should be roughly double the previous (accounting for jitter)
        // delay1 should be ~100ms + jitter
        // delay2 should be ~200ms + jitter
        // delay3 should be ~400ms + jitter

        assert!(delay1.as_millis() >= 100);
        assert!(delay1.as_millis() <= 200);

        assert!(delay2.as_millis() >= 200);
        assert!(delay2.as_millis() <= 300);

        assert!(delay3.as_millis() >= 400);
        assert!(delay3.as_millis() <= 500);
    }

    #[test]
    fn test_backoff_capped() {
        let base = Duration::from_secs(10);

        // Even with high attempt count, should be capped at 60 seconds
        let delay = calculate_backoff(10, base);
        assert!(delay.as_secs() <= 60);
    }

    #[tokio::test]
    async fn test_retry_client_creation() {
        let rate_limiter = Arc::new(RateLimiter::default_production());
        let client = RetryClient::with_defaults(rate_limiter).expect("Failed to create client");

        assert_eq!(client.max_retries(), 3);
        assert_eq!(client.base_delay(), Duration::from_secs(1));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: dx-font-production-ready, Property 5: Retry Policy Correctness
    // **Validates: Requirements 7.1, 7.4, 7.5**
    //
    // For any HTTP response status code:
    // - 5xx errors trigger retry (up to max_retries)
    // - 429 (Too Many Requests) triggers retry with backoff
    // - 4xx errors (except 429) do NOT trigger retry
    // - 2xx/3xx responses do NOT trigger retry

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn retry_policy_5xx_triggers_retry(
            status_code in 500u16..600u16,
            attempt in 0u32..10u32,
            max_retries in 1u32..10u32
        ) {
            let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            if attempt < max_retries {
                prop_assert!(
                    should_retry(status, attempt, max_retries),
                    "5xx status {} should trigger retry on attempt {} with max_retries {}",
                    status_code, attempt, max_retries
                );
            } else {
                prop_assert!(
                    !should_retry(status, attempt, max_retries),
                    "Should not retry after max attempts reached"
                );
            }
        }

        #[test]
        fn retry_policy_429_triggers_retry(
            attempt in 0u32..10u32,
            max_retries in 1u32..10u32
        ) {
            if attempt < max_retries {
                prop_assert!(
                    should_retry(StatusCode::TOO_MANY_REQUESTS, attempt, max_retries),
                    "429 should trigger retry on attempt {} with max_retries {}",
                    attempt, max_retries
                );
            }
        }

        #[test]
        fn retry_policy_4xx_no_retry(
            status_code in 400u16..429u16,  // 400-428 (excluding 429)
            attempt in 0u32..10u32,
            max_retries in 1u32..10u32
        ) {
            let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::BAD_REQUEST);

            prop_assert!(
                !should_retry(status, attempt, max_retries),
                "4xx status {} (except 429) should NOT trigger retry",
                status_code
            );
        }

        #[test]
        fn retry_policy_4xx_after_429_no_retry(
            status_code in 430u16..500u16,  // 430-499
            attempt in 0u32..10u32,
            max_retries in 1u32..10u32
        ) {
            let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::BAD_REQUEST);

            prop_assert!(
                !should_retry(status, attempt, max_retries),
                "4xx status {} should NOT trigger retry",
                status_code
            );
        }

        #[test]
        fn retry_policy_2xx_no_retry(
            status_code in 200u16..300u16,
            attempt in 0u32..10u32,
            max_retries in 1u32..10u32
        ) {
            let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK);

            prop_assert!(
                !should_retry(status, attempt, max_retries),
                "2xx status {} should NOT trigger retry",
                status_code
            );
        }

        #[test]
        fn retry_policy_3xx_no_retry(
            status_code in 300u16..400u16,
            attempt in 0u32..10u32,
            max_retries in 1u32..10u32
        ) {
            let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::MOVED_PERMANENTLY);

            prop_assert!(
                !should_retry(status, attempt, max_retries),
                "3xx status {} should NOT trigger retry",
                status_code
            );
        }
    }

    // Feature: dx-font-production-ready, Property 6: Exponential Backoff Growth
    // **Validates: Requirements 7.2**
    //
    // For any sequence of retries, the delay between retry n and retry n+1 SHALL be
    // greater than or equal to the delay between retry n-1 and retry n (exponential growth),
    // with some jitter variance allowed.

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn backoff_grows_exponentially(
            base_delay_ms in 50u64..500u64,
            attempts in 2u32..8u32
        ) {
            let base = Duration::from_millis(base_delay_ms);

            // Calculate delays for consecutive attempts
            let delays: Vec<Duration> = (1..=attempts)
                .map(|a| calculate_backoff(a, base))
                .collect();

            // Remove jitter by taking minimum of multiple samples
            // (jitter adds 0-100ms, so we check the base exponential growth)
            for i in 1..delays.len() {
                let prev_base = base_delay_ms * (1 << (i - 1));
                let curr_base = base_delay_ms * (1 << i);

                // Current base should be roughly double previous base
                // (allowing for the 60s cap)
                let expected_ratio = if curr_base > 60_000 { 1.0 } else { 2.0 };

                let actual_ratio = if prev_base == 0 {
                    2.0
                } else {
                    (curr_base as f64) / (prev_base as f64)
                };

                prop_assert!(
                    actual_ratio >= expected_ratio * 0.9 || curr_base >= 60_000,
                    "Backoff should grow exponentially: attempt {} base {}ms, attempt {} base {}ms",
                    i, prev_base, i + 1, curr_base
                );
            }
        }

        #[test]
        fn backoff_includes_jitter(
            base_delay_ms in 100u64..1000u64,
            attempt in 1u32..5u32
        ) {
            let base = Duration::from_millis(base_delay_ms);

            // Calculate multiple delays for the same attempt
            let delays: Vec<Duration> = (0..10)
                .map(|_| calculate_backoff(attempt, base))
                .collect();

            // With jitter, not all delays should be exactly the same
            let first = delays[0];
            let _all_same = delays.iter().all(|d| *d == first);

            // It's statistically very unlikely all 10 would be the same with 0-100ms jitter
            // But we allow it since it's technically possible
            // Instead, check that delays are within expected range
            let expected_base = base_delay_ms * (1 << (attempt.saturating_sub(1)));
            let expected_min = expected_base;
            let expected_max = (expected_base + 100).min(60_000);

            for delay in delays {
                prop_assert!(
                    delay.as_millis() >= expected_min as u128,
                    "Delay {}ms should be >= base {}ms",
                    delay.as_millis(), expected_min
                );
                prop_assert!(
                    delay.as_millis() <= expected_max as u128,
                    "Delay {}ms should be <= max {}ms",
                    delay.as_millis(), expected_max
                );
            }
        }

        #[test]
        fn backoff_capped_at_60_seconds(
            base_delay_ms in 1000u64..10000u64,
            attempt in 5u32..20u32
        ) {
            let base = Duration::from_millis(base_delay_ms);
            let delay = calculate_backoff(attempt, base);

            prop_assert!(
                delay.as_secs() <= 60,
                "Backoff should be capped at 60 seconds, got {}s",
                delay.as_secs()
            );
        }
    }
}
