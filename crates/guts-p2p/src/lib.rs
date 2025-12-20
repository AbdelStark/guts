//! # Guts P2P
//!
//! Peer-to-peer networking layer for Guts.
//!
//! This crate provides the networking infrastructure for connecting
//! Guts nodes in a decentralized mesh network.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod node;
mod peer;

pub use error::{P2pError, Result};
pub use node::{Node, NodeConfig};
pub use peer::{Peer, PeerId, PeerState};

use std::net::SocketAddr;

/// Default port for Guts P2P communication.
pub const DEFAULT_PORT: u16 = 9000;

/// Default maximum number of peers.
pub const DEFAULT_MAX_PEERS: usize = 50;
