//! Protocol error types.

use thiserror::Error;

/// Errors that can occur during protocol operations.
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// The message is malformed.
    #[error("malformed message: {0}")]
    Malformed(String),

    /// The message exceeds size limits.
    #[error("message too large: {size} bytes (max {max})")]
    TooLarge {
        /// The actual size.
        size: usize,
        /// The maximum allowed size.
        max: usize,
    },

    /// Unsupported protocol version.
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u32),

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Invalid magic bytes.
    #[error("invalid magic bytes")]
    InvalidMagic,
}

/// A specialized Result type for protocol operations.
pub type Result<T> = std::result::Result<T, ProtocolError>;
