//! Node configuration.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;

/// Node configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// API configuration.
    pub api: ApiConfig,
    /// P2P configuration.
    pub p2p: P2pConfig,
    /// Storage configuration.
    pub storage: StorageConfig,
}

/// API server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Listen address.
    pub listen_addr: SocketAddr,
    /// Enable CORS.
    pub cors_enabled: bool,
}

/// P2P network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    /// Listen address.
    pub listen_addr: SocketAddr,
    /// Bootstrap nodes.
    pub bootstrap_nodes: Vec<SocketAddr>,
    /// Maximum peers.
    pub max_peers: usize,
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory.
    pub data_dir: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".parse().unwrap(),
                cors_enabled: true,
            },
            p2p: P2pConfig {
                listen_addr: "0.0.0.0:9000".parse().unwrap(),
                bootstrap_nodes: vec![],
                max_peers: 50,
            },
            storage: StorageConfig {
                data_dir: "./data".to_string(),
            },
        }
    }
}

impl NodeConfig {
    /// Loads configuration from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: NodeConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Saves configuration to a TOML file.
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn config_roundtrip() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");

        let config = NodeConfig::default();
        config.to_file(&path).unwrap();

        let loaded = NodeConfig::from_file(&path).unwrap();
        assert_eq!(config.api.listen_addr, loaded.api.listen_addr);
    }
}
