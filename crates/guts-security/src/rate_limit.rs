//! Enhanced rate limiting with security features.
//!
//! This module provides an enhanced rate limiter with support for:
//! - Per-IP, per-user, and per-repository limits
//! - Adaptive rate limiting based on abuse patterns
//! - Suspicious activity detection

use crate::error::{Result, SecurityError};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Rate limit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per window for unauthenticated users.
    pub unauthenticated_limit: u32,
    /// Maximum requests per window for authenticated users.
    pub authenticated_limit: u32,
    /// Window duration in seconds.
    pub window_secs: u64,
    /// Enable adaptive rate limiting.
    pub adaptive_enabled: bool,
    /// Threshold for suspicious activity detection.
    pub suspicious_threshold: u32,
    /// Block duration for suspicious IPs in seconds.
    pub block_duration_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            unauthenticated_limit: 60,
            authenticated_limit: 5000,
            window_secs: 3600,
            adaptive_enabled: true,
            suspicious_threshold: 100,
            block_duration_secs: 3600,
        }
    }
}

/// Request context for rate limiting.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Client IP address.
    pub ip: IpAddr,
    /// User ID if authenticated.
    pub user_id: Option<String>,
    /// Repository key if applicable.
    pub repo_key: Option<String>,
    /// Request path.
    pub path: String,
    /// HTTP method.
    pub method: String,
    /// User agent.
    pub user_agent: Option<String>,
}

impl RequestContext {
    /// Creates a new request context.
    pub fn new(ip: IpAddr, path: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            ip,
            user_id: None,
            repo_key: None,
            path: path.into(),
            method: method.into(),
            user_agent: None,
        }
    }

    /// Sets the user ID.
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Sets the repository key.
    pub fn with_repo(mut self, repo_key: impl Into<String>) -> Self {
        self.repo_key = Some(repo_key.into());
        self
    }

    /// Sets the user agent.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Returns whether the request is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
}

/// Token bucket for rate limiting.
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Available tokens.
    tokens: u32,
    /// Maximum tokens.
    max_tokens: u32,
    /// Last refill timestamp.
    last_refill: u64,
    /// Window duration in seconds.
    window_secs: u64,
}

impl TokenBucket {
    fn new(max_tokens: u32, window_secs: u64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            last_refill: current_timestamp(),
            window_secs,
        }
    }

    fn refill_if_needed(&mut self) {
        let now = current_timestamp();
        if now >= self.last_refill + self.window_secs {
            self.tokens = self.max_tokens;
            self.last_refill = now;
        }
    }

    fn consume(&mut self) -> bool {
        self.refill_if_needed();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn remaining(&mut self) -> u32 {
        self.refill_if_needed();
        self.tokens
    }

    fn reset_time(&self) -> u64 {
        self.last_refill + self.window_secs
    }
}

/// Suspicious activity patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuspiciousPattern {
    /// Too many failed authentication attempts.
    AuthBruteForce,
    /// Rapid sequential requests.
    RapidRequests,
    /// Unusual path patterns.
    PathEnumeration,
    /// Many 4xx errors.
    ErrorSpike,
    /// Credential stuffing pattern.
    CredentialStuffing,
    /// Known malicious user agent.
    MaliciousUserAgent,
}

/// Record of suspicious activity.
#[derive(Debug, Clone)]
struct SuspiciousRecord {
    pattern: SuspiciousPattern,
    count: u32,
    #[allow(dead_code)]
    first_seen: u64,
    last_seen: u64,
}

/// Adaptive rate limiter with abuse detection.
#[derive(Debug)]
pub struct AdaptiveLimiter {
    /// Blocked IPs and their unblock time.
    blocked_ips: RwLock<HashMap<IpAddr, u64>>,
    /// Suspicious activity records by IP.
    suspicious: RwLock<HashMap<IpAddr, Vec<SuspiciousRecord>>>,
    /// Configuration.
    config: RateLimitConfig,
}

impl AdaptiveLimiter {
    /// Creates a new adaptive limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            blocked_ips: RwLock::new(HashMap::new()),
            suspicious: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Checks if an IP is blocked.
    pub fn is_blocked(&self, ip: &IpAddr) -> bool {
        let blocked = self.blocked_ips.read();
        if let Some(&unblock_time) = blocked.get(ip) {
            current_timestamp() < unblock_time
        } else {
            false
        }
    }

