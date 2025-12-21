//! # Resilience Module
//!
//! Production-grade resilience patterns including:
//!
//! - **Retry Policy**: Configurable retry with exponential backoff
//! - **Circuit Breaker**: Fail-fast for failing services
//! - **Timeout Management**: Request and operation timeouts
//! - **Rate Limiting**: Request rate limiting
//!
//! ## Usage
//!
//! ```rust,no_run
//! use guts_node::resilience::{RetryPolicy, CircuitBreaker, TimeoutConfig};
//! use std::time::Duration;
//!
//! let retry = RetryPolicy::default();
//! let circuit_breaker = CircuitBreaker::new(5, 3, Duration::from_secs(30));
//! let timeout = TimeoutConfig::default();
//! ```

use parking_lot::RwLock;
use std::future::Future;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Error kinds that can be retried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryableError {
    /// Network timeout.
    Timeout,
    /// Connection failed.
    ConnectionFailed,
    /// Service temporarily unavailable.
    ServiceUnavailable,
    /// Rate limited.
    RateLimited,
}

/// Retry policy configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier.
    pub multiplier: f64,
    /// Whether to add jitter to delays.
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy.
    pub fn new(max_attempts: u32, initial_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            ..Default::default()
        }
    }

    /// Calculate delay for a given attempt number.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let base_delay_ms = self.initial_delay.as_millis() as f64;
        let delay_ms = base_delay_ms * self.multiplier.powi(attempt as i32 - 1);
        let capped_delay =
            Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f64) as u64);

        if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = 1.0 + (rand::random::<f64>() * 0.25);
            Duration::from_millis((capped_delay.as_millis() as f64 * jitter_factor) as u64)
        } else {
            capped_delay
        }
    }

    /// Execute an operation with retry.
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.max_attempts {
                        tracing::warn!(
                            attempt = attempt,
                            max_attempts = self.max_attempts,
                            error = ?e,
                            "Retry exhausted"
                        );
                        return Err(e);
                    }

                    let delay = self.delay_for_attempt(attempt);
                    tracing::debug!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = ?e,
                        "Retrying after delay"
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally.
    Closed,
    /// Circuit is open, requests fail immediately.
    Open,
    /// Circuit is testing if the service has recovered.
    HalfOpen,
}

/// Circuit breaker for failing services.
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Number of failures before opening.
    failure_threshold: u32,
    /// Number of successes needed to close from half-open.
    success_threshold: u32,
    /// How long to stay open before transitioning to half-open.
    timeout: Duration,
    /// Current state.
    state: RwLock<CircuitState>,
    /// Current failure count.
    failure_count: AtomicU32,
    /// Current success count (in half-open state).
    success_count: AtomicU32,
    /// When the circuit was opened.
    opened_at: RwLock<Option<Instant>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            opened_at: RwLock::new(None),
        }
    }

    /// Get the current state.
    pub fn state(&self) -> CircuitState {
        self.maybe_transition_from_open();
        *self.state.read()
    }

    /// Check if requests should be allowed.
    pub fn allow_request(&self) -> bool {
        self.maybe_transition_from_open();
        let state = *self.state.read();
        matches!(state, CircuitState::Closed | CircuitState::HalfOpen)
    }

    /// Record a successful request.
    pub fn record_success(&self) {
        let state = *self.state.read();
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.success_threshold {
                    self.close();
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failed request.
    pub fn record_failure(&self) {
        let state = *self.state.read();
        match state {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.failure_threshold {
                    self.open();
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit
                self.open();
            }
            CircuitState::Open => {}
        }
    }

    /// Open the circuit.
    fn open(&self) {
        tracing::warn!("Circuit breaker opened");
        *self.state.write() = CircuitState::Open;
        *self.opened_at.write() = Some(Instant::now());
        self.success_count.store(0, Ordering::SeqCst);
    }

    /// Close the circuit.
    fn close(&self) {
        tracing::info!("Circuit breaker closed");
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.opened_at.write() = None;
    }

    /// Check if we should transition from open to half-open.
    fn maybe_transition_from_open(&self) {
        let state = *self.state.read();
        if state != CircuitState::Open {
            return;
        }

        if let Some(opened_at) = *self.opened_at.read() {
            if opened_at.elapsed() >= self.timeout {
                tracing::info!("Circuit breaker transitioning to half-open");
                *self.state.write() = CircuitState::HalfOpen;
                self.success_count.store(0, Ordering::SeqCst);
            }
        }
    }

    /// Execute an operation with circuit breaker protection.
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        if !self.allow_request() {
            return Err(CircuitBreakerError::Open);
        }

        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitBreakerError::Inner(e))
            }
        }
    }
}

