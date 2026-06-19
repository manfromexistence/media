
# Design Document: DX Media Production Ready

## Overview

This design document outlines the technical approach for making the `dx-media` crate production-ready. The work focuses on code quality improvements, safety enhancements, and documentation updates rather than new features. The goal is to transform a functional prototype into a professional, maintainable codebase suitable for production deployment.

## Architecture

The existing architecture remains unchanged. This effort focuses on improving the implementation quality of existing components: @tree[]

## Components and Interfaces

### 1. Clippy Configuration Refactoring

Current State: 50+ blanket `#![allow(clippy::...)]` attributes in `lib.rs` Target State: Maximum 10 crate-level suppressions for genuinely unavoidable warnings Approach: -Remove all blanket suppressions -Run `cargo clippy -- -D warnings` -For each warning:-Fix the underlying issue (preferred) -Add item-level `#[allow(...)]` with justification comment (if unfixable) -Keep only essential crate-level suppressions (e.g., `module_name_repetitions` for API design) Acceptable Crate-Level Suppressions:
```rust
// These are acceptable at crate level with justification:


#![allow(clippy::module_name_repetitions)] // API design choice: MediaType in media crate



#![allow(clippy::similar_names)] // Domain terms: asset/assets, provider/providers


```

### 2. Circuit Breaker Safe Lock Handling

Current State: Uses `.unwrap()` on `RwLock` operations which panics on lock poisoning Target State: Graceful recovery from lock poisoning Implementation:
```rust
// constants.rs (new file or in circuit_breaker.rs)
/// Default number of consecutive failures before opening the circuit.
/// /// This value balances between:
/// - Too low (1-2): Opens circuit on transient errors /// - Too high (5+): Allows too many failures before protection kicks in /// /// Recommended range: 3-5 for most API providers.
pub const DEFAULT_FAILURE_THRESHOLD: u32 = 3;
/// Default time to wait before attempting recovery (in seconds).
/// /// After this duration, the circuit transitions from Open to HalfOpen, /// allowing a single test request to check if the service recovered.
/// /// Recommended range: 30-120 seconds depending on provider SLA.
pub const DEFAULT_RESET_TIMEOUT_SECS: u64 = 60;
// circuit_breaker.rs impl CircuitBreaker { pub fn allow_request(&self) -> bool { // Use read() with recovery instead of unwrap()
let state = match self.state.read() { Ok(guard) => *guard, Err(poisoned) => { // Lock was poisoned by a panicking thread // Recover by reading the value and resetting tracing::warn!("Circuit breaker lock poisoned, recovering to Closed state");
let guard = poisoned.into_inner();
*guard }
};
// ... rest of logic }
pub fn record_success(&self) { self.failure_count.store(0, Ordering::Relaxed);
match self.state.write() { Ok(mut guard) => *guard = CircuitState::Closed, Err(poisoned) => { tracing::warn!("Circuit breaker lock poisoned during success, recovering");
let mut guard = poisoned.into_inner();
*guard = CircuitState::Closed;
}
}
}
pub fn record_failure(&self) { // Similar pattern for write operations }
}
```

### 3. Documented Constants Module

Current State: Magic numbers scattered throughout code Target State: Centralized, documented constants Implementation:
```rust
// src/constants.rs (new file)
//! Configuration constants for DX Media.
//!
//! All behavioral constants are documented here with their purpose, //! recommended values, and impact on system behavior.
/// Early-exit multiplier for Quantity search mode.
/// /// When searching in Quantity mode, the engine stops querying providers /// once it has gathered `requested_count * EARLY_EXIT_MULTIPLIER` results.
/// This provides a buffer for deduplication and filtering.
/// /// - Value of 3: Gathers 3x requested results before stopping /// - Higher values: More comprehensive but slower /// - Lower values: Faster but may not fill requested count after filtering pub const EARLY_EXIT_MULTIPLIER: usize = 3;
/// Default rate limit: requests per window.
pub const DEFAULT_RATE_LIMIT_REQUESTS: u32 = 100;
/// Default rate limit: window duration in seconds.
pub const DEFAULT_RATE_LIMIT_WINDOW_SECS: u64 = 60;
/// Base delay for exponential backoff in milliseconds.
/// /// Retry delays are calculated as: BASE_BACKOFF_MS * 2^attempt + jitter /// - Attempt 0: ~1000ms /// - Attempt 1: ~2000ms /// - Attempt 2: ~4000ms pub const BASE_BACKOFF_MS: u64 = 1000;
/// Maximum jitter added to backoff delays in milliseconds.
pub const MAX_BACKOFF_JITTER_MS: u64 = 500;
```

### 4. Honest User-Agent

Current State: Fake Chrome browser User-Agent Target State: Honest library identification Implementation:
```rust
// lib.rs /// User agent for API requests.
/// /// Identifies this library honestly to API providers. Some providers /// may require browser-like User-Agents; those should override this /// at the provider level with documented justification.
pub const USER_AGENT: &str = concat!( "dx-media/", env!("CARGO_PKG_VERSION"), " (https://github.com/anthropics/dx-media; Rust media acquisition library)"
);
```

