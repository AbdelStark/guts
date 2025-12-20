//! Node configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Configuration for the Guts node.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Ed25519 private key (hex encoded).
    pub private_key: String,
    /// BLS share (hex encoded).
    pub share: String,
    /// BLS polynomial (hex encoded).
    pub polynomial: String,

    /// P2P listen port.
    pub port: u16,
    /// Metrics HTTP port.
    pub metrics_port: u16,
    /// Data directory.
    pub directory: String,
    /// Number of worker threads.
    pub worker_threads: usize,
    /// Log level.
    pub log_level: String,

    /// Run in local mode.
    pub local: bool,
    /// Allowed peers (public keys).
    pub allowed_peers: Vec<String>,
    /// Bootstrap node addresses.
    pub bootstrappers: Vec<String>,

    /// Message backlog size.
    pub message_backlog: usize,
    /// Mailbox size.
    pub mailbox_size: usize,
    /// Deque size for pending messages.
    pub deque_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            private_key: String::new(),
            share: String::new(),
            polynomial: String::new(),
            port: 9000,
            metrics_port: 9090,
            directory: "./data".to_string(),
            worker_threads: 4,
            log_level: "info".to_string(),
            local: false,
            allowed_peers: Vec::new(),
            bootstrappers: Vec::new(),
            message_backlog: 1024,
            mailbox_size: 1024,
            deque_size: 10,
        }
    }
}

/// Peer addresses for local mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peers {
    /// Map of public key to socket address.
    pub addresses: HashMap<String, SocketAddr>,
}
