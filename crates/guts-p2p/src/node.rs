//! P2P node implementation.

use crate::{P2pError, Peer, PeerId, Result, DEFAULT_MAX_PEERS, DEFAULT_PORT};
use dashmap::DashMap;
use guts_identity::Keypair;
use std::net::SocketAddr;
use std::sync::Arc;

/// Configuration for a P2P node.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Address to listen on.
    pub listen_addr: SocketAddr,
    /// Maximum number of peers.
    pub max_peers: usize,
    /// Bootstrap nodes to connect to.
    pub bootstrap_nodes: Vec<SocketAddr>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT)),
            max_peers: DEFAULT_MAX_PEERS,
            bootstrap_nodes: Vec::new(),
        }
    }
}

/// A P2P network node.
pub struct Node {
    config: NodeConfig,
    keypair: Keypair,
    peers: Arc<DashMap<PeerId, Peer>>,
    running: std::sync::atomic::AtomicBool,
}

impl Node {
    /// Creates a new P2P node.
    #[must_use]
    pub fn new(config: NodeConfig, keypair: Keypair) -> Self {
        Self {
            config,
            keypair,
            peers: Arc::new(DashMap::new()),
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Creates a node with default configuration.
    #[must_use]
    pub fn with_defaults(keypair: Keypair) -> Self {
        Self::new(NodeConfig::default(), keypair)
    }

    /// Returns the node's public key.
    #[must_use]
    pub fn public_key(&self) -> guts_identity::PublicKey {
        self.keypair.public_key()
    }

    /// Returns the node's peer ID.
    #[must_use]
    pub fn peer_id(&self) -> PeerId {
        PeerId::from_public_key(&self.public_key())
    }

    /// Returns the listen address.
    #[must_use]
    pub fn listen_addr(&self) -> SocketAddr {
        self.config.listen_addr
    }

    /// Returns the current number of connected peers.
    #[must_use]
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Returns true if the node is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Starts the node.
    ///
    /// # Errors
    ///
    /// Returns an error if the node fails to start.
    pub async fn start(&self) -> Result<()> {
        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        tracing::info!(
            peer_id = %self.peer_id(),
            listen_addr = %self.listen_addr(),
            "P2P node started"
        );

        Ok(())
    }

    /// Stops the node.
    pub async fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.peers.clear();

        tracing::info!("P2P node stopped");
    }

    /// Connects to a peer.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub async fn connect(&self, addr: SocketAddr) -> Result<PeerId> {
        if !self.is_running() {
            return Err(P2pError::NotRunning);
        }

        if self.peers.len() >= self.config.max_peers {
            return Err(P2pError::MaxPeers(self.config.max_peers));
        }

        // In a real implementation, this would establish a TCP/QUIC connection
        // and perform the handshake

        tracing::debug!(addr = %addr, "Connecting to peer");

        // Placeholder: would return actual peer ID after handshake
        Err(P2pError::Connection("not implemented".into()))
    }

    /// Disconnects from a peer.
    pub async fn disconnect(&self, peer_id: &PeerId) {
        if let Some((_, peer)) = self.peers.remove(peer_id) {
            tracing::debug!(peer_id = %peer_id, "Disconnected from peer");
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("peer_id", &self.peer_id())
            .field("listen_addr", &self.config.listen_addr)
            .field("peer_count", &self.peer_count())
            .field("running", &self.is_running())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn node_start_stop() {
        let keypair = Keypair::generate();
        let node = Node::with_defaults(keypair);

        assert!(!node.is_running());

        node.start().await.unwrap();
        assert!(node.is_running());

        node.stop().await;
        assert!(!node.is_running());
    }
}
