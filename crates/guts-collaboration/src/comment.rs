//! Comment types for pull requests and issues.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Target of a comment (PR or Issue).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommentTarget {
    /// Comment on a pull request.
    PullRequest {
        /// Repository key (owner/repo).
        repo_key: String,
        /// Pull request number.
        number: u32,
    },
    /// Comment on an issue.
    Issue {
        /// Repository key (owner/repo).
        repo_key: String,
        /// Issue number.
        number: u32,
    },
}

impl CommentTarget {
    /// Creates a pull request comment target.
    pub fn pull_request(repo_key: impl Into<String>, number: u32) -> Self {
        Self::PullRequest {
            repo_key: repo_key.into(),
            number,
        }
    }

    /// Creates an issue comment target.
    pub fn issue(repo_key: impl Into<String>, number: u32) -> Self {
        Self::Issue {
            repo_key: repo_key.into(),
            number,
        }
    }

    /// Returns the repository key.
    pub fn repo_key(&self) -> &str {
        match self {
            CommentTarget::PullRequest { repo_key, .. } => repo_key,
            CommentTarget::Issue { repo_key, .. } => repo_key,
        }
    }

    /// Returns the number (PR or issue number).
    pub fn number(&self) -> u32 {
        match self {
            CommentTarget::PullRequest { number, .. } => *number,
            CommentTarget::Issue { number, .. } => *number,
        }
    }

    /// Returns true if this is a pull request comment.
    pub fn is_pull_request(&self) -> bool {
        matches!(self, CommentTarget::PullRequest { .. })
    }

    /// Returns true if this is an issue comment.
    pub fn is_issue(&self) -> bool {
        matches!(self, CommentTarget::Issue { .. })
    }
}

/// A comment on a pull request or issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Unique identifier.
    pub id: u64,
    /// Target of the comment (PR or Issue).
    pub target: CommentTarget,
    /// Author's public key (hex encoded).
    pub author: String,
    /// Comment body (Markdown).
    pub body: String,
    /// Unix timestamp when the comment was created.
    pub created_at: u64,
    /// Unix timestamp when the comment was last updated.
    pub updated_at: u64,
}

impl Comment {
    /// Creates a new comment.
    pub fn new(
        id: u64,
        target: CommentTarget,
        author: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            target,
            author: author.into(),
            body: body.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Updates the comment body.
    pub fn update_body(&mut self, body: impl Into<String>) {
        self.body = body.into();
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Returns true if this comment was edited.
    pub fn is_edited(&self) -> bool {
        self.updated_at > self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_creation() {
        let target = CommentTarget::pull_request("alice/repo", 1);
        let comment = Comment::new(1, target.clone(), "bob_pubkey", "Great work!");

        assert_eq!(comment.id, 1);
        assert_eq!(comment.author, "bob_pubkey");
        assert_eq!(comment.body, "Great work!");
        assert!(!comment.is_edited());
        assert!(target.is_pull_request());
    }

    #[test]
    fn test_issue_comment() {
        let target = CommentTarget::issue("alice/repo", 5);
        let comment = Comment::new(2, target.clone(), "carol_pubkey", "I can reproduce this bug");

        assert_eq!(comment.target.number(), 5);
        assert!(target.is_issue());
        assert!(!target.is_pull_request());
    }

    #[test]
    fn test_comment_update() {
        let target = CommentTarget::pull_request("alice/repo", 1);
        let mut comment = Comment::new(1, target, "bob_pubkey", "Original text");

        // Directly modify created_at to simulate passage of time (timestamps are in seconds)
        comment.created_at -= 1;

        comment.update_body("Updated text");
        assert_eq!(comment.body, "Updated text");
        assert!(comment.is_edited());
    }

    #[test]
    fn test_comment_target_helpers() {
        let pr_target = CommentTarget::pull_request("alice/repo", 1);
        assert_eq!(pr_target.repo_key(), "alice/repo");
        assert_eq!(pr_target.number(), 1);

        let issue_target = CommentTarget::issue("bob/project", 10);
        assert_eq!(issue_target.repo_key(), "bob/project");
        assert_eq!(issue_target.number(), 10);
    }
}
