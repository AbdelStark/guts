//! Genesis configuration for the consensus network.
//!
//! The genesis file defines the initial state of the network including
//! the initial validator set and consensus parameters.

use crate::error::{ConsensusError, Result};
use crate::transaction::SerializablePublicKey;
use crate::validator::{Validator, ValidatorConfig, ValidatorSet};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;

/// Genesis configuration for a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisValidator {
    /// Human-readable name.
    pub name: String,

    /// Public key (hex-encoded).
    pub pubkey: String,

    /// Voting weight.
    pub weight: u64,

    /// Network address.
    pub addr: String,
}

impl GenesisValidator {
    /// Converts to a `Validator`.
    pub fn into_validator(self) -> Result<Validator> {
        // Validate the public key format
        let pubkey_bytes =
            hex::decode(&self.pubkey).map_err(|e| ConsensusError::InvalidGenesis(e.to_string()))?;

        if pubkey_bytes.len() != 32 {
            return Err(ConsensusError::InvalidGenesis(format!(
                "invalid public key length: expected 32 bytes, got {}",
                pubkey_bytes.len()
            )));
        }

        let pubkey = SerializablePublicKey::from_hex(&self.pubkey);

        let addr: SocketAddr = self
            .addr
            .parse()
            .map_err(|e| ConsensusError::InvalidGenesis(format!("invalid address: {}", e)))?;

        Ok(Validator::new(pubkey, self.name, self.weight, addr))
    }
}

/// Genesis repository (for testnet seeding).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisRepository {
    /// Owner username.
    pub owner: String,

    /// Repository name.
    pub name: String,

    /// Description.
    pub description: String,

    /// Default branch.
    pub default_branch: String,
}

/// Consensus parameters from genesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusParams {
    /// Target block time in milliseconds.
    pub block_time_ms: u64,

    /// Maximum transactions per block.
    pub max_txs_per_block: usize,

    /// Maximum block size in bytes.
    pub max_block_size: usize,

    /// View timeout multiplier.
    pub view_timeout_multiplier: f64,

    /// Minimum validators.
    pub min_validators: usize,

    /// Maximum validators.
    pub max_validators: usize,
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            block_time_ms: 2000,
            max_txs_per_block: 1000,
            max_block_size: 10 * 1024 * 1024, // 10 MB
            view_timeout_multiplier: 2.0,
            min_validators: 4,
            max_validators: 100,
        }
    }
}

/// Complete genesis configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genesis {
    /// Network identifier (chain ID).
    pub chain_id: String,

    /// Genesis timestamp (unix milliseconds).
    pub timestamp: u64,

    /// Initial validators.
    pub validators: Vec<GenesisValidator>,

    /// Initial repositories (optional, for testnet).
    #[serde(default)]
    pub repositories: Vec<GenesisRepository>,

    /// Consensus parameters.
    #[serde(default)]
    pub consensus: ConsensusParams,
}

impl Genesis {
    /// Creates a new genesis configuration.
    pub fn new(chain_id: impl Into<String>, timestamp: u64) -> Self {
        Self {
            chain_id: chain_id.into(),
            timestamp,
            validators: Vec::new(),
            repositories: Vec::new(),
            consensus: ConsensusParams::default(),
        }
    }

    /// Adds a validator to the genesis.
    pub fn with_validator(mut self, validator: GenesisValidator) -> Self {
        self.validators.push(validator);
        self
    }

    /// Sets the consensus parameters.
    pub fn with_consensus_params(mut self, params: ConsensusParams) -> Self {
        self.consensus = params;
        self
    }