/// Circuit breaker error wrapper.
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open, request was not attempted.
    Open,
    /// The inner operation failed.
    Inner(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Circuit breaker is open"),
            Self::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}

/// Timeout configuration.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Connection timeout.
    pub connect: Duration,
    /// Read timeout.
    pub read: Duration,
    /// Write timeout.
    pub write: Duration,
    /// Total operation timeout.
    pub total: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(5),
            read: Duration::from_secs(30),
            write: Duration::from_secs(30),
            total: Duration::from_secs(60),
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout config.
    pub fn new(connect: Duration, total: Duration) -> Self {
        Self {
            connect,
            read: total,
            write: total,
            total,
        }
    }

    /// Execute an operation with timeout.
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, TimeoutError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        match tokio::time::timeout(self.total, operation()).await {
            Ok(result) => Ok(result),
            Err(_) => Err(TimeoutError {
                timeout: self.total,
            }),
        }
    }
}

/// Timeout error.
#[derive(Debug)]
pub struct TimeoutError {
    /// The timeout duration that was exceeded.
    pub timeout: Duration,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation timed out after {:?}", self.timeout)
    }
}

impl std::error::Error for TimeoutError {}

/// Rate limiter using token bucket algorithm.
#[derive(Debug)]
pub struct RateLimiter {
    /// Maximum tokens (requests) per window.
    max_tokens: u32,
    /// Current tokens available.
    tokens: AtomicU32,
    /// Last refill time (unix timestamp in millis).
    last_refill: AtomicU64,
    /// Refill rate (tokens per second).
    refill_rate: f64,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(requests_per_second: f64) -> Self {
        let max_tokens = (requests_per_second.ceil() as u32).max(1);
        Self {
            max_tokens,
            tokens: AtomicU32::new(max_tokens),
            last_refill: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            ),
            refill_rate: requests_per_second,
        }
    }

    /// Try to acquire a token (permission to make a request).
    pub fn try_acquire(&self) -> bool {
        self.refill();

        loop {
            let current = self.tokens.load(Ordering::SeqCst);
            if current == 0 {
                return false;
            }
            if self
                .tokens
                .compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let last = self.last_refill.load(Ordering::SeqCst);
        let elapsed_ms = now.saturating_sub(last);
        let elapsed_secs = elapsed_ms as f64 / 1000.0;
        let tokens_to_add = (elapsed_secs * self.refill_rate) as u32;

        if tokens_to_add > 0
            && self
                .last_refill
                .compare_exchange(last, now, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        {
            let current = self.tokens.load(Ordering::SeqCst);
            let new_tokens = (current + tokens_to_add).min(self.max_tokens);
            self.tokens.store(new_tokens, Ordering::SeqCst);
        }
    }

    /// Get remaining tokens.
    pub fn remaining(&self) -> u32 {
        self.refill();
        self.tokens.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: false,
        };

        assert_eq!(policy.delay_for_attempt(0), Duration::ZERO);
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(400));
    }

    #[test]
    fn test_circuit_breaker_states() {
        let cb = CircuitBreaker::new(2, 1, Duration::from_millis(100));

        // Initially closed
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(cb.allow_request());

        // Record success
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(10.0);

        // Should be able to acquire tokens
        for _ in 0..10 {
            assert!(limiter.try_acquire());
        }

        // Should be rate limited
        assert!(!limiter.try_acquire());
    }
}
