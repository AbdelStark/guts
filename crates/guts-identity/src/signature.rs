//! Ed25519 signature type.

use serde::{Deserialize, Serialize};
use std::fmt;

/// An Ed25519 signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    /// The length of a signature in bytes.
    pub const LEN: usize = 64;

    /// Creates a signature from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes of this signature.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Creates a signature from a byte slice.
    ///
    /// # Panics
    ///
    /// Panics if the slice is not exactly 64 bytes.
    #[must_use]
    pub fn from_slice(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 64];
        arr.copy_from_slice(bytes);
        Self(arr)
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({}...)", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
            if bytes.len() != Self::LEN {
                return Err(serde::de::Error::custom("invalid signature length"));
            }
            Ok(Self::from_slice(&bytes))
        } else {
            let bytes = <[u8; 64]>::deserialize(deserializer)?;
            Ok(Self(bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_roundtrip() {
        let bytes = [42u8; 64];
        let sig = Signature::from_bytes(bytes);
        assert_eq!(sig.as_bytes(), &bytes);
    }

    #[test]
    fn signature_serde_json() {
        let sig = Signature::from_bytes([1u8; 64]);
        let json = serde_json::to_string(&sig).unwrap();
        let sig2: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(sig, sig2);
    }
}
