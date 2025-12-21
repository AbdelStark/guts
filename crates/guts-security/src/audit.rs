//! Audit logging for security events.
//!
//! This module provides comprehensive audit logging capabilities for tracking
//! security-relevant events in the Guts platform.

use crate::error::{Result, SecurityError};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of audit entries to keep in memory.
const MAX_ENTRIES: usize = 100_000;

/// Types of security events that can be audited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication events
    /// User login attempt.
    Login,
    /// User logout.
    Logout,
    /// Failed login attempt.
    LoginFailed,
    /// API token created.
    TokenCreated,
    /// API token revoked.
    TokenRevoked,
    /// API token used.
    TokenUsed,

    // Authorization events
    /// Permission granted to user/team.
    PermissionGranted,
    /// Permission revoked from user/team.
    PermissionRevoked,
    /// Permission check failed.
    PermissionDenied,
    /// Access denied due to authorization.
    AccessDenied,

    // Repository events
    /// Repository created.
    RepoCreated,
    /// Repository deleted.
    RepoDeleted,
    /// Repository visibility changed.
    RepoVisibilityChanged,
    /// Branch protection rule changed.
    BranchProtectionChanged,

    // Key management events
    /// Cryptographic key rotated.
    KeyRotated,
    /// Key revoked.
    KeyRevoked,
    /// Key accessed.
    KeyAccessed,
    /// Key generation.
    KeyGenerated,

    // System events
    /// Configuration changed.
    ConfigChanged,
    /// Rate limit exceeded.
    RateLimitExceeded,
    /// Suspicious activity detected.
    SuspiciousActivity,
    /// System startup.
    SystemStartup,
    /// System shutdown.
    SystemShutdown,

    // Git operations
    /// Git push operation.
    GitPush,
    /// Git clone/fetch operation.
    GitFetch,
    /// Force push detected.
    ForcePush,

    // Collaboration events
    /// Pull request created.
    PullRequestCreated,
    /// Pull request merged.
    PullRequestMerged,
    /// Issue created.
    IssueCreated,

    // Organization events
    /// Organization created.
    OrgCreated,
    /// Organization member added.
    OrgMemberAdded,
    /// Organization member removed.
    OrgMemberRemoved,
    /// Team created.
    TeamCreated,
}

impl AuditEventType {
    /// Returns the severity level of this event type.
    pub fn severity(&self) -> AuditSeverity {
        match self {
            // Critical events
            AuditEventType::KeyRotated
            | AuditEventType::KeyRevoked
            | AuditEventType::RepoDeleted
            | AuditEventType::SuspiciousActivity
            | AuditEventType::ForcePush => AuditSeverity::Critical,

            // High severity events
            AuditEventType::LoginFailed
            | AuditEventType::PermissionDenied
            | AuditEventType::AccessDenied
            | AuditEventType::RateLimitExceeded
            | AuditEventType::TokenRevoked
            | AuditEventType::BranchProtectionChanged => AuditSeverity::High,

            // Medium severity events
            AuditEventType::Login
            | AuditEventType::Logout
            | AuditEventType::TokenCreated
            | AuditEventType::PermissionGranted
            | AuditEventType::PermissionRevoked
            | AuditEventType::RepoCreated
            | AuditEventType::RepoVisibilityChanged
            | AuditEventType::ConfigChanged
            | AuditEventType::OrgCreated
            | AuditEventType::OrgMemberAdded
            | AuditEventType::OrgMemberRemoved
            | AuditEventType::TeamCreated => AuditSeverity::Medium,

            // Low severity events
            AuditEventType::TokenUsed
            | AuditEventType::KeyAccessed
            | AuditEventType::KeyGenerated
            | AuditEventType::SystemStartup
            | AuditEventType::SystemShutdown
            | AuditEventType::GitPush
            | AuditEventType::GitFetch
            | AuditEventType::PullRequestCreated
            | AuditEventType::PullRequestMerged
            | AuditEventType::IssueCreated => AuditSeverity::Low,
        }
    }
}

