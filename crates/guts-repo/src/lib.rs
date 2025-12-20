//! # Guts Repo
//!
//! Git repository operations for Guts using gitoxide.
//!
//! This crate provides a high-level API for working with Git repositories
//! in the Guts network.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod repository;

pub use error::{RepoError, Result};
pub use repository::{GitRepository, RepoConfig};

/// A reference in a Git repository.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ref {
    /// The reference name (e.g., "refs/heads/main").
    pub name: String,
    /// The object ID this reference points to.
    pub target: String,
}

/// A Git object kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ObjectKind {
    /// A blob (file content).
    Blob,
    /// A tree (directory).
    Tree,
    /// A commit.
    Commit,
    /// An annotated tag.
    Tag,
}
