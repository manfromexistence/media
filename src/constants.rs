//! Constants for dx-media configuration.
//!
//! This module centralizes all magic numbers and default values used throughout
//! the crate, providing documentation for each constant's purpose.

// ═══════════════════════════════════════════════════════════════════════════════
// SEARCH ENGINE CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Multiplier for early exit threshold in quantity search mode.
///
/// When searching in quantity mode, the search engine will stop waiting for
/// slow providers once it has collected `requested_count * EARLY_EXIT_MULTIPLIER`
/// results. This allows faster response times while still providing more results
/// than requested to account for potential duplicates or filtering.
///
/// Example: If user requests 10 results, early exit triggers at 30 results.
pub const EARLY_EXIT_MULTIPLIER: usize = 3;

// ═══════════════════════════════════════════════════════════════════════════════
// CIRCUIT BREAKER CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Default number of consecutive failures before opening the circuit breaker.
///
/// When a provider fails this many times in a row, the circuit breaker opens
/// and subsequent requests are rejected until the reset timeout passes.
/// This prevents cascading failures and reduces load on failing providers.
pub const DEFAULT_FAILURE_THRESHOLD: u32 = 3;

/// Default timeout in seconds before a circuit breaker transitions from Open to HalfOpen.
///
/// After this duration, the circuit breaker allows a single test request through.
/// If the test succeeds, the circuit closes; if it fails, the circuit reopens.
pub const DEFAULT_RESET_TIMEOUT_SECS: u64 = 60;

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMITING CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Default maximum number of requests allowed per rate limit window.
///
/// This is the default rate limit applied to providers that don't specify
/// their own limits. It's conservative to avoid hitting API quotas.
pub const DEFAULT_RATE_LIMIT_REQUESTS: u32 = 100;

/// Default rate limit window duration in seconds.
///
/// Combined with `DEFAULT_RATE_LIMIT_REQUESTS`, this defines the default
/// rate of 100 requests per 60 seconds (~1.67 requests/second).
pub const DEFAULT_RATE_LIMIT_WINDOW_SECS: u64 = 60;

// ═══════════════════════════════════════════════════════════════════════════════
// HTTP RETRY CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Base delay in milliseconds for exponential backoff.
///
/// When a request fails and needs to be retried, the delay before the first
/// retry is `BASE_BACKOFF_MS`. Subsequent retries double this delay
/// (exponential backoff): 1s, 2s, 4s, 8s, etc.
pub const BASE_BACKOFF_MS: u64 = 1000;

/// Maximum jitter in milliseconds added to backoff delays.
///
/// A random value between 0 and `MAX_BACKOFF_JITTER_MS` is added to each
/// backoff delay to prevent thundering herd problems when multiple clients
/// retry simultaneously.
pub const MAX_BACKOFF_JITTER_MS: u64 = 500;
