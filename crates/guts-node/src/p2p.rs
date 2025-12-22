//! P2P networking integration for guts-node.
//!
//! This module provides the P2P layer integration using commonware-p2p's
//! authenticated network for production BFT consensus.
//!
//! # Channels
//!
//! The consensus engine requires several P2P channels:
//! - **Pending**: For pending consensus votes (channel 0)
//! - **Recovered**: For recovered messages after reconnection (channel 1)
//! - **Resolver**: For fetching missing certificates (channel 2)
//! - **Broadcast**: For block broadcast messages (channel 3)
//! - **Marshal**: For block sync messages (channel 4)

use commonware_codec::DecodeExt;
use commonware_consensus::marshal;
use commonware_cryptography::{
    ed25519::{PrivateKey, PublicKey},
    PrivateKeyExt, Signer,
};
use commonware_p2p::{
    authenticated::discovery as authenticated, utils::requester, Manager, Receiver, Sender,
};
use commonware_runtime::{Clock, Metrics, Network as RNetwork, Spawner, Storage};
use commonware_utils::{set::Ordered, union_unique};
use futures::channel::mpsc;
use governor::clock::{Clock as GClock, ReasonablyRealtime};
use governor::Quota;
use guts_consensus::simplex::{SimplexBlock, NAMESPACE};
use rand::{CryptoRng, Rng};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroU32,
    time::Duration,
};
use tracing::info;

/// Legacy P2P manager stub for backward compatibility.
///
/// This will be replaced by the authenticated network for real consensus.
#[derive(Clone)]
pub struct P2PManager {
    /// Our public key.
    pub public_key: PublicKey,
}

impl P2PManager {
    /// Create a new P2P manager stub.
    pub fn new(private_key: &PrivateKey) -> Self {
        Self {
            public_key: private_key.public_key(),
        }
    }

    /// Stub: Notify peers about a repository update.
    ///
    /// In the real consensus system, replication happens via the consensus layer.
    pub fn notify_update(
        &self,
        _repo_key: &str,
        _new_objects: Vec<guts_storage::ObjectId>,
        _refs: Vec<(String, guts_storage::ObjectId)>,
    ) {
        // Replication will be handled by consensus in the real implementation
        tracing::debug!("P2P notify_update called (stub)");
    }

    /// Stub: Register a repository for replication.
    ///
    /// In the real consensus system, repositories are replicated via consensus.
    pub fn register_repo(&self, _key: String, _repo: std::sync::Arc<guts_storage::Repository>) {
        // Registration will be handled differently in the real implementation
        tracing::debug!("P2P register_repo called (stub)");
    }
}

/// Channel IDs for consensus messaging.
pub const PENDING_CHANNEL: u64 = 0;
pub const RECOVERED_CHANNEL: u64 = 1;
pub const RESOLVER_CHANNEL: u64 = 2;
pub const BROADCAST_CHANNEL: u64 = 3;
pub const MARSHAL_CHANNEL: u64 = 4;

/// Maximum P2P message size (1MB).
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Configuration for the authenticated P2P network.
#[derive(Clone)]
pub struct AuthenticatedP2pConfig {
    /// Our private key for signing.
    pub private_key: PrivateKey,
    /// P2P listen address.
    pub listen_addr: SocketAddr,
    /// Our external address (for NAT traversal).
    pub external_addr: SocketAddr,
    /// Bootstrap nodes (public key, address).
    pub bootstrappers: Vec<(PublicKey, SocketAddr)>,
    /// Mailbox size for message channels.
    pub mailbox_size: usize,
    /// Message backlog size.
    pub message_backlog: usize,
    /// Whether running in local mode (relaxed timing).
    pub local: bool,
}

impl AuthenticatedP2pConfig {
    /// Creates a new configuration with sensible defaults.
    pub fn new(private_key: PrivateKey, listen_port: u16) -> Self {
        let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), listen_port);
        let external_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), listen_port);

        Self {
            private_key,
            listen_addr,
            external_addr,
            bootstrappers: Vec::new(),
            mailbox_size: 1024,
            message_backlog: 1024,
            local: true, // Default to local mode for development
        }
    }

    /// Set the external address.
    pub fn with_external_addr(mut self, addr: SocketAddr) -> Self {
        self.external_addr = addr;
        self
    }

    /// Add a bootstrapper.
    pub fn with_bootstrapper(mut self, key: PublicKey, addr: SocketAddr) -> Self {
        self.bootstrappers.push((key, addr));
        self
    }

    /// Set local mode.
    pub fn with_local(mut self, local: bool) -> Self {
        self.local = local;
        self
    }
}

