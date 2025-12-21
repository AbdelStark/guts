//! Repository collaborator types.

use crate::permission::Permission;
use serde::{Deserialize, Serialize};

/// A collaborator on a repository.
///
/// Collaborators are users who have been explicitly granted access to a repository,
/// separate from organization membership or team access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    /// Unique collaborator ID.
    pub id: u64,
    /// Repository key (e.g., "owner/repo").
    pub repo_key: String,
    /// User's public key (hex-encoded).
    pub user: String,
    /// Permission level granted.
    pub permission: Permission,
    /// Who added this collaborator (public key).
    pub added_by: String,
    /// When the collaborator was added (Unix timestamp).
    pub created_at: u64,
    /// When the permission was last updated (Unix timestamp).
    pub updated_at: u64,
}

impl Collaborator {
    /// Create a new collaborator.
    pub fn new(
        id: u64,
        repo_key: String,
        user: String,
        permission: Permission,
        added_by: String,
    ) -> Self {
        let now = Self::now();
        Self {
            id,
            repo_key,
            user,
            permission,
            added_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this collaborator has at least the required permission.
    pub fn has_permission(&self, required: Permission) -> bool {
        self.permission.has(required)
    }

    /// Update the permission level.
    pub fn set_permission(&mut self, permission: Permission) {
        self.permission = permission;
        self.updated_at = Self::now();
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Request to add or update a collaborator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorRequest {
    /// User's public key (hex-encoded).
    pub user: String,
    /// Permission level to grant.
    pub permission: Permission,
}

/// Response with collaborator information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorResponse {
    /// User's public key (hex-encoded).
    pub user: String,
    /// Permission level.
    pub permission: Permission,
    /// When the collaborator was added.
    pub created_at: u64,
}

impl From<&Collaborator> for CollaboratorResponse {
    fn from(c: &Collaborator) -> Self {
        Self {
            user: c.user.clone(),
            permission: c.permission,
            created_at: c.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaborator_creation() {
        let collab = Collaborator::new(
            1,
            "acme/api".into(),
            "user123".into(),
            Permission::Write,
            "owner".into(),
        );

        assert_eq!(collab.id, 1);
        assert_eq!(collab.repo_key, "acme/api");
        assert_eq!(collab.user, "user123");
        assert_eq!(collab.permission, Permission::Write);
        assert!(collab.has_permission(Permission::Read));
        assert!(collab.has_permission(Permission::Write));
        assert!(!collab.has_permission(Permission::Admin));
    }

    #[test]
    fn test_collaborator_permission_update() {
        let mut collab = Collaborator::new(
            1,
            "acme/api".into(),
            "user123".into(),
            Permission::Read,
            "owner".into(),
        );

        assert!(!collab.has_permission(Permission::Write));

        collab.set_permission(Permission::Admin);
        assert!(collab.has_permission(Permission::Admin));
        assert!(collab.updated_at >= collab.created_at);
    }
}
