# Milestone 4: Governance & Access Control

> **Status:** In Progress
> **Started:** 2025-12-21

## Overview

Milestone 4 implements governance features that enable secure, multi-user collaboration with proper access control. This includes repository permissions, organizations with teams, and a webhook system for CI/CD integration.

## Goals

1. **Repository Permissions**: Granular access control (Read, Write, Admin)
2. **Organizations**: Multi-user repository ownership with teams
3. **Teams**: Group users for easier permission management
4. **Branch Protection**: Protect branches from direct pushes
5. **Webhooks**: Event notifications for external integrations

## Architecture

### New Crate: `guts-auth`

```
crates/guts-auth/
├── src/
│   ├── lib.rs           # Public API
│   ├── permission.rs    # Permission types and checks
│   ├── organization.rs  # Organization management
│   ├── team.rs          # Team types and logic
│   ├── collaborator.rs  # Repository collaborators
│   ├── branch_protection.rs # Branch protection rules
│   ├── webhook.rs       # Webhook subscriptions
│   ├── store.rs         # In-memory auth store
│   └── error.rs         # Error types
└── Cargo.toml
```

### Data Models

#### Permission Levels

```rust
/// Permission level for repository access
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    /// Can read repository contents
    Read = 0,
    /// Can read and write (push)
    Write = 1,
    /// Full control including settings
    Admin = 2,
}
```

#### Organization

```rust
pub struct Organization {
    pub id: u64,
    pub name: String,              // Unique org name (e.g., "acme-corp")
    pub display_name: String,      // "Acme Corporation"
    pub description: Option<String>,
    pub owner: String,             // Public key (hex) of creator
    pub members: Vec<OrgMember>,   // Direct members
    pub teams: Vec<u64>,           // Team IDs
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct OrgMember {
    pub user: String,              // Public key (hex)
    pub role: OrgRole,
}

pub enum OrgRole {
    Owner,      // Full control of org
    Admin,      // Manage teams and repos
    Member,     // Default member access
}
```

#### Team

```rust
pub struct Team {
    pub id: u64,
    pub org_id: u64,
    pub name: String,              // "backend-team"
    pub description: Option<String>,
    pub members: Vec<String>,      // Public keys
    pub permission: Permission,    // Default permission for team repos
    pub repos: Vec<String>,        // repo_keys team has access to
    pub created_at: u64,
    pub updated_at: u64,
}
```

#### Collaborator

```rust
pub struct Collaborator {
    pub id: u64,
    pub repo_key: String,          // "owner/repo"
    pub user: String,              // Public key (hex)
    pub permission: Permission,
    pub added_by: String,          // Who invited them
    pub created_at: u64,
}
```

#### Branch Protection

```rust
pub struct BranchProtection {
    pub id: u64,
    pub repo_key: String,
    pub pattern: String,           // Branch pattern (e.g., "main", "release/*")
    pub require_pr: bool,          // Must use pull request
    pub require_reviews: u32,      // Minimum approving reviews
    pub require_status_checks: Vec<String>, // Required CI checks
    pub restrict_pushes: bool,     // Only admins can push
    pub allow_force_push: bool,
    pub allow_deletion: bool,
    pub created_at: u64,
    pub updated_at: u64,
}
```

#### Webhook

```rust
pub struct Webhook {
    pub id: u64,
    pub repo_key: String,
    pub url: String,               // Callback URL
    pub secret: Option<String>,    // HMAC signing secret
    pub events: Vec<WebhookEvent>,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

pub enum WebhookEvent {
    Push,
    PullRequest,
    PullRequestReview,
    Issue,
    IssueComment,
    Create,    // Branch/tag created
    Delete,    // Branch/tag deleted
}
```

### API Endpoints

#### Organizations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/orgs` | List organizations |
| POST | `/api/orgs` | Create organization |
| GET | `/api/orgs/{org}` | Get organization |
| PATCH | `/api/orgs/{org}` | Update organization |
| DELETE | `/api/orgs/{org}` | Delete organization |
| GET | `/api/orgs/{org}/members` | List members |
| PUT | `/api/orgs/{org}/members/{user}` | Add/update member |
| DELETE | `/api/orgs/{org}/members/{user}` | Remove member |

