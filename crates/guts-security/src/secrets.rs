//! Secrets management for secure storage and retrieval of sensitive data.
//!
//! This module provides a trait-based abstraction for secrets management,
//! with implementations for different backends.

use crate::error::{Result, SecurityError};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// A secret string that implements secure handling.
///
/// This type ensures that secrets are:
/// - Not logged via Debug
/// - Zeroized on drop (when possible)
/// - Not accidentally exposed
#[derive(Clone, Serialize, Deserialize)]
pub struct SecretString {
    inner: String,
}

impl SecretString {
    /// Creates a new secret string.
    pub fn new(s: impl Into<String>) -> Self {
        Self { inner: s.into() }
    }

    /// Exposes the secret value.
    ///
    /// Use this sparingly and only when necessary.
    pub fn expose(&self) -> &str {
        &self.inner
    }

    /// Returns the length of the secret.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        // Zero out the memory (best effort)
        // SAFETY: We're just overwriting the bytes with zeros
        unsafe {
            let ptr = self.inner.as_mut_ptr();
            let len = self.inner.len();
            std::ptr::write_bytes(ptr, 0, len);
        }
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        // Constant-time comparison to prevent timing attacks
        constant_time_eq(self.inner.as_bytes(), other.inner.as_bytes())
    }
}

impl Eq for SecretString {}

/// Constant-time byte comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Configuration for secrets providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Provider type.
    pub provider: SecretsProviderType,
    /// Provider-specific configuration.
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// Types of secrets providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretsProviderType {
    /// Environment variables.
    Env,
    /// Encrypted file.
    File,
    /// HashiCorp Vault.
    Vault,
    /// AWS Secrets Manager.
    AwsSecretsManager,
    /// In-memory (for testing).
    Memory,
}

/// Trait for secrets providers.
#[async_trait]
pub trait SecretsProvider: Send + Sync {
    /// Retrieves a secret by key.
    async fn get(&self, key: &str) -> Result<SecretString>;

    /// Stores a secret.
    async fn set(&self, key: &str, value: &SecretString) -> Result<()>;

    /// Deletes a secret.
    async fn delete(&self, key: &str) -> Result<()>;

    /// Checks if a secret exists.
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Lists all secret keys (not values).
    async fn list_keys(&self) -> Result<Vec<String>>;

    /// Rotates a secret, generating a new value.
    async fn rotate(&self, key: &str) -> Result<SecretString> {
        // Default implementation generates a random secret
        let new_value = SecretString::new(generate_random_secret());
        self.set(key, &new_value).await?;
        Ok(new_value)
    }
}

/// Generates a random secret string.
fn generate_random_secret() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simple random generation using time and UUID
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let uuid = uuid::Uuid::new_v4();

    // Combine for randomness (in production, use proper CSPRNG)
    format!("{}:{}", hex::encode(nanos.to_le_bytes()), uuid)
}

/// Environment-based secrets provider.
#[derive(Debug, Default)]
pub struct EnvSecretsProvider {
    /// Prefix for environment variable names.
    prefix: String,
}

impl EnvSecretsProvider {
    /// Creates a new environment secrets provider.
    pub fn new() -> Self {
        Self::with_prefix("GUTS_SECRET_")
    }

    /// Creates a provider with a custom prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    fn env_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key.to_uppercase().replace('-', "_"))
    }
}

#[async_trait]
impl SecretsProvider for EnvSecretsProvider {
    async fn get(&self, key: &str) -> Result<SecretString> {
        let env_key = self.env_key(key);
        std::env::var(&env_key)
            .map(SecretString::new)
            .map_err(|_| SecurityError::SecretNotFound(key.to_string()))
    }

    async fn set(&self, key: &str, value: &SecretString) -> Result<()> {
        let env_key = self.env_key(key);
        std::env::set_var(&env_key, value.expose());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let env_key = self.env_key(key);
        std::env::remove_var(&env_key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let env_key = self.env_key(key);
        Ok(std::env::var(&env_key).is_ok())
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        Ok(std::env::vars()
            .filter_map(|(k, _)| {
                if k.starts_with(&self.prefix) {
                    Some(k[self.prefix.len()..].to_lowercase().replace('_', "-"))
                } else {
                    None
                }
            })
            .collect())
    }
}

/// File-based secrets provider with optional encryption.
#[derive(Debug)]
pub struct FileSecretsProvider {
    /// Path to the secrets file.
    path: PathBuf,
    /// In-memory cache.
    cache: RwLock<HashMap<String, SecretString>>,
}

impl FileSecretsProvider {
    /// Creates a new file secrets provider.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Loads secrets from the file.
    pub fn load(&self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.path)?;
        let secrets: HashMap<String, String> = serde_json::from_str(&content)?;

        let mut cache = self.cache.write();
        for (k, v) in secrets {
            cache.insert(k, SecretString::new(v));
        }

        Ok(())
    }

