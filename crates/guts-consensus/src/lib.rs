//! Guts Consensus Engine
//!
//! This crate provides the BFT consensus layer for Guts, transforming it from
//! a replicated multi-node system into a truly decentralized network.
//!
//! # Architecture
//!
//! The consensus engine is built on top of commonware primitives and provides:
//!
//! - **Simplex BFT Consensus**: 2 network hops for proposals, 3 for finalization
//! - **Transaction Ordering**: Total ordering of all state-changing operations
//! - **Byzantine Fault Tolerance**: Tolerates f < n/3 Byzantine nodes
//! - **Validator Management**: Dynamic validator sets with epoch transitions
//!
//! # Components
//!
//! - [`Transaction`]: All state-changing operations (git push, PRs, issues, etc.)
//! - [`Block`]: Ordered container of transactions
//! - [`Mempool`]: Pending transaction pool
//! - [`ValidatorSet`]: Set of validators participating in consensus
//! - [`Genesis`]: Initial network configuration
//! - [`simplex`]: Real Simplex BFT consensus engine (production)
//!
//! # Real Simplex BFT Consensus
//!
//! The [`simplex`] module provides a production-ready BFT consensus implementation
//! using the commonware-consensus library. This is the recommended way to run
//! Guts in a multi-validator network.
//!
//! ```ignore
//! use guts_consensus::simplex::{Engine, Config};
//! use commonware_p2p::authenticated::discovery;
//!
//! // Create configuration
//! let config = Config::new(
//!     blocker,
//!     my_public_key,
//!     my_private_key,
//!     validator_public_keys,
//! );
//!
//! // Create and start engine with P2P channels
//! let engine = Engine::new(context, config).await;
//! engine.start(pending, recovered, resolver, broadcast, marshal);
//! ```
//!
//! # Transaction Flow
//!
//! ```text
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │   Client     │────▶│   API Layer  │────▶│   Mempool    │
//! │  (git push)  │     │  (validate)  │     │  (pending)   │
//! └──────────────┘     └──────────────┘     └──────┬───────┘
//!                                                   │
//!                      ┌────────────────────────────┘
//!                      ▼
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │  Broadcast   │◀────│   Leader     │◀────│  Consensus   │
//! │  to Peers    │     │  Proposes    │     │   Selects    │
//! └──────────────┘     │   Block      │     │    Leader    │
//!                      └──────┬───────┘     └──────────────┘
//!                             │
//!                             ▼
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │  Validators  │────▶│  Notarize    │────▶│   Finalize   │
//! │    Vote      │     │  (2f+1)      │     │   (2f+1)     │
//! └──────────────┘     └──────────────┘     └──────┬───────┘
//!                                                   │
//!                                                   ▼
//!                      ┌──────────────────────────────────┐
//!                      │         Apply to State           │
//!                      │  (git refs, PRs, issues, etc.)   │
//!                      └──────────────────────────────────┘
//! ```

mod block;
mod engine;
mod error;
mod genesis;
mod mempool;
mod message;
pub mod simplex;
mod transaction;
mod validator;

pub use block::{Block, BlockHeader, BlockId, FinalizedBlock};
pub use engine::{
    ConsensusApplication, ConsensusEngine, ConsensusEvent, EngineConfig, EngineState,
    NoOpApplication,
};
pub use error::{ConsensusError, Result};
pub use genesis::{
    generate_devnet_genesis, ConsensusParams, Genesis, GenesisRepository, GenesisValidator,
};
pub use mempool::{Mempool, MempoolConfig, MempoolStats};
pub use message::{
    ConsensusMessage, FinalizeMessage, NotarizeMessage, NullifyMessage, ProposeMessage,
    SyncRequestMessage, SyncResponseMessage, TransactionMessage, VoteCollector, CONSENSUS_CHANNEL,
    SYNC_CHANNEL, TRANSACTION_CHANNEL,
};
pub use transaction::{
    BranchProtectionSpec, CommentTargetSpec, IssueUpdate, OrgUpdate, PullRequestUpdate,
    SerializablePublicKey, SerializableSignature, TeamSpec, Transaction, TransactionId,
};
pub use validator::{Validator, ValidatorConfig, ValidatorSet};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_exports() {
        // Verify all public types are accessible
        let _: TransactionId;
        let _: BlockId;
    }
}
