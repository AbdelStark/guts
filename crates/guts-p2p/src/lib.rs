//! P2P networking layer for Guts decentralized code collaboration.
//!
//! This crate provides the peer-to-peer networking infrastructure for
//! replicating git repositories across multiple nodes.

mod error;
mod message;
mod protocol;

pub use error::P2PError;
pub use message::{Message, MessageType, ObjectData, RefUpdate, RepoAnnounce, SyncRequest};
pub use protocol::{ReplicationHandler, ReplicationProtocol};

/// Channel ID for replication messages.
pub const REPLICATION_CHANNEL: u64 = 1;

/// Maximum message size for replication (10 MB).
pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Result type for P2P operations.
pub type Result<T> = std::result::Result<T, P2PError>;
