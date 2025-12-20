//! Identifier types for Guts entities.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A 32-byte identifier used for content-addressed objects.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId([u8; 32]);

impl ObjectId {
    /// The length of an ObjectId in bytes.
    pub const LEN: usize = 32;

    /// Creates a new `ObjectId` from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes of this identifier.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Creates a null (all zeros) identifier.
    #[must_use]
    pub const fn null() -> Self {
        Self([0u8; 32])
    }

    /// Returns true if this is the null identifier.
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// Generates a random `ObjectId` for testing.
    #[cfg(any(test, feature = "test-utils"))]
    #[must_use]
    pub fn random() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(bytes)
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectId({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl TryFrom<&[u8]> for ObjectId {
    type Error = crate::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != Self::LEN {
            return Err(crate::Error::invalid_input(
                "bytes",
                format!("expected {} bytes, got {}", Self::LEN, bytes.len()),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }
}

/// A repository identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryId(ObjectId);

impl RepositoryId {
    /// Creates a new `RepositoryId` from an `ObjectId`.
    #[must_use]
    pub const fn new(id: ObjectId) -> Self {
        Self(id)
    }

    /// Returns the underlying `ObjectId`.
    #[must_use]
    pub const fn as_object_id(&self) -> &ObjectId {
        &self.0
    }

    /// Generates a new random repository ID.
    #[must_use]
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(ObjectId(bytes))
    }
}

impl fmt::Debug for RepositoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RepositoryId({})", self.0)
    }
}

impl fmt::Display for RepositoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A commit identifier (Git-compatible SHA-1 or SHA-256).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitId([u8; 32]);

impl CommitId {
    /// Creates a new `CommitId` from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes of this commit ID.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Creates a null (all zeros) commit ID.
    #[must_use]
    pub const fn null() -> Self {
        Self([0u8; 32])
    }

    /// Returns true if this is the null commit ID.
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl fmt::Debug for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommitId({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn object_id_null() {
        let id = ObjectId::null();
        assert!(id.is_null());
        assert_eq!(id.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn object_id_from_bytes() {
        let bytes = [1u8; 32];
        let id = ObjectId::from_bytes(bytes);
        assert!(!id.is_null());
        assert_eq!(id.as_bytes(), &bytes);
    }

    #[test]
    fn repository_id_generate() {
        let id1 = RepositoryId::generate();
        let id2 = RepositoryId::generate();
        assert_ne!(id1, id2);
    }
}
