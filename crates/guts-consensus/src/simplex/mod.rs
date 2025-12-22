//! Simplex BFT consensus implementation for Guts.
//!
//! This module provides a production-ready BFT consensus implementation
//! based on the Simplex consensus protocol from commonware. It provides:
//!
//! - Fast block times (2 network hops)
//! - Optimal finalization latency (3 network hops)
//! - Byzantine fault tolerance (up to f < n/3 faulty validators)
//! - Externalized uptime and fault proofs
//!
//! # Architecture
//!
//! The consensus engine is composed of several actors:
//!
//! - **Application**: Handles block proposal and verification
//! - **Marshal**: Manages block storage and synchronization
//! - **Buffer**: Buffers broadcast messages
//! - **Consensus (Voter/Batcher/Resolver)**: Core BFT logic
//!
//! # Usage
//!
//! ```ignore
//! use guts_consensus::simplex::{Engine, Config};
//!
//! // Create configuration
//! let config = Config::new(
//!     blocker,
//!     my_public_key,
//!     my_private_key,
//!     validator_public_keys,
//! );
//!
//! // Create engine
//! let engine = Engine::new(context, config).await;
//!
//! // Start with P2P channels
//! engine.start(pending, recovered, resolver, broadcast, marshal);
//! ```

pub mod application;
pub mod block;
pub mod engine;
pub mod types;

pub use application::{Actor as ApplicationActor, Config as ApplicationConfig, Mailbox, Message};
pub use block::SimplexBlock;
pub use engine::{Config, Engine, EngineMetrics, StaticSchemeProvider, EPOCH, NAMESPACE};
pub use types::{
    Activity, Finalization, Notarization, Scheme, ValidatorPrivateKey, ValidatorPublicKey,
};
