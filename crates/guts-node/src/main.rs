//! # Guts Node
//!
//! The main entry point for running a Guts node.

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

use config::NodeConfig;

/// Guts Node - Decentralized code collaboration
#[derive(Parser, Debug)]
#[command(name = "guts-node")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
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

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("guts={}", args.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting Guts node"
    );

    // Load configuration
    let config = if args.config.exists() {
        NodeConfig::from_file(&args.config)?
    } else {
        NodeConfig::default()
    };

    tracing::info!(
        api_addr = %args.api_addr,
        p2p_addr = %args.p2p_addr,
        data_dir = %args.data_dir.display(),
        "Node configuration loaded"
    );

    // Create data directory
    std::fs::create_dir_all(&args.data_dir)?;

    // Generate or load identity
    let keypair = guts_identity::Keypair::generate();
    tracing::info!(
        peer_id = %guts_p2p::PeerId::from_public_key(&keypair.public_key()),
        "Node identity initialized"
    );

    // Start API server
    let api_router = guts_api::create_router();
    let api_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(args.api_addr).await.unwrap();
        tracing::info!(addr = %args.api_addr, "API server listening");
        axum::serve(listener, api_router).await.unwrap();
    });

    // Start P2P node
    let p2p_config = guts_p2p::NodeConfig {
        listen_addr: args.p2p_addr,
        ..Default::default()
    };
    let p2p_node = guts_p2p::Node::new(p2p_config, keypair);
    p2p_node.start().await?;

    tracing::info!("Guts node running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    tracing::info!("Shutting down...");
    p2p_node.stop().await;

    Ok(())
}
