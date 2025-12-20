//! Guts Node - Decentralized code collaboration node.
//!
//! This is the main entry point for running a Guts validator node.

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

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

fn main() {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("guts={}", args.log_level).into()),
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
    if let Err(e) = std::fs::create_dir_all(&args.data_dir) {
        tracing::error!(error = %e, "Failed to create data directory");
        std::process::exit(1);
    }

    // TODO: Initialize commonware runtime and start node
    // This will be expanded with full P2P and consensus integration

    tracing::info!("Guts node initialized successfully");
    tracing::info!("Node is ready. Press Ctrl+C to stop.");

    // For now, just wait for interrupt
    std::thread::park();
}