    /// Loads genesis from a JSON file.
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConsensusError::InvalidGenesis(format!("failed to read file: {}", e)))?;

        let genesis: Genesis = serde_json::from_str(&content)?;
        genesis.validate()?;
        Ok(genesis)
    }

    /// Loads genesis from a YAML file.
    pub fn load_yaml(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConsensusError::InvalidGenesis(format!("failed to read file: {}", e)))?;

        let genesis: Genesis = serde_yaml::from_str(&content)
            .map_err(|e| ConsensusError::InvalidGenesis(e.to_string()))?;
        genesis.validate()?;
        Ok(genesis)
    }

    /// Validates the genesis configuration.
    pub fn validate(&self) -> Result<()> {
        if self.chain_id.is_empty() {
            return Err(ConsensusError::InvalidGenesis("chain_id is empty".into()));
        }

        if self.validators.is_empty() {
            return Err(ConsensusError::InvalidGenesis("no validators".into()));
        }

        if self.validators.len() < self.consensus.min_validators {
            return Err(ConsensusError::InvalidGenesis(format!(
                "need at least {} validators for BFT, got {}",
                self.consensus.min_validators,
                self.validators.len()
            )));
        }

        // Verify all public keys are valid
        for v in &self.validators {
            let _ = v.clone().into_validator()?;
        }

        // Check for duplicate names or pubkeys
        let mut seen_names = std::collections::HashSet::new();
        let mut seen_pubkeys = std::collections::HashSet::new();

        for v in &self.validators {
            if !seen_names.insert(&v.name) {
                return Err(ConsensusError::InvalidGenesis(format!(
                    "duplicate validator name: {}",
                    v.name
                )));
            }
            if !seen_pubkeys.insert(&v.pubkey) {
                return Err(ConsensusError::InvalidGenesis(format!(
                    "duplicate validator pubkey: {}",
                    v.pubkey
                )));
            }
        }

        Ok(())
    }

    /// Converts to a ValidatorSet.
    pub fn into_validator_set(self) -> Result<ValidatorSet> {
        let validators: Result<Vec<_>> = self
            .validators
            .into_iter()
            .map(|gv| gv.into_validator())
            .collect();

        let config = ValidatorConfig {
            min_validators: self.consensus.min_validators,
            max_validators: self.consensus.max_validators,
            quorum_threshold: 2.0 / 3.0,
            block_time_ms: self.consensus.block_time_ms,
        };

        ValidatorSet::new(validators?, 0, config)
    }

    /// Saves genesis to a JSON file.
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(ConsensusError::from)?;

        std::fs::write(path.as_ref(), content)
            .map_err(|e| ConsensusError::InvalidGenesis(format!("failed to write file: {}", e)))?;

        Ok(())
    }

    /// Saves genesis to a YAML file.
    pub fn save_yaml(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| ConsensusError::InvalidGenesis(e.to_string()))?;

        std::fs::write(path.as_ref(), content)
            .map_err(|e| ConsensusError::InvalidGenesis(format!("failed to write file: {}", e)))?;

        Ok(())
    }
}

/// Helper to generate a devnet genesis with test validators.
pub fn generate_devnet_genesis(validator_count: usize) -> Genesis {
    use commonware_cryptography::{ed25519, PrivateKeyExt, Signer};

    let validators: Vec<GenesisValidator> = (0..validator_count as u64)
        .map(|i| {
            let key = ed25519::PrivateKey::from_seed(i);
            let pubkey = hex::encode(key.public_key().as_ref());

            GenesisValidator {
                name: format!("validator-{}", i + 1),
                pubkey,
                weight: 100,
                addr: format!("127.0.0.1:{}", 9000 + i),
            }
        })
        .collect();

    Genesis {
        chain_id: "guts-devnet".into(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        validators,
        repositories: vec![],
        consensus: ConsensusParams::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_validation() {
        let genesis = generate_devnet_genesis(4);
        assert!(genesis.validate().is_ok());
    }

    #[test]
    fn test_genesis_too_few_validators() {
        let genesis = generate_devnet_genesis(2);
        assert!(matches!(
            genesis.validate(),
            Err(ConsensusError::InvalidGenesis(_))
        ));
    }

    #[test]
    fn test_genesis_to_validator_set() {
        let genesis = generate_devnet_genesis(4);
        let set = genesis.into_validator_set().unwrap();

        assert_eq!(set.len(), 4);
        assert_eq!(set.epoch(), 0);
    }

    #[test]
    fn test_genesis_serialization() {
        let genesis = generate_devnet_genesis(4);

        // JSON roundtrip
        let json = serde_json::to_string(&genesis).unwrap();
        let parsed: Genesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.chain_id, genesis.chain_id);
        assert_eq!(parsed.validators.len(), 4);
    }

    #[test]
    fn test_genesis_duplicate_name() {
        let mut genesis = generate_devnet_genesis(4);
        genesis.validators[1].name = genesis.validators[0].name.clone();

        assert!(matches!(
            genesis.validate(),
            Err(ConsensusError::InvalidGenesis(msg)) if msg.contains("duplicate validator name")
        ));
    }

    #[test]
    fn test_genesis_duplicate_pubkey() {
        let mut genesis = generate_devnet_genesis(4);
        genesis.validators[1].pubkey = genesis.validators[0].pubkey.clone();

        assert!(matches!(
            genesis.validate(),
            Err(ConsensusError::InvalidGenesis(msg)) if msg.contains("duplicate validator pubkey")
        ));
    }
}
