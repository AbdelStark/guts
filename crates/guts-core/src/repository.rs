//! Repository types and metadata.

use crate::{RepositoryId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Visibility level for a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Repository is visible to everyone.
    #[default]
    Public,
    /// Repository is only visible to the owner and collaborators.
    Private,
    /// Repository is visible to organization members.
    Internal,
}

/// Metadata about a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Short description of the repository.
    pub description: Option<String>,
    /// Default branch name.
    pub default_branch: String,
    /// Repository visibility.
    pub visibility: Visibility,
    /// Custom key-value metadata.
    pub custom: HashMap<String, String>,
}

impl Default for RepositoryMetadata {
    fn default() -> Self {
        Self {
            description: None,
            default_branch: String::from("main"),
            visibility: Visibility::Public,
            custom: HashMap::new(),
        }
    }
}

/// A Guts repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Unique identifier for the repository.
    pub id: RepositoryId,
    /// Human-readable name.
    pub name: String,
    /// Owner's identity (public key hash).
    pub owner: String,
    /// When the repository was created.
    pub created_at: Timestamp,
    /// When the repository was last updated.
    pub updated_at: Timestamp,
    /// Repository metadata.
    pub metadata: RepositoryMetadata,
}

impl Repository {
    /// Creates a new repository with the given name and owner.
    #[must_use]
    pub fn new(name: impl Into<String>, owner: impl Into<String>) -> Self {
        let now = Timestamp::now();
        Self {
            id: RepositoryId::generate(),
            name: name.into(),
            owner: owner.into(),
            created_at: now,
            updated_at: now,
            metadata: RepositoryMetadata::default(),
        }
    }

    /// Returns the repository's full path (owner/name).
    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// Updates the repository's timestamp.
    pub fn touch(&mut self) {
        self.updated_at = Timestamp::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn repository_new() {
        let repo = Repository::new("my-repo", "alice");
        assert_eq!(repo.name, "my-repo");
        assert_eq!(repo.owner, "alice");
        assert_eq!(repo.full_name(), "alice/my-repo");
    }

    #[test]
    fn repository_metadata_default() {
        let meta = RepositoryMetadata::default();
        assert_eq!(meta.default_branch, "main");
        assert_eq!(meta.visibility, Visibility::Public);
    }
}