    /// Blocks an IP for the configured duration.
    pub fn block_ip(&self, ip: IpAddr, reason: SuspiciousPattern) {
        let unblock_time = current_timestamp() + self.config.block_duration_secs;
        self.blocked_ips.write().insert(ip, unblock_time);

        tracing::warn!(
            ip = %ip,
            pattern = ?reason,
            unblock_time = unblock_time,
            "IP blocked due to suspicious activity"
        );
    }

    /// Unblocks an IP.
    pub fn unblock_ip(&self, ip: &IpAddr) {
        self.blocked_ips.write().remove(ip);
    }

    /// Records a suspicious activity.
    pub fn record_suspicious(&self, ip: IpAddr, pattern: SuspiciousPattern) {
        let now = current_timestamp();
        let mut suspicious = self.suspicious.write();

        let records = suspicious.entry(ip).or_default();

        // Find or create record for this pattern
        if let Some(record) = records.iter_mut().find(|r| r.pattern == pattern) {
            record.count += 1;
            record.last_seen = now;
        } else {
            records.push(SuspiciousRecord {
                pattern,
                count: 1,
                first_seen: now,
                last_seen: now,
            });
        }

        // Check if threshold exceeded
        let total_count: u32 = records.iter().map(|r| r.count).sum();
        if total_count >= self.config.suspicious_threshold {
            drop(suspicious);
            self.block_ip(ip, pattern);
        }
    }

    /// Checks the request against abuse patterns.
    pub fn check(&self, ctx: &RequestContext) -> Result<()> {
        if !self.config.adaptive_enabled {
            return Ok(());
        }

        // Check if IP is blocked
        if self.is_blocked(&ctx.ip) {
            let blocked = self.blocked_ips.read();
            let unblock_time = blocked.get(&ctx.ip).copied().unwrap_or(0);
            let _retry_after = unblock_time.saturating_sub(current_timestamp());

            return Err(SecurityError::SuspiciousActivity {
                pattern: "IP temporarily blocked".to_string(),
                actor: ctx.ip.to_string(),
            });
        }

        // Check for known malicious user agents
        if let Some(ref ua) = ctx.user_agent {
            if is_malicious_user_agent(ua) {
                self.record_suspicious(ctx.ip, SuspiciousPattern::MaliciousUserAgent);
            }
        }

        Ok(())
    }

    /// Cleans up expired blocks and old records.
    pub fn cleanup(&self) {
        let now = current_timestamp();

        // Remove expired blocks
        self.blocked_ips
            .write()
            .retain(|_, &mut unblock| unblock > now);

        // Remove old suspicious records (older than 24 hours)
        let cutoff = now.saturating_sub(24 * 60 * 60);
        let mut suspicious = self.suspicious.write();
        for records in suspicious.values_mut() {
            records.retain(|r| r.last_seen > cutoff);
        }
        suspicious.retain(|_, records| !records.is_empty());
    }

    /// Returns the number of currently blocked IPs.
    pub fn blocked_count(&self) -> usize {
        let now = current_timestamp();
        self.blocked_ips
            .read()
            .values()
            .filter(|&&unblock| unblock > now)
            .count()
    }
}

/// Checks if a user agent is known to be malicious.
fn is_malicious_user_agent(ua: &str) -> bool {
    let ua_lower = ua.to_lowercase();

    // Known malicious patterns
    let malicious_patterns = [
        "sqlmap",
        "nikto",
        "nessus",
        "nmap",
        "masscan",
        "zgrab",
        "gobuster",
        "dirbuster",
        "nuclei",
        "wpscan",
    ];

    malicious_patterns.iter().any(|p| ua_lower.contains(p))
}

/// Enhanced rate limiter with multiple limit types.
#[derive(Debug)]
pub struct EnhancedRateLimiter {
    /// Per-IP limits.
    ip_limits: RwLock<HashMap<IpAddr, TokenBucket>>,
    /// Per-user limits.
    user_limits: RwLock<HashMap<String, TokenBucket>>,
    /// Per-repository limits.
    repo_limits: RwLock<HashMap<String, TokenBucket>>,
    /// Adaptive limiter.
    adaptive: AdaptiveLimiter,
    /// Configuration.
    config: RateLimitConfig,
}

