//! P2P error types.

use thiserror::Error;

/// Errors that can occur during P2P operations.
#[derive(Debug, Error)]
pub enum P2pError {
    /// Connection failed.
    #[error("connection failed: {0}")]
    Connection(String),

    /// Peer not found.
    #[error("peer not found: {0}")]
    PeerNotFound(String),

    /// Maximum peers reached.
    #[error("maximum peers reached: {0}")]
    MaxPeers(usize),

    /// Protocol error.
    #[error("protocol error: {0}")]
    Protocol(#[from] guts_protocol::ProtocolError),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The node is not running.
    #[error("node not running")]
    NotRunning,
}

/// A specialized Result type for P2P operations.
pub type Result<T> = std::result::Result<T, P2pError>;
