//! Webhook types for event notifications.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Events that can trigger webhooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    /// Push to repository.
    Push,
    /// Pull request opened, closed, merged, etc.
    PullRequest,
    /// Review submitted on a pull request.
    PullRequestReview,
    /// Comment on a pull request.
    PullRequestComment,
    /// Issue opened, closed, etc.
    Issue,
    /// Comment on an issue.
    IssueComment,
    /// Branch or tag created.
    Create,
    /// Branch or tag deleted.
    Delete,
    /// Repository forked.
    Fork,
    /// Repository starred.
    Star,
}

impl WebhookEvent {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "push" => Some(WebhookEvent::Push),
            "pull_request" | "pr" => Some(WebhookEvent::PullRequest),
            "pull_request_review" | "pr_review" => Some(WebhookEvent::PullRequestReview),
            "pull_request_comment" | "pr_comment" => Some(WebhookEvent::PullRequestComment),
            "issue" | "issues" => Some(WebhookEvent::Issue),
            "issue_comment" => Some(WebhookEvent::IssueComment),
            "create" => Some(WebhookEvent::Create),
            "delete" => Some(WebhookEvent::Delete),
            "fork" => Some(WebhookEvent::Fork),
            "star" => Some(WebhookEvent::Star),
            _ => None,
        }
    }

    /// Get all available events.
    pub fn all() -> Vec<WebhookEvent> {
        vec![
            WebhookEvent::Push,
            WebhookEvent::PullRequest,
            WebhookEvent::PullRequestReview,
            WebhookEvent::PullRequestComment,
            WebhookEvent::Issue,
            WebhookEvent::IssueComment,
            WebhookEvent::Create,
            WebhookEvent::Delete,
            WebhookEvent::Fork,
            WebhookEvent::Star,
        ]
    }
}

impl std::fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookEvent::Push => write!(f, "push"),
            WebhookEvent::PullRequest => write!(f, "pull_request"),
            WebhookEvent::PullRequestReview => write!(f, "pull_request_review"),
            WebhookEvent::PullRequestComment => write!(f, "pull_request_comment"),
            WebhookEvent::Issue => write!(f, "issue"),
            WebhookEvent::IssueComment => write!(f, "issue_comment"),
            WebhookEvent::Create => write!(f, "create"),
            WebhookEvent::Delete => write!(f, "delete"),
            WebhookEvent::Fork => write!(f, "fork"),
            WebhookEvent::Star => write!(f, "star"),
        }
    }
}

/// A webhook subscription for a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Unique webhook ID.
    pub id: u64,
    /// Repository key (e.g., "owner/repo").
    pub repo_key: String,
    /// Callback URL for webhook delivery.
    pub url: String,
    /// Optional HMAC secret for payload signing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Events that trigger this webhook.
    pub events: HashSet<WebhookEvent>,
    /// Whether the webhook is active.
    pub active: bool,
    /// Content type for delivery (application/json or application/x-www-form-urlencoded).
    pub content_type: String,
    /// Whether to send SSL verification.
    pub insecure_ssl: bool,
    /// When the webhook was created (Unix timestamp).
    pub created_at: u64,
    /// When the webhook was last updated (Unix timestamp).
    pub updated_at: u64,
    /// Number of recent deliveries.
    pub delivery_count: u64,
    /// Number of failed deliveries.
    pub failure_count: u64,
}

impl Webhook {
    /// Create a new webhook.
    pub fn new(id: u64, repo_key: String, url: String, events: HashSet<WebhookEvent>) -> Self {
        let now = Self::now();
        Self {
            id,
            repo_key,
            url,
            secret: None,
            events,
            active: true,
            content_type: "application/json".into(),
            insecure_ssl: false,
            created_at: now,
            updated_at: now,
            delivery_count: 0,
            failure_count: 0,
        }
    }

    /// Set the webhook secret for HMAC signing.
    pub fn with_secret(mut self, secret: String) -> Self {
        self.secret = Some(secret);
        self
    }

    /// Check if this webhook should fire for an event.
    pub fn should_fire(&self, event: WebhookEvent) -> bool {
        self.active && self.events.contains(&event)
    }

    /// Add an event to trigger this webhook.
    pub fn add_event(&mut self, event: WebhookEvent) {
        self.events.insert(event);
        self.updated_at = Self::now();
    }