impl EnhancedRateLimiter {
    /// Creates a new enhanced rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            ip_limits: RwLock::new(HashMap::new()),
            user_limits: RwLock::new(HashMap::new()),
            repo_limits: RwLock::new(HashMap::new()),
            adaptive: AdaptiveLimiter::new(config.clone()),
            config,
        }
    }

    /// Checks all applicable rate limits.
    pub fn check(&self, ctx: &RequestContext) -> Result<RateLimitInfo> {
        // First check adaptive limiter
        self.adaptive.check(ctx)?;

        // Check IP limit
        let ip_result = self.check_ip(&ctx.ip, ctx.is_authenticated());

        // Check user limit if authenticated
        let user_result = ctx.user_id.as_ref().map(|user_id| self.check_user(user_id));

        // Check repo limit if applicable
        let repo_result = ctx
            .repo_key
            .as_ref()
            .map(|repo_key| self.check_repo(repo_key));

        // If any limit is exceeded, return error
        if !ip_result.allowed {
            return Err(SecurityError::RateLimitExceeded {
                resource: "ip".to_string(),
                retry_after: ip_result.reset - current_timestamp(),
            });
        }

        if let Some(ref user) = user_result {
            if !user.allowed {
                return Err(SecurityError::RateLimitExceeded {
                    resource: "user".to_string(),
                    retry_after: user.reset - current_timestamp(),
                });
            }
        }

        if let Some(ref repo) = repo_result {
            if !repo.allowed {
                return Err(SecurityError::RateLimitExceeded {
                    resource: "repository".to_string(),
                    retry_after: repo.reset - current_timestamp(),
                });
            }
        }

        // Return the most restrictive limit info
        Ok(RateLimitInfo {
            allowed: true,
            limit: ip_result.limit,
            remaining: ip_result.remaining,
            reset: ip_result.reset,
            resource: "ip".to_string(),
        })
    }

    /// Checks IP-based rate limit.
    fn check_ip(&self, ip: &IpAddr, authenticated: bool) -> RateLimitInfo {
        let limit = if authenticated {
            self.config.authenticated_limit
        } else {
            self.config.unauthenticated_limit
        };

        let mut limits = self.ip_limits.write();
        let bucket = limits
            .entry(*ip)
            .or_insert_with(|| TokenBucket::new(limit, self.config.window_secs));

        let allowed = bucket.consume();
        RateLimitInfo {
            allowed,
            limit,
            remaining: bucket.remaining(),
            reset: bucket.reset_time(),
            resource: "ip".to_string(),
        }
    }

    /// Checks user-based rate limit.
    fn check_user(&self, user_id: &str) -> RateLimitInfo {
        let limit = self.config.authenticated_limit;

        let mut limits = self.user_limits.write();
        let bucket = limits
            .entry(user_id.to_string())
            .or_insert_with(|| TokenBucket::new(limit, self.config.window_secs));

        let allowed = bucket.consume();
        RateLimitInfo {
            allowed,
            limit,
            remaining: bucket.remaining(),
            reset: bucket.reset_time(),
            resource: "user".to_string(),
        }
    }

    /// Checks repository-based rate limit.
    fn check_repo(&self, repo_key: &str) -> RateLimitInfo {
        // Repos get higher limits
        let limit = self.config.authenticated_limit * 2;

        let mut limits = self.repo_limits.write();
        let bucket = limits
            .entry(repo_key.to_string())
            .or_insert_with(|| TokenBucket::new(limit, self.config.window_secs));

        let allowed = bucket.consume();
        RateLimitInfo {
            allowed,
            limit,
            remaining: bucket.remaining(),
            reset: bucket.reset_time(),
            resource: "repository".to_string(),
        }
    }

    /// Records a failed authentication attempt.
    pub fn record_auth_failure(&self, ip: IpAddr) {
        self.adaptive
            .record_suspicious(ip, SuspiciousPattern::AuthBruteForce);
    }

    /// Records a suspicious request pattern.
    pub fn record_suspicious(&self, ip: IpAddr, pattern: SuspiciousPattern) {
        self.adaptive.record_suspicious(ip, pattern);
    }

    /// Cleans up expired state.
    pub fn cleanup(&self) {
        self.adaptive.cleanup();

        // Clean up IP limits (remove entries older than 2x window)
        let cutoff = current_timestamp() - (self.config.window_secs * 2);
        self.ip_limits
            .write()
            .retain(|_, bucket| bucket.last_refill > cutoff);
        self.user_limits
            .write()
            .retain(|_, bucket| bucket.last_refill > cutoff);
        self.repo_limits
            .write()
            .retain(|_, bucket| bucket.last_refill > cutoff);
    }

    /// Returns the adaptive limiter for direct access.
    pub fn adaptive(&self) -> &AdaptiveLimiter {
        &self.adaptive
    }
}

