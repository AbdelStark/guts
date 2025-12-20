//! Consensus proposals.

use guts_core::Timestamp;
use guts_identity::PublicKey;
use serde::{Deserialize, Serialize};

/// A unique identifier for a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId([u8; 32]);

impl ProposalId {
    /// Creates a new proposal ID from bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// A consensus proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique identifier.
    pub id: ProposalId,
    /// Round number.
    pub round: u64,
    /// The proposer's public key.
    pub proposer: PublicKey,
    /// The proposed data.
    pub data: Vec<u8>,
    /// Timestamp of the proposal.
    pub timestamp: Timestamp,
}

impl Proposal {
    /// Creates a new proposal.
    #[must_use]
    pub fn new(round: u64, proposer: PublicKey, data: Vec<u8>) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(round.to_be_bytes());
        hasher.update(proposer.as_bytes());
        hasher.update(&data);

        let hash = hasher.finalize();
        let id = ProposalId::from_bytes(hash.into());

        Self {
            id,
            round,
            proposer,
            data,
            timestamp: Timestamp::now(),
        }
    }
}
