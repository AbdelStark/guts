//! Content hashing using BLAKE3.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A BLAKE3 content hash.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// The length of a content hash in bytes.
    pub const LEN: usize = 32;

    /// Computes the content hash of the given data.
    #[must_use]
    pub fn compute(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(*hash.as_bytes())
    }

    /// Creates a content hash from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes of this hash.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the hash as a hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Creates a content hash from a hex string.
    ///
    /// # Errors
    ///
    /// Returns an error if the hex string is invalid.
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != Self::LEN {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", &self.to_hex()[..16])
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn content_hash_deterministic() {
        let data = b"Hello, world!";
        let hash1 = ContentHash::compute(data);
        let hash2 = ContentHash::compute(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn content_hash_different_data() {
        let hash1 = ContentHash::compute(b"Hello");
        let hash2 = ContentHash::compute(b"World");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn content_hash_hex_roundtrip() {
        let hash = ContentHash::compute(b"test");
        let hex = hash.to_hex();
        let hash2 = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash, hash2);
    }
}
