//! Organization types for multi-user repository ownership.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Role within an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrgRole {
    /// Regular member with default access.
    Member,
    /// Can manage teams and repositories.
    Admin,
    /// Full control of the organization.
    Owner,
}

impl OrgRole {
    /// Check if this role has at least the required role level.
    pub fn has(&self, required: OrgRole) -> bool {
        *self >= required
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "member" => Some(OrgRole::Member),
            "admin" => Some(OrgRole::Admin),
            "owner" => Some(OrgRole::Owner),
            _ => None,
        }
    }
}

impl std::fmt::Display for OrgRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrgRole::Member => write!(f, "member"),
            OrgRole::Admin => write!(f, "admin"),
            OrgRole::Owner => write!(f, "owner"),
        }
    }
}

impl Default for OrgRole {
    fn default() -> Self {
        OrgRole::Member
    }
}

/// A member of an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMember {
    /// User's public key (hex-encoded).
    pub user: String,
    /// Role within the organization.
    pub role: OrgRole,
    /// When the member was added (Unix timestamp).
    pub added_at: u64,
    /// Who added this member (public key hex).
    pub added_by: String,
}

impl OrgMember {
    /// Create a new organization member.
    pub fn new(user: String, role: OrgRole, added_by: String) -> Self {
        Self {
            user,
            role,
            added_at: Self::now(),
            added_by,
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// An organization for multi-user repository ownership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Unique organization ID.
    pub id: u64,
    /// Unique organization name (URL-safe, e.g., "acme-corp").
    pub name: String,
    /// Display name (e.g., "Acme Corporation").
    pub display_name: String,
    /// Optional description.
    pub description: Option<String>,
    /// The creator's public key (hex).
    pub created_by: String,
    /// Organization members.
    pub members: Vec<OrgMember>,
    /// Team IDs belonging to this organization.
    pub teams: HashSet<u64>,
    /// Repository keys owned by this organization.
    pub repos: HashSet<String>,
    /// When the organization was created (Unix timestamp).
    pub created_at: u64,
    /// When the organization was last updated (Unix timestamp).
    pub updated_at: u64,
}

impl Organization {
    /// Create a new organization.
    pub fn new(id: u64, name: String, display_name: String, created_by: String) -> Self {
        let now = Self::now();
        let founder = OrgMember {
            user: created_by.clone(),
            role: OrgRole::Owner,
            added_at: now,
            added_by: created_by.clone(),
        };

        Self {
            id,
            name,
            display_name,
            description: None,
            created_by,
            members: vec![founder],
            teams: HashSet::new(),
            repos: HashSet::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the organization description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self.updated_at = Self::now();
        self
    }

    /// Get a member by their public key.
    pub fn get_member(&self, user: &str) -> Option<&OrgMember> {
        self.members.iter().find(|m| m.user == user)
    }

    /// Check if a user is a member.
    pub fn is_member(&self, user: &str) -> bool {
        self.get_member(user).is_some()
    }

    /// Check if a user has at least the specified role.
    pub fn has_role(&self, user: &str, required: OrgRole) -> bool {
        self.get_member(user)
            .map(|m| m.role.has(required))
            .unwrap_or(false)
    }

    /// Check if a user is an owner.
    pub fn is_owner(&self, user: &str) -> bool {
        self.has_role(user, OrgRole::Owner)
    }

    /// Check if a user is an admin (or owner).
    pub fn is_admin(&self, user: &str) -> bool {
        self.has_role(user, OrgRole::Admin)
    }

    /// Add a member to the organization.
    pub fn add_member(&mut self, member: OrgMember) -> bool {
        if self.is_member(&member.user) {
            return false;
        }
        self.members.push(member);
        self.updated_at = Self::now();
        true
    }

    /// Remove a member from the organization.
    /// Returns an error string if this would remove the last owner.
    pub fn remove_member(&mut self, user: &str) -> Result<bool, &'static str> {
        // Check if this is an owner and they're the last one
        if let Some(member) = self.get_member(user) {
            if member.role == OrgRole::Owner {
                let owner_count = self.members.iter().filter(|m| m.role == OrgRole::Owner).count();
                if owner_count <= 1 {
                    return Err("cannot remove last owner");
                }
            }
        }

        let before = self.members.len();
        self.members.retain(|m| m.user != user);
        let removed = self.members.len() < before;

        if removed {
            self.updated_at = Self::now();
        }

        Ok(removed)
    }

    /// Update a member's role.
    /// Returns an error if demoting the last owner.
    pub fn update_member_role(&mut self, user: &str, new_role: OrgRole) -> Result<bool, &'static str> {
        // Check if demoting the last owner
        if let Some(member) = self.get_member(user) {
            if member.role == OrgRole::Owner && new_role != OrgRole::Owner {
                let owner_count = self.members.iter().filter(|m| m.role == OrgRole::Owner).count();
                if owner_count <= 1 {
                    return Err("cannot demote last owner");
                }
            }
        }

        for member in &mut self.members {
            if member.user == user {
                member.role = new_role;
                self.updated_at = Self::now();
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Add a team to this organization.
    pub fn add_team(&mut self, team_id: u64) {
        self.teams.insert(team_id);
        self.updated_at = Self::now();
    }

    /// Remove a team from this organization.
    pub fn remove_team(&mut self, team_id: u64) -> bool {
        let removed = self.teams.remove(&team_id);
        if removed {
            self.updated_at = Self::now();
        }
        removed
    }

    /// Add a repository to this organization.
    pub fn add_repo(&mut self, repo_key: String) {
        self.repos.insert(repo_key);
        self.updated_at = Self::now();
    }

    /// Remove a repository from this organization.
    pub fn remove_repo(&mut self, repo_key: &str) -> bool {
        let removed = self.repos.remove(repo_key);
        if removed {
            self.updated_at = Self::now();
        }
        removed
    }

    /// Count the number of owners.
    pub fn owner_count(&self) -> usize {
        self.members.iter().filter(|m| m.role == OrgRole::Owner).count()
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
    fn test_org_role_ordering() {
        assert!(OrgRole::Member < OrgRole::Admin);
        assert!(OrgRole::Admin < OrgRole::Owner);
    }

    #[test]
    fn test_org_creation() {
        let org = Organization::new(1, "acme".into(), "Acme Corp".into(), "abc123".into());

        assert_eq!(org.id, 1);
        assert_eq!(org.name, "acme");
        assert_eq!(org.display_name, "Acme Corp");
        assert_eq!(org.members.len(), 1);
        assert!(org.is_owner("abc123"));
    }

    #[test]
    fn test_add_remove_member() {
        let mut org = Organization::new(1, "acme".into(), "Acme Corp".into(), "owner".into());

        // Add a member
        let member = OrgMember::new("user1".into(), OrgRole::Member, "owner".into());
        assert!(org.add_member(member));
        assert!(org.is_member("user1"));
        assert!(!org.is_admin("user1"));

        // Add admin
        let admin = OrgMember::new("admin1".into(), OrgRole::Admin, "owner".into());
        assert!(org.add_member(admin));
        assert!(org.is_admin("admin1"));
        assert!(!org.is_owner("admin1"));

        // Remove member
        assert!(org.remove_member("user1").unwrap());
        assert!(!org.is_member("user1"));

        // Cannot remove last owner
        assert!(org.remove_member("owner").is_err());
    }

    #[test]
    fn test_role_update() {
        let mut org = Organization::new(1, "acme".into(), "Acme Corp".into(), "owner1".into());

        // Add another owner
        let owner2 = OrgMember::new("owner2".into(), OrgRole::Owner, "owner1".into());
        org.add_member(owner2);

        // Now we can demote owner1
        assert!(org.update_member_role("owner1", OrgRole::Admin).unwrap());
        assert!(org.is_admin("owner1"));
        assert!(!org.is_owner("owner1"));

        // But cannot demote the last owner
        assert!(org.update_member_role("owner2", OrgRole::Member).is_err());
    }
}
