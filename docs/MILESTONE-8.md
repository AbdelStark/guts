# Milestone 8: Git & GitHub Compatibility

> **Status:** In Progress
> **Started:** 2025-12-21

## Overview

Milestone 8 focuses on making Guts fully compatible with existing Git and GitHub tooling. This includes user accounts with personal access tokens (PATs), GitHub-compatible API responses, repository contents API, releases/tags management, archive downloads, and SSH key management. These features enable seamless integration with existing CI/CD tools, IDE plugins, and developer workflows.

## Goals

1. **User Accounts**: Full user account management with profile, settings, and identity mapping
2. **Personal Access Tokens**: Token-based authentication for API and Git operations
3. **GitHub API Compatibility**: Standard headers, pagination, rate limits, and error formats
4. **Contents API**: Browse and download files without cloning the repository
5. **Releases & Tags**: Release management with assets and changelogs
6. **Archive Downloads**: Tarball and zipball downloads for any ref
7. **SSH Key Management**: User SSH key storage for future SSH protocol support

## Architecture

### New Crate: `guts-compat`

```
crates/guts-compat/
├── src/
│   ├── lib.rs           # Public API
│   ├── error.rs         # Error types
│   ├── user.rs          # User account types
│   ├── token.rs         # Personal access token types
│   ├── ssh_key.rs       # SSH key management
│   ├── release.rs       # Release and tag types
│   ├── contents.rs      # Repository contents types
│   ├── archive.rs       # Archive generation
│   ├── pagination.rs    # GitHub-style pagination
│   ├── rate_limit.rs    # Rate limiting
│   ├── middleware.rs    # Axum middleware for compatibility
│   └── store.rs         # Compatibility data storage
└── Cargo.toml
```

### Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Token Generation | CSPRNG + Base62 | Secure, URL-safe tokens |
| Token Hashing | Argon2id | Industry standard password hashing |
| SSH Keys | ed25519-dalek | Ed25519 key validation |
| Archives | flate2 + tar | Standard compression formats |
| Rate Limiting | Token bucket | Standard rate limiting algorithm |
| Pagination | Link headers | GitHub API standard |

### Core Types

#### User Account

```rust
/// A user account in the system
pub struct User {
    pub id: UserId,
    pub username: String,            // Unique username (lowercase, alphanumeric)
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub avatar_url: Option<String>,
    pub public_key: String,          // Ed25519 public key (identity)
    pub created_at: u64,
    pub updated_at: u64,
}

/// User profile response (public)
pub struct UserProfile {
    pub login: String,               // GitHub compatibility
    pub id: u64,
    pub avatar_url: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub public_repos: u64,
    pub followers: u64,
    pub following: u64,
    pub created_at: String,          // ISO 8601
}
```

#### Personal Access Token

```rust
/// A personal access token for authentication
pub struct PersonalAccessToken {
    pub id: TokenId,
    pub user_id: UserId,
    pub name: String,                // User-provided name
    pub token_hash: String,          // Argon2id hash (never store plaintext)
    pub token_prefix: String,        // First 8 chars for identification
    pub scopes: Vec<TokenScope>,
    pub expires_at: Option<u64>,
    pub last_used_at: Option<u64>,
    pub created_at: u64,
}

/// Token scopes for fine-grained permissions
pub enum TokenScope {
    // Repository
    RepoRead,                        // Read access to repos
    RepoWrite,                       // Push access to repos
    RepoAdmin,                       // Admin access to repos
    RepoDelete,                      // Delete repos

    // User
    UserRead,                        // Read user profile
    UserWrite,                       // Update user profile
    UserEmail,                       // Access email addresses

    // Organization
    OrgRead,                         // Read org info
    OrgWrite,                        // Manage org
    OrgAdmin,                        // Admin org operations

    // SSH Keys
    SshKeyRead,                      // List SSH keys
    SshKeyWrite,                     // Add/remove SSH keys

    // Workflow
    WorkflowRead,                    // Read workflows
    WorkflowWrite,                   // Trigger workflows

    // Webhooks
    WebhookRead,                     // Read webhooks
    WebhookWrite,                    // Manage webhooks

    // Admin (superuser)
    Admin,                           // Full admin access
}

/// Token format: guts_<prefix>_<secret>
/// Example: guts_abc12345_XXXXXXXXXXXXXXXXXXXXXXXXXXX
pub struct TokenValue {
    pub prefix: String,              // 8 chars
    pub secret: String,              // 32 chars
}
```

