//! Real-time event types.

use guts_auth::WebhookEvent;
use serde::{Deserialize, Serialize};

/// A real-time event that can be broadcast to connected clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeEvent {
    /// Event type identifier.
    #[serde(rename = "type")]
    pub event_type: String,

    /// Channel this event belongs to.
    pub channel: String,

    /// The underlying event kind.
    pub event: EventKind,

    /// Event payload data.
    pub data: serde_json::Value,

    /// Unix timestamp when the event occurred.
    pub timestamp: u64,

    /// Unique event ID.
    pub event_id: String,
}

impl RealtimeEvent {
    /// Create a new real-time event.
    pub fn new(channel: String, event: EventKind, data: serde_json::Value) -> Self {
        Self {
            event_type: "event".to_string(),
            channel,
            event,
            data,
            timestamp: Self::now(),
            event_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Specific event types for real-time updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    // Repository events
    /// Code pushed to repository.
    Push,
    /// Branch created.
    BranchCreated,
    /// Branch deleted.
    BranchDeleted,
    /// Tag created.
    TagCreated,
    /// Tag deleted.
    TagDeleted,

    // Pull request events
    /// Pull request opened.
    PrOpened,
    /// Pull request closed (without merge).
    PrClosed,
    /// Pull request merged.
    PrMerged,
    /// Pull request updated (new commits).
    PrUpdated,
    /// Pull request reopened.
    PrReopened,
    /// Review submitted.
    PrReview,
    /// Comment added to PR.
    PrComment,

    // Issue events
    /// Issue opened.
    IssueOpened,
    /// Issue closed.
    IssueClosed,
    /// Issue reopened.
    IssueReopened,
    /// Issue updated.
    IssueUpdated,
    /// Comment added to issue.
    IssueComment,

    // Label events
    /// Label added.
    LabelAdded,
    /// Label removed.
    LabelRemoved,

    // Repository metadata
    /// Repository created.
    RepoCreated,
    /// Repository settings updated.
    RepoUpdated,

    // Collaboration events
    /// Collaborator added.
    CollaboratorAdded,
    /// Collaborator removed.
    CollaboratorRemoved,
}

impl EventKind {
    /// Convert from WebhookEvent to EventKind.
    pub fn from_webhook(event: WebhookEvent) -> Self {
        match event {
            WebhookEvent::Push => EventKind::Push,
            WebhookEvent::PullRequest => EventKind::PrOpened,
            WebhookEvent::PullRequestReview => EventKind::PrReview,
            WebhookEvent::PullRequestComment => EventKind::PrComment,
            WebhookEvent::Issue => EventKind::IssueOpened,
            WebhookEvent::IssueComment => EventKind::IssueComment,
            WebhookEvent::Create => EventKind::BranchCreated,
            WebhookEvent::Delete => EventKind::BranchDeleted,
            WebhookEvent::Fork => EventKind::RepoCreated,
            WebhookEvent::Star => EventKind::RepoUpdated,
        }
    }

    /// Get all event kinds.
    pub fn all() -> Vec<EventKind> {
        vec![
            EventKind::Push,
            EventKind::BranchCreated,
            EventKind::BranchDeleted,
            EventKind::TagCreated,
            EventKind::TagDeleted,
            EventKind::PrOpened,
            EventKind::PrClosed,
            EventKind::PrMerged,
            EventKind::PrUpdated,
            EventKind::PrReopened,
            EventKind::PrReview,
            EventKind::PrComment,
            EventKind::IssueOpened,
            EventKind::IssueClosed,
            EventKind::IssueReopened,
            EventKind::IssueUpdated,
            EventKind::IssueComment,
            EventKind::LabelAdded,
            EventKind::LabelRemoved,
            EventKind::RepoCreated,
            EventKind::RepoUpdated,
            EventKind::CollaboratorAdded,
            EventKind::CollaboratorRemoved,
        ]
    }
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventKind::Push => write!(f, "push"),
            EventKind::BranchCreated => write!(f, "branch.created"),
            EventKind::BranchDeleted => write!(f, "branch.deleted"),
            EventKind::TagCreated => write!(f, "tag.created"),
            EventKind::TagDeleted => write!(f, "tag.deleted"),
            EventKind::PrOpened => write!(f, "pr.opened"),
            EventKind::PrClosed => write!(f, "pr.closed"),
            EventKind::PrMerged => write!(f, "pr.merged"),
            EventKind::PrUpdated => write!(f, "pr.updated"),
            EventKind::PrReopened => write!(f, "pr.reopened"),
            EventKind::PrReview => write!(f, "pr.review"),
            EventKind::PrComment => write!(f, "pr.comment"),
            EventKind::IssueOpened => write!(f, "issue.opened"),
            EventKind::IssueClosed => write!(f, "issue.closed"),
            EventKind::IssueReopened => write!(f, "issue.reopened"),
            EventKind::IssueUpdated => write!(f, "issue.updated"),
            EventKind::IssueComment => write!(f, "issue.comment"),
            EventKind::LabelAdded => write!(f, "label.added"),
            EventKind::LabelRemoved => write!(f, "label.removed"),
            EventKind::RepoCreated => write!(f, "repo.created"),
            EventKind::RepoUpdated => write!(f, "repo.updated"),
            EventKind::CollaboratorAdded => write!(f, "collaborator.added"),
            EventKind::CollaboratorRemoved => write!(f, "collaborator.removed"),
        }
    }
}