/// Information about a rate limit check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Maximum requests in the window.
    pub limit: u32,
    /// Remaining requests in the window.
    pub remaining: u32,
    /// Unix timestamp when the window resets.
    pub reset: u64,
    /// Resource type (ip, user, repo).
    pub resource: String,
}

impl RateLimitInfo {
    /// Returns headers for the rate limit response.
    pub fn headers(&self) -> Vec<(String, String)> {
        vec![
            ("X-RateLimit-Limit".to_string(), self.limit.to_string()),
            (
                "X-RateLimit-Remaining".to_string(),
                self.remaining.to_string(),
            ),
            ("X-RateLimit-Reset".to_string(), self.reset.to_string()),
            ("X-RateLimit-Resource".to_string(), self.resource.clone()),
        ]
    }
}

/// Gets the current Unix timestamp.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    }

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(5, 3600);

        // Should have 5 tokens
        assert_eq!(bucket.remaining(), 5);

        // Consume 3
        assert!(bucket.consume());
        assert!(bucket.consume());
        assert!(bucket.consume());
        assert_eq!(bucket.remaining(), 2);

        // Consume remaining
        assert!(bucket.consume());
        assert!(bucket.consume());
        assert_eq!(bucket.remaining(), 0);

        // Should fail now
        assert!(!bucket.consume());
    }

    #[test]
    fn test_rate_limiter_unauthenticated() {
        let config = RateLimitConfig {
            unauthenticated_limit: 3,
            authenticated_limit: 10,
            window_secs: 3600,
            ..Default::default()
        };

        let limiter = EnhancedRateLimiter::new(config);
        let ctx = RequestContext::new(test_ip(), "/api/test", "GET");

        // Should allow 3 requests
        assert!(limiter.check(&ctx).is_ok());
        assert!(limiter.check(&ctx).is_ok());
        assert!(limiter.check(&ctx).is_ok());

        // 4th should fail
        let result = limiter.check(&ctx);
        assert!(matches!(
            result,
            Err(SecurityError::RateLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_rate_limiter_authenticated() {
        let config = RateLimitConfig {
            unauthenticated_limit: 3,
            authenticated_limit: 5,
            window_secs: 3600,
            ..Default::default()
        };

        let limiter = EnhancedRateLimiter::new(config);
        let ctx = RequestContext::new(test_ip(), "/api/test", "GET").with_user("user123");

        // Should allow 5 requests (authenticated limit)
        for _ in 0..5 {
            assert!(limiter.check(&ctx).is_ok());
        }

        // 6th should fail
        let result = limiter.check(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_limit_info_headers() {
        let info = RateLimitInfo {
            allowed: true,
            limit: 5000,
            remaining: 4999,
            reset: 1234567890,
            resource: "ip".to_string(),
        };

        let headers = info.headers();
        assert_eq!(headers.len(), 4);
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Limit" && v == "5000"));
    }

    #[test]
    fn test_adaptive_limiter_blocking() {
        let config = RateLimitConfig {
            suspicious_threshold: 3,
            block_duration_secs: 60,
            ..Default::default()
        };

        let limiter = AdaptiveLimiter::new(config);
        let ip = test_ip();

        assert!(!limiter.is_blocked(&ip));

        // Record suspicious activity
        limiter.record_suspicious(ip, SuspiciousPattern::AuthBruteForce);
        limiter.record_suspicious(ip, SuspiciousPattern::AuthBruteForce);
        assert!(!limiter.is_blocked(&ip));

        // Third should trigger block
        limiter.record_suspicious(ip, SuspiciousPattern::AuthBruteForce);
        assert!(limiter.is_blocked(&ip));
    }

    #[test]
    fn test_malicious_user_agent_detection() {
        assert!(is_malicious_user_agent("sqlmap/1.5"));
        assert!(is_malicious_user_agent(
            "Mozilla/5.0 (compatible; Nikto/2.1.5)"
        ));
        assert!(!is_malicious_user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0"
        ));
    }

    #[test]
    fn test_request_context() {
        let ctx = RequestContext::new(test_ip(), "/api/repos", "POST")
            .with_user("alice")
            .with_repo("owner/repo")
            .with_user_agent("git/2.30.0");

        assert!(ctx.is_authenticated());
        assert_eq!(ctx.user_id, Some("alice".to_string()));
        assert_eq!(ctx.repo_key, Some("owner/repo".to_string()));
    }
}
