//! Peer management.

use guts_core::Timestamp;
use guts_identity::PublicKey;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// A peer identifier derived from public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId([u8; 32]);

impl PeerId {
    /// Creates a peer ID from a public key.
    #[must_use]
    pub fn from_public_key(key: &PublicKey) -> Self {
        Self(*key.as_bytes())
    }

    /// Returns the raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns a short hex representation.
    #[must_use]
    pub fn short_id(&self) -> String {
        hex::encode(&self.0[..8])
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.short_id())
    }
}

/// The state of a peer connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    /// Connecting to the peer.
    Connecting,
    /// Connected and handshaking.
    Handshaking,
    /// Fully connected.
    Connected,
    /// Disconnected.
    Disconnected,
}

/// A connected peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// The peer's unique identifier.
    pub id: PeerId,
    /// The peer's address.
    pub address: SocketAddr,
    /// The peer's public key.
    pub public_key: PublicKey,
    /// Current connection state.
    pub state: PeerState,
    /// When the connection was established.
    pub connected_at: Timestamp,
    /// When we last heard from this peer.
    pub last_seen: Timestamp,
}

impl Peer {
    /// Creates a new peer.
    #[must_use]
    pub fn new(address: SocketAddr, public_key: PublicKey) -> Self {
        let now = Timestamp::now();
        Self {
            id: PeerId::from_public_key(&public_key),
            address,
            public_key,
            state: PeerState::Connecting,
            connected_at: now,
            last_seen: now,
        }
    }

    /// Marks the peer as connected.
    pub fn mark_connected(&mut self) {
        self.state = PeerState::Connected;
        self.last_seen = Timestamp::now();
    }

    /// Updates the last seen timestamp.
    pub fn touch(&mut self) {
        self.last_seen = Timestamp::now();
    }
}
