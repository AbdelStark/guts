//! Consensus error types.

use thiserror::Error;

/// Errors that can occur during consensus operations.
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Invalid transaction.
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Invalid block.
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    /// Invalid signature.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid genesis configuration.
    #[error("invalid genesis: {0}")]
    InvalidGenesis(String),

    /// Validator not found.
    #[error("validator not found: {0}")]
    ValidatorNotFound(String),

    /// No quorum reached.
    #[error("no quorum: {0}")]
    NoQuorum(String),

    /// Mempool error.
    #[error("mempool error: {0}")]
    MempoolError(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Storage error.
    #[error("storage error: {0}")]
    StorageError(String),

    /// Consensus engine error.
    #[error("engine error: {0}")]
    EngineError(String),

    /// Network error.
    #[error("network error: {0}")]
    NetworkError(String),

    /// Transaction already exists.
    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(String),

    /// Block not found.
    #[error("block not found at height {0}")]
    BlockNotFound(u64),

    /// Invalid state transition.
    #[error("invalid state transition: {0}")]
    InvalidStateTransition(String),

    /// Transaction execution failed.
    #[error("transaction failed: {0}")]
    TransactionFailed(String),
}

/// Result type for consensus operations.
pub type Result<T> = std::result::Result<T, ConsensusError>;

impl From<serde_json::Error> for ConsensusError {
    fn from(err: serde_json::Error) -> Self {
        ConsensusError::SerializationError(err.to_string())
    }
}