#### Teams

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/orgs/{org}/teams` | List teams |
| POST | `/api/orgs/{org}/teams` | Create team |
| GET | `/api/orgs/{org}/teams/{team}` | Get team |
| PATCH | `/api/orgs/{org}/teams/{team}` | Update team |
| DELETE | `/api/orgs/{org}/teams/{team}` | Delete team |
| GET | `/api/orgs/{org}/teams/{team}/members` | List team members |
| PUT | `/api/orgs/{org}/teams/{team}/members/{user}` | Add member |
| DELETE | `/api/orgs/{org}/teams/{team}/members/{user}` | Remove member |
| GET | `/api/orgs/{org}/teams/{team}/repos` | List team repos |
| PUT | `/api/orgs/{org}/teams/{team}/repos/{repo}` | Add repo to team |
| DELETE | `/api/orgs/{org}/teams/{team}/repos/{repo}` | Remove repo from team |

#### Collaborators

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{repo}/collaborators` | List collaborators |
| PUT | `/api/repos/{owner}/{repo}/collaborators/{user}` | Add collaborator |
| DELETE | `/api/repos/{owner}/{repo}/collaborators/{user}` | Remove collaborator |
| GET | `/api/repos/{owner}/{repo}/collaborators/{user}/permission` | Check permission |

#### Branch Protection

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{repo}/branches/{branch}/protection` | Get protection |
| PUT | `/api/repos/{owner}/{repo}/branches/{branch}/protection` | Set protection |
| DELETE | `/api/repos/{owner}/{repo}/branches/{branch}/protection` | Remove protection |

#### Webhooks

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{repo}/hooks` | List webhooks |
| POST | `/api/repos/{owner}/{repo}/hooks` | Create webhook |
| GET | `/api/repos/{owner}/{repo}/hooks/{id}` | Get webhook |
| PATCH | `/api/repos/{owner}/{repo}/hooks/{id}` | Update webhook |
| DELETE | `/api/repos/{owner}/{repo}/hooks/{id}` | Delete webhook |
| POST | `/api/repos/{owner}/{repo}/hooks/{id}/ping` | Ping webhook |

### P2P Message Types

New message types for governance data replication:

```rust
pub enum AuthMessage {
    // Organization messages
    OrganizationCreated(Organization),
    OrganizationUpdated { id: u64, display_name: Option<String>, description: Option<String> },
    OrganizationDeleted { id: u64 },
    OrgMemberAdded { org_id: u64, member: OrgMember },
    OrgMemberRemoved { org_id: u64, user: String },

    // Team messages
    TeamCreated(Team),
    TeamUpdated { id: u64, name: Option<String>, permission: Option<Permission> },
    TeamDeleted { id: u64 },
    TeamMemberAdded { team_id: u64, user: String },
    TeamMemberRemoved { team_id: u64, user: String },
    TeamRepoAdded { team_id: u64, repo_key: String },
    TeamRepoRemoved { team_id: u64, repo_key: String },

    // Collaborator messages
    CollaboratorAdded(Collaborator),
    CollaboratorUpdated { repo_key: String, user: String, permission: Permission },
    CollaboratorRemoved { repo_key: String, user: String },

    // Branch protection messages
    BranchProtectionSet(BranchProtection),
    BranchProtectionRemoved { repo_key: String, pattern: String },

    // Webhook messages (local only, not replicated)

    // Sync messages
    SyncAuthRequest { scope: AuthSyncScope },
    SyncAuthResponse { data: AuthSyncData },
}
```

### CLI Commands

