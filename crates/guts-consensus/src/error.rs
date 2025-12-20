//! Consensus error types.

use thiserror::Error;

/// Errors that can occur during consensus operations.
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Not enough validators.
    #[error("not enough validators: have {have}, need {need}")]
    NotEnoughValidators {
        /// Current validator count.
        have: usize,
        /// Required validator count.
        need: usize,
    },

    /// Invalid proposal.
    #[error("invalid proposal: {0}")]
    InvalidProposal(String),

    /// Invalid vote.
    #[error("invalid vote: {0}")]
    InvalidVote(String),

    /// Timeout occurred.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Not a validator.
    #[error("not a validator")]
    NotValidator,
}

/// A specialized Result type for consensus operations.
pub type Result<T> = std::result::Result<T, ConsensusError>;