/// Consensus P2P channels for the simplex engine.
pub struct ConsensusChannels<S: Sender, R: Receiver, Res> {
    /// Pending channel (sender, receiver).
    pub pending: (S, R),
    /// Recovered channel (sender, receiver).
    pub recovered: (S, R),
    /// Resolver channel (sender, receiver).
    pub resolver: (S, R),
    /// Broadcast channel (sender, receiver).
    pub broadcast: (S, R),
    /// Marshal resolver (receiver, resolver).
    pub marshal: (
        mpsc::Receiver<marshal::ingress::handler::Message<SimplexBlock>>,
        Res,
    ),
}

/// Authenticated P2P network for BFT consensus.
pub struct AuthenticatedNetwork<E>
where
    E: Clock
        + GClock
        + ReasonablyRealtime
        + Rng
        + CryptoRng
        + Spawner
        + Storage
        + Metrics
        + RNetwork
        + Clone,
{
    /// The network context.
    #[allow(dead_code)]
    context: E,
    /// Our public key.
    pub public_key: PublicKey,
    /// The network oracle for peer management.
    pub oracle: authenticated::Oracle<PublicKey>,
}

impl<E> AuthenticatedNetwork<E>
where
    E: Clock
        + GClock
        + ReasonablyRealtime
        + Rng
        + CryptoRng
        + Spawner
        + Storage
        + Metrics
        + RNetwork
        + Clone,
{
    /// Creates and starts a new authenticated P2P network.
    ///
    /// Returns the network handle and the consensus channels.
    pub async fn new(
        context: E,
        config: AuthenticatedP2pConfig,
        participants: Vec<PublicKey>,
    ) -> (
        Self,
        ConsensusChannels<
            authenticated::Sender<PublicKey>,
            authenticated::Receiver<PublicKey>,
            impl commonware_resolver::Resolver<Key = marshal::ingress::handler::Request<SimplexBlock>>,
        >,
        commonware_runtime::Handle<()>,
    ) {
        let public_key = config.private_key.public_key();

        // Create P2P namespace
        let p2p_namespace = union_unique(NAMESPACE, b"_P2P");

        // Configure the network
        let mut p2p_cfg = if config.local {
            authenticated::Config::local(
                config.private_key.clone(),
                &p2p_namespace,
                config.listen_addr,
                config.external_addr,
                config.bootstrappers.clone(),
                MAX_MESSAGE_SIZE,
            )
        } else {
            authenticated::Config::recommended(
                config.private_key.clone(),
                &p2p_namespace,
                config.listen_addr,
                config.external_addr,
                config.bootstrappers.clone(),
                MAX_MESSAGE_SIZE,
            )
        };
        p2p_cfg.mailbox_size = config.mailbox_size;

        // Create the network
        let (mut network, mut oracle) = authenticated::Network::new(context.clone(), p2p_cfg);

        // Provide authorized peers
        let participants_ordered: Ordered<PublicKey> = participants.into_iter().collect();
        oracle.update(0, participants_ordered).await;

        // Register consensus channels
        let pending_quota = Quota::per_second(NonZeroU32::new(128).unwrap());
        let pending = network.register(PENDING_CHANNEL, pending_quota, config.message_backlog);

        let recovered_quota = Quota::per_second(NonZeroU32::new(128).unwrap());
        let recovered =
            network.register(RECOVERED_CHANNEL, recovered_quota, config.message_backlog);

        let resolver_quota = Quota::per_second(NonZeroU32::new(128).unwrap());
        let resolver = network.register(RESOLVER_CHANNEL, resolver_quota, config.message_backlog);

        let broadcast_quota = Quota::per_second(NonZeroU32::new(8).unwrap());
        let broadcast =
            network.register(BROADCAST_CHANNEL, broadcast_quota, config.message_backlog);

        let marshal_quota = Quota::per_second(NonZeroU32::new(8).unwrap());
        let marshal_channel =
            network.register(MARSHAL_CHANNEL, marshal_quota, config.message_backlog);

        // Start the network
        let network_handle = network.start();

        // Create marshal resolver
        let marshal_resolver_cfg = marshal::resolver::p2p::Config {
            public_key: public_key.clone(),
            manager: oracle.clone(),
            mailbox_size: config.mailbox_size,
            requester_config: requester::Config {
                me: Some(public_key.clone()),
                rate_limit: Quota::per_second(NonZeroU32::new(5).unwrap()),
                initial: Duration::from_secs(1),
                timeout: Duration::from_secs(2),
            },
            fetch_retry_timeout: Duration::from_millis(100),
            priority_requests: false,
            priority_responses: false,
        };
        let marshal_resolver =
            marshal::resolver::p2p::init(&context, marshal_resolver_cfg, marshal_channel);

        info!(
            ?public_key,
            listen_addr = %config.listen_addr,
            external_addr = %config.external_addr,
            bootstrappers = config.bootstrappers.len(),
            "P2P network started"
        );

        let channels = ConsensusChannels {
            pending,
            recovered,
            resolver,
            broadcast,
            marshal: marshal_resolver,
        };

        (
            Self {
                context,
                public_key,
                oracle,
            },
            channels,
            network_handle,
        )
    }

    /// Update the set of authorized participants.
    pub async fn update_participants(&mut self, epoch: u64, participants: Vec<PublicKey>) {
        let participants_ordered: Ordered<PublicKey> = participants.into_iter().collect();
        self.oracle.update(epoch, participants_ordered).await;
    }
}