/// Push event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEventData {
    /// Reference that was pushed (e.g., "refs/heads/main").
    #[serde(rename = "ref")]
    pub ref_name: String,
    /// SHA before the push.
    pub before: String,
    /// SHA after the push.
    pub after: String,
    /// Username of the pusher.
    pub pusher: String,
    /// Number of commits.
    pub commit_count: usize,
}

/// Pull request event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestEventData {
    /// Pull request number.
    pub number: u64,
    /// Pull request title.
    pub title: String,
    /// Author username.
    pub author: String,
    /// Source branch.
    pub source_branch: String,
    /// Target branch.
    pub target_branch: String,
    /// Current state (open, closed, merged).
    pub state: String,
}

/// Issue event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEventData {
    /// Issue number.
    pub number: u64,
    /// Issue title.
    pub title: String,
    /// Author username.
    pub author: String,
    /// Current state (open, closed).
    pub state: String,
}

/// Comment event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentEventData {
    /// Comment ID.
    pub id: u64,
    /// Parent number (PR or Issue number).
    pub parent_number: u64,
    /// Author username.
    pub author: String,
    /// Comment body (truncated for notification).
    pub body_preview: String,
}

/// Review event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEventData {
    /// Review ID.
    pub id: u64,
    /// Pull request number.
    pub pr_number: u64,
    /// Reviewer username.
    pub reviewer: String,
    /// Review state (approved, changes_requested, commented).
    pub state: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let data = serde_json::json!({
            "ref": "refs/heads/main",
            "before": "abc123",
            "after": "def456"
        });

        let event = RealtimeEvent::new("repo:alice/myrepo".to_string(), EventKind::Push, data);

        assert_eq!(event.event_type, "event");
        assert_eq!(event.channel, "repo:alice/myrepo");
        assert_eq!(event.event, EventKind::Push);
        assert!(!event.event_id.is_empty());
    }

    #[test]
    fn test_event_kind_display() {
        assert_eq!(EventKind::Push.to_string(), "push");
        assert_eq!(EventKind::PrOpened.to_string(), "pr.opened");
        assert_eq!(EventKind::IssueComment.to_string(), "issue.comment");
    }

    #[test]
    fn test_event_serialization() {
        let data = serde_json::json!({"test": "value"});
        let event = RealtimeEvent::new("repo:test/repo".to_string(), EventKind::Push, data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"event\""));
        assert!(json.contains("\"channel\":\"repo:test/repo\""));
    }
}
