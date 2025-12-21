//! Git object types and utilities.

use crate::{Result, StorageError};
use bytes::Bytes;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha1::{Digest, Sha1};
use std::fmt;

/// A 20-byte SHA-1 object identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId([u8; 20]);

impl Serialize for ObjectId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ObjectId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ObjectId::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl ObjectId {
    /// Creates an ObjectId from raw bytes.
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Creates an ObjectId from a hex string.
    pub fn from_hex(hex: &str) -> Result<Self> {
        if hex.len() != 40 {
            return Err(StorageError::InvalidObject(format!(
                "invalid object id length: {}",
                hex.len()
            )));
        }
        let mut bytes = [0u8; 20];
        hex::decode_to_slice(hex, &mut bytes)
            .map_err(|e| StorageError::InvalidObject(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// Returns the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Returns the hex representation.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Computes the SHA-1 hash of data with a git object header.
    pub fn hash_object(object_type: ObjectType, data: &[u8]) -> Self {
        let header = format!("{} {}\0", object_type.as_str(), data.len());
        let mut hasher = Sha1::new();
        hasher.update(header.as_bytes());
        hasher.update(data);
        let result = hasher.finalize();
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&result);
        Self(bytes)
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectId({})", self.to_hex())
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Git object types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    /// File content.
    Blob,
    /// Directory listing.
    Tree,
    /// Commit object.
    Commit,
    /// Annotated tag.
    Tag,
}

impl ObjectType {
    /// Returns the string representation used in git.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
            Self::Tag => "tag",
        }
    }

    /// Parses an object type from a string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            "tag" => Ok(Self::Tag),
            _ => Err(StorageError::InvalidObject(format!(
                "unknown object type: {}",
                s
            ))),
        }
    }

    /// Returns the type code used in pack files.
    pub fn pack_type(&self) -> u8 {
        match self {
            Self::Commit => 1,
            Self::Tree => 2,
            Self::Blob => 3,
            Self::Tag => 4,
        }
    }

    /// Parses an object type from a pack file type code.
    pub fn from_pack_type(code: u8) -> Result<Self> {
        match code {
            1 => Ok(Self::Commit),
            2 => Ok(Self::Tree),
            3 => Ok(Self::Blob),
            4 => Ok(Self::Tag),
            _ => Err(StorageError::InvalidObject(format!(
                "unknown pack type: {}",
                code
            ))),
        }
    }
}

/// A git object (blob, tree, commit, or tag).
#[derive(Debug, Clone)]
pub struct GitObject {
    /// The object's unique identifier (SHA-1 hash).
    pub id: ObjectId,
    /// The type of object.
    pub object_type: ObjectType,
    /// The raw object data (uncompressed).
    pub data: Bytes,
}

impl GitObject {
    /// Creates a new git object, computing its ID from the data.
    pub fn new(object_type: ObjectType, data: impl Into<Bytes>) -> Self {
        let data = data.into();
        let id = ObjectId::hash_object(object_type, &data);
        Self {
            id,
            object_type,
            data,
        }
    }

    /// Creates a blob object from file content.
    pub fn blob(content: impl Into<Bytes>) -> Self {
        Self::new(ObjectType::Blob, content)
    }

    /// Creates a commit object.
    pub fn commit(
        tree_id: &ObjectId,
        parents: &[ObjectId],
        author: &str,
        committer: &str,
        message: &str,
    ) -> Self {
        let mut content = format!("tree {}\n", tree_id);
        for parent in parents {
            content.push_str(&format!("parent {}\n", parent));
        }
        content.push_str(&format!("author {}\n", author));
        content.push_str(&format!("committer {}\n", committer));
        content.push_str(&format!("\n{}", message));
        Self::new(ObjectType::Commit, content.into_bytes())
    }

    /// Returns the size of the object data.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_hex_roundtrip() {
        let hex = "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3";
        let id = ObjectId::from_hex(hex).unwrap();
        assert_eq!(id.to_hex(), hex);
    }

    #[test]
    fn test_blob_hash() {
        // "hello\n" should hash to a well-known value
        let obj = GitObject::blob(b"hello\n".to_vec());
        // This is the actual git hash for "hello\n"
        assert_eq!(obj.id.to_hex(), "ce013625030ba8dba906f756967f9e9ca394464a");
    }

    #[test]
    fn test_object_type_roundtrip() {
        for ot in [
            ObjectType::Blob,
            ObjectType::Tree,
            ObjectType::Commit,
            ObjectType::Tag,
        ] {
            let s = ot.as_str();
            let parsed = ObjectType::parse(s).unwrap();
            assert_eq!(ot, parsed);
        }
    }
}
