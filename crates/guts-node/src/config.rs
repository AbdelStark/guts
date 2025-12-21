//! # Node Configuration
//!
//! Production-grade configuration management with:
//!
//! - Environment variable support (12-factor app)
//! - Configuration file loading (YAML/TOML)
//! - Comprehensive validation
//! - Sensible defaults
//!
//! ## Environment Variables
//!
//! | Variable | Description | Default |
//! |----------|-------------|---------|
//! | `GUTS_API_ADDR` | HTTP API address | `127.0.0.1:8080` |
//! | `GUTS_P2P_ADDR` | P2P listen address | `0.0.0.0:9000` |
//! | `GUTS_METRICS_ADDR` | Metrics endpoint | `0.0.0.0:9090` |
//! | `GUTS_LOG_LEVEL` | Log level | `info` |
//! | `GUTS_LOG_FORMAT` | Log format (json/pretty) | `json` |
//! | `GUTS_PRIVATE_KEY` | Ed25519 private key (hex) | *required for P2P* |
//! | `GUTS_DATA_DIR` | Data directory | `./data` |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use guts_node::config::NodeConfig;
//!
//! let config = NodeConfig::from_env().expect("Invalid configuration");
//! config.validate_config().expect("Configuration validation failed");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use validator::Validate;

/// Configuration validation errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Invalid configuration value.
    #[error("Invalid configuration: {0}")]
    Invalid(String),

    /// Missing required configuration.
    #[error("Missing required configuration: {0}")]
    Missing(String),

    /// Environment variable parsing error.
    #[error("Failed to parse environment variable {key}: {message}")]
    EnvParse { key: String, message: String },

    /// File loading error.
    #[error("Failed to load configuration file: {0}")]
    FileLoad(String),

    /// Validation error.
    #[error("Validation failed: {0}")]
    Validation(String),
}

/// Main node configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Validate)]
pub struct NodeConfig {
    /// HTTP API configuration.
    #[validate(nested)]
    #[serde(default)]
    pub api: ApiConfig,

    /// P2P network configuration.
    #[validate(nested)]
    #[serde(default)]
    pub p2p: P2pConfig,

    /// Metrics configuration.
    #[validate(nested)]
    #[serde(default)]
    pub metrics: MetricsConfig,

    /// Logging configuration.
    #[validate(nested)]
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Storage configuration.
    #[validate(nested)]
    #[serde(default)]
    pub storage: StorageConfig,

    /// Resilience configuration.
    #[validate(nested)]
    #[serde(default)]
    pub resilience: ResilienceConfig,
}

