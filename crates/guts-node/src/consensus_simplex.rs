//! Simplex BFT consensus integration for guts-node.
//!
//! This module provides the integration between the HTTP API server and the
//! real Simplex BFT consensus engine from commonware.
//!
//! # Architecture
//!
//! The consensus engine runs in its own commonware runtime context, while the
//! HTTP server runs in the main Tokio runtime. They communicate via:
//!
//! - **Mempool**: Transactions are submitted via the HTTP API
//! - **State**: Finalized blocks update shared application state
//! - **P2P**: Consensus messages are exchanged via authenticated channels

use crate::p2p::{parse_bootstrapper, parse_private_key, TokioContext};
use commonware_cryptography::{ed25519::PublicKey, Signer};
use commonware_runtime::{tokio as cw_tokio, Runner};
use guts_consensus::simplex::{Config as SimplexConfig, Engine as SimplexEngine};
use std::{net::SocketAddr, path::PathBuf};
use tracing::{error, info, warn};

/// Configuration for the Simplex BFT consensus.
#[derive(Clone)]
pub struct SimplexConsensusConfig {
    /// Private key hex string for this validator.
    pub private_key_hex: String,
    /// P2P listen address.
    pub p2p_addr: SocketAddr,
    /// External P2P address (for NAT).
    pub external_addr: Option<SocketAddr>,
    /// Bootstrapper addresses (format: "pubkey@host:port").
    pub bootstrappers: Vec<String>,
    /// Participant public keys (hex strings).
    pub participants: Vec<String>,
    /// Storage directory.
    pub data_dir: PathBuf,
    /// Local mode (relaxed timing for development).
    pub local: bool,
    /// Mailbox size.
    pub mailbox_size: usize,
    /// Message backlog size.
    pub message_backlog: usize,
    /// Worker threads for the consensus runtime.
    pub worker_threads: usize,
}

impl Default for SimplexConsensusConfig {
    fn default() -> Self {
        Self {
            private_key_hex: String::new(),
            p2p_addr: "0.0.0.0:9000".parse().unwrap(),
            external_addr: None,
            bootstrappers: Vec::new(),
            participants: Vec::new(),
            data_dir: PathBuf::from("./data"),
            local: true,
            mailbox_size: 1024,
            message_backlog: 1024,
            worker_threads: 4,
        }
    }
}

/// Handle to a running Simplex consensus engine.
pub struct SimplexConsensusHandle {
    /// The public key of this validator.
    pub public_key: PublicKey,
}

/// Start the Simplex BFT consensus engine.
///
/// This spawns a new thread that runs the commonware runtime with the
/// consensus engine. The engine will connect to other validators via P2P
/// and participate in BFT consensus.
///
/// Returns a handle that can be used to interact with the consensus engine.
pub fn start_simplex_consensus(
    config: SimplexConsensusConfig,
) -> Result<SimplexConsensusHandle, String> {
    // Parse private key
    let private_key = parse_private_key(&config.private_key_hex)?;
    let public_key = private_key.public_key();

    info!(
        public_key = hex::encode(public_key.as_ref()),
        p2p_addr = %config.p2p_addr,
        "Starting Simplex BFT consensus"
    );

    // Parse participants
    let mut participants: Vec<PublicKey> = Vec::new();
    for pk_hex in &config.participants {
        let pk = crate::p2p::parse_public_key(pk_hex)?;
        participants.push(pk);
    }

    // Make sure we're in the participant list
    if !participants.contains(&public_key) {
        participants.push(public_key.clone());
    }

    info!(
        participant_count = participants.len(),
        "Parsed validator set"
    );

    // Parse bootstrappers
    let mut bootstrappers = Vec::new();
    for bs in &config.bootstrappers {
        let (pk, addr) = parse_bootstrapper(bs)?;
        bootstrappers.push((pk, addr));
    }

    // Clone config values for the thread
    let p2p_addr = config.p2p_addr;
    let external_addr = config.external_addr.unwrap_or(p2p_addr);
    let data_dir = config.data_dir.clone();
    let local = config.local;
    let mailbox_size = config.mailbox_size;
    let message_backlog = config.message_backlog;
    let worker_threads = config.worker_threads;
    let pk_clone = public_key.clone();

    // Spawn the consensus engine in a separate thread with its own runtime
    std::thread::spawn(move || {
        // Configure commonware tokio runtime
        let runtime_cfg = cw_tokio::Config::default()
            .with_tcp_nodelay(Some(true))
            .with_worker_threads(worker_threads)
            .with_storage_directory(data_dir)
            .with_catch_panics(false);

        let executor = cw_tokio::Runner::new(runtime_cfg);

        // Start the runtime
        executor.start(move |context: TokioContext| async move {
            run_consensus_engine(
                context,
                private_key,
                participants,
                bootstrappers,
                p2p_addr,
                external_addr,
                local,
                mailbox_size,
                message_backlog,
            )
            .await;
        });
    });

    Ok(SimplexConsensusHandle {
        public_key: pk_clone,
    })
}

/// Run the consensus engine (called within the commonware runtime).
#[allow(clippy::too_many_arguments)]
async fn run_consensus_engine(
    context: TokioContext,
    private_key: commonware_cryptography::ed25519::PrivateKey,
    participants: Vec<PublicKey>,
    bootstrappers: Vec<(PublicKey, SocketAddr)>,
    p2p_addr: SocketAddr,
    external_addr: SocketAddr,
    local: bool,
    mailbox_size: usize,
    message_backlog: usize,
) {
    use crate::p2p::{AuthenticatedNetwork, AuthenticatedP2pConfig};
    use futures::future::try_join_all;

    let public_key = private_key.public_key();

    info!(
        public_key = hex::encode(public_key.as_ref()),
        "Consensus engine starting"
    );

    // Create P2P config
    let mut p2p_config = AuthenticatedP2pConfig::new(private_key.clone(), p2p_addr.port());
    p2p_config.listen_addr = p2p_addr;
    p2p_config.external_addr = external_addr;
    p2p_config.bootstrappers = bootstrappers.clone();
    p2p_config.mailbox_size = mailbox_size;
    p2p_config.message_backlog = message_backlog;
    p2p_config.local = local;

    // Create the P2P network
    let (network, channels, network_handle) =
        AuthenticatedNetwork::new(context.clone(), p2p_config, participants.clone()).await;

    info!(
        public_key = hex::encode(network.public_key.as_ref()),
        "P2P network initialized"
    );

    // Create the simplex engine config
    let simplex_config = SimplexConfig::new(
        network.oracle.clone(),
        public_key.clone(),
        private_key,
        participants,
    );

    // Create the simplex engine
    let simplex_engine = SimplexEngine::new(context.clone(), simplex_config).await;

    info!("Simplex BFT engine created, starting...");

    // Start the simplex engine with P2P channels
    let engine_handle = simplex_engine.start(
        channels.pending,
        channels.recovered,
        channels.resolver,
        channels.broadcast,
        channels.marshal,
    );

    info!("Simplex BFT consensus running");

    // Wait for any task to complete (they should run forever)
    if let Err(e) = try_join_all(vec![network_handle, engine_handle]).await {
        error!(?e, "Consensus engine task failed");
    } else {
        warn!("Consensus engine stopped unexpectedly");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SimplexConsensusConfig::default();
        assert!(config.local);
        assert_eq!(config.worker_threads, 4);
    }
}