#### SSH Key

```rust
/// An SSH public key for authentication
pub struct SshKey {
    pub id: SshKeyId,
    pub user_id: UserId,
    pub title: String,
    pub key_type: SshKeyType,
    pub public_key: String,          // Full public key string
    pub fingerprint: String,         // SHA256 fingerprint
    pub created_at: u64,
    pub last_used_at: Option<u64>,
}

pub enum SshKeyType {
    Ed25519,
    Rsa,
    Ecdsa,
}
```

#### Release

```rust
/// A release (tagged version)
pub struct Release {
    pub id: ReleaseId,
    pub repo_key: String,
    pub tag_name: String,            // e.g., "v1.0.0"
    pub target_commitish: String,    // Branch or commit SHA
    pub name: Option<String>,        // Release title
    pub body: Option<String>,        // Markdown body
    pub draft: bool,
    pub prerelease: bool,
    pub author: String,              // Username
    pub assets: Vec<ReleaseAsset>,
    pub created_at: u64,
    pub published_at: Option<u64>,
}

/// An asset attached to a release
pub struct ReleaseAsset {
    pub id: AssetId,
    pub release_id: ReleaseId,
    pub name: String,                // Filename
    pub label: Option<String>,
    pub content_type: String,
    pub size: u64,
    pub download_count: u64,
    pub content_hash: String,        // SHA-256
    pub created_at: u64,
    pub uploader: String,
}
```

#### Repository Contents

```rust
/// Content entry (file or directory)
pub struct ContentEntry {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub content_type: ContentType,
    pub encoding: Option<String>,    // "base64" for files
    pub content: Option<String>,     // Base64 content (files only)
    pub download_url: Option<String>,
    pub html_url: Option<String>,
}

pub enum ContentType {
    File,
    Dir,
    Symlink,
    Submodule,
}
```

### Middleware Types

```rust
/// Rate limit state
pub struct RateLimitState {
    pub limit: u32,                  // Requests per hour
    pub remaining: u32,              // Remaining requests
    pub reset: u64,                  // Unix timestamp when limit resets
    pub used: u32,                   // Requests used this hour
}

/// Pagination info for Link header
pub struct PaginationLinks {
    pub first: Option<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
    pub last: Option<String>,
}
```

## Data Flow

### Token Authentication Flow

```
Client Request
     │
     │ Authorization: Bearer guts_abc12345_XXXXX
     ▼
┌─────────────────────────────────┐
│      Token Middleware           │
│  1. Extract token from header   │
│  2. Parse prefix + secret       │
│  3. Lookup by prefix            │
│  4. Verify hash (Argon2id)      │
│  5. Check expiration            │
│  6. Validate scopes             │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│     Request Context             │
│  - User ID                      │
│  - Username                     │
│  - Scopes                       │
│  - Rate limit state             │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│       Route Handler             │
│  - Check required scope         │
│  - Process request              │
│  - Return response              │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│     Response Middleware         │
│  - Add X-RateLimit-* headers    │
│  - Add Link header (pagination) │
│  - Format errors consistently   │
└─────────────────────────────────┘
```

### Contents API Flow

