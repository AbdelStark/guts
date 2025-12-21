//! P2P networking integration for guts-node.
//!
//! This module provides the P2P layer integration using commonware-p2p's
//! simulated network for development/testing and authenticated network for production.

use bytes::Bytes;
use commonware_cryptography::{ed25519, PrivateKeyExt, Signer};
use commonware_p2p::simulated::{Config as SimConfig, Link, Network, Oracle};
use commonware_p2p::{Recipients, Sender as P2PSender};
use commonware_runtime::{deterministic, Metrics};
use guts_p2p::{ReplicationProtocol, REPLICATION_CHANNEL};
use guts_storage::Repository;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// P2P network manager for multi-node replication.
#[allow(dead_code)]
pub struct P2PManager {
    /// The replication protocol handler.
    protocol: Arc<ReplicationProtocol>,
    /// Sender for broadcasting messages.
    sender: Arc<RwLock<Option<SimulatedSender>>>,
    /// Our public key.
    our_pk: ed25519::PublicKey,
}

/// Wrapper around the simulated sender for thread-safe access.
struct SimulatedSender {
    sender: commonware_p2p::simulated::Sender<ed25519::PublicKey>,
}

#[allow(dead_code)]
impl P2PManager {
    /// Create a new P2P manager with a given private key.
    pub fn new(private_key: &ed25519::PrivateKey) -> Self {
        let our_pk = private_key.public_key();
        let protocol = Arc::new(ReplicationProtocol::new());

        Self {
            protocol,
            sender: Arc::new(RwLock::new(None)),
            our_pk,
        }
    }

    /// Get our public key.
    pub fn public_key(&self) -> ed25519::PublicKey {
        self.our_pk.clone()
    }

    /// Get the replication protocol.
    pub fn protocol(&self) -> Arc<ReplicationProtocol> {
        self.protocol.clone()
    }

    /// Register a repository for replication.
    pub fn register_repo(&self, key: String, repo: Arc<Repository>) {
        self.protocol.register_repo(key, repo);
    }

    /// Set the sender for broadcasting messages.
    pub fn set_sender(&self, sender: commonware_p2p::simulated::Sender<ed25519::PublicKey>) {
        *self.sender.write() = Some(SimulatedSender { sender });
    }

    /// Broadcast a message to all peers.
    pub fn broadcast(&self, message: Bytes) {
        if let Some(sender) = self.sender.read().as_ref() {
            let mut sender_clone = sender.sender.clone();
            let msg = message.clone();
            // We need to spawn this since send is async
            tokio::spawn(async move {
                if let Err(e) = sender_clone.send(Recipients::All, msg, false).await {
                    error!("Failed to broadcast message: {:?}", e);
                }
            });
        }
    }

    /// Handle an incoming message from a peer.
    pub fn handle_message(&self, peer_id: &[u8], data: &[u8]) -> Option<Bytes> {
        match self.protocol.handle_message(peer_id, data) {
            Ok(Some(response)) => Some(response.encode()),
            Ok(None) => None,
            Err(e) => {
                warn!("Error handling P2P message: {}", e);
                None
            }
        }
    }

    /// Notify peers about a repository update.
    pub fn notify_update(
        &self,
        repo_key: &str,
        new_objects: Vec<guts_storage::ObjectId>,
        refs: Vec<(String, guts_storage::ObjectId)>,
    ) {
        let announce = guts_p2p::RepoAnnounce {
            repo_key: repo_key.to_string(),
            object_ids: new_objects,
            refs,
        };

        info!(
            repo = %repo_key,
            objects = announce.object_ids.len(),
            refs = announce.refs.len(),
            "Broadcasting repository update"
        );

        self.broadcast(announce.encode());
    }
}

/// Configuration for multi-node test setup.
#[allow(dead_code)]
pub struct MultiNodeConfig {
    /// Number of nodes to create.
    pub node_count: usize,
    /// Link latency between nodes.
    pub latency: Duration,
    /// Link success rate (1.0 = 100%).
    pub success_rate: f64,
}

impl Default for MultiNodeConfig {
    fn default() -> Self {
        Self {
            node_count: 3,
            latency: Duration::from_millis(10),
            success_rate: 1.0,
        }
    }
}

