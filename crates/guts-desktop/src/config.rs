//! # Configuration Persistence
//!
//! Save and load settings to/from disk.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::auth::Credentials;

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// URL of the guts-node to connect to.
    pub node_url: String,

    /// Currently active credentials (if logged in).
    #[serde(default)]
    pub credentials: Option<Credentials>,

    /// All saved accounts for quick switching.
    ///
    /// Each account stores the username, public key, and token.
    /// Users can switch between accounts without re-registering.
    #[serde(default)]
    pub accounts: Vec<Credentials>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_url: "http://127.0.0.1:8080".to_string(),
            credentials: None,
            accounts: Vec::new(),
        }
    }
}

impl Config {
    /// Returns the config file path.
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("guts").join("config.json"))
    }

    /// Loads configuration from disk, or returns default if not found.
    ///
    /// Also performs migration: if credentials exist but aren't in accounts list,
    /// they will be added automatically.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            tracing::warn!("Could not determine config directory");
            return Self::default();
        };

        if !path.exists() {
            tracing::debug!(?path, "Config file not found, using defaults");
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(mut config) => {
                    tracing::info!(?path, "Loaded configuration");

                    // Migration: ensure credentials are in accounts list
                    Self::migrate_credentials_to_accounts(&mut config);

                    config
                }
                Err(e) => {
                    tracing::warn!(?path, error = %e, "Failed to parse config, using defaults");
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!(?path, error = %e, "Failed to read config, using defaults");
                Self::default()
            }
        }
    }

    /// Migrate existing credentials to accounts list if not already present.
    fn migrate_credentials_to_accounts(config: &mut Self) {
        if let Some(ref creds) = config.credentials {
            // Check if credentials already exist in accounts
            let exists = config.accounts.iter().any(|a| a.username == creds.username);
            if !exists && creds.token.is_some() {
                tracing::info!(username = %creds.username, "Migrating credentials to accounts list");
                config.accounts.push(creds.clone());
                // Save the migrated config
                if let Err(e) = config.save() {
                    tracing::warn!("Failed to save migrated config: {}", e);
                }
            }
        }
    }

    /// Saves configuration to disk.
    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::config_path() else {
            return Err("Could not determine config directory".to_string());
        };

        // Create config directory if needed
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!("Failed to create config directory: {}", e));
            }
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, contents).map_err(|e| format!("Failed to write config: {}", e))?;

        tracing::info!(?path, "Saved configuration");
        Ok(())
    }
}
