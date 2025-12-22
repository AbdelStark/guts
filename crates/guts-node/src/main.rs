//! Guts Node - Decentralized code collaboration node.
//!
//! This is the main entry point for running a Guts validator node.
//!
//! ## Configuration
//!
//! The node can be configured via command-line arguments or environment variables:
//!
//! - `GUTS_API_ADDR` - HTTP API listen address (default: 127.0.0.1:8080)
//! - `GUTS_P2P_ADDR` - P2P listen address (default: 0.0.0.0:9000)
//! - `GUTS_METRICS_ADDR` - Metrics endpoint address (default: 0.0.0.0:9090)
//! - `GUTS_LOG_LEVEL` - Log level (default: info)
//! - `GUTS_LOG_FORMAT` - Log format: json or pretty (default: json)
//! - `GUTS_DATA_DIR` - Data directory (default: ./data)

use clap::Parser;
use guts_auth::AuthStore;
use guts_ci::CiStore;
use guts_collaboration::CollaborationStore;
use guts_compat::CompatStore;
use guts_consensus::{
    ConsensusEngine, EngineConfig, Genesis, Mempool, MempoolConfig, ValidatorConfig, ValidatorSet,
};
use guts_node::api::{create_router, AppState};
use guts_node::config::NodeConfig;
use guts_node::consensus_app::GutsApplication;
use guts_node::health::HealthState;
use guts_node::observability::{init_logging, LogFormat};
use guts_realtime::EventHub;
use guts_storage::RepoStore;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Guts Node - Decentralized code collaboration infrastructure
#[derive(Parser, Debug)]
#[command(name = "guts-node")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,

    /// API listen address (overrides config file and env)
    #[arg(long)]
    api_addr: Option<SocketAddr>,

    /// P2P listen address (overrides config file and env)
    #[arg(long)]
    p2p_addr: Option<SocketAddr>,

    /// Metrics listen address
    #[arg(long)]
    metrics_addr: Option<SocketAddr>,

    /// Data directory
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long)]
    log_level: Option<String>,

    /// Log format (json, pretty)
    #[arg(long)]
    log_format: Option<String>,

    /// Run in local development mode (uses pretty logging)
    #[arg(long)]
    local: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration
    let mut config = if args.config.exists() {
        NodeConfig::from_file(&args.config).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to load config file: {}. Using defaults.",
                e
            );
            NodeConfig::default()
        })
    } else {
        NodeConfig::default()
    };

    // Merge environment variables
    if let Err(e) = config.merge_env() {
        eprintln!("Warning: Failed to merge environment config: {}", e);
    }

    // Override with CLI arguments
    if let Some(addr) = args.api_addr {
        config.api.addr = addr;
    }
    if let Some(addr) = args.p2p_addr {
        config.p2p.addr = addr;
    }
    if let Some(addr) = args.metrics_addr {
        config.metrics.addr = addr;
    }
    if let Some(dir) = args.data_dir {
        config.storage.data_dir = dir;
    }
    if let Some(level) = args.log_level {
        config.logging.level = level;
    }
    if let Some(format) = args.log_format {
        config.logging.format = format;
    }

    // Local mode uses pretty logging
    if args.local {
        config.logging.format = "pretty".to_string();
    }

    // Validate configuration
    if let Err(e) = config.validate_config() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Initialize logging
    let json_format = LogFormat::parse(&config.logging.format) == LogFormat::Json;
    init_logging(&config.logging.level, json_format);

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Starting Guts node");

    tracing::info!(
        api_addr = %config.api.addr,
        p2p_addr = %config.p2p.addr,
        metrics_addr = %config.metrics.addr,
        data_dir = %config.storage.data_dir.display(),
        log_level = %config.logging.level,
        "Node configuration"
    );

    // Create data directory
    if let Err(e) = std::fs::create_dir_all(&config.storage.data_dir) {
        tracing::error!(error = %e, "Failed to create data directory");
        return Err(e.into());
    }

    // Initialize health state
    let health_state = HealthState::new();

    // Create shared stores
    let repos = Arc::new(RepoStore::new());
    let collaboration = Arc::new(CollaborationStore::new());
    let auth = Arc::new(AuthStore::new());
    let realtime = Arc::new(EventHub::new());
    let ci = Arc::new(CiStore::new());
    let compat = Arc::new(CompatStore::new());

    // Initialize real Simplex BFT consensus if enabled
    let simplex_handle = if config.consensus.enabled && config.consensus.use_simplex_bft {
        tracing::info!("Initializing real Simplex BFT consensus");

        // Validate required configuration
        let private_key_hex = config.p2p.private_key.clone().ok_or_else(|| {
            anyhow::anyhow!("GUTS_PRIVATE_KEY is required for Simplex BFT consensus")
        })?;

        // Build simplex config
        let simplex_config = guts_node::consensus_simplex::SimplexConsensusConfig {
            private_key_hex,
            p2p_addr: config.p2p.addr,
            external_addr: None, // Will be set from config if needed
            bootstrappers: config.p2p.bootstrappers.clone(),
            participants: config.p2p.allowed_peers.clone(),
            data_dir: config.storage.data_dir.clone(),
            local: true, // Local mode for development
            mailbox_size: config.p2p.mailbox_size,
            message_backlog: config.p2p.message_backlog,
            worker_threads: 4,
        };

        // Start the simplex consensus engine
        match guts_node::consensus_simplex::start_simplex_consensus(simplex_config) {
            Ok(handle) => {
                tracing::info!(
                    public_key = hex::encode(handle.public_key.as_ref()),
                    "Simplex BFT consensus started"
                );
                Some(handle)
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to start Simplex BFT consensus");
                return Err(anyhow::anyhow!("Failed to start Simplex BFT: {}", e));
            }
        }
    } else {
        None
    };

    // Initialize simulation-based consensus if enabled (fallback when simplex is not used)
    let (consensus, mempool, guts_app) =
        if config.consensus.enabled && !config.consensus.use_simplex_bft {
            tracing::info!("Initializing simulation-based consensus engine");

            // Create mempool
            let mempool_config = MempoolConfig {
                max_transactions: config.consensus.mempool_max_txs,
                max_transaction_age: Duration::from_secs(config.consensus.mempool_ttl_secs),
                max_transactions_per_block: config.consensus.max_txs_per_block,
            };
            let mempool = Arc::new(Mempool::new(mempool_config));

            // Create consensus engine config
            let engine_config = EngineConfig {
                block_time: Duration::from_millis(config.consensus.block_time_ms),
                max_txs_per_block: config.consensus.max_txs_per_block,
                max_block_size: config.consensus.max_block_size,
                view_timeout_multiplier: config.consensus.view_timeout_multiplier,
                consensus_enabled: config.consensus.enabled,
            };

            // Parse validator key from P2P config using PrivateKeyExt
            use commonware_cryptography::PrivateKeyExt;
            let validator_key = config.p2p.private_key.as_ref().and_then(|key_hex| {
                let key_hex = key_hex.strip_prefix("0x").unwrap_or(key_hex);
                hex::decode(key_hex).ok().and_then(|bytes| {
                    if bytes.len() >= 8 {
                        // Use the first 8 bytes as a seed
                        let seed = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                        Some(commonware_cryptography::ed25519::PrivateKey::from_seed(
                            seed,
                        ))
                    } else {
                        None
                    }
                })
            });

            // Load validator set from genesis file if available, otherwise single-node mode
            let validators = if let Some(ref genesis_path) = config.consensus.genesis_file {
                tracing::info!(path = %genesis_path.display(), "Loading genesis file");
                match Genesis::load_json(genesis_path) {
                    Ok(genesis) => {
                        tracing::info!(
                            chain_id = %genesis.chain_id,
                            validator_count = genesis.validators.len(),
                            "Genesis loaded successfully"
                        );
                        genesis
                            .into_validator_set()
                            .expect("Failed to create validator set from genesis")
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to load genesis file");
                        return Err(anyhow::anyhow!("Failed to load genesis: {}", e));
                    }
                }
            } else {
                // Fallback to single-node mode
                tracing::info!("No genesis file, using single-node mode");
                let validator_config = ValidatorConfig {
                    min_validators: 0, // Allow single-node mode
                    max_validators: 100,
                    quorum_threshold: 2.0 / 3.0,
                    block_time_ms: config.consensus.block_time_ms,
                };

                if let Some(ref key) = validator_key {
                    use guts_consensus::{SerializablePublicKey, Validator};
                    let pubkey = SerializablePublicKey::from_pubkey(
                        &commonware_cryptography::Signer::public_key(key),
                    );
                    let validator = Validator::new(pubkey, "local", 1, config.p2p.addr);
                    ValidatorSet::new(vec![validator], 0, validator_config)
                        .expect("Failed to create validator set")
                } else {
                    ValidatorSet::new(vec![], 0, validator_config)
                        .expect("Failed to create empty validator set")
                }
            };

            // Create consensus engine
            let consensus = Arc::new(ConsensusEngine::new(
                engine_config,
                validator_key,
                validators,
                mempool.clone(),
            ));

            // Create the Guts application that applies finalized blocks to state
            let guts_app = Arc::new(GutsApplication::new(
                repos.clone(),
                collaboration.clone(),
                auth.clone(),
                realtime.clone(),
            ));

            tracing::info!(
                enabled = config.consensus.enabled,
                block_time_ms = config.consensus.block_time_ms,
                max_txs_per_block = config.consensus.max_txs_per_block,
                "Simulation-based consensus engine initialized"
            );

            (Some(consensus), Some(mempool), Some(guts_app))
        } else if !config.consensus.enabled {
            tracing::info!("Consensus disabled, running in single-node mode");
            (None, None, None)
        } else {
            // Simplex BFT is enabled, no simulation consensus needed
            (None, None, None)
        };

    // Log simplex consensus status
    if simplex_handle.is_some() {
        tracing::info!("Using real Simplex BFT consensus");
    }

    // Create application state
    let state = AppState {
        repos,
        p2p: None, // P2P is optional, enabled via configuration
        consensus,
        mempool,
        collaboration,
        auth,
        realtime,
        ci,
        compat,
    };

    // Mark storage and realtime as healthy
    health_state.set_storage_healthy(true);
    health_state.set_realtime_healthy(true, 0);

    // Spawn consensus engine task if enabled
    if let (Some(ref consensus), Some(ref app)) = (&state.consensus, &guts_app) {
        let consensus = consensus.clone();
        let app = app.clone();
        tokio::spawn(async move {
            if let Err(e) = consensus.run(app).await {
                tracing::error!(error = %e, "Consensus engine error");
            }
        });
        tracing::info!("Consensus engine started");
    }

    // Create router with health state
    let app = create_router(state, health_state.clone());

    // Create TCP listener
    let listener = match tokio::net::TcpListener::bind(&config.api.addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, addr = %config.api.addr, "Failed to bind to address");
            return Err(e.into());
        }
    };

    tracing::info!(addr = %config.api.addr, "HTTP server listening");

    // Mark startup as complete and service as ready
    health_state.set_startup_complete(true);
    health_state.set_ready(true);

    tracing::info!("Node startup complete, ready to accept connections");

    // Start server with graceful shutdown
    let shutdown_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;

    if let Err(e) = shutdown_result {
        tracing::error!(error = %e, "Server error");
        return Err(e.into());
    }

    tracing::info!("Guts node stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!(error = %e, "Failed to install CTRL+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to install SIGTERM handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, initiating graceful shutdown");
}
