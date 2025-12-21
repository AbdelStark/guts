//! Hardware Security Module (HSM) integration.
//!
//! This module provides an abstraction layer for HSM operations,
//! allowing cryptographic operations to be performed in secure hardware.

use crate::error::{Result, SecurityError};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HSM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    /// HSM provider type.
    pub provider: HsmProviderType,
    /// Connection string or endpoint.
    pub endpoint: Option<String>,
    /// Slot or partition identifier.
    pub slot: Option<String>,
    /// PIN or authentication credential.
    #[serde(skip_serializing)]
    pub pin: Option<String>,
    /// Additional provider-specific options.
    #[serde(default)]
    pub options: HashMap<String, String>,
}

impl Default for HsmConfig {
    fn default() -> Self {
        Self {
            provider: HsmProviderType::Mock,
            endpoint: None,
            slot: None,
            pin: None,
            options: HashMap::new(),
        }
    }
}

/// Types of HSM providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HsmProviderType {
    /// Mock HSM for testing.
    Mock,
    /// PKCS#11 compatible HSM.
    Pkcs11,
    /// AWS CloudHSM.
    AwsCloudHsm,
    /// YubiHSM.
    YubiHsm,
    /// Azure Key Vault HSM.
    AzureKeyVault,
    /// Google Cloud HSM.
    GoogleCloudHsm,
}

/// Trait for HSM providers.
#[async_trait]
pub trait HsmProvider: Send + Sync + std::fmt::Debug {
    /// Generates a new key pair in the HSM.
    async fn generate_key(&self, key_id: &str) -> Result<Vec<u8>>;

    /// Signs a message using a key stored in the HSM.
    async fn sign(&self, key_id: &str, message: &[u8]) -> Result<Vec<u8>>;

    /// Verifies a signature using a key stored in the HSM.
    async fn verify(&self, key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool>;

    /// Retrieves the public key for a given key ID.
    async fn get_public_key(&self, key_id: &str) -> Result<Vec<u8>>;

    /// Deletes a key from the HSM.
    async fn delete_key(&self, key_id: &str) -> Result<()>;

    /// Lists all key IDs in the HSM.
    async fn list_keys(&self) -> Result<Vec<String>>;

    /// Checks if the HSM connection is healthy.
    async fn health_check(&self) -> Result<bool>;
}

/// Mock HSM provider for testing and development.
#[derive(Debug, Default)]
pub struct MockHsmProvider {
    /// Stored key pairs (key_id -> (private_key, public_key)).
    #[allow(clippy::type_complexity)]
    keys: RwLock<HashMap<String, (Vec<u8>, Vec<u8>)>>,
}

impl MockHsmProvider {
    /// Creates a new mock HSM provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a mock key pair.
    fn generate_mock_keypair() -> (Vec<u8>, Vec<u8>) {
        // In a real implementation, this would use proper Ed25519 key generation
        // For the mock, we just generate random-looking bytes
        let private = (0..32)
            .map(|i| ((i * 17 + 42) % 256) as u8)
            .collect::<Vec<_>>();
        let public = (0..32)
            .map(|i| ((i * 23 + 37) % 256) as u8)
            .collect::<Vec<_>>();
        (private, public)
    }

    /// Creates a mock signature.
    fn mock_sign(private_key: &[u8], message: &[u8]) -> Vec<u8> {
        // Mock signature is a simple XOR of key and message hash
        let mut sig = vec![0u8; 64];
        for (i, byte) in sig.iter_mut().enumerate() {
            let key_byte = private_key.get(i % 32).copied().unwrap_or(0);
            let msg_byte = message.get(i % message.len()).copied().unwrap_or(0);
            *byte = key_byte ^ msg_byte;
        }
        sig
    }
}

#[async_trait]
impl HsmProvider for MockHsmProvider {
    async fn generate_key(&self, key_id: &str) -> Result<Vec<u8>> {
        let (private, public) = Self::generate_mock_keypair();

        self.keys
            .write()
            .insert(key_id.to_string(), (private, public.clone()));

        tracing::debug!(key_id = %key_id, "mock HSM: key generated");
        Ok(public)
    }

