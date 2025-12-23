# ADR-005: Permission and Access Control Hierarchy

## Status

Accepted

## Date

2025-12-20

## Context

Guts requires a comprehensive access control system to:

1. Control who can read/write to repositories
2. Manage organizations with multiple users
3. Support team-based access for groups
4. Protect branches from unauthorized changes
5. Enable webhook integrations

The system must be flexible enough to handle:
- Personal repositories
- Organization-owned repositories
- External collaborators
- Team-based access

## Decision

We implement a hierarchical permission system in `guts-auth` crate:

### Permission Levels

```rust
pub enum Permission {
    Read,   // Clone, view
    Write,  // Push, create branches
    Admin,  // Settings, manage access
}

impl Permission {
    /// Check if this permission level includes another
    pub fn has(&self, other: Permission) -> bool {
        match (self, other) {
            (Permission::Admin, _) => true,
            (Permission::Write, Permission::Read) => true,
            (Permission::Write, Permission::Write) => true,
            (Permission::Read, Permission::Read) => true,
            _ => false,
        }
    }
}
```

### Permission Sources (Priority Order)

1. **Repository Owner**: Always has Admin
2. **Collaborator**: Direct permission grant on repository
3. **Team Member**: Inherits team's default permission
4. **Organization Member**: Inherits org-level access
5. **Public**: Read access if repository is public

### Resolution Algorithm

```rust
pub async fn resolve_permission(
    &self,
    repo_key: &str,
    user: &str,
) -> Result<Option<Permission>> {
    // 1. Check if owner
    if self.is_owner(repo_key, user)? {
        return Ok(Some(Permission::Admin));
    }

    // 2. Check direct collaborator
    if let Some(perm) = self.get_collaborator(repo_key, user)? {
        return Ok(Some(perm));
    }

    // 3. Check team membership
    if let Some(perm) = self.get_team_permission(repo_key, user)? {
        return Ok(Some(perm));
    }

    // 4. Check organization membership
    if let Some(perm) = self.get_org_permission(repo_key, user)? {
        return Ok(Some(perm));
    }

    // 5. No permission
    Ok(None)
}
```

### Entity Model

```rust
/// Organization - group of users and teams
pub struct Organization {
    pub id: Uuid,
    pub name: String,           // Unique slug
    pub display_name: String,
    pub members: Vec<Member>,   // Users with roles
    pub created_at: DateTime<Utc>,
}

/// Organization Member
pub struct Member {
    pub user: String,
    pub role: OrgRole,          // Owner, Admin, Member
}

/// Team - subset of org for access control
pub struct Team {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub default_permission: Permission,
    pub members: Vec<String>,
    pub repos: Vec<String>,     // repo keys with access
}

/// Collaborator - direct repo access grant
pub struct Collaborator {
    pub user: String,
    pub repo_key: String,
    pub permission: Permission,
}

/// Branch Protection - rules for protected branches
pub struct BranchProtection {
    pub id: Uuid,
    pub repo_key: String,
    pub branch: String,         // Branch pattern (e.g., "main")
    pub require_pr: bool,       // Must use PR to merge
    pub required_reviews: u32,  // Minimum approvals
    pub require_linear: bool,   // No merge commits
}
```

### Organization Roles

```
Owner:  Full control, billing, can delete org
Admin:  Manage members, teams, settings
Member: Access based on team membership
```

## Consequences

### Positive

- **Flexible**: Supports personal, team, and org workflows
- **Hierarchical**: Clear precedence for permission resolution
- **Familiar**: Mirrors GitHub's access model
- **Secure**: Deny by default, explicit grants required

### Negative

- **Complexity**: Multiple permission sources to check
- **Performance**: May need caching for hot paths
- **Edge cases**: Team+collaborator overlap needs handling

### Neutral

- Branch protection is per-repository
- No cross-organization permissions

## Branch Protection Details

Protected branches enforce:

```rust
pub struct BranchProtection {
    // Core settings
    pub require_pr: bool,           // Direct push blocked
    pub required_reviews: u32,      // Min approving reviews

    // Future enhancements
    pub dismiss_stale_reviews: bool,
    pub require_code_owner_review: bool,
    pub restrict_push_access: Vec<String>,
}
```

Protection check flow:
1. Intercept push to protected branch
2. Verify push is from merge operation
3. Check PR has required approvals
4. Allow or reject push

## Webhook Integration

Webhooks notify external systems of events:

```rust
pub struct Webhook {
    pub id: Uuid,
    pub repo_key: String,
    pub url: String,
    pub secret: Option<String>,     // HMAC signing
    pub events: Vec<WebhookEvent>,  // push, pr, issue, etc.
    pub active: bool,
}
```

## Alternatives Considered

### Role-Based Access Control (RBAC)

Pure RBAC with predefined roles.

**Rejected because:**
- Less flexible than permission levels
- Harder to customize per-repository

### Access Control Lists (ACL)

Pure ACL without hierarchy.

**Rejected because:**
- No organization grouping
- Harder to manage at scale

### Capability-Based Security

Token-based access with embedded permissions.

**Deferred because:**
- Adds complexity
- Current model sufficient for MVP
- Can be added for API tokens later

## References

- [GitHub Organization Permissions](https://docs.github.com/en/organizations/managing-access-to-your-organizations-repositories)
- [GitHub Branch Protection](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches)
- [Guts Milestone 4 Spec](../MILESTONE-4.md)
