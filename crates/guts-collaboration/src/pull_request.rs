//! Pull Request types and state management.

use guts_storage::ObjectId;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{CollaborationError, Label, Result};

/// State of a pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    /// Pull request is open and can be reviewed/merged.
    Open,
    /// Pull request was closed without merging.
    Closed,
    /// Pull request was merged into the target branch.
    Merged,
}

impl std::fmt::Display for PullRequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PullRequestState::Open => write!(f, "open"),
            PullRequestState::Closed => write!(f, "closed"),
            PullRequestState::Merged => write!(f, "merged"),
        }
    }
}

/// A pull request for proposing changes to a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Unique identifier within the store.
    pub id: u64,
    /// Repository key (owner/repo).
    pub repo_key: String,
    /// Pull request number within the repository (#1, #2, etc.).
    pub number: u32,
    /// Title of the pull request.
    pub title: String,
    /// Description/body of the pull request (Markdown).
    pub description: String,
    /// Author's public key (hex encoded).
    pub author: String,
    /// Current state of the pull request.
    pub state: PullRequestState,
    /// Source branch name.
    pub source_branch: String,
    /// Target branch name (usually "main" or "master").
    pub target_branch: String,
    /// Head commit of the source branch.
    pub source_commit: ObjectId,
    /// Head commit of the target branch at PR creation.
    pub target_commit: ObjectId,
    /// Labels applied to this pull request.
    pub labels: Vec<Label>,
    /// Unix timestamp when the PR was created.
    pub created_at: u64,
    /// Unix timestamp when the PR was last updated.
    pub updated_at: u64,
    /// Unix timestamp when the PR was merged (if merged).
    pub merged_at: Option<u64>,
    /// Public key of the user who merged the PR (if merged).
    pub merged_by: Option<String>,
}

impl PullRequest {
    /// Creates a new pull request.
    pub fn new(
        id: u64,
        repo_key: impl Into<String>,
        number: u32,
        title: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
        source_branch: impl Into<String>,
        target_branch: impl Into<String>,
        source_commit: ObjectId,
        target_commit: ObjectId,
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
            state: PullRequestState::Open,
            source_branch: source_branch.into(),
            target_branch: target_branch.into(),
            source_commit,
            target_commit,
            labels: Vec::new(),
            created_at: now,
            updated_at: now,
            merged_at: None,
            merged_by: None,
        }
    }

    /// Returns true if the pull request is open.
    pub fn is_open(&self) -> bool {
        self.state == PullRequestState::Open
    }

    /// Returns true if the pull request is merged.
    pub fn is_merged(&self) -> bool {
        self.state == PullRequestState::Merged
    }

    /// Returns true if the pull request is closed (not merged).
    pub fn is_closed(&self) -> bool {
        self.state == PullRequestState::Closed
    }

    /// Closes the pull request without merging.
    pub fn close(&mut self) -> Result<()> {
        if self.state == PullRequestState::Merged {
            return Err(CollaborationError::InvalidStateTransition {
                action: "close".to_string(),
                current_state: self.state.to_string(),
            });
        }

        self.state = PullRequestState::Closed;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }

    /// Reopens a closed pull request.
    pub fn reopen(&mut self) -> Result<()> {
        if self.state != PullRequestState::Closed {
            return Err(CollaborationError::InvalidStateTransition {
                action: "reopen".to_string(),
                current_state: self.state.to_string(),
            });
        }

        self.state = PullRequestState::Open;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }

    /// Merges the pull request.
    pub fn merge(&mut self, merged_by: impl Into<String>) -> Result<()> {
        if self.state != PullRequestState::Open {
            return Err(CollaborationError::InvalidStateTransition {
                action: "merge".to_string(),
                current_state: self.state.to_string(),
            });
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.state = PullRequestState::Merged;
        self.merged_at = Some(now);
        self.merged_by = Some(merged_by.into());
        self.updated_at = now;
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

    /// Updates the source commit (when new commits are pushed).
    pub fn update_source_commit(&mut self, commit: ObjectId) {
        self.source_commit = commit;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pr() -> PullRequest {
        PullRequest::new(
            1,
            "alice/repo",
            1,
            "Add feature X",
            "This PR adds feature X",
            "alice_pubkey",
            "feature-x",
            "main",
            ObjectId::from_bytes([1u8; 20]),
            ObjectId::from_bytes([2u8; 20]),
        )
    }

    #[test]
    fn test_pr_creation() {
        let pr = create_test_pr();
        assert_eq!(pr.number, 1);
        assert_eq!(pr.title, "Add feature X");
        assert!(pr.is_open());
        assert!(!pr.is_merged());
        assert!(!pr.is_closed());
    }

    #[test]
    fn test_pr_close_and_reopen() {
        let mut pr = create_test_pr();

        pr.close().unwrap();
        assert!(pr.is_closed());
        assert!(!pr.is_open());

        pr.reopen().unwrap();
        assert!(pr.is_open());
        assert!(!pr.is_closed());
    }

    #[test]
    fn test_pr_merge() {
        let mut pr = create_test_pr();

        pr.merge("bob_pubkey").unwrap();
        assert!(pr.is_merged());
        assert!(!pr.is_open());
        assert!(pr.merged_at.is_some());
        assert_eq!(pr.merged_by, Some("bob_pubkey".to_string()));
    }

    #[test]
    fn test_cannot_merge_closed_pr() {
        let mut pr = create_test_pr();
        pr.close().unwrap();

        let result = pr.merge("bob");
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_close_merged_pr() {
        let mut pr = create_test_pr();
        pr.merge("bob").unwrap();

        let result = pr.close();
        assert!(result.is_err());
    }

    #[test]
    fn test_labels() {
        let mut pr = create_test_pr();

        pr.add_label(Label::bug());
        pr.add_label(Label::enhancement());
        assert_eq!(pr.labels.len(), 2);

        // Adding same label twice doesn't duplicate
        pr.add_label(Label::bug());
        assert_eq!(pr.labels.len(), 2);

        pr.remove_label("bug");
        assert_eq!(pr.labels.len(), 1);
        assert_eq!(pr.labels[0].name, "enhancement");
    }
}
