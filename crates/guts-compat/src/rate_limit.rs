//! Rate limiting with GitHub-compatible headers.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default rate limit (requests per hour).
pub const DEFAULT_RATE_LIMIT: u32 = 5000;

/// Rate limit for unauthenticated requests.
pub const UNAUTHENTICATED_RATE_LIMIT: u32 = 60;

/// Rate limit resource types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitResource {
    /// Core API endpoints.
    Core,
    /// Search endpoints (lower limit).
    Search,
    /// GraphQL endpoint.
    Graphql,
    /// Git operations (clone, fetch, push).
    Git,
    /// Code scanning.
    CodeScanning,
}

impl RateLimitResource {
    /// Get the default limit for this resource.
    pub fn default_limit(&self, authenticated: bool) -> u32 {
        if !authenticated {
            return UNAUTHENTICATED_RATE_LIMIT;
        }

        match self {
            Self::Core => 5000,
            Self::Search => 30,
            Self::Graphql => 5000,
            Self::Git => 5000,
            Self::CodeScanning => 1000,
        }
    }

    /// Get the reset interval for this resource.
    pub fn reset_interval(&self) -> Duration {
        match self {
            Self::Core => Duration::from_secs(3600),         // 1 hour
            Self::Search => Duration::from_secs(60),         // 1 minute
            Self::Graphql => Duration::from_secs(3600),      // 1 hour
            Self::Git => Duration::from_secs(3600),          // 1 hour
            Self::CodeScanning => Duration::from_secs(3600), // 1 hour
        }
    }
}

impl std::fmt::Display for RateLimitResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Search => write!(f, "search"),
            Self::Graphql => write!(f, "graphql"),
            Self::Git => write!(f, "git"),
            Self::CodeScanning => write!(f, "code_scanning"),
        }
    }
}

/// Rate limit state for a specific user/resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitState {
    /// Maximum requests allowed.
    pub limit: u32,
    /// Remaining requests in current window.
    pub remaining: u32,
    /// Unix timestamp when the limit resets.
    pub reset: u64,
    /// Requests used in current window.
    pub used: u32,
    /// Resource type.
    pub resource: RateLimitResource,
}

impl RateLimitState {
    /// Create a new rate limit state.
    pub fn new(limit: u32, resource: RateLimitResource) -> Self {
        let reset = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + resource.reset_interval().as_secs();

        Self {
            limit,
            remaining: limit,
            reset,
            used: 0,
            resource,
        }
    }

    /// Check if the rate limit is exceeded.
    pub fn is_exceeded(&self) -> bool {
        self.remaining == 0 && !self.is_reset()
    }

    /// Check if the window has reset.
    pub fn is_reset(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.reset
    }

    /// Consume one request from the limit.
    pub fn consume(&mut self) -> bool {
        // Reset if window expired
        if self.is_reset() {
            self.reset_window();
        }

        if self.remaining > 0 {
            self.remaining -= 1;
            self.used += 1;
            true
        } else {
            false
        }
    }

    /// Reset the window.
    pub fn reset_window(&mut self) {
        self.remaining = self.limit;
        self.used = 0;
        self.reset = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + self.resource.reset_interval().as_secs();
    }

    /// Get the time until reset in seconds.
    pub fn time_until_reset(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.reset.saturating_sub(now)
    }
}

/// Rate limit response for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    /// Resources with their limits.
    pub resources: RateLimitResources,
    /// Rate limit state for the primary resource.
    pub rate: RateLimitInfo,
}

/// Rate limit resources in response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResources {
    /// Core API limit.
    pub core: RateLimitInfo,
    /// Search API limit.
    pub search: RateLimitInfo,
    /// GraphQL API limit.
    pub graphql: RateLimitInfo,
}

/// Rate limit info for a single resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Maximum requests allowed.
    pub limit: u32,
    /// Remaining requests.
    pub remaining: u32,
    /// Unix timestamp when limit resets.
    pub reset: u64,
    /// Requests used.
    pub used: u32,
}

impl From<&RateLimitState> for RateLimitInfo {
    fn from(state: &RateLimitState) -> Self {
        Self {
            limit: state.limit,
            remaining: state.remaining,
            reset: state.reset,
            used: state.used,
        }
    }
}