// Type alias for the tokio runtime context
/// Type alias for the commonware tokio runtime.
pub type TokioContext = commonware_runtime::tokio::Context;

/// Parse a public key from hex string.
pub fn parse_public_key(hex_str: &str) -> Result<PublicKey, String> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex: {}", e))?;
    PublicKey::decode(bytes.as_ref()).map_err(|e| format!("Invalid public key: {:?}", e))
}

/// Parse a private key from hex string (using seed derivation).
pub fn parse_private_key(hex_str: &str) -> Result<PrivateKey, String> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex: {}", e))?;

    if bytes.len() >= 8 {
        // Use the first 8 bytes as a seed
        let seed = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        Ok(PrivateKey::from_seed(seed))
    } else {
        Err("Private key hex must be at least 8 bytes".to_string())
    }
}

/// Parse bootstrapper string in format "pubkey@host:port".
pub fn parse_bootstrapper(s: &str) -> Result<(PublicKey, SocketAddr), String> {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid bootstrapper format '{}', expected 'pubkey@host:port'",
            s
        ));
    }

    let public_key = parse_public_key(parts[0])?;
    let addr: SocketAddr = parts[1]
        .parse()
        .map_err(|e| format!("Invalid address '{}': {}", parts[1], e))?;

    Ok((public_key, addr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_public_key() {
        // Generate a key and convert to hex
        let private_key = PrivateKey::from_seed(42);
        let public_key = private_key.public_key();
        let hex_str = hex::encode(public_key.as_ref());

        let parsed = parse_public_key(&hex_str).unwrap();
        assert_eq!(parsed, public_key);

        // With 0x prefix
        let hex_with_prefix = format!("0x{}", hex_str);
        let parsed = parse_public_key(&hex_with_prefix).unwrap();
        assert_eq!(parsed, public_key);
    }

    #[test]
    fn test_parse_private_key() {
        let hex_str = "0123456789abcdef0123456789abcdef";
        let key = parse_private_key(hex_str).unwrap();
        assert!(!key.public_key().as_ref().is_empty());
    }

    #[test]
    fn test_parse_bootstrapper() {
        let private_key = PrivateKey::from_seed(42);
        let public_key = private_key.public_key();
        let hex_str = hex::encode(public_key.as_ref());

        let bootstrapper_str = format!("{}@127.0.0.1:9000", hex_str);
        let (parsed_key, addr) = parse_bootstrapper(&bootstrapper_str).unwrap();

        assert_eq!(parsed_key, public_key);
        assert_eq!(addr, "127.0.0.1:9000".parse().unwrap());
    }
}
