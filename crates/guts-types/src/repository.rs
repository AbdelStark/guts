//! Repository types for Guts.

use commonware_cryptography::{Hasher, Sha256};
use serde::{Deserialize, Serialize};

/// A unique identifier for a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryId([u8; 32]);

impl RepositoryId {
    /// Creates a new repository ID from raw bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Generates a repository ID from the repository name and owner.
    pub fn generate(name: &str, owner: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(b":");
        hasher.update(owner.as_bytes());
        let digest = hasher.finalize();
        let bytes: [u8; 32] = digest
            .as_ref()
            .try_into()
            .expect("SHA256 produces 32 bytes");
        Self(bytes)
    }

    /// Returns the ID as a hex string.
    pub fn to_hex(&self) -> String {
        commonware_utils::hex(&self.0)
    }
}

impl std::fmt::Display for RepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Visibility of a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Public repository.
    #[default]
    Public,
    /// Private repository.
    Private,
}

/// A repository in the Guts network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Unique identifier.
    pub id: RepositoryId,
    /// Human-readable name.
    pub name: String,
    /// Owner's public key (hex encoded).
    pub owner: String,
    /// Optional description.
    pub description: Option<String>,
    /// Default branch name.
    pub default_branch: String,
    /// Repository visibility.
    pub visibility: Visibility,
    /// Creation timestamp (unix millis).
    pub created_at: u64,
}

impl Repository {
    /// Creates a new repository.
    pub fn new(name: impl Into<String>, owner: impl Into<String>) -> Self {
        let name = name.into();
        let owner = owner.into();
        let id = RepositoryId::generate(&name, &owner);

        Self {
            id,
            name,
            owner,
            description: None,
            default_branch: "main".to_string(),
            visibility: Visibility::Public,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Returns the full name (owner/name).
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_id_generation() {
        let id1 = RepositoryId::generate("my-repo", "alice");
        let id2 = RepositoryId::generate("my-repo", "alice");
        let id3 = RepositoryId::generate("other-repo", "alice");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_repository_creation() {
        let repo = Repository::new("test-repo", "user123");
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.owner, "user123");
        assert_eq!(repo.default_branch, "main");
        assert_eq!(repo.full_name(), "user123/test-repo");
    }
}