/// Severity levels for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    /// Low severity - informational events.
    Low = 0,
    /// Medium severity - notable events.
    Medium = 1,
    /// High severity - security-relevant events.
    High = 2,
    /// Critical severity - immediate attention required.
    Critical = 3,
}

/// An audit event representing a security-relevant action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Type of the event.
    pub event_type: AuditEventType,
    /// Actor who triggered the event (public key, username, or system).
    pub actor: String,
    /// Resource affected by the event.
    pub resource: String,
    /// Result of the action (success, failure reason).
    pub result: String,
    /// IP address of the request origin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// User agent of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl AuditEvent {
    /// Creates a new audit event.
    pub fn new(
        event_type: AuditEventType,
        actor: impl Into<String>,
        resource: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self {
            event_type,
            actor: actor.into(),
            resource: resource.into(),
            result: result.into(),
            ip_address: None,
            user_agent: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the IP address for the event.
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Sets the user agent for the event.
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Sets additional metadata for the event.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// A stored audit log entry with ID and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique ID of the entry.
    pub id: u64,
    /// Unix timestamp when the event occurred.
    pub timestamp: u64,
    /// The audit event.
    #[serde(flatten)]
    pub event: AuditEvent,
    /// Signature of the entry (if signed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl AuditEntry {
    /// Returns the severity of this entry.
    pub fn severity(&self) -> AuditSeverity {
        self.event.event_type.severity()
    }

    /// Returns the canonical bytes for signing.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        // Create a deterministic representation for signing
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(
            serde_json::to_string(&self.event)
                .unwrap_or_default()
                .as_bytes(),
        );
        bytes
    }
}

/// Query parameters for searching audit logs.
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    /// Filter by event types.
    pub event_types: Option<Vec<AuditEventType>>,
    /// Filter by actor.
    pub actor: Option<String>,
    /// Filter by resource.
    pub resource: Option<String>,
    /// Minimum timestamp.
    pub from_timestamp: Option<u64>,
    /// Maximum timestamp.
    pub to_timestamp: Option<u64>,
    /// Minimum severity.
    pub min_severity: Option<AuditSeverity>,
    /// Maximum number of results.
    pub limit: Option<usize>,
    /// Offset for pagination.
    pub offset: Option<usize>,
}

/// Builder for constructing audit queries.
#[derive(Debug, Clone, Default)]
pub struct AuditQueryBuilder {
    query: AuditQuery,
}

impl AuditQueryBuilder {
    /// Creates a new query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by specific event types.
    pub fn event_types(mut self, types: Vec<AuditEventType>) -> Self {
        self.query.event_types = Some(types);
        self
    }

    /// Filters by actor.
    pub fn actor(mut self, actor: impl Into<String>) -> Self {
        self.query.actor = Some(actor.into());
        self
    }

    /// Filters by resource.
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.query.resource = Some(resource.into());
        self
    }

    /// Filters by time range.
    pub fn time_range(mut self, from: u64, to: u64) -> Self {
        self.query.from_timestamp = Some(from);
        self.query.to_timestamp = Some(to);
        self
    }

    /// Filters by minimum severity.
    pub fn min_severity(mut self, severity: AuditSeverity) -> Self {
        self.query.min_severity = Some(severity);
        self
    }

    /// Sets the maximum number of results.
    pub fn limit(mut self, limit: usize) -> Self {
        self.query.limit = Some(limit);
        self
    }

    /// Sets the offset for pagination.
    pub fn offset(mut self, offset: usize) -> Self {
        self.query.offset = Some(offset);
        self
    }

    /// Builds the query.
    pub fn build(self) -> AuditQuery {
        self.query
    }
}

