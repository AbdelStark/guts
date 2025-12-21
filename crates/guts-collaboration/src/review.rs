//! Code review types for pull requests.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// State of a code review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    /// Review approves the changes.
    Approved,
    /// Review requests changes before merging.
    ChangesRequested,
    /// Review is just a comment without approval/rejection.
    Commented,
    /// Review was dismissed.
    Dismissed,
}

impl std::fmt::Display for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewState::Approved => write!(f, "approved"),
            ReviewState::ChangesRequested => write!(f, "changes_requested"),
            ReviewState::Commented => write!(f, "commented"),
            ReviewState::Dismissed => write!(f, "dismissed"),
        }
    }
}

/// A code review on a pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Unique identifier.
    pub id: u64,
    /// Repository key (owner/repo).
    pub repo_key: String,
    /// Pull request number.
    pub pr_number: u32,
    /// Reviewer's public key (hex encoded).
    pub author: String,
    /// Review state.
    pub state: ReviewState,
    /// Review body/summary (optional).
    pub body: Option<String>,
    /// Commit SHA that was reviewed.
    pub commit_id: String,
    /// Unix timestamp when the review was created.
    pub created_at: u64,
    /// Unix timestamp when the review was submitted.
    pub submitted_at: Option<u64>,
}

impl Review {
    /// Creates a new review.
    pub fn new(
        id: u64,
        repo_key: impl Into<String>,
        pr_number: u32,
        author: impl Into<String>,
        state: ReviewState,
        commit_id: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            repo_key: repo_key.into(),
            pr_number,
            author: author.into(),
            state,
            body: None,
            commit_id: commit_id.into(),
            created_at: now,
            submitted_at: Some(now),
        }
    }

    /// Creates a new review with a body.
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Returns true if this review approves the PR.
    pub fn is_approved(&self) -> bool {
        self.state == ReviewState::Approved
    }

    /// Returns true if this review requests changes.
    pub fn requests_changes(&self) -> bool {
        self.state == ReviewState::ChangesRequested
    }

    /// Returns true if this review was dismissed.
    pub fn is_dismissed(&self) -> bool {
        self.state == ReviewState::Dismissed
    }

    /// Dismisses this review.
    pub fn dismiss(&mut self) {
        self.state = ReviewState::Dismissed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_creation() {
        let review = Review::new(
            1,
            "alice/repo",
            1,
            "bob_pubkey",
            ReviewState::Approved,
            "abc123",
        )
        .with_body("LGTM!");

        assert_eq!(review.id, 1);
        assert_eq!(review.pr_number, 1);
        assert_eq!(review.author, "bob_pubkey");
        assert!(review.is_approved());
        assert!(!review.requests_changes());
        assert_eq!(review.body, Some("LGTM!".to_string()));
    }

    #[test]
    fn test_changes_requested() {
        let review = Review::new(
            2,
            "alice/repo",
            1,
            "carol_pubkey",
            ReviewState::ChangesRequested,
            "def456",
        )
        .with_body("Please add tests");

        assert!(!review.is_approved());
        assert!(review.requests_changes());
        assert!(!review.is_dismissed());
    }

    #[test]
    fn test_dismiss_review() {
        let mut review = Review::new(
            3,
            "alice/repo",
            1,
            "dave_pubkey",
            ReviewState::ChangesRequested,
            "ghi789",
        );

        review.dismiss();
        assert!(review.is_dismissed());
        assert!(!review.requests_changes());
    }

    #[test]
    fn test_commented_review() {
        let review = Review::new(
            4,
            "alice/repo",
            2,
            "eve_pubkey",
            ReviewState::Commented,
            "jkl012",
        )
        .with_body("Have you considered using a different approach?");

        assert!(!review.is_approved());
        assert!(!review.requests_changes());
        assert!(!review.is_dismissed());
        assert_eq!(review.state, ReviewState::Commented);
    }
}
