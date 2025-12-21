//! Storage for compatibility layer data.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::{CompatError, Result};
use crate::rate_limit::RateLimiter;
use crate::release::{AssetId, Release, ReleaseAsset, ReleaseId};
use crate::ssh_key::{SshKey, SshKeyId};
use crate::token::{PersonalAccessToken, TokenId, TokenScope, TokenValue};
use crate::user::{User, UserId};

/// Compatibility layer data store.
#[derive(Debug, Clone)]
pub struct CompatStore {
    /// User storage.
    pub users: UserStore,
    /// Token storage.
    pub tokens: TokenStore,
    /// SSH key storage.
    pub ssh_keys: SshKeyStore,
    /// Release storage.
    pub releases: ReleaseStore,
    /// Rate limiter.
    pub rate_limiter: RateLimiter,
}

impl Default for CompatStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CompatStore {
    /// Create a new compatibility store.
    pub fn new() -> Self {
        Self {
            users: UserStore::new(),
            tokens: TokenStore::new(),
            ssh_keys: SshKeyStore::new(),
            releases: ReleaseStore::new(),
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Get statistics about stored data.
    pub fn stats(&self) -> CompatStats {
        CompatStats {
            users: self.users.count(),
            tokens: self.tokens.count(),
            ssh_keys: self.ssh_keys.count(),
            releases: self.releases.count(),
        }
    }
}

/// User storage.
#[derive(Debug, Clone)]
pub struct UserStore {
    /// Users by ID.
    users: Arc<RwLock<HashMap<UserId, User>>>,
    /// Username to ID index.
    username_index: Arc<RwLock<HashMap<String, UserId>>>,
    /// Public key to ID index.
    pubkey_index: Arc<RwLock<HashMap<String, UserId>>>,
    /// Next user ID.
    next_id: Arc<AtomicU64>,
}

impl Default for UserStore {
    fn default() -> Self {
        Self::new()
    }
}

impl UserStore {
    /// Create a new user store.
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            username_index: Arc::new(RwLock::new(HashMap::new())),
            pubkey_index: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create a new user.
    pub fn create(&self, username: String, public_key: String) -> Result<User> {
        // Validate username
        User::validate_username(&username).map_err(CompatError::InvalidUsername)?;

        let mut users = self.users.write();
        let mut username_index = self.username_index.write();
        let mut pubkey_index = self.pubkey_index.write();

        // Check for duplicate username
        if username_index.contains_key(&username) {
            return Err(CompatError::UsernameExists(username));
        }

        // Check for duplicate public key
        if pubkey_index.contains_key(&public_key) {
            return Err(CompatError::UsernameExists(
                "public key already registered".to_string(),
            ));
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let user = User::new(id, username.clone(), public_key.clone());

        username_index.insert(username, id);
        pubkey_index.insert(public_key, id);
        users.insert(id, user.clone());

        Ok(user)
    }

    /// Get a user by ID.
    pub fn get(&self, id: UserId) -> Option<User> {
        self.users.read().get(&id).cloned()
    }

    /// Get a user by username.
    pub fn get_by_username(&self, username: &str) -> Option<User> {
        let username_index = self.username_index.read();
        let id = username_index.get(username)?;
        self.users.read().get(id).cloned()
    }

    /// Get a user by public key.
    pub fn get_by_public_key(&self, public_key: &str) -> Option<User> {
        let pubkey_index = self.pubkey_index.read();
        let id = pubkey_index.get(public_key)?;
        self.users.read().get(id).cloned()
    }

    /// Update a user.
    pub fn update(&self, user: User) -> Result<User> {
        let mut users = self.users.write();
        if !users.contains_key(&user.id) {
            return Err(CompatError::UserNotFound(user.id.to_string()));
        }
        users.insert(user.id, user.clone());
        Ok(user)
    }

    /// List all users.
    pub fn list(&self) -> Vec<User> {
        self.users.read().values().cloned().collect()
    }

    /// Count users.
    pub fn count(&self) -> usize {
        self.users.read().len()
    }
}

/// Token storage.
#[derive(Debug, Clone)]
pub struct TokenStore {
    /// Tokens by ID.
    tokens: Arc<RwLock<HashMap<TokenId, PersonalAccessToken>>>,
    /// Token prefix to ID index.
    prefix_index: Arc<RwLock<HashMap<String, TokenId>>>,
    /// Next token ID.
    next_id: Arc<AtomicU64>,
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenStore {
    /// Create a new token store.
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            prefix_index: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create a new token.
    ///
    /// Returns the token struct and the plaintext token (only shown once).
    pub fn create(
        &self,
        user_id: UserId,
        name: String,
        scopes: Vec<TokenScope>,
        expires_at: Option<u64>,
    ) -> Result<(PersonalAccessToken, String)> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (token, plaintext) =
            PersonalAccessToken::generate(id, user_id, name, scopes, expires_at)?;

        let mut tokens = self.tokens.write();
        let mut prefix_index = self.prefix_index.write();

        prefix_index.insert(token.token_prefix.clone(), id);
        tokens.insert(id, token.clone());

        Ok((token, plaintext))
    }

    /// Get a token by ID.
    pub fn get(&self, id: TokenId) -> Option<PersonalAccessToken> {
        self.tokens.read().get(&id).cloned()
    }

    /// Get a token by prefix.
    pub fn get_by_prefix(&self, prefix: &str) -> Option<PersonalAccessToken> {
        let prefix_index = self.prefix_index.read();
        let id = prefix_index.get(prefix)?;
        self.tokens.read().get(id).cloned()
    }

    /// Verify a token and return the user ID if valid.
    pub fn verify(&self, token_string: &str) -> Result<(UserId, Vec<TokenScope>)> {
        let token_value = TokenValue::parse(token_string)?;

        let prefix_index = self.prefix_index.read();
        let id = prefix_index
            .get(&token_value.prefix)
            .ok_or(CompatError::TokenNotFound)?;

        let mut tokens = self.tokens.write();
        let token = tokens.get_mut(id).ok_or(CompatError::TokenNotFound)?;

        // Verify the secret
        token.verify(&token_value.secret)?;

        // Check expiration
        if token.is_expired() {
            return Err(CompatError::TokenExpired);
        }

        // Update last used
        token.touch();

        Ok((token.user_id, token.scopes.clone()))
    }

    /// Revoke (delete) a token.
    pub fn revoke(&self, id: TokenId) -> Result<()> {
        let mut tokens = self.tokens.write();
        let mut prefix_index = self.prefix_index.write();

        let token = tokens.remove(&id).ok_or(CompatError::TokenNotFound)?;
        prefix_index.remove(&token.token_prefix);

        Ok(())
    }

    /// List tokens for a user (without secrets).
    pub fn list_for_user(&self, user_id: UserId) -> Vec<PersonalAccessToken> {
        self.tokens
            .read()
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Count tokens.
    pub fn count(&self) -> usize {
        self.tokens.read().len()
    }
}

/// SSH key storage.
#[derive(Debug, Clone)]
pub struct SshKeyStore {
    /// Keys by ID.
    keys: Arc<RwLock<HashMap<SshKeyId, SshKey>>>,
    /// Fingerprint to ID index.
    fingerprint_index: Arc<RwLock<HashMap<String, SshKeyId>>>,
    /// Next key ID.
    next_id: Arc<AtomicU64>,
}

impl Default for SshKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SshKeyStore {
    /// Create a new SSH key store.
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            fingerprint_index: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Add an SSH key.
    pub fn add(&self, user_id: UserId, title: String, public_key: String) -> Result<SshKey> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let key = SshKey::new(id, user_id, title, public_key)?;

        let mut keys = self.keys.write();
        let mut fingerprint_index = self.fingerprint_index.write();

        // Check for duplicate fingerprint
        if fingerprint_index.contains_key(&key.fingerprint) {
            return Err(CompatError::SshKeyExists(key.fingerprint));
        }

        fingerprint_index.insert(key.fingerprint.clone(), id);
        keys.insert(id, key.clone());

        Ok(key)
    }

    /// Get an SSH key by ID.
    pub fn get(&self, id: SshKeyId) -> Option<SshKey> {
        self.keys.read().get(&id).cloned()
    }

    /// Get an SSH key by fingerprint.
    pub fn get_by_fingerprint(&self, fingerprint: &str) -> Option<SshKey> {
        let fingerprint_index = self.fingerprint_index.read();
        let id = fingerprint_index.get(fingerprint)?;
        self.keys.read().get(id).cloned()
    }

    /// Remove an SSH key.
    pub fn remove(&self, id: SshKeyId) -> Result<SshKey> {
        let mut keys = self.keys.write();
        let mut fingerprint_index = self.fingerprint_index.write();

        let key = keys.remove(&id).ok_or(CompatError::SshKeyNotFound)?;
        fingerprint_index.remove(&key.fingerprint);

        Ok(key)
    }

    /// List SSH keys for a user.
    pub fn list_for_user(&self, user_id: UserId) -> Vec<SshKey> {
        self.keys
            .read()
            .values()
            .filter(|k| k.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Count SSH keys.
    pub fn count(&self) -> usize {
        self.keys.read().len()
    }
}

/// Release storage.
#[derive(Debug, Clone)]
pub struct ReleaseStore {
    /// Releases by ID.
    releases: Arc<RwLock<HashMap<ReleaseId, Release>>>,
    /// (repo_key, tag_name) to ID index.
    tag_index: Arc<RwLock<HashMap<(String, String), ReleaseId>>>,
    /// Asset content storage (hash -> bytes).
    asset_content: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// Next release ID.
    next_release_id: Arc<AtomicU64>,
    /// Next asset ID.
    next_asset_id: Arc<AtomicU64>,
}

impl Default for ReleaseStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReleaseStore {
    /// Create a new release store.
    pub fn new() -> Self {
        Self {
            releases: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            asset_content: Arc::new(RwLock::new(HashMap::new())),
            next_release_id: Arc::new(AtomicU64::new(1)),
            next_asset_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create a new release.
    pub fn create(
        &self,
        repo_key: String,
        tag_name: String,
        target_commitish: String,
        author: String,
    ) -> Result<Release> {
        let mut releases = self.releases.write();
        let mut tag_index = self.tag_index.write();

        // Check for duplicate tag
        let key = (repo_key.clone(), tag_name.clone());
        if tag_index.contains_key(&key) {
            return Err(CompatError::ReleaseExists(tag_name));
        }

        let id = self.next_release_id.fetch_add(1, Ordering::SeqCst);
        let release = Release::new(id, repo_key, tag_name, target_commitish, author);

        tag_index.insert(key, id);
        releases.insert(id, release.clone());

        Ok(release)
    }

    /// Get a release by ID.
    pub fn get(&self, id: ReleaseId) -> Option<Release> {
        self.releases.read().get(&id).cloned()
    }

    /// Get a release by tag.
    pub fn get_by_tag(&self, repo_key: &str, tag_name: &str) -> Option<Release> {
        let tag_index = self.tag_index.read();
        let id = tag_index.get(&(repo_key.to_string(), tag_name.to_string()))?;
        self.releases.read().get(id).cloned()
    }

    /// Get the latest release for a repository.
    pub fn get_latest(&self, repo_key: &str) -> Option<Release> {
        self.releases
            .read()
            .values()
            .filter(|r| r.repo_key == repo_key && r.is_publishable())
            .max_by_key(|r| r.published_at)
            .cloned()
    }

    /// Update a release.
    pub fn update(&self, release: Release) -> Result<Release> {
        let mut releases = self.releases.write();
        if !releases.contains_key(&release.id) {
            return Err(CompatError::ReleaseNotFound(release.id.to_string()));
        }
        releases.insert(release.id, release.clone());
        Ok(release)
    }

    /// Delete a release.
    pub fn delete(&self, id: ReleaseId) -> Result<Release> {
        let mut releases = self.releases.write();
        let mut tag_index = self.tag_index.write();

        let release = releases
            .remove(&id)
            .ok_or_else(|| CompatError::ReleaseNotFound(id.to_string()))?;
        tag_index.remove(&(release.repo_key.clone(), release.tag_name.clone()));

        Ok(release)
    }

    /// List releases for a repository.
    pub fn list(&self, repo_key: &str) -> Vec<Release> {
        let mut releases: Vec<_> = self
            .releases
            .read()
            .values()
            .filter(|r| r.repo_key == repo_key)
            .cloned()
            .collect();
        releases.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        releases
    }

    /// Add an asset to a release.
    pub fn add_asset(
        &self,
        release_id: ReleaseId,
        name: String,
        content_type: String,
        content: Vec<u8>,
        uploader: String,
    ) -> Result<ReleaseAsset> {
        let mut releases = self.releases.write();
        let mut asset_content = self.asset_content.write();

        let release = releases
            .get_mut(&release_id)
            .ok_or_else(|| CompatError::ReleaseNotFound(release_id.to_string()))?;

        // Check for duplicate asset name
        if release.assets.iter().any(|a| a.name == name) {
            return Err(CompatError::AssetExists(name));
        }

        // Calculate hash
        use sha2::{Digest, Sha256};
        let hash = hex::encode(Sha256::digest(&content));

        let id = self.next_asset_id.fetch_add(1, Ordering::SeqCst);
        let asset = ReleaseAsset::new(
            id,
            release_id,
            name,
            content_type,
            content.len() as u64,
            hash.clone(),
            uploader,
        );

        // Store content
        asset_content.insert(hash, content);

        // Add to release
        release.add_asset(asset.clone());

        Ok(asset)
    }

    /// Get asset content.
    pub fn get_asset_content(&self, content_hash: &str) -> Option<Vec<u8>> {
        self.asset_content.read().get(content_hash).cloned()
    }

    /// Delete an asset.
    pub fn delete_asset(&self, release_id: ReleaseId, asset_id: AssetId) -> Result<ReleaseAsset> {
        let mut releases = self.releases.write();
        let mut asset_content = self.asset_content.write();

        let release = releases
            .get_mut(&release_id)
            .ok_or_else(|| CompatError::ReleaseNotFound(release_id.to_string()))?;

        let asset = release
            .remove_asset(asset_id)
            .ok_or_else(|| CompatError::AssetNotFound(asset_id.to_string()))?;

        // Remove content
        asset_content.remove(&asset.content_hash);

        Ok(asset)
    }

    /// Count releases.
    pub fn count(&self) -> usize {
        self.releases.read().len()
    }
}

/// Statistics about stored data.
#[derive(Debug, Clone, Serialize)]
pub struct CompatStats {
    /// Number of users.
    pub users: usize,
    /// Number of tokens.
    pub tokens: usize,
    /// Number of SSH keys.
    pub ssh_keys: usize,
    /// Number of releases.
    pub releases: usize,
}

use serde::Serialize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_store() {
        let store = UserStore::new();

        // Create a user
        let user = store
            .create("alice".to_string(), "pubkey123".to_string())
            .unwrap();
        assert_eq!(user.username, "alice");

        // Get by ID
        let found = store.get(user.id).unwrap();
        assert_eq!(found.username, "alice");

        // Get by username
        let found = store.get_by_username("alice").unwrap();
        assert_eq!(found.id, user.id);

        // Duplicate username should fail
        let result = store.create("alice".to_string(), "pubkey456".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_token_store() {
        let store = TokenStore::new();

        // Create a token
        let (token, plaintext) = store
            .create(
                1,
                "test token".to_string(),
                vec![TokenScope::RepoRead],
                None,
            )
            .unwrap();

        assert!(plaintext.starts_with("guts_"));

        // Verify the token
        let (user_id, scopes) = store.verify(&plaintext).unwrap();
        assert_eq!(user_id, 1);
        assert!(scopes.contains(&TokenScope::RepoRead));

        // Revoke the token
        store.revoke(token.id).unwrap();

        // Verification should fail
        let result = store.verify(&plaintext);
        assert!(result.is_err());
    }

    #[test]
    fn test_ssh_key_store() {
        let store = SshKeyStore::new();

        // Add a key
        let key = store
            .add(
                1,
                "My Key".to_string(),
                "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl test@example.com".to_string(),
            )
            .unwrap();

        // Get by ID
        let found = store.get(key.id).unwrap();
        assert_eq!(found.title, "My Key");

        // List for user
        let keys = store.list_for_user(1);
        assert_eq!(keys.len(), 1);

        // Remove
        store.remove(key.id).unwrap();
        assert!(store.get(key.id).is_none());
    }

    #[test]
    fn test_release_store() {
        let store = ReleaseStore::new();

        // Create a release
        let release = store
            .create(
                "alice/repo".to_string(),
                "v1.0.0".to_string(),
                "main".to_string(),
                "alice".to_string(),
            )
            .unwrap();

        assert_eq!(release.tag_name, "v1.0.0");

        // Get by ID
        let found = store.get(release.id).unwrap();
        assert_eq!(found.tag_name, "v1.0.0");

        // Get by tag
        let found = store.get_by_tag("alice/repo", "v1.0.0").unwrap();
        assert_eq!(found.id, release.id);

        // Get latest
        let latest = store.get_latest("alice/repo").unwrap();
        assert_eq!(latest.id, release.id);

        // Add asset
        let asset = store
            .add_asset(
                release.id,
                "app.tar.gz".to_string(),
                "application/gzip".to_string(),
                b"test content".to_vec(),
                "alice".to_string(),
            )
            .unwrap();

        assert_eq!(asset.name, "app.tar.gz");

        // Get asset content
        let content = store.get_asset_content(&asset.content_hash).unwrap();
        assert_eq!(content, b"test content");

        // Delete release
        store.delete(release.id).unwrap();
        assert!(store.get(release.id).is_none());
    }
}
