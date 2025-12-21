//! Notification types for user alerts.

use serde::{Deserialize, Serialize};

/// A notification for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique notification ID.
    pub id: String,
    /// User who receives this notification.
    pub user_id: String,
    /// Notification type.
    pub notification_type: NotificationType,
    /// Brief title.
    pub title: String,
    /// Notification body/description.
    pub body: String,
    /// URL to navigate to when clicked.
    pub url: Option<String>,
    /// Whether the notification has been read.
    pub read: bool,
    /// When the notification was created (Unix timestamp).
    pub created_at: u64,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: NotificationMetadata,
}

impl Notification {
    /// Create a new notification.
    pub fn new(
        user_id: String,
        notification_type: NotificationType,
        title: String,
        body: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            notification_type,
            title,
            body,
            url: None,
            read: false,
            created_at: Self::now(),
            metadata: NotificationMetadata::default(),
        }
    }

    /// Set the URL.
    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    /// Set metadata.
    pub fn with_metadata(mut self, metadata: NotificationMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Mark as read.
    pub fn mark_read(&mut self) {
        self.read = true;
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Types of notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Mentioned in a comment.
    Mention,
    /// Assigned to an issue or PR.
    Assigned,
    /// Review requested on a PR.
    ReviewRequested,
    /// PR you authored was merged.
    PrMerged,
    /// PR you authored was closed.
    PrClosed,
    /// New comment on your issue/PR.
    NewComment,
    /// Review submitted on your PR.
    ReviewSubmitted,
    /// Issue you authored was closed.
    IssueClosed,
    /// Push to a watched repository.
    Push,
    /// Team added to repository.
    TeamAdded,
    /// Collaborator added.
    CollaboratorAdded,
    /// Workflow completed (CI/CD).
    WorkflowCompleted,
}

impl NotificationType {
    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            NotificationType::Mention => "Mentioned",
            NotificationType::Assigned => "Assigned",
            NotificationType::ReviewRequested => "Review Requested",
            NotificationType::PrMerged => "PR Merged",
            NotificationType::PrClosed => "PR Closed",
            NotificationType::NewComment => "New Comment",
            NotificationType::ReviewSubmitted => "Review Submitted",
            NotificationType::IssueClosed => "Issue Closed",
            NotificationType::Push => "New Push",
            NotificationType::TeamAdded => "Team Added",
            NotificationType::CollaboratorAdded => "Collaborator Added",
            NotificationType::WorkflowCompleted => "Workflow Completed",
        }
    }

    /// Get icon hint for the notification type.
    pub fn icon(&self) -> &'static str {
        match self {
            NotificationType::Mention => "at",
            NotificationType::Assigned => "user",
            NotificationType::ReviewRequested => "eye",
            NotificationType::PrMerged => "git-merge",
            NotificationType::PrClosed => "git-pull-request",
            NotificationType::NewComment => "message",
            NotificationType::ReviewSubmitted => "check-circle",
            NotificationType::IssueClosed => "issue-closed",
            NotificationType::Push => "upload",
            NotificationType::TeamAdded => "users",
            NotificationType::CollaboratorAdded => "user-plus",
            NotificationType::WorkflowCompleted => "activity",
        }
    }
}

/// Additional metadata for notifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationMetadata {
    /// Repository key if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_key: Option<String>,
    /// PR number if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u64>,
    /// Issue number if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_number: Option<u64>,
    /// Username of the actor who triggered this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
}

/// User notification preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// User ID.
    pub user_id: String,
    /// Enabled notification types.
    pub enabled_types: Vec<NotificationType>,
    /// Whether to receive email notifications.
    pub email_enabled: bool,
    /// Whether to receive web push notifications.
    pub push_enabled: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            enabled_types: vec![
                NotificationType::Mention,
                NotificationType::Assigned,
                NotificationType::ReviewRequested,
                NotificationType::PrMerged,
                NotificationType::NewComment,
                NotificationType::ReviewSubmitted,
            ],
            email_enabled: false,
            push_enabled: false,
        }
    }
}

impl NotificationPreferences {
    /// Create preferences for a user with defaults.
    pub fn for_user(user_id: String) -> Self {
        Self {
            user_id,
            ..Default::default()
        }
    }

    /// Check if a notification type is enabled.
    pub fn is_enabled(&self, notification_type: NotificationType) -> bool {
        self.enabled_types.contains(&notification_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            "alice".to_string(),
            NotificationType::Mention,
            "You were mentioned".to_string(),
            "Bob mentioned you in a comment".to_string(),
        );

        assert!(!notification.id.is_empty());
        assert_eq!(notification.user_id, "alice");
        assert_eq!(notification.notification_type, NotificationType::Mention);
        assert!(!notification.read);
    }

    #[test]
    fn test_notification_with_url() {
        let notification = Notification::new(
            "alice".to_string(),
            NotificationType::NewComment,
            "New comment".to_string(),
            "Bob commented on your PR".to_string(),
        )
        .with_url("/alice/myrepo/pull/1#comment-5".to_string());

        assert_eq!(
            notification.url,
            Some("/alice/myrepo/pull/1#comment-5".to_string())
        );
    }

    #[test]
    fn test_notification_mark_read() {
        let mut notification = Notification::new(
            "alice".to_string(),
            NotificationType::Mention,
            "Test".to_string(),
            "Test body".to_string(),
        );

        assert!(!notification.read);
        notification.mark_read();
        assert!(notification.read);
    }

    #[test]
    fn test_notification_preferences() {
        let prefs = NotificationPreferences::for_user("alice".to_string());

        assert!(prefs.is_enabled(NotificationType::Mention));
        assert!(prefs.is_enabled(NotificationType::ReviewRequested));
        assert!(!prefs.is_enabled(NotificationType::Push));
    }

    #[test]
    fn test_notification_type_label() {
        assert_eq!(NotificationType::Mention.label(), "Mentioned");
        assert_eq!(NotificationType::PrMerged.label(), "PR Merged");
    }
}
