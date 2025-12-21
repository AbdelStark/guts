//! Team types for group-based repository access.

use crate::permission::Permission;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A team within an organization.
///
/// Teams allow grouping users for easier permission management.
/// A team can have access to multiple repositories with a specific permission level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Unique team ID.
    pub id: u64,
    /// Organization ID this team belongs to.
    pub org_id: u64,
    /// Team name (URL-safe, unique within org).
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Team members (public keys).
    pub members: HashSet<String>,
    /// Default permission level for team repositories.
    pub permission: Permission,
    /// Repository keys this team has access to.
    pub repos: HashSet<String>,
    /// When the team was created (Unix timestamp).
    pub created_at: u64,
    /// When the team was last updated (Unix timestamp).
    pub updated_at: u64,
    /// Who created this team (public key).
    pub created_by: String,
}

impl Team {
    /// Create a new team.
    pub fn new(
        id: u64,
        org_id: u64,
        name: String,
        permission: Permission,
        created_by: String,
    ) -> Self {
        let now = Self::now();
        Self {
            id,
            org_id,
            name,
            description: None,
            members: HashSet::new(),
            permission,
            repos: HashSet::new(),
            created_at: now,
            updated_at: now,
            created_by,
        }
    }

    /// Set the team description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self.updated_at = Self::now();
        self
    }

    /// Check if a user is a member of this team.
    pub fn is_member(&self, user: &str) -> bool {
        self.members.contains(user)
    }

    /// Add a member to the team.
    pub fn add_member(&mut self, user: String) -> bool {
        let added = self.members.insert(user);
        if added {
            self.updated_at = Self::now();
        }
        added
    }

    /// Remove a member from the team.
    pub fn remove_member(&mut self, user: &str) -> bool {
        let removed = self.members.remove(user);
        if removed {
            self.updated_at = Self::now();
        }
        removed
    }

    /// Check if the team has access to a repository.
    pub fn has_repo(&self, repo_key: &str) -> bool {
        self.repos.contains(repo_key)
    }

    /// Add a repository to the team.
    pub fn add_repo(&mut self, repo_key: String) -> bool {
        let added = self.repos.insert(repo_key);
        if added {
            self.updated_at = Self::now();
        }
        added
    }

    /// Remove a repository from the team.
    pub fn remove_repo(&mut self, repo_key: &str) -> bool {
        let removed = self.repos.remove(repo_key);
        if removed {
            self.updated_at = Self::now();
        }
        removed
    }

    /// Get the permission level for a specific repository.
    /// Returns the team's permission if the repo is in the team, None otherwise.
    pub fn get_repo_permission(&self, repo_key: &str) -> Option<Permission> {
        if self.repos.contains(repo_key) {
            Some(self.permission)
        } else {
            None
        }
    }

    /// Update the team's default permission level.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_creation() {
        let team = Team::new(1, 1, "backend".into(), Permission::Write, "creator".into());

        assert_eq!(team.id, 1);
        assert_eq!(team.org_id, 1);
        assert_eq!(team.name, "backend");
        assert_eq!(team.permission, Permission::Write);
        assert!(team.members.is_empty());
        assert!(team.repos.is_empty());
    }

    #[test]
    fn test_team_members() {
        let mut team = Team::new(1, 1, "backend".into(), Permission::Write, "creator".into());

        assert!(team.add_member("user1".into()));
        assert!(team.is_member("user1"));
        assert!(!team.add_member("user1".into())); // Already exists

        assert!(team.add_member("user2".into()));
        assert_eq!(team.members.len(), 2);

        assert!(team.remove_member("user1"));
        assert!(!team.is_member("user1"));
        assert!(!team.remove_member("user1")); // Already removed
    }

    #[test]
    fn test_team_repos() {
        let mut team = Team::new(1, 1, "backend".into(), Permission::Write, "creator".into());

        assert!(team.add_repo("acme/api".into()));
        assert!(team.has_repo("acme/api"));
        assert_eq!(
            team.get_repo_permission("acme/api"),
            Some(Permission::Write)
        );
        assert_eq!(team.get_repo_permission("acme/other"), None);

        assert!(team.remove_repo("acme/api"));
        assert!(!team.has_repo("acme/api"));
    }

    #[test]
    fn test_team_permission_update() {
        let mut team = Team::new(1, 1, "backend".into(), Permission::Read, "creator".into());
        team.add_repo("acme/api".into());

        assert_eq!(team.get_repo_permission("acme/api"), Some(Permission::Read));

        team.set_permission(Permission::Admin);
        assert_eq!(
            team.get_repo_permission("acme/api"),
            Some(Permission::Admin)
        );
    }
}
