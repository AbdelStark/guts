//! Ed25519 public key for verification.

use crate::{IdentityError, Result, Signature};
use ed25519_dalek::{Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fmt;

/// An Ed25519 public key for signature verification.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey {
    key: VerifyingKey,
}

impl PublicKey {
    /// The length of a public key in bytes.
    pub const LEN: usize = 32;

    /// Creates a public key from a verifying key.
    pub(crate) fn from_verifying_key(key: VerifyingKey) -> Self {
        Self { key }
    }

    /// Creates a public key from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes do not represent a valid public key.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::LEN {
            return Err(IdentityError::InvalidPublicKey(format!(
                "expected {} bytes, got {}",
                Self::LEN,
                bytes.len()
            )));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);

        let key = VerifyingKey::from_bytes(&arr)
            .map_err(|e| IdentityError::InvalidPublicKey(e.to_string()))?;

        Ok(Self { key })
    }

    /// Returns the raw bytes of this public key.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.key.as_bytes()
    }

    /// Returns a short identifier (first 8 bytes as hex).
    #[must_use]
    pub fn short_id(&self) -> String {
        hex::encode(&self.as_bytes()[..8])
    }

    /// Verifies a signature against a message.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature is invalid.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        let sig = ed25519_dalek::Signature::from_bytes(signature.as_bytes());
        self.key
            .verify(message, &sig)
            .map_err(|_| IdentityError::InvalidSignature)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", self.short_id())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_bytes()))
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(self.as_bytes()))
        } else {
            serializer.serialize_bytes(self.as_bytes())
        }
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
            PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
        } else {
            let bytes = <[u8; 32]>::deserialize(deserializer)?;
            PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keypair;

    #[test]
    fn public_key_short_id() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let short = pk.short_id();
        assert_eq!(short.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn public_key_roundtrip() {
        let kp = Keypair::generate();
        let pk1 = kp.public_key();

        let bytes = pk1.as_bytes();
        let pk2 = PublicKey::from_bytes(bytes).unwrap();

        assert_eq!(pk1, pk2);
    }

    #[test]
    fn public_key_serde_json() {
        let kp = Keypair::generate();
        let pk = kp.public_key();

        let json = serde_json::to_string(&pk).unwrap();
        let pk2: PublicKey = serde_json::from_str(&json).unwrap();

        assert_eq!(pk, pk2);
    }
}
