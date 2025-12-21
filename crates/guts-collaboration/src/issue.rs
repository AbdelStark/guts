//! Issue types and state management.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{CollaborationError, Label, Result};

/// State of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    /// Issue is open.
    Open,
    /// Issue is closed.
    Closed,
}

impl std::fmt::Display for IssueState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueState::Open => write!(f, "open"),
            IssueState::Closed => write!(f, "closed"),
        }
    }
}

/// An issue for tracking bugs, features, and tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique identifier within the store.
    pub id: u64,
    /// Repository key (owner/repo).
    pub repo_key: String,
    /// Issue number within the repository (#1, #2, etc.).
    pub number: u32,
    /// Title of the issue.
    pub title: String,
    /// Description/body of the issue (Markdown).
    pub description: String,
    /// Author's public key (hex encoded).
    pub author: String,
    /// Current state of the issue.
    pub state: IssueState,
    /// Labels applied to this issue.
    pub labels: Vec<Label>,
    /// Unix timestamp when the issue was created.
    pub created_at: u64,
    /// Unix timestamp when the issue was last updated.
    pub updated_at: u64,
    /// Unix timestamp when the issue was closed (if closed).
    pub closed_at: Option<u64>,
    /// Public key of the user who closed the issue (if closed).
    pub closed_by: Option<String>,
}

impl Issue {
    /// Creates a new issue.
    pub fn new(
        id: u64,
        repo_key: impl Into<String>,
        number: u32,
        title: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            repo_key: repo_key.into(),
            number,
            title: title.into(),
            description: description.into(),
            author: author.into(),
            state: IssueState::Open,
            labels: Vec::new(),
            created_at: now,
            updated_at: now,
            closed_at: None,
            closed_by: None,
        }
    }

    /// Returns true if the issue is open.
    pub fn is_open(&self) -> bool {
        self.state == IssueState::Open
    }

    /// Returns true if the issue is closed.
    pub fn is_closed(&self) -> bool {
        self.state == IssueState::Closed
    }

    /// Closes the issue.
    pub fn close(&mut self, closed_by: impl Into<String>) -> Result<()> {
        if self.state == IssueState::Closed {
            return Err(CollaborationError::InvalidStateTransition {
                action: "close".to_string(),
                current_state: self.state.to_string(),
            });
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.state = IssueState::Closed;
        self.closed_at = Some(now);
        self.closed_by = Some(closed_by.into());
        self.updated_at = now;
        Ok(())
    }

    /// Reopens the issue.
    pub fn reopen(&mut self) -> Result<()> {
        if self.state == IssueState::Open {
            return Err(CollaborationError::InvalidStateTransition {
                action: "reopen".to_string(),
                current_state: self.state.to_string(),
            });
        }

        self.state = IssueState::Open;
        self.closed_at = None;
        self.closed_by = None;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }

    /// Updates the title.
    pub fn update_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Updates the description.
    pub fn update_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Adds a label.
    pub fn add_label(&mut self, label: Label) {
        if !self.labels.iter().any(|l| l.name == label.name) {
            self.labels.push(label);
            self.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    /// Removes a label by name.
    pub fn remove_label(&mut self, name: &str) {
        let before = self.labels.len();
        self.labels.retain(|l| l.name != name);
        if self.labels.len() != before {
            self.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_issue() -> Issue {
        Issue::new(
            1,
            "alice/repo",
            1,
            "Bug: Something is broken",
            "Steps to reproduce...",
            "alice_pubkey",
        )
    }

    #[test]
    fn test_issue_creation() {
        let issue = create_test_issue();
        assert_eq!(issue.number, 1);
        assert_eq!(issue.title, "Bug: Something is broken");
        assert!(issue.is_open());
        assert!(!issue.is_closed());
    }

    #[test]
    fn test_issue_close_and_reopen() {
        let mut issue = create_test_issue();

        issue.close("bob_pubkey").unwrap();
        assert!(issue.is_closed());
        assert!(!issue.is_open());
        assert!(issue.closed_at.is_some());
        assert_eq!(issue.closed_by, Some("bob_pubkey".to_string()));

        issue.reopen().unwrap();
        assert!(issue.is_open());
        assert!(!issue.is_closed());
        assert!(issue.closed_at.is_none());
        assert!(issue.closed_by.is_none());
    }

    #[test]
    fn test_cannot_close_closed_issue() {
        let mut issue = create_test_issue();
        issue.close("bob").unwrap();

        let result = issue.close("alice");
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_reopen_open_issue() {
        let mut issue = create_test_issue();

        let result = issue.reopen();
        assert!(result.is_err());
    }

    #[test]
    fn test_labels() {
        let mut issue = create_test_issue();

        issue.add_label(Label::bug());
        issue.add_label(Label::help_wanted());
        assert_eq!(issue.labels.len(), 2);

        // Adding same label twice doesn't duplicate
        issue.add_label(Label::bug());
        assert_eq!(issue.labels.len(), 2);

        issue.remove_label("bug");
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.labels[0].name, "help wanted");
    }

    #[test]
    fn test_update_title_and_description() {
        let mut issue = create_test_issue();
        let original_updated = issue.updated_at;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        issue.update_title("New title");
        assert_eq!(issue.title, "New title");
        assert!(issue.updated_at >= original_updated);

        issue.update_description("New description");
        assert_eq!(issue.description, "New description");
    }
}