```
GET /repos/{owner}/{repo}/contents/{path}?ref=main
     │
     ▼
┌─────────────────────────────────┐
│     Resolve Reference           │
│  - Parse ref (branch/tag/sha)   │
│  - Get commit object            │
│  - Get tree from commit         │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│      Traverse Tree              │
│  - Walk path segments           │
│  - Find target entry            │
│  - Determine type (file/dir)    │
└────────────────┬────────────────┘
                 │
                 ▼
     ┌───────────┴───────────┐
     │                       │
   File?                   Dir?
     │                       │
     ▼                       ▼
┌─────────────┐     ┌─────────────┐
│ Get Blob    │     │ List Tree   │
│ Base64 enc  │     │ Entries     │
└─────────────┘     └─────────────┘
```

## Implementation Plan

### Phase 1: Core Types & Storage

1. [ ] Create `guts-compat` crate structure
2. [ ] Define User and UserProfile types
3. [ ] Define PersonalAccessToken types
4. [ ] Define SshKey types
5. [ ] Create compatibility data store
6. [ ] Add user storage with username index

### Phase 2: User Accounts

1. [ ] POST `/api/users` - Create user account
2. [ ] GET `/api/users` - List users
3. [ ] GET `/api/users/{username}` - Get user profile
4. [ ] PATCH `/api/users/{username}` - Update user profile
5. [ ] GET `/api/user` - Get authenticated user
6. [ ] PATCH `/api/user` - Update authenticated user
7. [ ] GET `/api/users/{username}/repos` - List user repos

### Phase 3: Personal Access Tokens

1. [ ] Implement token generation (CSPRNG)
2. [ ] Implement Argon2id hashing
3. [ ] POST `/api/user/tokens` - Create token
4. [ ] GET `/api/user/tokens` - List tokens (no secrets)
5. [ ] DELETE `/api/user/tokens/{id}` - Revoke token
6. [ ] Create token authentication middleware
7. [ ] Support Basic auth (username:token)
8. [ ] Support Bearer token auth

### Phase 4: SSH Key Management

1. [ ] POST `/api/user/keys` - Add SSH key
2. [ ] GET `/api/user/keys` - List SSH keys
3. [ ] GET `/api/user/keys/{id}` - Get SSH key
4. [ ] DELETE `/api/user/keys/{id}` - Remove SSH key
5. [ ] Validate SSH key format
6. [ ] Calculate SSH key fingerprint

### Phase 5: GitHub API Compatibility Middleware

1. [ ] Rate limiting middleware (X-RateLimit-* headers)
2. [ ] Pagination middleware (Link header)
3. [ ] Error response formatting
4. [ ] ETag/If-None-Match support
5. [ ] GitHub media type handling
6. [ ] CORS headers for API

### Phase 6: Repository Contents API

1. [ ] GET `/api/repos/{owner}/{repo}/contents/{path}` - Get contents
2. [ ] GET `/api/repos/{owner}/{repo}/readme` - Get README
3. [ ] GET `/api/repos/{owner}/{repo}/license` - Get license
4. [ ] Support `ref` query parameter
5. [ ] Support directory listing
6. [ ] Support symlink resolution

### Phase 7: Archive Downloads

1. [ ] GET `/api/repos/{owner}/{repo}/tarball/{ref}` - Download tarball
2. [ ] GET `/api/repos/{owner}/{repo}/zipball/{ref}` - Download zipball
3. [ ] Implement tar archive generation
4. [ ] Implement zip archive generation
5. [ ] Stream archives for large repos

### Phase 8: Releases & Tags

1. [ ] POST `/api/repos/{owner}/{repo}/releases` - Create release
2. [ ] GET `/api/repos/{owner}/{repo}/releases` - List releases
3. [ ] GET `/api/repos/{owner}/{repo}/releases/{id}` - Get release
4. [ ] PATCH `/api/repos/{owner}/{repo}/releases/{id}` - Update release
5. [ ] DELETE `/api/repos/{owner}/{repo}/releases/{id}` - Delete release
6. [ ] GET `/api/repos/{owner}/{repo}/releases/latest` - Get latest
7. [ ] GET `/api/repos/{owner}/{repo}/releases/tags/{tag}` - Get by tag
8. [ ] POST `/api/repos/{owner}/{repo}/releases/{id}/assets` - Upload asset
9. [ ] GET `/api/repos/{owner}/{repo}/releases/{id}/assets` - List assets
10. [ ] DELETE `/api/repos/{owner}/{repo}/releases/{id}/assets/{asset_id}` - Delete asset