```bash
# Organization commands
guts org list
guts org create <name> --display-name "Display Name" --description "..."
guts org show <name>
guts org delete <name>
guts org members <name>
guts org add-member <org> <user> --role member|admin|owner
guts org remove-member <org> <user>

# Team commands
guts team list --org <org>
guts team create --org <org> --name <name> --permission read|write|admin
guts team show --org <org> <team>
guts team delete --org <org> <team>
guts team add-member --org <org> <team> <user>
guts team remove-member --org <org> <team> <user>
guts team add-repo --org <org> <team> <repo>
guts team remove-repo --org <org> <team> <repo>

# Collaborator commands
guts collaborator list --repo owner/repo
guts collaborator add --repo owner/repo <user> --permission read|write|admin
guts collaborator remove --repo owner/repo <user>

# Branch protection commands
guts protect <branch> --repo owner/repo --require-pr --require-reviews 2
guts unprotect <branch> --repo owner/repo

# Webhook commands
guts webhook list --repo owner/repo
guts webhook create --repo owner/repo --url https://... --events push,pr
guts webhook delete --repo owner/repo <id>
guts webhook ping --repo owner/repo <id>
```

## Implementation Plan

### Phase 1: Core Types (guts-auth crate)

1. Create crate structure
2. Implement `Permission` enum with ordering
3. Implement `Organization` with member management
4. Implement `Team` with repo associations
5. Implement `Collaborator` type
6. Implement `BranchProtection` rules
7. Implement `Webhook` type
8. Create `AuthStore` for in-memory storage
9. Add comprehensive unit tests

### Phase 2: Permission Checking

1. Implement permission resolution algorithm
2. Handle org → team → user permission inheritance
3. Implement branch protection checks
4. Add authorization middleware for API

### Phase 3: API Integration (guts-node)

1. Add `guts-auth` dependency
2. Integrate `AuthStore` into `AppState`
3. Implement organization endpoints
4. Implement team endpoints
5. Implement collaborator endpoints
6. Implement branch protection endpoints
7. Implement webhook endpoints
8. Add authorization checks to existing endpoints
9. Add API integration tests

### Phase 4: P2P Replication (guts-p2p)

1. Add auth message types to P2P protocol
2. Implement auth event broadcasting
3. Implement auth sync protocol
4. Handle concurrent updates
5. Add P2P integration tests

### Phase 5: CLI Commands (guts-cli)

1. Add `org` subcommand
2. Add `team` subcommand
3. Add `collaborator` subcommand
4. Add `protect`/`unprotect` commands
5. Add `webhook` subcommand

### Phase 6: E2E Testing

1. Create multi-user permission test
2. Test organization workflow
3. Test team permissions inheritance
4. Test branch protection enforcement
5. Test webhook delivery

## Success Criteria

- [ ] Create and manage organizations via API
- [ ] Create teams within organizations
- [ ] Add collaborators with specific permissions
- [ ] Permission checks on all repository operations
- [ ] Branch protection rules enforced
- [ ] Webhooks fire on repository events
- [ ] P2P replication of auth data across nodes
- [ ] CLI commands for governance operations
- [ ] E2E test passing for permission scenarios

## Dependencies

- `guts-types`: Core types (Identity, ObjectId)
- `guts-collaboration`: PR/Issue integration
- `guts-p2p`: P2P networking layer
- `serde`: Serialization
- `thiserror`: Error handling

## Security Considerations

1. **Permission Escalation**: Prevent users from granting higher permissions than they have
2. **Org Takeover**: Ensure at least one owner exists
3. **Webhook Security**: HMAC signing of payloads, secret storage
4. **Rate Limiting**: Prevent brute-force permission enumeration
5. **Audit Logging**: Track all permission changes

## Future Considerations

These features are out of scope for Milestone 4 but should be considered:

1. **Fine-grained Permissions**: Path-based permissions
2. **Permission Templates**: Predefined permission sets
3. **Audit Trail**: Complete history of permission changes
4. **SSO Integration**: External identity providers
5. **API Keys**: Non-user authentication for CI/CD
6. **2FA Enforcement**: Require 2FA for sensitive operations

## References

- [GitHub Organizations API](https://docs.github.com/en/rest/orgs)
- [GitHub Branch Protection API](https://docs.github.com/en/rest/branches/branch-protection)
- [GitHub Webhooks API](https://docs.github.com/en/rest/webhooks)
- [Guts PRD](./PRD.md)
