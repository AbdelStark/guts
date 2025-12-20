//! Git protocol error types.

use thiserror::Error;

/// Errors that can occur during git protocol operations.
#[derive(Debug, Error)]
pub enum GitError {
    /// Invalid pack file format.
    #[error("invalid pack file: {0}")]
    InvalidPack(String),

    /// Invalid pkt-line format.
    #[error("invalid pkt-line: {0}")]
    InvalidPktLine(String),

    /// Protocol error.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Object not found.
    #[error("object not found: {0}")]
    ObjectNotFound(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(#[from] guts_storage::StorageError),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