### Phase 9: CLI Commands

1. [ ] `guts auth login` - Interactive login
2. [ ] `guts auth token create` - Create PAT
3. [ ] `guts auth token list` - List tokens
4. [ ] `guts auth token revoke` - Revoke token
5. [ ] `guts ssh-key add` - Add SSH key
6. [ ] `guts ssh-key list` - List SSH keys
7. [ ] `guts release create` - Create release
8. [ ] `guts release list` - List releases
9. [ ] `guts release download` - Download release assets

### Phase 10: Tests & Documentation

1. [ ] Unit tests for token generation/verification
2. [ ] Unit tests for SSH key parsing
3. [ ] E2E tests for user account CRUD
4. [ ] E2E tests for token authentication
5. [ ] E2E tests for contents API
6. [ ] E2E tests for releases API
7. [ ] Update API documentation
8. [ ] Add compatibility guide

## API Reference

### User Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/users` | Create user account |
| GET | `/api/users` | List users |
| GET | `/api/users/{username}` | Get user profile |
| PATCH | `/api/users/{username}` | Update user profile |
| GET | `/api/user` | Get authenticated user |
| PATCH | `/api/user` | Update authenticated user |
| GET | `/api/users/{username}/repos` | List user repos |

### Token Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/user/tokens` | Create personal access token |
| GET | `/api/user/tokens` | List tokens (no secrets) |
| GET | `/api/user/tokens/{id}` | Get token metadata |
| DELETE | `/api/user/tokens/{id}` | Revoke token |

### SSH Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/user/keys` | Add SSH key |
| GET | `/api/user/keys` | List SSH keys |
| GET | `/api/user/keys/{id}` | Get SSH key |
| DELETE | `/api/user/keys/{id}` | Remove SSH key |

### Contents Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos/{owner}/{repo}/contents` | Get root contents |
| GET | `/api/repos/{owner}/{repo}/contents/{path}` | Get file/directory |
| GET | `/api/repos/{owner}/{repo}/readme` | Get README |
| GET | `/api/repos/{owner}/{repo}/license` | Get license |

### Archive Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos/{owner}/{repo}/tarball/{ref}` | Download tarball |
| GET | `/api/repos/{owner}/{repo}/zipball/{ref}` | Download zipball |

### Release Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/repos/{owner}/{repo}/releases` | Create release |
| GET | `/api/repos/{owner}/{repo}/releases` | List releases |
| GET | `/api/repos/{owner}/{repo}/releases/latest` | Get latest release |
| GET | `/api/repos/{owner}/{repo}/releases/tags/{tag}` | Get by tag |
| GET | `/api/repos/{owner}/{repo}/releases/{id}` | Get release |
| PATCH | `/api/repos/{owner}/{repo}/releases/{id}` | Update release |
| DELETE | `/api/repos/{owner}/{repo}/releases/{id}` | Delete release |
| POST | `/api/repos/{owner}/{repo}/releases/{id}/assets` | Upload asset |
| GET | `/api/repos/{owner}/{repo}/releases/{id}/assets` | List assets |
| DELETE | `/api/repos/{owner}/{repo}/releases/{id}/assets/{asset_id}` | Delete asset |

### Rate Limit Endpoint

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/rate_limit` | Get current rate limit status |

## Response Headers

All API responses include GitHub-compatible headers:

```
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4999
X-RateLimit-Reset: 1234567890
X-RateLimit-Used: 1
X-RateLimit-Resource: core

Link: <https://api.guts.network/users?page=2>; rel="next",
      <https://api.guts.network/users?page=5>; rel="last"

