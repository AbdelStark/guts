//! Core traits for Guts components.

use crate::Result;
use async_trait::async_trait;

/// A trait for types that can be serialized to bytes.
pub trait ToBytes {
    /// Serializes the value to bytes.
    fn to_bytes(&self) -> Vec<u8>;
}

/// A trait for types that can be deserialized from bytes.
pub trait FromBytes: Sized {
    /// Deserializes a value from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes cannot be deserialized.
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

/// A trait for types that have a content-based identifier.
pub trait ContentAddressed {
    /// Returns the content hash of this value.
    fn content_id(&self) -> crate::ObjectId;
}

/// A trait for verifiable signatures.
#[async_trait]
pub trait Verifiable {
    /// The public key type used for verification.
    type PublicKey;

    /// Verifies this item's signature.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature is invalid.
    async fn verify(&self, public_key: &Self::PublicKey) -> Result<bool>;
}

/// A trait for types that can be signed.
#[async_trait]
pub trait Signable {
    /// The signature type produced.
    type Signature;

    /// Returns the bytes to be signed.
    fn signable_bytes(&self) -> Vec<u8>;
}
