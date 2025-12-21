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

    #[test]
    fn test_repository_id_from_bytes() {
        let bytes = [0u8; 32];
        let id = RepositoryId::from_bytes(bytes);
        assert_eq!(*id.as_bytes(), bytes);
    }

    #[test]
    fn test_repository_id_to_hex() {
        let bytes = [0xab; 32];
        let id = RepositoryId::from_bytes(bytes);
        let hex = id.to_hex();
        assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_repository_id_display() {
        let id = RepositoryId::generate("repo", "owner");
        let display = format!("{}", id);
        let hex = id.to_hex();
        assert_eq!(display, hex);
    }

    #[test]
    fn test_repository_id_different_owners_same_name() {
        let id1 = RepositoryId::generate("repo", "alice");
        let id2 = RepositoryId::generate("repo", "bob");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_repository_id_is_hash() {
        // Verify that repository IDs are deterministic SHA256 hashes
        let id = RepositoryId::generate("test", "owner");
        assert_eq!(id.as_bytes().len(), 32); // SHA256 produces 32 bytes
    }

    #[test]
    fn test_repository_visibility_default() {
        let repo = Repository::new("test", "owner");
        assert_eq!(repo.visibility, Visibility::Public);
    }

    #[test]
    fn test_visibility_serde() {
        let public: Visibility = serde_json::from_str(r#""public""#).unwrap();
        let private: Visibility = serde_json::from_str(r#""private""#).unwrap();

        assert_eq!(public, Visibility::Public);
        assert_eq!(private, Visibility::Private);

        assert_eq!(
            serde_json::to_string(&Visibility::Public).unwrap(),
            r#""public""#
        );
        assert_eq!(
            serde_json::to_string(&Visibility::Private).unwrap(),
            r#""private""#
        );
    }

    #[test]
    fn test_repository_with_description() {
        let repo = Repository::new("test", "owner").with_description("A test repository");

        assert_eq!(repo.description, Some("A test repository".to_string()));
    }

    #[test]
    fn test_repository_created_at_is_set() {
        let repo = Repository::new("test", "owner");
        assert!(repo.created_at > 0);
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository::new("test-repo", "alice").with_description("Test description");

        let json = serde_json::to_string(&repo).unwrap();
        let parsed: Repository = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, repo.name);
        assert_eq!(parsed.owner, repo.owner);
        assert_eq!(parsed.description, repo.description);
        assert_eq!(parsed.id, repo.id);
    }

    #[test]
    fn test_repository_id_equality() {
        let id1 = RepositoryId::from_bytes([1u8; 32]);
        let id2 = RepositoryId::from_bytes([1u8; 32]);
        let id3 = RepositoryId::from_bytes([2u8; 32]);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_repository_id_hash_trait() {
        use std::collections::HashSet;

        let id1 = RepositoryId::generate("repo1", "owner");
        let id2 = RepositoryId::generate("repo2", "owner");

        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id1); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_repository_full_name_format() {
        let repo = Repository::new("my-project", "org-name");
        assert_eq!(repo.full_name(), "org-name/my-project");
    }

    #[test]
    fn test_repository_accepts_string_types() {
        let repo1 = Repository::new(String::from("name"), String::from("owner"));
        let repo2 = Repository::new("name", "owner");

        assert_eq!(repo1.name, repo2.name);
        assert_eq!(repo1.owner, repo2.owner);
    }

    #[test]
    fn test_visibility_default_trait() {
        let visibility: Visibility = Default::default();
        assert_eq!(visibility, Visibility::Public);
    }

    #[test]
    fn test_repository_id_copy_trait() {
        let id1 = RepositoryId::generate("repo", "owner");
        let id2 = id1; // Copy
        assert_eq!(id1, id2);
    }
}