/// Thread-safe audit log for recording security events.
#[derive(Debug)]
pub struct AuditLog {
    /// Stored entries.
    entries: RwLock<VecDeque<AuditEntry>>,
    /// Next ID counter.
    next_id: AtomicU64,
    /// Maximum entries to store.
    max_entries: usize,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLog {
    /// Creates a new audit log with default capacity.
    pub fn new() -> Self {
        Self::with_capacity(MAX_ENTRIES)
    }

    /// Creates a new audit log with specified capacity.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(VecDeque::with_capacity(max_entries.min(MAX_ENTRIES))),
            next_id: AtomicU64::new(1),
            max_entries,
        }
    }

    /// Records an audit event.
    pub fn record(&self, event: AuditEvent) -> AuditEntry {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = AuditEntry {
            id,
            timestamp,
            event,
            signature: None,
        };

        let mut entries = self.entries.write();

        // Remove oldest entries if at capacity
        while entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry.clone());

        // Log the event
        tracing::info!(
            event_type = ?entry.event.event_type,
            actor = %entry.event.actor,
            resource = %entry.event.resource,
            result = %entry.event.result,
            severity = ?entry.severity(),
            "audit event recorded"
        );

        entry
    }

    /// Gets an entry by ID.
    pub fn get(&self, id: u64) -> Result<AuditEntry> {
        self.entries
            .read()
            .iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or_else(|| SecurityError::AuditLogNotFound(id.to_string()))
    }

    /// Queries entries matching the given criteria.
    pub fn query(&self, query: &AuditQuery) -> Vec<AuditEntry> {
        let entries = self.entries.read();

        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| {
                // Filter by event types
                if let Some(ref types) = query.event_types {
                    if !types.contains(&e.event.event_type) {
                        return false;
                    }
                }

                // Filter by actor
                if let Some(ref actor) = query.actor {
                    if !e.event.actor.contains(actor) {
                        return false;
                    }
                }

                // Filter by resource
                if let Some(ref resource) = query.resource {
                    if !e.event.resource.contains(resource) {
                        return false;
                    }
                }

                // Filter by timestamp range
                if let Some(from) = query.from_timestamp {
                    if e.timestamp < from {
                        return false;
                    }
                }
                if let Some(to) = query.to_timestamp {
                    if e.timestamp > to {
                        return false;
                    }
                }

                // Filter by severity
                if let Some(min_sev) = query.min_severity {
                    if e.severity() < min_sev {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);

        filtered.into_iter().skip(offset).take(limit).collect()
    }

    /// Returns the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Returns whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Returns recent entries up to the specified limit.
    pub fn recent(&self, limit: usize) -> Vec<AuditEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Returns entries by severity level.
    pub fn by_severity(&self, severity: AuditSeverity, limit: usize) -> Vec<AuditEntry> {
        self.query(
            &AuditQueryBuilder::new()
                .min_severity(severity)
                .limit(limit)
                .build(),
        )
    }

    /// Clears all entries (for testing).
    #[cfg(test)]
    pub fn clear(&self) {
        self.entries.write().clear();
    }

    /// Exports entries as JSON.
    pub fn export_json(&self) -> Result<String> {
        let entries = self.entries.read();
        serde_json::to_string_pretty(&*entries).map_err(SecurityError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_audit_event() {
        let event = AuditEvent::new(AuditEventType::Login, "user123", "session", "success");

        assert_eq!(event.event_type, AuditEventType::Login);
        assert_eq!(event.actor, "user123");
        assert_eq!(event.resource, "session");
        assert_eq!(event.result, "success");
    }

    #[test]
    fn test_audit_event_with_metadata() {
        let event = AuditEvent::new(AuditEventType::Login, "user123", "session", "success")
            .with_ip("127.0.0.1")
            .with_user_agent("curl/7.64.1")
            .with_metadata(serde_json::json!({"mfa": true}));

        assert_eq!(event.ip_address, Some("127.0.0.1".to_string()));
        assert_eq!(event.user_agent, Some("curl/7.64.1".to_string()));
        assert_eq!(event.metadata["mfa"], true);
    }

    #[test]
    fn test_record_and_get_entry() {
        let log = AuditLog::new();
        let event = AuditEvent::new(AuditEventType::Login, "user123", "session", "success");

        let entry = log.record(event);
        assert!(entry.id > 0);
        assert!(entry.timestamp > 0);

        let retrieved = log.get(entry.id).unwrap();
        assert_eq!(retrieved.id, entry.id);
        assert_eq!(retrieved.event.actor, "user123");
    }

    #[test]
    fn test_query_by_event_type() {
        let log = AuditLog::new();

        log.record(AuditEvent::new(
            AuditEventType::Login,
            "user1",
            "session",
            "success",
        ));
        log.record(AuditEvent::new(
            AuditEventType::Logout,
            "user1",
            "session",
            "success",
        ));
        log.record(AuditEvent::new(
            AuditEventType::Login,
            "user2",
            "session",
            "success",
        ));

        let query = AuditQueryBuilder::new()
            .event_types(vec![AuditEventType::Login])
            .build();

        let results = log.query(&query);
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|e| e.event.event_type == AuditEventType::Login));
    }

    #[test]
    fn test_query_by_actor() {
        let log = AuditLog::new();

        log.record(AuditEvent::new(
            AuditEventType::Login,
            "user1",
            "session",
            "success",
        ));
        log.record(AuditEvent::new(
            AuditEventType::Login,
            "user2",
            "session",
            "success",
        ));

        let query = AuditQueryBuilder::new().actor("user1").build();

        let results = log.query(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event.actor, "user1");
    }

    #[test]
    fn test_query_by_severity() {
        let log = AuditLog::new();

        log.record(AuditEvent::new(
            AuditEventType::Login,
            "user1",
            "session",
            "success",
        ));
        log.record(AuditEvent::new(
            AuditEventType::LoginFailed,
            "user1",
            "session",
            "invalid password",
        ));
        log.record(AuditEvent::new(
            AuditEventType::KeyRotated,
            "system",
            "node-key",
            "success",
        ));

        let high_sev = log.by_severity(AuditSeverity::High, 100);
        assert_eq!(high_sev.len(), 2); // LoginFailed and KeyRotated (Critical >= High)

        let critical = log.by_severity(AuditSeverity::Critical, 100);
        assert_eq!(critical.len(), 1); // Only KeyRotated
    }

    #[test]
    fn test_pagination() {
        let log = AuditLog::new();

        for i in 0..10 {
            log.record(AuditEvent::new(
                AuditEventType::Login,
                format!("user{}", i),
                "session",
                "success",
            ));
        }

        let query = AuditQueryBuilder::new().limit(3).offset(5).build();

        let results = log.query(&query);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].event.actor, "user5");
    }

    #[test]
    fn test_capacity_limit() {
        let log = AuditLog::with_capacity(5);

        for i in 0..10 {
            log.record(AuditEvent::new(
                AuditEventType::Login,
                format!("user{}", i),
                "session",
                "success",
            ));
        }

        assert_eq!(log.len(), 5);

        // Should contain the last 5 entries
        let recent = log.recent(5);
        assert_eq!(recent[0].event.actor, "user5");
        assert_eq!(recent[4].event.actor, "user9");
    }

    #[test]
    fn test_event_severity() {
        assert_eq!(AuditEventType::Login.severity(), AuditSeverity::Medium);
        assert_eq!(AuditEventType::LoginFailed.severity(), AuditSeverity::High);
        assert_eq!(
            AuditEventType::KeyRotated.severity(),
            AuditSeverity::Critical
        );
        assert_eq!(AuditEventType::GitPush.severity(), AuditSeverity::Low);
    }

    #[test]
    fn test_canonical_bytes() {
        let entry = AuditEntry {
            id: 1,
            timestamp: 1000,
            event: AuditEvent::new(AuditEventType::Login, "user", "session", "success"),
            signature: None,
        };

        let bytes = entry.canonical_bytes();
        assert!(!bytes.is_empty());

        // Same entry should produce same bytes
        let bytes2 = entry.canonical_bytes();
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn test_export_json() {
        let log = AuditLog::new();
        log.record(AuditEvent::new(
            AuditEventType::Login,
            "user",
            "session",
            "success",
        ));

        let json = log.export_json().unwrap();
        // serde uses snake_case so "Login" becomes "login"
        assert!(json.contains("login"));
        assert!(json.contains("user"));
    }
}
