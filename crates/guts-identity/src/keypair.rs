//! Ed25519 keypair for signing and verification.

use crate::{IdentityError, PublicKey, Result, Signature};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use zeroize::Zeroizing;

/// An Ed25519 keypair for signing and verification.
pub struct Keypair {
    signing_key: SigningKey,
}

impl Keypair {
    /// Generates a new random keypair.
    #[must_use]
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Creates a keypair from a secret key (32 bytes).
    ///
    /// # Errors
    ///
    /// Returns an error if the secret key is invalid.
    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(IdentityError::InvalidSecretKey);
        }

        let mut secret = [0u8; 32];
        secret.copy_from_slice(bytes);
        let secret = Zeroizing::new(secret);

        let signing_key = SigningKey::from_bytes(&secret);
        Ok(Self { signing_key })
    }

    /// Returns the public key for this keypair.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_verifying_key(self.signing_key.verifying_key())
    }

    /// Signs a message with this keypair.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        let sig = self.signing_key.sign(message);
        Signature::from_bytes(sig.to_bytes())
    }

    /// Verifies a signature against a message.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature is invalid.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        self.public_key().verify(message, signature)
    }

    /// Returns the secret key bytes.
    ///
    /// # Security
    ///
    /// Handle with care. The returned bytes should be zeroized after use.
    #[must_use]
    pub fn secret_bytes(&self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(self.signing_key.to_bytes())
    }
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keypair")
            .field("public_key", &self.public_key())
            .finish_non_exhaustive()
    }
}

impl Clone for Keypair {
    fn clone(&self) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&self.signing_key.to_bytes()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn keypair_generate() {
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();
        assert_ne!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn keypair_sign_verify() {
        let kp = Keypair::generate();
        let message = b"Hello, Guts!";

        let signature = kp.sign(message);
        assert!(kp.verify(message, &signature).is_ok());
    }

    #[test]
    fn keypair_wrong_message() {
        let kp = Keypair::generate();
        let signature = kp.sign(b"message 1");
        assert!(kp.verify(b"message 2", &signature).is_err());
    }

    #[test]
    fn keypair_from_secret_bytes() {
        let kp1 = Keypair::generate();
        let secret = kp1.secret_bytes();

        let kp2 = Keypair::from_secret_bytes(&*secret).unwrap();
        assert_eq!(kp1.public_key(), kp2.public_key());
    }
}
