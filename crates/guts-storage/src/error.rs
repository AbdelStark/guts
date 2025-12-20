//! Storage error types.

use thiserror::Error;

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The requested item was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// A corruption was detected.
    #[error("corruption detected: {0}")]
    Corruption(String),

    /// The storage is full.
    #[error("storage full")]
    StorageFull,
}

/// A specialized Result type for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;
