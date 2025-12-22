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
use guts_node::api::{create_router, AppState};
use guts_node::config::NodeConfig;
use guts_node::health::HealthState;
use guts_node::observability::{init_logging, LogFormat};
use guts_realtime::EventHub;
use guts_storage::RepoStore;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

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

    // Create application state
    let state = AppState {
        repos: Arc::new(RepoStore::new()),
        p2p: None,       // P2P is optional, enabled via configuration
        consensus: None, // Consensus is optional, enabled via configuration
        mempool: None,   // Mempool is created with consensus
        collaboration: Arc::new(CollaborationStore::new()),
        auth: Arc::new(AuthStore::new()),
        realtime: Arc::new(EventHub::new()),
        ci: Arc::new(CiStore::new()),
        compat: Arc::new(CompatStore::new()),
    };

    // Mark storage and realtime as healthy
    health_state.set_storage_healthy(true);
    health_state.set_realtime_healthy(true, 0);

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
