//! Permission levels and access control.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Permission level for repository access.
///
/// Permissions are ordered: Read < Write < Admin
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Can read repository contents (clone, pull).
    Read,
    /// Can read and write (push commits).
    Write,
    /// Full control including settings, collaborators, and deletion.
    Admin,
}

impl Permission {
    /// Check if this permission level grants at least the required level.
    pub fn has(&self, required: Permission) -> bool {
        *self >= required
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" => Some(Permission::Read),
            "write" | "push" => Some(Permission::Write),
            "admin" | "owner" => Some(Permission::Admin),
            _ => None,
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Write => write!(f, "write"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

impl Default for Permission {
    fn default() -> Self {
        Permission::Read
    }
}

/// A permission grant for a specific resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// The resource key (e.g., "owner/repo").
    pub resource: String,
    /// The granted permission level.
    pub permission: Permission,
    /// Source of this permission (e.g., "owner", "collaborator", "team:backend").
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::Read < Permission::Write);
        assert!(Permission::Write < Permission::Admin);
        assert!(Permission::Read < Permission::Admin);
    }

    #[test]
    fn test_permission_has() {
        assert!(Permission::Admin.has(Permission::Read));
        assert!(Permission::Admin.has(Permission::Write));
        assert!(Permission::Admin.has(Permission::Admin));

        assert!(Permission::Write.has(Permission::Read));
        assert!(Permission::Write.has(Permission::Write));
        assert!(!Permission::Write.has(Permission::Admin));

        assert!(Permission::Read.has(Permission::Read));
        assert!(!Permission::Read.has(Permission::Write));
        assert!(!Permission::Read.has(Permission::Admin));
    }

    #[test]
    fn test_permission_from_str() {
        assert_eq!(Permission::from_str("read"), Some(Permission::Read));
        assert_eq!(Permission::from_str("write"), Some(Permission::Write));
        assert_eq!(Permission::from_str("push"), Some(Permission::Write));
        assert_eq!(Permission::from_str("admin"), Some(Permission::Admin));
        assert_eq!(Permission::from_str("owner"), Some(Permission::Admin));
        assert_eq!(Permission::from_str("ADMIN"), Some(Permission::Admin));
        assert_eq!(Permission::from_str("invalid"), None);
    }

    #[test]
    fn test_permission_display() {
        assert_eq!(format!("{}", Permission::Read), "read");
        assert_eq!(format!("{}", Permission::Write), "write");
        assert_eq!(format!("{}", Permission::Admin), "admin");
    }
}
