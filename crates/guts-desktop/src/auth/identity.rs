//! # Identity Management
//!
//! Ed25519 keypair generation and storage for user authentication.
//!
//! This module follows the same pattern as `guts-cli` for identity handling.

use commonware_cryptography::{ed25519::PrivateKey, PrivateKeyExt, Signer};
use rand::rngs::OsRng;

/// User identity backed by an Ed25519 keypair.
///
/// The private key is used to prove ownership during registration,
/// while the public key serves as the user's unique identifier.
#[derive(Clone)]
pub struct Identity {
    private_key: PrivateKey,
}

impl Identity {
    /// Generate a new random Ed25519 identity.
    ///
    /// This creates a cryptographically secure keypair using the OS random
    /// number generator, following the same pattern as `guts-cli`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let identity = Identity::generate();
    /// let public_key = identity.public_key_hex();
    /// println!("Your public key: {}", public_key);
    /// ```
    #[must_use]
    pub fn generate() -> Self {
        let private_key = PrivateKey::from_rng(&mut OsRng);
        Self { private_key }
    }

    /// Export the private key as a hex string for storage.
    ///
    /// **Security**: This value should be stored securely and never logged.
    #[must_use]
    #[allow(dead_code)]
    pub fn to_hex(&self) -> String {
        commonware_utils::hex(self.private_key.as_ref())
    }

    /// Get the public key as a hex string.
    ///
    /// This is used for user registration with the API.
    /// The API stores this hex string to identify the user.
    #[must_use]
    pub fn public_key_hex(&self) -> String {
        commonware_utils::hex(self.private_key.public_key().as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let identity = Identity::generate();
        let public_key = identity.public_key_hex();

        // Ed25519 public key is 32 bytes = 64 hex chars
        assert_eq!(public_key.len(), 64);
    }

    #[test]
    fn test_private_key_hex() {
        let identity = Identity::generate();
        let private_key_hex = identity.to_hex();

        // Ed25519 private key (seed) is 32 bytes = 64 hex chars
        assert_eq!(private_key_hex.len(), 64);
    }

    #[test]
    fn test_unique_identities() {
        let id1 = Identity::generate();
        let id2 = Identity::generate();

        // Two generated identities should be different
        assert_ne!(id1.public_key_hex(), id2.public_key_hex());
        assert_ne!(id1.to_hex(), id2.to_hex());
    }

    #[test]
    fn test_consistent_public_key() {
        let identity = Identity::generate();

        // Public key should be consistent across calls
        let pk1 = identity.public_key_hex();
        let pk2 = identity.public_key_hex();
        assert_eq!(pk1, pk2);
    }
}