ETag: "abc123"
```

## Error Response Format

Errors follow GitHub's format:

```json
{
  "message": "Not Found",
  "documentation_url": "https://docs.guts.network/rest/repos/contents"
}
```

For validation errors:

```json
{
  "message": "Validation Failed",
  "errors": [
    {
      "resource": "Release",
      "field": "tag_name",
      "code": "already_exists"
    }
  ],
  "documentation_url": "https://docs.guts.network/rest/releases"
}
```

## Token Format

Tokens follow a structured format for easy identification:

```
guts_<prefix>_<secret>

Examples:
guts_abc12345_Yq3bH7mN9xKpL2vR4wF6jT8dC1gS5aE0
guts_xyz98765_Mn4cJ6kP8qW2tY5uB9vX3zA7dF1gH0iL
```

- Prefix: 8 characters (lowercase alphanumeric)
- Secret: 32 characters (mixed case alphanumeric)
- Only the first 8 characters are stored; the full token is hashed

## Authentication Methods

### Bearer Token (Recommended)

```bash
curl -H "Authorization: Bearer guts_abc12345_XXXXX" \
  https://api.guts.network/user
```

### Basic Auth (Token as password)

```bash
curl -u "username:guts_abc12345_XXXXX" \
  https://api.guts.network/user
```

### Git Credential Helper

```bash
git config --global credential.helper store
echo "https://username:guts_abc12345_XXXXX@guts.network" >> ~/.git-credentials
```

## Success Criteria

- [ ] Users can create accounts with unique usernames
- [ ] Users can generate and manage personal access tokens
- [ ] Token authentication works for all API endpoints
- [ ] Git push/pull works with token authentication
- [ ] Contents API returns file/directory listings
- [ ] Archives can be downloaded for any ref
- [ ] Releases can be created and managed
- [ ] Rate limiting headers present on all responses
- [ ] Pagination works with Link headers
- [ ] Error responses match GitHub format
- [ ] SSH keys can be stored and validated

## Security Considerations

1. **Token Storage**: Tokens are hashed with Argon2id; plaintext never stored
2. **Token Rotation**: Support for token expiration and revocation
3. **Scope Limitation**: Fine-grained scopes limit token capabilities
4. **Rate Limiting**: Prevent abuse with per-user rate limits
5. **Audit Logging**: Log all authentication events
6. **Input Validation**: Validate all user inputs strictly
7. **SSH Key Validation**: Verify SSH key format and type

## Performance Considerations

1. **Token Lookup**: Index tokens by prefix for O(1) lookup
2. **Caching**: Cache user profiles and rate limit state
3. **Streaming**: Stream large archives to avoid memory issues
4. **Compression**: Compress archive responses
5. **Pagination**: Limit default page size to 30 items

## Future Enhancements

These features are out of scope for Milestone 8 but planned:

1. **SSH Protocol**: Full SSH Git protocol support
2. **OAuth 2.0**: OAuth apps and authorization flow
3. **GitHub App Compatibility**: App installation and authentication
4. **Two-Factor Authentication**: TOTP/WebAuthn support
5. **GPG Key Management**: Commit signing verification
6. **GraphQL API**: GitHub GraphQL compatibility
7. **Starring/Watching**: Repository stars and watch notifications

## Dependencies

- `guts-storage`: Object storage for contents API
- `guts-git`: Git protocol for archive generation
- `guts-auth`: Permission checking
- `argon2`: Password hashing
- `base64`: Content encoding
- `flate2`: Gzip compression
- `tar`: Tar archive creation
- `zip`: Zip archive creation
- `rand`: Secure random token generation

## References

- [GitHub REST API](https://docs.github.com/en/rest)
- [GitHub Authentication](https://docs.github.com/en/authentication)
- [Git Smart HTTP Protocol](https://git-scm.com/docs/http-protocol)
- [Argon2 Password Hashing](https://github.com/P-H-C/phc-winner-argon2)
- [RFC 6749 OAuth 2.0](https://tools.ietf.org/html/rfc6749)