impl NodeConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // API configuration
        if let Ok(addr) = std::env::var("GUTS_API_ADDR") {
            config.api.addr = addr.parse().map_err(|_| ConfigError::EnvParse {
                key: "GUTS_API_ADDR".to_string(),
                message: "Invalid socket address".to_string(),
            })?;
        }

        if let Ok(timeout) = std::env::var("GUTS_REQUEST_TIMEOUT") {
            config.api.request_timeout_secs =
                timeout.parse().map_err(|_| ConfigError::EnvParse {
                    key: "GUTS_REQUEST_TIMEOUT".to_string(),
                    message: "Invalid timeout value".to_string(),
                })?;
        }

        // P2P configuration
        if let Ok(addr) = std::env::var("GUTS_P2P_ADDR") {
            config.p2p.addr = addr.parse().map_err(|_| ConfigError::EnvParse {
                key: "GUTS_P2P_ADDR".to_string(),
                message: "Invalid socket address".to_string(),
            })?;
        }

        if let Ok(key) = std::env::var("GUTS_PRIVATE_KEY") {
            config.p2p.private_key = Some(key);
        }

        if let Ok(enabled) = std::env::var("GUTS_P2P_ENABLED") {
            config.p2p.enabled = enabled.parse().unwrap_or(true);
        }

        // Metrics configuration
        if let Ok(addr) = std::env::var("GUTS_METRICS_ADDR") {
            config.metrics.addr = addr.parse().map_err(|_| ConfigError::EnvParse {
                key: "GUTS_METRICS_ADDR".to_string(),
                message: "Invalid socket address".to_string(),
            })?;
        }

        if let Ok(enabled) = std::env::var("GUTS_METRICS_ENABLED") {
            config.metrics.enabled = enabled.parse().unwrap_or(true);
        }

        // Logging configuration
        if let Ok(level) = std::env::var("GUTS_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(format) = std::env::var("GUTS_LOG_FORMAT") {
            config.logging.format = format;
        }

        // Storage configuration
        if let Ok(dir) = std::env::var("GUTS_DATA_DIR") {
            config.storage.data_dir = PathBuf::from(dir);
        }

        Ok(config)
    }

    /// Load configuration from a YAML file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::FileLoad(e.to_string()))?;

        serde_yaml::from_str(&content).map_err(|e| ConfigError::FileLoad(e.to_string()))
    }

    /// Merge configuration from environment variables.
    pub fn merge_env(&mut self) -> Result<(), ConfigError> {
        let env_config = Self::from_env()?;

        // Only override if explicitly set in environment
        if std::env::var("GUTS_API_ADDR").is_ok() {
            self.api.addr = env_config.api.addr;
        }
        if std::env::var("GUTS_P2P_ADDR").is_ok() {
            self.p2p.addr = env_config.p2p.addr;
        }
        if std::env::var("GUTS_PRIVATE_KEY").is_ok() {
            self.p2p.private_key = env_config.p2p.private_key;
        }
        if std::env::var("GUTS_METRICS_ADDR").is_ok() {
            self.metrics.addr = env_config.metrics.addr;
        }
        if std::env::var("GUTS_LOG_LEVEL").is_ok() {
            self.logging.level = env_config.logging.level;
        }
        if std::env::var("GUTS_LOG_FORMAT").is_ok() {
            self.logging.format = env_config.logging.format;
        }
        if std::env::var("GUTS_DATA_DIR").is_ok() {
            self.storage.data_dir = env_config.storage.data_dir;
        }

        Ok(())
    }

    /// Validate the configuration.
    pub fn validate_config(&self) -> Result<(), ConfigError> {
        // Use validator crate
        self.validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;

        // Custom validations
        if let Some(ref key) = self.p2p.private_key {
            validate_hex_key(key, "private_key", 64)?;
        }

        if let Some(ref share) = self.p2p.share {
            validate_hex_key(share, "share", 64)?;
        }

        if let Some(ref polynomial) = self.p2p.polynomial {
            validate_hex_key(polynomial, "polynomial", 64)?;
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.to_lowercase().as_str()) {
            return Err(ConfigError::Invalid(format!(
                "Invalid log level '{}'. Valid values: {:?}",
                self.logging.level, valid_levels
            )));
        }

        // Validate log format
        let valid_formats = ["json", "pretty"];
        if !valid_formats.contains(&self.logging.format.to_lowercase().as_str()) {
            return Err(ConfigError::Invalid(format!(
                "Invalid log format '{}'. Valid values: {:?}",
                self.logging.format, valid_formats
            )));
        }

        Ok(())
    }
}

/// API server configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct ApiConfig {
    /// Listen address.
    pub addr: SocketAddr,

    /// Request timeout in seconds.
    #[validate(range(min = 1, max = 3600))]
    pub request_timeout_secs: u32,

    /// Maximum request body size in bytes.
    #[validate(range(min = 1024, max = 104857600))] // 1KB to 100MB
    pub max_body_size: usize,

    /// Maximum concurrent connections.
    #[validate(range(min = 1, max = 100000))]
    pub max_connections: u32,

    /// Enable CORS.
    pub cors_enabled: bool,

    /// CORS allowed origins (empty = all).
    pub cors_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".parse().expect("Invalid default address"),
            request_timeout_secs: 30,
            max_body_size: 50 * 1024 * 1024, // 50MB
            max_connections: 10000,
            cors_enabled: true,
            cors_origins: vec![],
        }
    }
}

/// P2P network configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct P2pConfig {
    /// Whether P2P is enabled.
    pub enabled: bool,

    /// P2P listen address.
    pub addr: SocketAddr,

    /// Ed25519 private key (hex encoded).
    pub private_key: Option<String>,

    /// BLS share (hex encoded).
    pub share: Option<String>,

    /// BLS polynomial (hex encoded).
    pub polynomial: Option<String>,

    /// Allowed peers (public keys).
    pub allowed_peers: Vec<String>,

    /// Bootstrap node addresses.
    pub bootstrappers: Vec<String>,

    /// Message backlog size.
    #[validate(range(min = 16, max = 65536))]
    pub message_backlog: usize,

    /// Mailbox size.
    #[validate(range(min = 16, max = 65536))]
    pub mailbox_size: usize,

    /// P2P operation timeout in seconds.
    #[validate(range(min = 1, max = 300))]
    pub timeout_secs: u32,
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            addr: "0.0.0.0:9000".parse().expect("Invalid default address"),
            private_key: None,
            share: None,
            polynomial: None,
            allowed_peers: Vec::new(),
            bootstrappers: Vec::new(),
            message_backlog: 1024,
            mailbox_size: 1024,
            timeout_secs: 10,
        }
    }
}