    /// Remove an event.
    pub fn remove_event(&mut self, event: WebhookEvent) -> bool {
        let removed = self.events.remove(&event);
        if removed {
            self.updated_at = Self::now();
        }
        removed
    }

    /// Enable the webhook.
    pub fn enable(&mut self) {
        self.active = true;
        self.updated_at = Self::now();
    }

    /// Disable the webhook.
    pub fn disable(&mut self) {
        self.active = false;
        self.updated_at = Self::now();
    }

    /// Record a successful delivery.
    pub fn record_success(&mut self) {
        self.delivery_count += 1;
    }

    /// Record a failed delivery.
    pub fn record_failure(&mut self) {
        self.delivery_count += 1;
        self.failure_count += 1;
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Request to create a webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    /// Callback URL.
    pub url: String,
    /// Optional secret for HMAC signing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Events to trigger the webhook.
    pub events: Vec<String>,
    /// Content type (default: application/json).
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// Whether to verify SSL (default: true, so insecure_ssl is false).
    #[serde(default)]
    pub insecure_ssl: bool,
}

fn default_content_type() -> String {
    "application/json".into()
}

/// Request to update a webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhookRequest {
    /// New callback URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// New secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// New events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// New active status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Webhook delivery payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type.
    pub event: WebhookEvent,
    /// Delivery ID (unique per delivery attempt).
    pub delivery_id: String,
    /// Repository information.
    pub repository: WebhookRepository,
    /// Event-specific payload.
    pub payload: serde_json::Value,
    /// Timestamp of the event.
    pub timestamp: u64,
}

/// Repository information for webhook payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRepository {
    /// Repository key.
    pub key: String,
    /// Repository name.
    pub name: String,
    /// Repository owner.
    pub owner: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_parsing() {
        assert_eq!(WebhookEvent::from_str("push"), Some(WebhookEvent::Push));
        assert_eq!(WebhookEvent::from_str("PUSH"), Some(WebhookEvent::Push));
        assert_eq!(WebhookEvent::from_str("pull_request"), Some(WebhookEvent::PullRequest));
        assert_eq!(WebhookEvent::from_str("pr"), Some(WebhookEvent::PullRequest));
        assert_eq!(WebhookEvent::from_str("invalid"), None);
    }

    #[test]
    fn test_webhook_creation() {
        let mut events = HashSet::new();
        events.insert(WebhookEvent::Push);
        events.insert(WebhookEvent::PullRequest);

        let webhook = Webhook::new(1, "acme/api".into(), "https://example.com/hook".into(), events);

        assert_eq!(webhook.id, 1);
        assert!(webhook.active);
        assert!(webhook.should_fire(WebhookEvent::Push));
        assert!(webhook.should_fire(WebhookEvent::PullRequest));
        assert!(!webhook.should_fire(WebhookEvent::Issue));
    }

    #[test]
    fn test_webhook_disable() {
        let mut events = HashSet::new();
        events.insert(WebhookEvent::Push);

        let mut webhook = Webhook::new(1, "acme/api".into(), "https://example.com/hook".into(), events);

        assert!(webhook.should_fire(WebhookEvent::Push));

        webhook.disable();
        assert!(!webhook.should_fire(WebhookEvent::Push));

        webhook.enable();
        assert!(webhook.should_fire(WebhookEvent::Push));
    }

    #[test]
    fn test_webhook_events() {
        let events = HashSet::new();
        let mut webhook = Webhook::new(1, "acme/api".into(), "https://example.com/hook".into(), events);

        assert!(!webhook.should_fire(WebhookEvent::Push));

        webhook.add_event(WebhookEvent::Push);
        assert!(webhook.should_fire(WebhookEvent::Push));

        webhook.remove_event(WebhookEvent::Push);
        assert!(!webhook.should_fire(WebhookEvent::Push));
    }

    #[test]
    fn test_webhook_delivery_tracking() {
        let events = HashSet::new();
        let mut webhook = Webhook::new(1, "acme/api".into(), "https://example.com/hook".into(), events);

        assert_eq!(webhook.delivery_count, 0);
        assert_eq!(webhook.failure_count, 0);

        webhook.record_success();
        assert_eq!(webhook.delivery_count, 1);
        assert_eq!(webhook.failure_count, 0);

        webhook.record_failure();
        assert_eq!(webhook.delivery_count, 2);
        assert_eq!(webhook.failure_count, 1);
    }
}
