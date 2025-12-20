//! # Guts Consensus
//!
//! BFT consensus integration for the Guts network.
//!
//! This crate provides abstractions for Byzantine Fault Tolerant consensus,
//! designed to integrate with commonware's consensus primitives.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod proposal;
mod validator;

pub use error::{ConsensusError, Result};
pub use proposal::{Proposal, ProposalId};
pub use validator::{Validator, ValidatorSet};

/// The state of a consensus round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundState {
    /// Waiting for a proposal.
    Proposing,
    /// Voting on a proposal.
    Voting,
    /// Round has been committed.
    Committed,
    /// Round failed.
    Failed,
}

/// Configuration for the consensus engine.
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Minimum number of validators required.
    pub min_validators: usize,
    /// Timeout for proposal phase.
    pub proposal_timeout_ms: u64,
    /// Timeout for voting phase.
    pub vote_timeout_ms: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_validators: 4,
            proposal_timeout_ms: 5000,
            vote_timeout_ms: 3000,
        }
    }
}