/// Metrics configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct MetricsConfig {
    /// Whether metrics are enabled.
    pub enabled: bool,

    /// Metrics endpoint address.
    pub addr: SocketAddr,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            addr: "0.0.0.0:9090".parse().expect("Invalid default address"),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error).
    pub level: String,

    /// Log format (json, pretty).
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct StorageConfig {
    /// Data directory.
    pub data_dir: PathBuf,

    /// Enable data persistence (future feature).
    pub persistent: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            persistent: false,
        }
    }
}

/// Resilience configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Validate)]
pub struct ResilienceConfig {
    /// Retry configuration.
    #[validate(nested)]
    #[serde(default)]
    pub retry: RetryConfig,

    /// Circuit breaker configuration.
    #[validate(nested)]
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
}


/// Retry configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct RetryConfig {
    /// Maximum retry attempts.
    #[validate(range(min = 0, max = 10))]
    pub max_attempts: u32,

    /// Initial delay in milliseconds.
    #[validate(range(min = 10, max = 60000))]
    pub initial_delay_ms: u64,

    /// Maximum delay in milliseconds.
    #[validate(range(min = 100, max = 300000))]
    pub max_delay_ms: u64,

    /// Backoff multiplier.
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Convert to RetryPolicy.
    pub fn to_policy(&self) -> crate::resilience::RetryPolicy {
        crate::resilience::RetryPolicy {
            max_attempts: self.max_attempts,
            initial_delay: Duration::from_millis(self.initial_delay_ms),
            max_delay: Duration::from_millis(self.max_delay_ms),
            multiplier: self.multiplier,
            jitter: true,
        }
    }
}

/// Circuit breaker configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening.
    #[validate(range(min = 1, max = 100))]
    pub failure_threshold: u32,

    /// Number of successes to close from half-open.
    #[validate(range(min = 1, max = 100))]
    pub success_threshold: u32,

    /// Timeout in seconds before transitioning to half-open.
    #[validate(range(min = 1, max = 3600))]
    pub timeout_secs: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_secs: 30,
        }
    }
}

impl CircuitBreakerConfig {
    /// Convert to CircuitBreaker.
    pub fn to_circuit_breaker(&self) -> crate::resilience::CircuitBreaker {
        crate::resilience::CircuitBreaker::new(
            self.failure_threshold,
            self.success_threshold,
            Duration::from_secs(self.timeout_secs as u64),
        )
    }
}

/// Validate a hexadecimal key.
fn validate_hex_key(key: &str, name: &str, expected_len: usize) -> Result<(), ConfigError> {
    // Remove 0x prefix if present
    let key = key.strip_prefix("0x").unwrap_or(key);

    if key.len() != expected_len {
        return Err(ConfigError::Invalid(format!(
            "{} must be {} hex characters, got {}",
            name,
            expected_len,
            key.len()
        )));
    }

    if !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ConfigError::Invalid(format!(
            "{} contains non-hexadecimal characters",
            name
        )));
    }

    Ok(())
}

/// Legacy configuration for backwards compatibility.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_hex_key_validation() {
        // Valid key
        assert!(validate_hex_key(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "test",
            64
        )
        .is_ok());

        // With 0x prefix
        assert!(validate_hex_key(
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "test",
            64
        )
        .is_ok());

        // Wrong length
        assert!(validate_hex_key("0123456789abcdef", "test", 64).is_err());

        // Invalid characters
        assert!(validate_hex_key(
            "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
            "test",
            64
        )
        .is_err());
    }

    #[test]
    fn test_log_level_validation() {
        let mut config = NodeConfig::default();

        // Valid levels
        for level in &["trace", "debug", "info", "warn", "error"] {
            config.logging.level = level.to_string();
            assert!(config.validate_config().is_ok());
        }

        // Invalid level
        config.logging.level = "invalid".to_string();
        assert!(config.validate_config().is_err());
    }
}
