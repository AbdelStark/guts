//! Storage error types.

use thiserror::Error;

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Object not found in storage.
    #[error("object not found: {0}")]
    ObjectNotFound(String),

    /// Reference not found.
    #[error("reference not found: {0}")]
    RefNotFound(String),

    /// Repository not found.
    #[error("repository not found: {0}")]
    RepoNotFound(String),

    /// Repository already exists.
    #[error("repository already exists: {0}")]
    RepoExists(String),

    /// Invalid object format.
    #[error("invalid object: {0}")]
    InvalidObject(String),

    /// Invalid reference name.
    #[error("invalid reference: {0}")]
    InvalidRef(String),

    /// Compression/decompression error.
    #[error("compression error: {0}")]
    Compression(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