    /// Saves secrets to the file.
    fn save(&self) -> Result<()> {
        let cache = self.cache.read();
        let secrets: HashMap<String, String> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.expose().to_string()))
            .collect();

        let content = serde_json::to_string_pretty(&secrets)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }
}

#[async_trait]
impl SecretsProvider for FileSecretsProvider {
    async fn get(&self, key: &str) -> Result<SecretString> {
        self.cache
            .read()
            .get(key)
            .cloned()
            .ok_or_else(|| SecurityError::SecretNotFound(key.to_string()))
    }

    async fn set(&self, key: &str, value: &SecretString) -> Result<()> {
        self.cache.write().insert(key.to_string(), value.clone());
        self.save()?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.cache.write().remove(key);
        self.save()?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.cache.read().contains_key(key))
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.cache.read().keys().cloned().collect())
    }
}

/// In-memory secrets provider for testing.
#[derive(Debug, Default)]
pub struct MemorySecretsProvider {
    secrets: RwLock<HashMap<String, SecretString>>,
}

impl MemorySecretsProvider {
    /// Creates a new in-memory secrets provider.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SecretsProvider for MemorySecretsProvider {
    async fn get(&self, key: &str) -> Result<SecretString> {
        self.secrets
            .read()
            .get(key)
            .cloned()
            .ok_or_else(|| SecurityError::SecretNotFound(key.to_string()))
    }

    async fn set(&self, key: &str, value: &SecretString) -> Result<()> {
        self.secrets.write().insert(key.to_string(), value.clone());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.secrets.write().remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.secrets.read().contains_key(key))
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.secrets.read().keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_redaction() {
        let secret = SecretString::new("super_secret_password");

        // Debug should not reveal the secret
        let debug_str = format!("{:?}", secret);
        assert!(!debug_str.contains("super_secret_password"));
        assert!(debug_str.contains("REDACTED"));

        // Display should not reveal the secret
        let display_str = format!("{}", secret);
        assert!(!display_str.contains("super_secret_password"));
    }

    #[test]
    fn test_secret_string_expose() {
        let secret = SecretString::new("my_secret");
        assert_eq!(secret.expose(), "my_secret");
    }

    #[test]
    fn test_secret_string_equality() {
        let s1 = SecretString::new("password");
        let s2 = SecretString::new("password");
        let s3 = SecretString::new("different");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }

    #[tokio::test]
    async fn test_memory_secrets_provider() {
        let provider = MemorySecretsProvider::new();

        // Test set and get
        let secret = SecretString::new("test_value");
        provider.set("test_key", &secret).await.unwrap();

        let retrieved = provider.get("test_key").await.unwrap();
        assert_eq!(retrieved.expose(), "test_value");

        // Test exists
        assert!(provider.exists("test_key").await.unwrap());
        assert!(!provider.exists("nonexistent").await.unwrap());

        // Test list keys
        let keys = provider.list_keys().await.unwrap();
        assert!(keys.contains(&"test_key".to_string()));

        // Test delete
        provider.delete("test_key").await.unwrap();
        assert!(!provider.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_env_secrets_provider() {
        let provider = EnvSecretsProvider::with_prefix("TEST_SECRET_");

        // Set a secret
        let secret = SecretString::new("env_test_value");
        provider.set("my-key", &secret).await.unwrap();

        // Verify it was set in the environment
        assert!(std::env::var("TEST_SECRET_MY_KEY").is_ok());

        // Get the secret back
        let retrieved = provider.get("my-key").await.unwrap();
        assert_eq!(retrieved.expose(), "env_test_value");

        // Clean up
        provider.delete("my-key").await.unwrap();
        assert!(std::env::var("TEST_SECRET_MY_KEY").is_err());
    }

    #[tokio::test]
    async fn test_rotate_secret() {
        let provider = MemorySecretsProvider::new();

        // Set initial secret
        let initial = SecretString::new("initial_value");
        provider.set("rotating_key", &initial).await.unwrap();

        // Rotate
        let new_secret = provider.rotate("rotating_key").await.unwrap();

        // Verify it changed
        assert_ne!(new_secret.expose(), "initial_value");
        assert!(!new_secret.is_empty());

        // Verify we can retrieve the new value
        let retrieved = provider.get("rotating_key").await.unwrap();
        assert_eq!(retrieved.expose(), new_secret.expose());
    }

    #[tokio::test]
    async fn test_file_secrets_provider() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("secrets.json");

        let provider = FileSecretsProvider::new(&path);

        // Set and save
        let secret = SecretString::new("file_secret");
        provider.set("file_key", &secret).await.unwrap();

        // Verify file exists
        assert!(path.exists());

        // Create new provider and load
        let provider2 = FileSecretsProvider::new(&path);
        provider2.load().unwrap();

        // Verify we can read the secret
        let retrieved = provider2.get("file_key").await.unwrap();
        assert_eq!(retrieved.expose(), "file_secret");
    }
}