/// Rate limit headers for HTTP responses.
#[derive(Debug, Clone, Default)]
pub struct RateLimitHeaders {
    /// X-RateLimit-Limit header value.
    pub limit: String,
    /// X-RateLimit-Remaining header value.
    pub remaining: String,
    /// X-RateLimit-Reset header value.
    pub reset: String,
    /// X-RateLimit-Used header value.
    pub used: String,
    /// X-RateLimit-Resource header value.
    pub resource: String,
}

impl From<&RateLimitState> for RateLimitHeaders {
    fn from(state: &RateLimitState) -> Self {
        Self {
            limit: state.limit.to_string(),
            remaining: state.remaining.to_string(),
            reset: state.reset.to_string(),
            used: state.used.to_string(),
            resource: state.resource.to_string(),
        }
    }
}

/// Rate limiter that tracks limits per user and resource.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// User states keyed by (user_id, resource).
    states: Arc<Mutex<HashMap<(String, RateLimitResource), RateLimitState>>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a rate limit state for a user/resource.
    pub fn get_state(
        &self,
        user_id: &str,
        resource: RateLimitResource,
        authenticated: bool,
    ) -> RateLimitState {
        let mut states = self.states.lock();
        let key = (user_id.to_string(), resource);

        states
            .entry(key)
            .or_insert_with(|| {
                let limit = resource.default_limit(authenticated);
                RateLimitState::new(limit, resource)
            })
            .clone()
    }

    /// Check and consume a request for a user/resource.
    ///
    /// Returns the updated state if allowed, or None if rate limited.
    pub fn check_and_consume(
        &self,
        user_id: &str,
        resource: RateLimitResource,
        authenticated: bool,
    ) -> Option<RateLimitState> {
        let mut states = self.states.lock();
        let key = (user_id.to_string(), resource);

        let state = states.entry(key).or_insert_with(|| {
            let limit = resource.default_limit(authenticated);
            RateLimitState::new(limit, resource)
        });

        if state.consume() {
            Some(state.clone())
        } else {
            None
        }
    }

    /// Get the rate limit response for a user.
    pub fn get_response(&self, user_id: &str, authenticated: bool) -> RateLimitResponse {
        let core = self.get_state(user_id, RateLimitResource::Core, authenticated);
        let search = self.get_state(user_id, RateLimitResource::Search, authenticated);
        let graphql = self.get_state(user_id, RateLimitResource::Graphql, authenticated);

        RateLimitResponse {
            resources: RateLimitResources {
                core: (&core).into(),
                search: (&search).into(),
                graphql: (&graphql).into(),
            },
            rate: (&core).into(),
        }
    }

    /// Clean up expired states to free memory.
    pub fn cleanup(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut states = self.states.lock();
        states.retain(|_, state| {
            // Keep states that haven't expired yet or have been used
            state.reset > now || state.used > 0
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_state() {
        let mut state = RateLimitState::new(100, RateLimitResource::Core);

        assert_eq!(state.limit, 100);
        assert_eq!(state.remaining, 100);
        assert_eq!(state.used, 0);
        assert!(!state.is_exceeded());

        // Consume a request
        assert!(state.consume());
        assert_eq!(state.remaining, 99);
        assert_eq!(state.used, 1);
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let mut state = RateLimitState::new(2, RateLimitResource::Core);

        assert!(state.consume());
        assert!(state.consume());
        assert!(!state.consume()); // Exceeded

        assert!(state.is_exceeded());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new();

        // First request should succeed
        let state = limiter.check_and_consume("user1", RateLimitResource::Core, true);
        assert!(state.is_some());

        // Get state
        let state = limiter.get_state("user1", RateLimitResource::Core, true);
        assert_eq!(state.used, 1);
    }

    #[test]
    fn test_unauthenticated_limit() {
        let limiter = RateLimiter::new();
        let state = limiter.get_state("anon", RateLimitResource::Core, false);

        assert_eq!(state.limit, UNAUTHENTICATED_RATE_LIMIT);
    }

    #[test]
    fn test_rate_limit_headers() {
        let state = RateLimitState::new(5000, RateLimitResource::Core);
        let headers = RateLimitHeaders::from(&state);

        assert_eq!(headers.limit, "5000");
        assert_eq!(headers.remaining, "5000");
        assert_eq!(headers.resource, "core");
    }

    #[test]
    fn test_resource_default_limits() {
        assert_eq!(RateLimitResource::Core.default_limit(true), 5000);
        assert_eq!(RateLimitResource::Search.default_limit(true), 30);
        assert_eq!(RateLimitResource::Core.default_limit(false), 60);
    }
}
