//! # Guts Compatibility Layer
//!
//! Git and GitHub compatibility layer for the Guts code collaboration platform.
//!
//! This crate provides:
//! - **User Accounts**: User registration and profile management
//! - **Personal Access Tokens**: Token-based authentication for API and Git operations
//! - **SSH Keys**: SSH key management for future SSH protocol support
//! - **Releases**: Release and asset management
//! - **Contents API**: Repository file browsing
//! - **Archive Downloads**: Tarball and zipball generation
//! - **Rate Limiting**: GitHub-compatible rate limiting
//! - **Pagination**: GitHub-style Link header pagination
//!
//! ## Example
//!
//! ```rust
//! use guts_compat::{CompatStore, TokenScope};
//!
//! // Create a store
//! let store = CompatStore::new();
//!
//! // Create a user
//! let user = store.users.create(
//!     "alice".to_string(),
//!     "ed25519_pubkey_hex".to_string(),
//! ).unwrap();
//!
//! // Create a personal access token
//! let (token, plaintext) = store.tokens.create(
//!     user.id,
//!     "CI/CD Token".to_string(),
//!     vec![TokenScope::RepoRead, TokenScope::RepoWrite],
//!     None, // No expiration
//! ).unwrap();
//!
//! println!("Token: {}", plaintext);
//!
//! // Verify the token later
//! let (user_id, scopes) = store.tokens.verify(&plaintext).unwrap();
//! assert_eq!(user_id, user.id);
//! ```
//!
//! ## Authentication
//!
//! Tokens can be used in several ways:
//!
//! ```text
//! # Bearer token (recommended)
//! curl -H "Authorization: Bearer guts_abc12345_XXXXX" https://api.guts.network/user
//!
//! # Token header (GitHub-style)
//! curl -H "Authorization: token guts_abc12345_XXXXX" https://api.guts.network/user
//!
//! # Basic auth (username:token)
//! curl -u "alice:guts_abc12345_XXXXX" https://api.guts.network/user
//! ```
//!
//! ## Rate Limiting
//!
//! All API responses include rate limit headers:
//!
//! ```text
//! X-RateLimit-Limit: 5000
//! X-RateLimit-Remaining: 4999
//! X-RateLimit-Reset: 1234567890
//! X-RateLimit-Used: 1
//! X-RateLimit-Resource: core
//! ```

pub mod archive;
pub mod contents;
pub mod error;
pub mod middleware;
pub mod pagination;
pub mod rate_limit;
pub mod release;
pub mod ssh_key;
pub mod store;
pub mod token;
pub mod user;

// Re-export main types
pub use archive::{ArchiveEntry, ArchiveFormat, TarGzBuilder, ZipBuilder, create_archive};
pub use contents::{
    ContentEntry, ContentType, ContentsQuery, LicenseResponse, ReadmeResponse,
    base64_encode, detect_spdx_id, is_readme_file, recognize_license_file,
};
pub use error::{CompatError, Result};
pub use middleware::{
    AuthContext, AuthorizationValue, ErrorResponse, ResponseHeaders, ValidationError,
    ValidationErrorCode, parse_authorization_header, resource_from_path,
};
pub use pagination::{
    PaginatedResponse, PaginationLinks, PaginationParams, paginate,
    DEFAULT_PER_PAGE, MAX_PER_PAGE,
};
pub use rate_limit::{
    RateLimitHeaders, RateLimitInfo, RateLimitResource, RateLimitResponse,
    RateLimitResources, RateLimitState, RateLimiter,
    DEFAULT_RATE_LIMIT, UNAUTHENTICATED_RATE_LIMIT,
};
pub use release::{
    AssetId, AssetResponse, AuthorInfo, CreateReleaseRequest, Release, ReleaseAsset,
    ReleaseId, ReleaseResponse, UpdateReleaseRequest,
};
pub use ssh_key::{AddSshKeyRequest, SshKey, SshKeyId, SshKeyResponse, SshKeyType};
pub use store::{
    CompatStats, CompatStore, ReleaseStore, SshKeyStore, TokenStore, UserStore,
};
pub use token::{
    CreateTokenRequest, PersonalAccessToken, TokenId, TokenResponse, TokenScope, TokenValue,
};
pub use user::{CreateUserRequest, UpdateUserRequest, User, UserId, UserProfile};