### 5. Integration Tests with Wiremock

Current State: No HTTP-level integration tests Target State: Comprehensive mocked integration tests Test Structure: @tree:tests[] Example Test:
```rust
// tests/integration/nasa_integration_test.rs use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use dx_media::providers::NasaImagesProvider;


#[tokio::test]


async fn test_nasa_search_parses_valid_response() { let mock_server = MockServer::start().await;
Mock::given(method("GET"))
.and(path("/search"))
.and(query_param("q", "mars"))
.respond_with(ResponseTemplate::new(200)
.set_body_json(include_str!("fixtures/nasa_success.json")))
.mount(&mock_server)
.await;
let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
let results = provider.search("mars", 10).await.unwrap();
assert!(!results.is_empty());
assert!(results.iter().all(|a| !a.id.is_empty()));
assert!(results.iter().all(|a| !a.download_url.is_empty()));
}


#[tokio::test]


async fn test_nasa_handles_malformed_response() { let mock_server = MockServer::start().await;
Mock::given(method("GET"))
.respond_with(ResponseTemplate::new(200)
.set_body_string("not valid json"))
.mount(&mock_server)
.await;
let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
let result = provider.search("mars", 10).await;
assert!(result.is_err());
}
```

### 6. Builder Error Handling

Current State: `try_build()` silently returns `None` on validation failure Target State: Explicit error handling with logging Implementation:
```rust
// types.rs impl MediaAssetBuilder { /// Build the media asset, returning None if required fields are missing.
/// /// **Deprecated:** Use `build()` for explicit error handling or /// `build_or_log()` for logged failures.


#[deprecated(since = "1.0.0", note = "Use build() or build_or_log() instead")]



#[must_use]


pub fn try_build(self) -> Option<MediaAsset> { match self.build() { Ok(asset) => Some(asset), Err(e) => { tracing::warn!("MediaAssetBuilder.try_build() failed: {}", e);
None }
}
}
/// Build the media asset, logging any errors before returning None.
/// /// Useful in iterator chains where you want to skip invalid assets /// but still have visibility into failures.


#[must_use]


pub fn build_or_log(self) -> Option<MediaAsset> { match self.build() { Ok(asset) => Some(asset), Err(e) => { tracing::debug!("MediaAssetBuilder validation failed: {}", e);
None }
}
}
}
```

### 7. Dead Code Cleanup

Files to Review: -`http.rs`: Remove or use `timeout` field -Any `#[allow(dead_code)]` with "future use" comments Approach: -Search for `#[allow(dead_code)]` annotations -For each:-If truly unused: remove the code -If needed for future: feature-gate it -If used but marked incorrectly: remove the annotation

## Data Models

No changes to data models. The `MediaAsset` and related types remain unchanged.

## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property 1: Lock Poisoning Recovery

For any circuit breaker instance and any sequence of operations where a lock becomes poisoned, the circuit breaker SHALL recover to a functional state without panicking and continue to allow or reject requests based on its recovered state. Validates: Requirements 2.1, 2.3, 2.4

### Property 2: Provider Response Parsing Correctness

For any provider API response (valid JSON or malformed), parsing SHALL either: -Produce a valid `MediaAsset` with all required fields populated (for valid responses), OR -Return a descriptive error (for invalid/malformed responses) The parser SHALL NOT panic or produce assets with empty required fields. Validates: Requirements 5.3, 5.4

### Property 3: Builder Error Message Specificity

For any `MediaAssetBuilder` instance missing one or more required fields, calling `build()` SHALL return an error that names the specific missing field(s). Validates: Requirements 6.4

## Error Handling

### Lock Poisoning

- Strategy: Recover and continue
- Implementation: Use `into_inner()` on poisoned locks to extract value
- Logging: Warn-level log on recovery
- State: Reset to safe default (Closed for circuit breaker)

### Builder Validation

- Strategy: Explicit errors with field names
- Implementation: `DxError::BuilderValidation { field: String }`
- Logging: Debug-level for `build_or_log()`, Warn-level for deprecated `try_build()`

### Provider Parsing

- Strategy: Return errors, don't panic
- Implementation: Use `?` operator, wrap parsing in Result
- Logging: Debug-level for parsing failures

## Testing Strategy

### Unit Tests

- Test circuit breaker state transitions
- Test builder validation error messages
- Test constant values are within expected ranges

### Property-Based Tests

- Property 1: Simulate lock poisoning scenarios with proptest
- Property 2: Generate random JSON responses and verify parsing behavior
- Property 3: Generate builders with random missing fields, verify error messages

### Integration Tests

- Use wiremock to mock provider APIs
- Test happy path parsing
- Test error handling for malformed responses
- Test rate limiting behavior

### Test Configuration

- Property tests: minimum 100 iterations
- Integration tests: use realistic sample responses from actual APIs
- Coverage target: 80% on provider parsing logic

### Property-Based Testing Framework

- Library: proptest (already in dev-dependencies)
- Iteration count: 100 minimum per property
- Tag format: `// Feature: dx-media-production-ready, Property N: description`
