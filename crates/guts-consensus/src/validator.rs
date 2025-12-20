//! Validator management.

use guts_identity::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A validator in the consensus network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// The validator's public key.
    pub public_key: PublicKey,
    /// Voting power (stake weight).
    pub voting_power: u64,
    /// Whether the validator is active.
    pub active: bool,
}

impl Validator {
    /// Creates a new validator.
    #[must_use]
    pub fn new(public_key: PublicKey, voting_power: u64) -> Self {
        Self {
            public_key,
            voting_power,
            active: true,
        }
    }
}

/// A set of validators.
#[derive(Debug, Clone, Default)]
pub struct ValidatorSet {
    validators: Vec<Validator>,
}

impl ValidatorSet {
    /// Creates an empty validator set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a validator to the set.
    pub fn add(&mut self, validator: Validator) {
        self.validators.push(validator);
    }

    /// Returns the number of validators.
    #[must_use]
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Returns true if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Returns the total voting power.
    #[must_use]
    pub fn total_power(&self) -> u64 {
        self.validators
            .iter()
            .filter(|v| v.active)
            .map(|v| v.voting_power)
            .sum()
    }

    /// Returns the quorum threshold (2/3 + 1 of total power).
    #[must_use]
    pub fn quorum_threshold(&self) -> u64 {
        let total = self.total_power();
        (total * 2 / 3) + 1
    }

    /// Checks if the given public key is a validator.
    #[must_use]
    pub fn contains(&self, public_key: &PublicKey) -> bool {
        self.validators.iter().any(|v| &v.public_key == public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guts_identity::Keypair;

    #[test]
    fn validator_set_quorum() {
        let mut set = ValidatorSet::new();

        for _ in 0..4 {
            let kp = Keypair::generate();
            set.add(Validator::new(kp.public_key(), 100));
        }

        assert_eq!(set.total_power(), 400);
        assert_eq!(set.quorum_threshold(), 267); // 2/3 * 400 + 1
    }
}