    async fn sign(&self, key_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        let keys = self.keys.read();
        let (private, _) = keys
            .get(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        let signature = Self::mock_sign(private, message);
        tracing::debug!(key_id = %key_id, msg_len = message.len(), "mock HSM: signed");
        Ok(signature)
    }

    async fn verify(&self, key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        let keys = self.keys.read();
        let (private, _) = keys
            .get(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        // Recreate expected signature and compare
        let expected = Self::mock_sign(private, message);
        let valid = signature == expected.as_slice();

        tracing::debug!(key_id = %key_id, valid = valid, "mock HSM: verified");
        Ok(valid)
    }

    async fn get_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        let keys = self.keys.read();
        let (_, public) = keys
            .get(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        Ok(public.clone())
    }

    async fn delete_key(&self, key_id: &str) -> Result<()> {
        self.keys.write().remove(key_id);
        tracing::debug!(key_id = %key_id, "mock HSM: key deleted");
        Ok(())
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.keys.read().keys().cloned().collect())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// PKCS#11 HSM provider stub.
///
/// This is a placeholder implementation. A real implementation would use
/// a PKCS#11 library like `pkcs11` or `cryptoki`.
#[derive(Debug)]
pub struct Pkcs11HsmProvider {
    /// Configuration.
    config: HsmConfig,
    /// Connected status.
    connected: RwLock<bool>,
}

impl Pkcs11HsmProvider {
    /// Creates a new PKCS#11 HSM provider.
    pub fn new(config: HsmConfig) -> Self {
        Self {
            config,
            connected: RwLock::new(false),
        }
    }

    /// Connects to the HSM.
    pub async fn connect(&self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Load the PKCS#11 library
        // 2. Initialize the library
        // 3. Open a session to the specified slot
        // 4. Login with the PIN

        if self.config.endpoint.is_none() {
            return Err(SecurityError::Configuration(
                "PKCS#11 library path not configured".to_string(),
            ));
        }

        *self.connected.write() = true;
        tracing::info!(slot = ?self.config.slot, "PKCS#11 HSM connected");
        Ok(())
    }

    /// Disconnects from the HSM.
    pub async fn disconnect(&self) -> Result<()> {
        *self.connected.write() = false;
        tracing::info!("PKCS#11 HSM disconnected");
        Ok(())
    }

    fn check_connected(&self) -> Result<()> {
        if !*self.connected.read() {
            return Err(SecurityError::HsmNotConfigured);
        }
        Ok(())
    }
}

#[async_trait]
impl HsmProvider for Pkcs11HsmProvider {
    async fn generate_key(&self, key_id: &str) -> Result<Vec<u8>> {
        self.check_connected()?;

        // In a real implementation, this would use PKCS#11 to generate a key
        tracing::info!(key_id = %key_id, "PKCS#11: generating key (stub)");

        // Return placeholder public key
        Err(SecurityError::HsmError(
            "PKCS#11 provider not fully implemented".to_string(),
        ))
    }

    async fn sign(&self, key_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        self.check_connected()?;

        tracing::info!(key_id = %key_id, msg_len = message.len(), "PKCS#11: signing (stub)");

        Err(SecurityError::HsmError(
            "PKCS#11 provider not fully implemented".to_string(),
        ))
    }

    async fn verify(&self, key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        self.check_connected()?;

        tracing::info!(
            key_id = %key_id,
            msg_len = message.len(),
            sig_len = signature.len(),
            "PKCS#11: verifying (stub)"
        );

        Err(SecurityError::HsmError(
            "PKCS#11 provider not fully implemented".to_string(),
        ))
    }

    async fn get_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        self.check_connected()?;

        tracing::info!(key_id = %key_id, "PKCS#11: getting public key (stub)");

        Err(SecurityError::HsmError(
            "PKCS#11 provider not fully implemented".to_string(),
        ))
    }

    async fn delete_key(&self, key_id: &str) -> Result<()> {
        self.check_connected()?;

        tracing::info!(key_id = %key_id, "PKCS#11: deleting key (stub)");

        Err(SecurityError::HsmError(
            "PKCS#11 provider not fully implemented".to_string(),
        ))
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        self.check_connected()?;

        tracing::info!("PKCS#11: listing keys (stub)");

        Err(SecurityError::HsmError(
            "PKCS#11 provider not fully implemented".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(*self.connected.read())
    }
}

/// Creates an HSM provider based on configuration.
pub fn create_hsm_provider(config: &HsmConfig) -> Result<Box<dyn HsmProvider>> {
    match config.provider {
        HsmProviderType::Mock => Ok(Box::new(MockHsmProvider::new())),
        HsmProviderType::Pkcs11 => Ok(Box::new(Pkcs11HsmProvider::new(config.clone()))),
        HsmProviderType::AwsCloudHsm => Err(SecurityError::Configuration(
            "AWS CloudHSM provider not implemented".to_string(),
        )),
        HsmProviderType::YubiHsm => Err(SecurityError::Configuration(
            "YubiHSM provider not implemented".to_string(),
        )),
        HsmProviderType::AzureKeyVault => Err(SecurityError::Configuration(
            "Azure Key Vault HSM provider not implemented".to_string(),
        )),
        HsmProviderType::GoogleCloudHsm => Err(SecurityError::Configuration(
            "Google Cloud HSM provider not implemented".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_hsm_generate_key() {
        let hsm = MockHsmProvider::new();

        let public_key = hsm.generate_key("test-key").await.unwrap();
        assert_eq!(public_key.len(), 32);

        // Key should be listed
        let keys = hsm.list_keys().await.unwrap();
        assert!(keys.contains(&"test-key".to_string()));
    }

    #[tokio::test]
    async fn test_mock_hsm_sign_verify() {
        let hsm = MockHsmProvider::new();

        hsm.generate_key("signing-key").await.unwrap();

        let message = b"Hello, World!";
        let signature = hsm.sign("signing-key", message).await.unwrap();

        assert_eq!(signature.len(), 64);

        // Verify should succeed with correct message
        let valid = hsm
            .verify("signing-key", message, &signature)
            .await
            .unwrap();
        assert!(valid);

        // Verify should fail with wrong message
        let invalid = hsm
            .verify("signing-key", b"Wrong message", &signature)
            .await
            .unwrap();
        assert!(!invalid);
    }

    #[tokio::test]
    async fn test_mock_hsm_get_public_key() {
        let hsm = MockHsmProvider::new();

        let generated = hsm.generate_key("pk-test").await.unwrap();
        let retrieved = hsm.get_public_key("pk-test").await.unwrap();

        assert_eq!(generated, retrieved);
    }

    #[tokio::test]
    async fn test_mock_hsm_delete_key() {
        let hsm = MockHsmProvider::new();

        hsm.generate_key("to-delete").await.unwrap();
        assert!(hsm
            .list_keys()
            .await
            .unwrap()
            .contains(&"to-delete".to_string()));

        hsm.delete_key("to-delete").await.unwrap();
        assert!(!hsm
            .list_keys()
            .await
            .unwrap()
            .contains(&"to-delete".to_string()));
    }

    #[tokio::test]
    async fn test_mock_hsm_key_not_found() {
        let hsm = MockHsmProvider::new();

        let result = hsm.sign("nonexistent", b"message").await;
        assert!(matches!(result, Err(SecurityError::KeyNotFound(_))));
    }

    #[tokio::test]
    async fn test_mock_hsm_health_check() {
        let hsm = MockHsmProvider::new();
        assert!(hsm.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_pkcs11_hsm_not_connected() {
        let config = HsmConfig {
            provider: HsmProviderType::Pkcs11,
            endpoint: Some("/usr/lib/softhsm/libsofthsm2.so".to_string()),
            slot: Some("0".to_string()),
            pin: Some("1234".to_string()),
            options: HashMap::new(),
        };

        let hsm = Pkcs11HsmProvider::new(config);

        // Should fail because not connected
        let result = hsm.sign("key", b"message").await;
        assert!(matches!(result, Err(SecurityError::HsmNotConfigured)));
    }

    #[test]
    fn test_create_hsm_provider() {
        let mock_config = HsmConfig::default();
        let provider = create_hsm_provider(&mock_config);
        assert!(provider.is_ok());

        let unsupported = HsmConfig {
            provider: HsmProviderType::YubiHsm,
            ..Default::default()
        };
        let result = create_hsm_provider(&unsupported);
        assert!(result.is_err());
    }
}
