//! Circuit breaker implementation for provider resilience.
//!
//! Implements the circuit breaker pattern to prevent cascading failures
//! when providers become unavailable or start failing.
//!
//! # Lock Poisoning Recovery
//!
//! All methods that access the internal `RwLock` handle poisoned locks gracefully.
//! If a thread panics while holding the lock, subsequent operations will recover
//! by extracting the inner value and logging a warning. This ensures the circuit
//! breaker remains functional even after thread panics.

use crate::constants::{DEFAULT_FAILURE_THRESHOLD, DEFAULT_RESET_TIMEOUT_SECS};
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use tracing::warn;

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests are allowed.
    Closed,
    /// Failing - requests are rejected.
    Open,
    /// Testing if service recovered - one request allowed.
    HalfOpen,
}

/// Circuit breaker for a single provider.
///
/// The circuit breaker tracks failures and opens the circuit when
/// the failure threshold is reached, preventing further requests
/// until the reset timeout passes.
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: AtomicU64, // Unix timestamp in seconds
    state: RwLock<CircuitState>,

    // Configuration
    failure_threshold: u32,
    reset_timeout_secs: u64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `failure_threshold` - Number of consecutive failures before opening the circuit.
    /// * `reset_timeout` - Duration to wait before transitioning from Open to HalfOpen.
    #[must_use]
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            last_failure: AtomicU64::new(0),
            state: RwLock::new(CircuitState::Closed),
            failure_threshold,
            reset_timeout_secs: reset_timeout.as_secs(),
        }
    }

    /// Create a circuit breaker with default settings (3 failures, 60s timeout).
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(
            DEFAULT_FAILURE_THRESHOLD,
            Duration::from_secs(DEFAULT_RESET_TIMEOUT_SECS),
        )
    }

    /// Check if a request should be allowed.
    ///
    /// Returns `true` if the request should proceed, `false` if it should be rejected.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the internal lock is poisoned (due to a thread panic), this method
    /// recovers by extracting the inner state and logs a warning. The circuit
    /// breaker continues to function normally after recovery.
    pub fn allow_request(&self) -> bool {
        let state = match self.state.read() {
            Ok(guard) => *guard,
            Err(poisoned) => {
                warn!("Circuit breaker lock was poisoned during read, recovering state");
                *poisoned.into_inner()
            }
        };

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if reset timeout has passed
                let last = self.last_failure.load(Ordering::Relaxed);
                let now = Self::current_timestamp();
                // Use >= for the comparison to handle 0 timeout correctly
                if now.saturating_sub(last) >= self.reset_timeout_secs {
                    // Transition to half-open
                    match self.state.write() {
                        Ok(mut guard) => *guard = CircuitState::HalfOpen,
                        Err(poisoned) => {
                            warn!(
                                "Circuit breaker lock was poisoned during write, recovering to HalfOpen"
                            );
                            *poisoned.into_inner() = CircuitState::HalfOpen;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true, // Allow test request
        }
    }

    /// Record a successful request.
    ///
    /// Resets the failure count and closes the circuit.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the internal lock is poisoned, this method recovers by extracting
    /// the inner state and setting it to Closed. This ensures the circuit
    /// breaker can resume normal operation after a thread panic.
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        match self.state.write() {
            Ok(mut guard) => *guard = CircuitState::Closed,
            Err(poisoned) => {
                warn!(
                    "Circuit breaker lock was poisoned during record_success, recovering to Closed state"
                );
                *poisoned.into_inner() = CircuitState::Closed;
            }
        }
    }

    /// Record a failed request.
    ///
    /// Increments the failure count and opens the circuit if the threshold is reached.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the internal lock is poisoned, this method recovers by extracting
    /// the inner state and continuing with the failure recording logic.
    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.last_failure
            .store(Self::current_timestamp(), Ordering::Relaxed);

        let current_state = match self.state.read() {
            Ok(guard) => *guard,
            Err(poisoned) => {
                warn!(
                    "Circuit breaker lock was poisoned during record_failure read, recovering state"
                );
                *poisoned.into_inner()
            }
        };

        if current_state == CircuitState::HalfOpen {
            // Failed during half-open test, go back to open
            match self.state.write() {
                Ok(mut guard) => *guard = CircuitState::Open,
                Err(poisoned) => {
                    warn!(
                        "Circuit breaker lock was poisoned during record_failure write, recovering to Open"
                    );
                    *poisoned.into_inner() = CircuitState::Open;
                }
            }
        } else if count >= self.failure_threshold {
            match self.state.write() {
                Ok(mut guard) => *guard = CircuitState::Open,
                Err(poisoned) => {
                    warn!(
                        "Circuit breaker lock was poisoned during threshold check, recovering to Open"
                    );
                    *poisoned.into_inner() = CircuitState::Open;
                }
            }
        }
    }

    /// Get the current state of the circuit breaker.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the internal lock is poisoned, this method recovers by extracting
    /// the inner state and returning it.
    #[must_use]
    pub fn state(&self) -> CircuitState {
        match self.state.read() {
            Ok(guard) => *guard,
            Err(poisoned) => {
                warn!("Circuit breaker lock was poisoned during state read, recovering");
                *poisoned.into_inner()
            }
        }
    }

    /// Get the current failure count.
    #[must_use]
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Reset the circuit breaker to its initial state.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the internal lock is poisoned, this method recovers by extracting
    /// the inner state and resetting it to Closed.
    pub fn reset(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.last_failure.store(0, Ordering::Relaxed);
        match self.state.write() {
            Ok(mut guard) => *guard = CircuitState::Closed,
            Err(poisoned) => {
                warn!("Circuit breaker lock was poisoned during reset, recovering to Closed state");
                *poisoned.into_inner() = CircuitState::Closed;
            }
        }
    }

    /// Get the current Unix timestamp in seconds.
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));

        // Record 3 failures
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Requests should be rejected
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_success_resets() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));

        // Record 2 failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count(), 2);

        // Success resets the count
        cb.record_success();
        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_success() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(0));

        // Open the circuit
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // With 0 timeout, allow_request should immediately transition to HalfOpen
        let allowed = cb.allow_request();
        assert!(allowed, "Should allow request after timeout");
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Success closes the circuit
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_failure() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(0));

        // Open the circuit
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // With 0 timeout, allow_request should immediately transition to HalfOpen
        let allowed = cb.allow_request();
        assert!(allowed, "Should allow request after timeout");
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Failure goes back to Open
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60));

        // Open the circuit
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Reset
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
        assert!(cb.allow_request());
    }
}
