//! Validator set management.
//!
//! Validators are the nodes that participate in consensus. They propose blocks,
//! vote on proposals, and earn the right to finalize blocks.

use crate::error::{ConsensusError, Result};
use crate::transaction::SerializablePublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// A validator in the consensus network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Validator {
    /// Validator's public key (identity).
    pub pubkey: SerializablePublicKey,

    /// Human-readable name.
    pub name: String,

    /// Voting weight.
    pub weight: u64,

    /// Network address for P2P communication.
    pub addr: SocketAddr,

    /// Epoch when the validator joined.
    pub joined_epoch: u64,

    /// Whether the validator is active (participating in consensus).
    pub active: bool,
}

impl Validator {
    /// Creates a new validator.
    pub fn new(
        pubkey: SerializablePublicKey,
        name: impl Into<String>,
        weight: u64,
        addr: SocketAddr,
    ) -> Self {
        Self {
            pubkey,
            name: name.into(),
            weight,
            addr,
            joined_epoch: 0,
            active: true,
        }
    }

    /// Sets the epoch when the validator joined.
    pub fn with_joined_epoch(mut self, epoch: u64) -> Self {
        self.joined_epoch = epoch;
        self
    }

    /// Sets the active status.
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

/// Configuration for validator set behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Minimum number of validators for network operation.
    pub min_validators: usize,

    /// Maximum number of validators.
    pub max_validators: usize,

    /// Quorum threshold (fraction of weight required for consensus).
    /// For BFT, this is typically 2/3.
    pub quorum_threshold: f64,

    /// Target block time in milliseconds.
    pub block_time_ms: u64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            min_validators: 4,
            max_validators: 100,
            quorum_threshold: 2.0 / 3.0,
            block_time_ms: 2000,
        }
    }
}

/// The validator set for a given epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    /// Current validators.
    validators: Vec<Validator>,

    /// Epoch number (changes when validator set changes).
    epoch: u64,

    /// Configuration.
    config: ValidatorConfig,

    /// Index for fast lookup by public key (hex string).
    #[serde(skip)]
    index: HashMap<String, usize>,
}

impl ValidatorSet {
    /// Creates a new validator set.
    pub fn new(validators: Vec<Validator>, epoch: u64, config: ValidatorConfig) -> Result<Self> {
        if validators.len() < config.min_validators {
            return Err(ConsensusError::InvalidGenesis(format!(
                "need at least {} validators, got {}",
                config.min_validators,
                validators.len()
            )));
        }

        if validators.len() > config.max_validators {
            return Err(ConsensusError::InvalidGenesis(format!(
                "too many validators: {} > {}",
                validators.len(),
                config.max_validators
            )));
        }

        let mut set = Self {
            validators,
            epoch,
            config,
            index: HashMap::new(),
        };

        set.rebuild_index();
        Ok(set)
    }

    /// Creates a genesis validator set.
    pub fn genesis(validators: Vec<Validator>) -> Result<Self> {
        Self::new(validators, 0, ValidatorConfig::default())
    }

