//! Guts Node - Decentralized code collaboration node.
//!
//! This is the main entry point for running a Guts validator node.

use clap::Parser;
use guts_auth::AuthStore;
use guts_ci::CiStore;
use guts_collaboration::CollaborationStore;
use guts_node::api::{create_router, AppState};
use guts_realtime::EventHub;
use guts_storage::RepoStore;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Guts Node - Decentralized code collaboration infrastructure
#[derive(Parser, Debug)]
#[command(name = "guts-node")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,

    /// API listen address
    #[arg(long, default_value = "127.0.0.1:8080")]
    api_addr: SocketAddr,

    /// P2P listen address
    #[arg(long, default_value = "0.0.0.0:9000")]
    p2p_addr: SocketAddr,

    /// Data directory
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Run in local development mode
    #[arg(long)]
    local: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("guts={},tower_http=debug", args.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Starting Guts node");

    tracing::info!(
        api_addr = %args.api_addr,
        p2p_addr = %args.p2p_addr,
        data_dir = %args.data_dir.display(),
        local = args.local,
        "Node configuration"
    );

    // Create data directory
    std::fs::create_dir_all(&args.data_dir)?;

    // Create application state
    let state = AppState {
        repos: Arc::new(RepoStore::new()),
        p2p: None, // P2P is optional, enabled via configuration
        collaboration: Arc::new(CollaborationStore::new()),
        auth: Arc::new(AuthStore::new()),
        realtime: Arc::new(EventHub::new()),
        ci: Arc::new(CiStore::new()),
    };

    // Create router
    let app = create_router(state);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&args.api_addr).await?;
    tracing::info!(addr = %args.api_addr, "HTTP server listening");

    // Start server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Guts node stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
    tracing::info!("Shutdown signal received");
}
