//! Repository error types.

use thiserror::Error;

/// Errors that can occur during repository operations.
#[derive(Debug, Error)]
pub enum RepoError {
    /// The repository was not found.
    #[error("repository not found: {0}")]
    NotFound(String),

    /// The repository already exists.
    #[error("repository already exists: {0}")]
    AlreadyExists(String),

    /// A Git operation failed.
    #[error("git error: {0}")]
    Git(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A reference was not found.
    #[error("reference not found: {0}")]
    RefNotFound(String),

    /// An object was not found.
    #[error("object not found: {0}")]
    ObjectNotFound(String),
}

/// A specialized Result type for repository operations.
pub type Result<T> = std::result::Result<T, RepoError>;
