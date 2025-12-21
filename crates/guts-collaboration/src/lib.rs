//! Collaboration features for Guts: Pull Requests, Issues, Comments, Reviews.
//!
//! This crate provides the core types and storage for decentralized code collaboration,
//! enabling developers to create pull requests, track issues, and conduct code reviews.

mod comment;
mod error;
mod issue;
mod label;
mod pull_request;
mod review;
mod store;

pub use comment::{Comment, CommentTarget};
pub use error::CollaborationError;
pub use issue::{Issue, IssueState};
pub use label::Label;
pub use pull_request::{PullRequest, PullRequestState};
pub use review::{Review, ReviewState};
pub use store::CollaborationStore;

/// Result type for collaboration operations.
pub type Result<T> = std::result::Result<T, CollaborationError>;
