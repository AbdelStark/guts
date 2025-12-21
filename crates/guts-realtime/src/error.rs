//! Error types for the real-time module.

use thiserror::Error;

/// Errors that can occur in real-time operations.
#[derive(Debug, Error)]
pub enum RealtimeError {
    /// Invalid channel format.
    #[error("invalid channel: {0}")]
    InvalidChannel(String),

    /// Subscription limit exceeded.
    #[error("subscription limit exceeded: max {0} subscriptions")]
    SubscriptionLimit(usize),

    /// Client not found.
    #[error("client not found: {0}")]
    ClientNotFound(String),

    /// Send failed.
    #[error("failed to send message: {0}")]
    SendFailed(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Channel closed.
    #[error("channel closed")]
    ChannelClosed,
}