/// A multi-node test environment using simulated networking.
#[allow(dead_code)]
pub struct MultiNodeTestEnv {
    /// The simulated network oracle.
    pub oracle: Oracle<ed25519::PublicKey>,
    /// Node public keys.
    pub node_pks: Vec<ed25519::PublicKey>,
    /// Node private keys.
    pub node_sks: Vec<ed25519::PrivateKey>,
    /// P2P managers for each node.
    pub managers: Vec<Arc<P2PManager>>,
    /// Senders for each node.
    pub senders: Vec<commonware_p2p::simulated::Sender<ed25519::PublicKey>>,
    /// Receivers for each node.
    pub receivers: Vec<commonware_p2p::simulated::Receiver<ed25519::PublicKey>>,
}

#[allow(dead_code)]
impl MultiNodeTestEnv {
    /// Create a new multi-node test environment.
    ///
    /// This must be called from within a deterministic runtime context.
    pub async fn new(context: deterministic::Context, config: MultiNodeConfig) -> Self {
        // Create the simulated network
        let (network, mut oracle) = Network::new(
            context.with_label("network"),
            SimConfig {
                max_size: guts_p2p::MAX_MESSAGE_SIZE,
                disconnect_on_block: true,
                tracked_peer_sets: None,
            },
        );

        // Start the network
        network.start();

        // Generate node keys
        let mut node_sks = Vec::with_capacity(config.node_count);
        let mut node_pks = Vec::with_capacity(config.node_count);
        for i in 0..config.node_count {
            let sk = ed25519::PrivateKey::from_seed(i as u64);
            let pk = sk.public_key();
            node_sks.push(sk);
            node_pks.push(pk);
        }

        // Register each node and get sender/receiver
        let mut senders = Vec::with_capacity(config.node_count);
        let mut receivers = Vec::with_capacity(config.node_count);
        let mut managers = Vec::with_capacity(config.node_count);

        for (i, sk) in node_sks.iter().enumerate() {
            let pk = node_pks[i].clone();
            let (sender, receiver) = oracle
                .control(pk.clone())
                .register(REPLICATION_CHANNEL)
                .await
                .expect("Failed to register node");

            let manager = Arc::new(P2PManager::new(sk));
            manager.set_sender(sender.clone());

            senders.push(sender);
            receivers.push(receiver);
            managers.push(manager);
        }

        // Create bidirectional links between all nodes
        for i in 0..config.node_count {
            for j in 0..config.node_count {
                if i != j {
                    oracle
                        .add_link(
                            node_pks[i].clone(),
                            node_pks[j].clone(),
                            Link {
                                latency: config.latency,
                                jitter: Duration::from_millis(1),
                                success_rate: config.success_rate,
                            },
                        )
                        .await
                        .expect("Failed to add link");
                }
            }
        }

        Self {
            oracle,
            node_pks,
            node_sks,
            managers,
            senders,
            receivers,
        }
    }

    /// Get the P2P manager for a specific node.
    pub fn manager(&self, index: usize) -> Arc<P2PManager> {
        self.managers[index].clone()
    }

    /// Register a repository on a specific node.
    pub fn register_repo(&self, node_index: usize, key: &str, repo: Arc<Repository>) {
        self.managers[node_index].register_repo(key.to_string(), repo);
    }

    /// Get a repository from a specific node.
    pub fn get_repo(&self, node_index: usize, key: &str) -> Option<Arc<Repository>> {
        self.managers[node_index].protocol().get_repo(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::{PrivateKeyExt, Signer};
    use commonware_runtime::Runner;

    #[test]
    fn test_p2p_manager_creation() {
        let sk = ed25519::PrivateKey::from_seed(0);
        let manager = P2PManager::new(&sk);

        assert_eq!(manager.public_key(), sk.public_key());
    }

    #[test]
    fn test_multi_node_env_creation() {
        let executor = deterministic::Runner::default();
        executor.start(|context| async move {
            let env = MultiNodeTestEnv::new(
                context,
                MultiNodeConfig {
                    node_count: 3,
                    latency: Duration::from_millis(10),
                    success_rate: 1.0,
                },
            )
            .await;

            assert_eq!(env.node_pks.len(), 3);
            assert_eq!(env.managers.len(), 3);
            assert_eq!(env.senders.len(), 3);
            assert_eq!(env.receivers.len(), 3);
        });
    }
}
