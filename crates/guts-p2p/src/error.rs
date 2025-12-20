//! P2P error types.

use thiserror::Error;

/// Errors that can occur in P2P operations.
#[derive(Debug, Error)]
pub enum P2PError {
    /// Message encoding/decoding error.
    #[error("codec error: {0}")]
    Codec(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(#[from] guts_storage::StorageError),

    /// Invalid message.
    #[error("invalid message: {0}")]
    InvalidMessage(String),

    /// Repository not found.
    #[error("repository not found: {0}")]
    RepoNotFound(String),

    /// Channel closed.
    #[error("channel closed")]
    ChannelClosed,
}
