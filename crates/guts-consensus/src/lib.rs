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
//! - [`ConsensusEngine`]: Main consensus engine implementation
//!
//! # Example
//!
//! ```rust,no_run
//! use guts_consensus::{
//!     ConsensusEngine, EngineConfig, Genesis, Mempool, MempoolConfig,
//!     Transaction, ValidatorSet,
//! };
//! use commonware_cryptography::PrivateKeyExt;
//! use std::sync::Arc;
//!
//! // Load genesis configuration
//! let genesis = Genesis::load_json("genesis.json").unwrap();
//! let validators = genesis.into_validator_set().unwrap();
//!
//! // Create mempool
//! let mempool = Arc::new(Mempool::new(MempoolConfig::default()));
//!
//! // Create consensus engine
//! let config = EngineConfig::default();
//! let validator_key = commonware_cryptography::ed25519::PrivateKey::from_seed(0);
//! let engine = ConsensusEngine::new(config, Some(validator_key), validators, mempool);
//!
//! // Subscribe to events
//! let mut events = engine.subscribe();
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