/// Version of the compatibility layer.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api() {
        // Verify main types are accessible
        let store = CompatStore::new();
        assert_eq!(store.users.count(), 0);
        assert_eq!(store.tokens.count(), 0);
    }

    #[test]
    fn test_full_user_token_flow() {
        let store = CompatStore::new();

        // Create user
        let user = store
            .users
            .create("alice".to_string(), "pubkey123".to_string())
            .unwrap();
        assert_eq!(user.username, "alice");

        // Create token
        let (token, plaintext) = store
            .tokens
            .create(
                user.id,
                "My Token".to_string(),
                vec![TokenScope::RepoRead, TokenScope::RepoWrite],
                None,
            )
            .unwrap();

        assert!(plaintext.starts_with("guts_"));

        // Verify token
        let (user_id, scopes) = store.tokens.verify(&plaintext).unwrap();
        assert_eq!(user_id, user.id);
        assert!(scopes.contains(&TokenScope::RepoRead));
        assert!(scopes.contains(&TokenScope::RepoWrite));

        // Check token has correct scopes
        assert!(token.has_scope(TokenScope::RepoRead));
        assert!(token.has_scope(TokenScope::RepoWrite));
        assert!(!token.has_scope(TokenScope::Admin));
    }

    #[test]
    fn test_rate_limiting() {
        let limiter = RateLimiter::new();

        // Authenticated users get higher limits
        let state = limiter.get_state("user1", RateLimitResource::Core, true);
        assert_eq!(state.limit, 5000);

        // Unauthenticated users get lower limits
        let state = limiter.get_state("anon", RateLimitResource::Core, false);
        assert_eq!(state.limit, 60);
    }

    #[test]
    fn test_pagination() {
        let items: Vec<i32> = (1..=100).collect();
        let params = PaginationParams::new(2, 10);
        let response = paginate(&items, &params);

        assert_eq!(response.items.len(), 10);
        assert_eq!(response.total_count, 100);
        assert_eq!(response.page, 2);
        assert!(response.has_next_page());
        assert!(response.has_prev_page());
    }

    #[test]
    fn test_release_management() {
        let store = CompatStore::new();

        // Create a release
        let release = store
            .releases
            .create(
                "alice/repo".to_string(),
                "v1.0.0".to_string(),
                "main".to_string(),
                "alice".to_string(),
            )
            .unwrap();

        // Add an asset
        let asset = store
            .releases
            .add_asset(
                release.id,
                "app-linux-amd64.tar.gz".to_string(),
                "application/gzip".to_string(),
                b"binary content".to_vec(),
                "alice".to_string(),
            )
            .unwrap();

        assert_eq!(asset.name, "app-linux-amd64.tar.gz");

        // Get asset content
        let content = store.releases.get_asset_content(&asset.content_hash);
        assert!(content.is_some());
    }

    #[test]
    fn test_archive_generation() {
        let entries = vec![
            ArchiveEntry::file("README.md".to_string(), b"# My Project".to_vec()),
            ArchiveEntry::file("src/main.rs".to_string(), b"fn main() {}".to_vec()),
        ];

        let archive = create_archive(ArchiveFormat::TarGz, "my-project-v1.0.0".to_string(), entries);
        assert!(archive.is_ok());
    }

    #[test]
    fn test_ssh_key_management() {
        let store = CompatStore::new();

        // Add an SSH key
        let key = store.ssh_keys.add(
            1,
            "My Laptop".to_string(),
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl user@laptop".to_string(),
        ).unwrap();

        assert!(key.fingerprint.starts_with("SHA256:"));
        assert_eq!(key.key_type, SshKeyType::Ed25519);

        // List keys
        let keys = store.ssh_keys.list_for_user(1);
        assert_eq!(keys.len(), 1);
    }
}
