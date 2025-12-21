//! Error types for collaboration operations.

use thiserror::Error;

/// Errors that can occur during collaboration operations.
#[derive(Debug, Error)]
pub enum CollaborationError {
    /// Pull request not found.
    #[error("pull request not found: {repo_key}#{number}")]
    PullRequestNotFound { repo_key: String, number: u32 },

    /// Issue not found.
    #[error("issue not found: {repo_key}#{number}")]
    IssueNotFound { repo_key: String, number: u32 },

    /// Comment not found.
    #[error("comment not found: {id}")]
    CommentNotFound { id: u64 },

    /// Review not found.
    #[error("review not found: {id}")]
    ReviewNotFound { id: u64 },

    /// Pull request already exists.
    #[error("pull request already exists: {repo_key}#{number}")]
    PullRequestExists { repo_key: String, number: u32 },

    /// Issue already exists.
    #[error("issue already exists: {repo_key}#{number}")]
    IssueExists { repo_key: String, number: u32 },

    /// Invalid state transition.
    #[error("invalid state transition: cannot {action} when state is {current_state}")]
    InvalidStateTransition {
        action: String,
        current_state: String,
    },

    /// Pull request already merged.
    #[error("pull request already merged: {repo_key}#{number}")]
    AlreadyMerged { repo_key: String, number: u32 },

    /// Pull request is closed.
    #[error("pull request is closed: {repo_key}#{number}")]
    PullRequestClosed { repo_key: String, number: u32 },

    /// Issue is closed.
    #[error("issue is closed: {repo_key}#{number}")]
    IssueClosed { repo_key: String, number: u32 },

    /// Repository not found.
    #[error("repository not found: {repo_key}")]
    RepoNotFound { repo_key: String },

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