    /// Rebuilds the lookup index.
    fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, v) in self.validators.iter().enumerate() {
            self.index.insert(v.pubkey.0.clone(), i);
        }
    }

    /// Returns the current epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Returns all validators.
    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    /// Returns all active validators.
    pub fn active_validators(&self) -> Vec<&Validator> {
        self.validators.iter().filter(|v| v.active).collect()
    }

    /// Returns the number of validators.
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Returns true if there are no validators.
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Returns the number of active validators.
    pub fn active_count(&self) -> usize {
        self.validators.iter().filter(|v| v.active).count()
    }

    /// Gets a validator by public key.
    pub fn get(&self, pubkey: &SerializablePublicKey) -> Option<&Validator> {
        self.index
            .get(&pubkey.0)
            .and_then(|&i| self.validators.get(i))
    }

    /// Checks if a public key belongs to a validator.
    pub fn is_validator(&self, pubkey: &SerializablePublicKey) -> bool {
        self.index.contains_key(&pubkey.0)
    }

    /// Checks if a public key belongs to an active validator.
    pub fn is_active_validator(&self, pubkey: &SerializablePublicKey) -> bool {
        self.get(pubkey).map(|v| v.active).unwrap_or(false)
    }

    /// Returns the total voting weight.
    pub fn total_weight(&self) -> u64 {
        self.validators.iter().map(|v| v.weight).sum()
    }

    /// Returns the total active voting weight.
    pub fn active_weight(&self) -> u64 {
        self.validators
            .iter()
            .filter(|v| v.active)
            .map(|v| v.weight)
            .sum()
    }

    /// Returns the quorum weight required for consensus.
    pub fn quorum_weight(&self) -> u64 {
        let total = self.active_weight();
        ((total as f64) * self.config.quorum_threshold).ceil() as u64
    }

    /// Returns the maximum Byzantine weight that can be tolerated.
    /// For f < n/3, this is floor((n-1)/3).
    pub fn max_byzantine_weight(&self) -> u64 {
        let total = self.active_weight();
        total / 3
    }

    /// Gets the leader for a given view using round-robin selection.
    pub fn leader_for_view(&self, view: u64) -> Option<&Validator> {
        let active: Vec<_> = self.validators.iter().filter(|v| v.active).collect();
        if active.is_empty() {
            return None;
        }
        let idx = (view as usize) % active.len();
        Some(active[idx])
    }

    /// Gets the leader index for a given view.
    pub fn leader_index_for_view(&self, view: u64) -> Option<usize> {
        let active_count = self.active_count();
        if active_count == 0 {
            return None;
        }
        Some((view as usize) % active_count)
    }

    /// Returns the target block time.
    pub fn block_time_ms(&self) -> u64 {
        self.config.block_time_ms
    }

    /// Returns the configuration.
    pub fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    /// Checks if a set of signers meets quorum.
    pub fn has_quorum(&self, signers: &[SerializablePublicKey]) -> bool {
        let signed_weight: u64 = signers
            .iter()
            .filter_map(|pk| self.get(pk))
            .filter(|v| v.active)
            .map(|v| v.weight)
            .sum();

        signed_weight >= self.quorum_weight()
    }

    /// Gets the network addresses of all active validators.
    pub fn active_addresses(&self) -> Vec<SocketAddr> {
        self.validators
            .iter()
            .filter(|v| v.active)
            .map(|v| v.addr)
            .collect()
    }

    /// Gets the public keys of all active validators.
    pub fn active_pubkeys(&self) -> Vec<SerializablePublicKey> {
        self.validators
            .iter()
            .filter(|v| v.active)
            .map(|v| v.pubkey.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::{ed25519, PrivateKeyExt, Signer};

    fn test_validators(count: usize) -> Vec<Validator> {
        (0..count as u64)
            .map(|i| {
                let key = ed25519::PrivateKey::from_seed(i);
                Validator::new(
                    SerializablePublicKey::from_pubkey(&key.public_key()),
                    format!("validator-{}", i),
                    100,
                    format!("127.0.0.1:{}", 9000 + i).parse().unwrap(),
                )
            })
            .collect()
    }

    #[test]
    fn test_validator_set_genesis() {
        let validators = test_validators(4);
        let set = ValidatorSet::genesis(validators).unwrap();

        assert_eq!(set.epoch(), 0);
        assert_eq!(set.len(), 4);
        assert_eq!(set.active_count(), 4);
    }

    #[test]
    fn test_validator_set_too_few() {
        let validators = test_validators(2);
        let result = ValidatorSet::genesis(validators);

        assert!(matches!(result, Err(ConsensusError::InvalidGenesis(_))));
    }

    #[test]
    fn test_validator_lookup() {
        let validators = test_validators(4);
        let pubkey = validators[2].pubkey.clone();
        let set = ValidatorSet::genesis(validators).unwrap();

        assert!(set.is_validator(&pubkey));
        assert!(set.is_active_validator(&pubkey));

        let v = set.get(&pubkey).unwrap();
        assert_eq!(v.name, "validator-2");
    }

    #[test]
    fn test_quorum_weight() {
        let validators = test_validators(4); // 4 * 100 = 400 total weight
        let set = ValidatorSet::genesis(validators).unwrap();

        // 2/3 of 400 = 266.67, ceil = 267
        assert_eq!(set.quorum_weight(), 267);
    }

    #[test]
    fn test_leader_rotation() {
        let validators = test_validators(4);
        let set = ValidatorSet::genesis(validators).unwrap();

        let leader0 = set.leader_for_view(0).unwrap();
        let leader1 = set.leader_for_view(1).unwrap();
        let leader4 = set.leader_for_view(4).unwrap();

        // View 4 should wrap around to validator 0
        assert_eq!(leader0.pubkey, leader4.pubkey);
        assert_ne!(leader0.pubkey, leader1.pubkey);
    }

    #[test]
    fn test_has_quorum() {
        let validators = test_validators(4);
        let pubkeys: Vec<_> = validators.iter().map(|v| v.pubkey.clone()).collect();
        let set = ValidatorSet::genesis(validators).unwrap();

        // 3 of 4 validators (300/400 = 75% > 66.7%)
        assert!(set.has_quorum(&pubkeys[0..3]));

        // 2 of 4 validators (200/400 = 50% < 66.7%)
        assert!(!set.has_quorum(&pubkeys[0..2]));
    }

    #[test]
    fn test_active_addresses() {
        let validators = test_validators(4);
        let set = ValidatorSet::genesis(validators).unwrap();

        let addrs = set.active_addresses();
        assert_eq!(addrs.len(), 4);
    }
}
