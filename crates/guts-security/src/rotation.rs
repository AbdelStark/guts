//! Key rotation management for cryptographic keys.
//!
//! This module provides infrastructure for managing key lifecycles,
//! including rotation policies, overlap periods, and key state tracking.

use crate::error::{Result, SecurityError};
use crate::hsm::HsmProvider;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Key rotation policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    /// Maximum age of a key before rotation (in seconds).
    pub max_age_secs: u64,
    /// Overlap period during which old key remains valid (in seconds).
    pub overlap_period_secs: u64,
    /// Warning period before key expiry (in seconds).
    pub warn_before_secs: u64,
    /// Whether to auto-rotate expired keys.
    pub auto_rotate: bool,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self {
            // 90 days
            max_age_secs: 90 * 24 * 60 * 60,
            // 7 days overlap
            overlap_period_secs: 7 * 24 * 60 * 60,
            // Warn 14 days before
            warn_before_secs: 14 * 24 * 60 * 60,
            auto_rotate: true,
        }
    }
}

impl KeyRotationPolicy {
    /// Creates a new policy with custom max age.
    pub fn with_max_age(max_age: Duration) -> Self {
        Self {
            max_age_secs: max_age.as_secs(),
            ..Default::default()
        }
    }

    /// Returns the maximum age as a Duration.
    pub fn max_age(&self) -> Duration {
        Duration::from_secs(self.max_age_secs)
    }

    /// Returns the overlap period as a Duration.
    pub fn overlap_period(&self) -> Duration {
        Duration::from_secs(self.overlap_period_secs)
    }

    /// Returns the warning threshold as a Duration.
    pub fn warn_before(&self) -> Duration {
        Duration::from_secs(self.warn_before_secs)
    }
}

/// State of a managed key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyState {
    /// Key is active and primary.
    Active,
    /// Key is being rotated (overlap period).
    Rotating,
    /// Key is deprecated but still valid for verification.
    Deprecated,
    /// Key is revoked and should not be used.
    Revoked,
    /// Key has expired.
    Expired,
}

impl KeyState {
    /// Returns whether this key can be used for signing.
    pub fn can_sign(&self) -> bool {
        matches!(self, KeyState::Active)
    }

    /// Returns whether this key can be used for verification.
    pub fn can_verify(&self) -> bool {
        matches!(
            self,
            KeyState::Active | KeyState::Rotating | KeyState::Deprecated
        )
    }
}

/// Metadata for a managed key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique key identifier.
    pub key_id: String,
    /// Current state of the key.
    pub state: KeyState,
    /// Unix timestamp when the key was created.
    pub created_at: u64,
    /// Unix timestamp when the key was last rotated.
    pub rotated_at: Option<u64>,
    /// Unix timestamp when the key expires.
    pub expires_at: u64,
    /// Public key (hex-encoded).
    pub public_key: String,
    /// Key algorithm.
    pub algorithm: String,
    /// Whether this key is stored in HSM.
    pub hsm_backed: bool,
}

impl KeyMetadata {
    /// Returns whether the key is expired.
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        now >= self.expires_at
    }

    /// Returns the remaining time until expiry.
    pub fn time_until_expiry(&self) -> Option<Duration> {
        let now = current_timestamp();
        if now >= self.expires_at {
            None
        } else {
            Some(Duration::from_secs(self.expires_at - now))
        }
    }

    /// Returns whether the key should be rotated soon.
    pub fn should_warn(&self, policy: &KeyRotationPolicy) -> bool {
        if let Some(remaining) = self.time_until_expiry() {
            remaining <= policy.warn_before()
        } else {
            true
        }
    }
}

/// Event emitted during key rotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationEvent {
    /// Type of rotation event.
    pub event_type: RotationEventType,
    /// Key ID affected.
    pub key_id: String,
    /// Unix timestamp of the event.
    pub timestamp: u64,
    /// Additional details.
    pub details: Option<String>,
}

/// Types of rotation events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationEventType {
    /// New key generated.
    KeyGenerated,
    /// Rotation started.
    RotationStarted,
    /// Rotation completed.
    RotationCompleted,
    /// Key deprecated.
    KeyDeprecated,
    /// Key revoked.
    KeyRevoked,
    /// Key expired.
    KeyExpired,
    /// Warning about upcoming expiry.
    ExpiryWarning,
}

/// Key manager for handling key lifecycle and rotation.
pub struct KeyManager {
    /// Active keys by ID.
    keys: RwLock<HashMap<String, KeyMetadata>>,
    /// Rotation policy.
    policy: KeyRotationPolicy,
    /// Optional HSM provider.
    hsm: Option<Arc<dyn HsmProvider>>,
    /// Rotation event history.
    events: RwLock<Vec<RotationEvent>>,
}

impl KeyManager {
    /// Creates a new key manager with the given policy.
    pub fn new(policy: KeyRotationPolicy) -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            policy,
            hsm: None,
            events: RwLock::new(Vec::new()),
        }
    }

    /// Creates a key manager with HSM support.
    pub fn with_hsm(policy: KeyRotationPolicy, hsm: Arc<dyn HsmProvider>) -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            policy,
            hsm: Some(hsm),
            events: RwLock::new(Vec::new()),
        }
    }

    /// Registers a new key with the manager.
    pub fn register_key(&self, key_id: &str, public_key: &str, hsm_backed: bool) -> KeyMetadata {
        let now = current_timestamp();
        let expires_at = now + self.policy.max_age_secs;

        let metadata = KeyMetadata {
            key_id: key_id.to_string(),
            state: KeyState::Active,
            created_at: now,
            rotated_at: None,
            expires_at,
            public_key: public_key.to_string(),
            algorithm: "Ed25519".to_string(),
            hsm_backed,
        };

        self.keys
            .write()
            .insert(key_id.to_string(), metadata.clone());
        self.emit_event(RotationEventType::KeyGenerated, key_id, None);

        metadata
    }

    /// Gets key metadata by ID.
    pub fn get_key(&self, key_id: &str) -> Result<KeyMetadata> {
        self.keys
            .read()
            .get(key_id)
            .cloned()
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))
    }

    /// Gets the current active signing key.
    pub fn get_active_key(&self) -> Result<KeyMetadata> {
        self.keys
            .read()
            .values()
            .find(|k| k.state == KeyState::Active)
            .cloned()
            .ok_or_else(|| SecurityError::KeyNotFound("no active key".to_string()))
    }

    /// Gets all keys that can be used for verification.
    pub fn get_verification_keys(&self) -> Vec<KeyMetadata> {
        self.keys
            .read()
            .values()
            .filter(|k| k.state.can_verify())
            .cloned()
            .collect()
    }

    /// Starts key rotation, creating a new key and deprecating the old one.
    pub async fn rotate_key(&self, key_id: &str) -> Result<KeyMetadata> {
        let old_key = self.get_key(key_id)?;

        if old_key.state == KeyState::Revoked {
            return Err(SecurityError::RotationFailed(
                "cannot rotate revoked key".to_string(),
            ));
        }

        // Mark old key as rotating
        {
            let mut keys = self.keys.write();
            if let Some(key) = keys.get_mut(key_id) {
                key.state = KeyState::Rotating;
                key.rotated_at = Some(current_timestamp());
            }
        }

        self.emit_event(RotationEventType::RotationStarted, key_id, None);

        // Generate new key
        let new_key_id = format!("{}-{}", key_id.split('-').next().unwrap_or(key_id), uuid());
        let new_public_key = if let Some(ref hsm) = self.hsm {
            let pk = hsm.generate_key(&new_key_id).await?;
            hex::encode(pk)
        } else {
            // In non-HSM mode, we'd generate locally
            // For now, return a placeholder
            format!("pk_{}", uuid())
        };

        let new_metadata = self.register_key(&new_key_id, &new_public_key, self.hsm.is_some());

        self.emit_event(
            RotationEventType::RotationCompleted,
            &new_key_id,
            Some(format!("rotated from {}", key_id)),
        );

        // Schedule deprecation of old key after overlap period
        // In production, this would be handled by a background task
        tracing::info!(
            old_key = %key_id,
            new_key = %new_key_id,
            overlap_secs = self.policy.overlap_period_secs,
            "key rotation completed"
        );

        Ok(new_metadata)
    }

    /// Deprecates a key, marking it as no longer valid for signing.
    pub fn deprecate_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.write();
        let key = keys
            .get_mut(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        if key.state == KeyState::Revoked {
            return Err(SecurityError::RotationFailed(
                "cannot deprecate revoked key".to_string(),
            ));
        }

        key.state = KeyState::Deprecated;
        drop(keys);

        self.emit_event(RotationEventType::KeyDeprecated, key_id, None);
        Ok(())
    }

    /// Revokes a key, marking it as completely invalid.
    pub fn revoke_key(&self, key_id: &str, reason: Option<&str>) -> Result<()> {
        let mut keys = self.keys.write();
        let key = keys
            .get_mut(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        key.state = KeyState::Revoked;
        drop(keys);

        self.emit_event(
            RotationEventType::KeyRevoked,
            key_id,
            reason.map(String::from),
        );

        tracing::warn!(key_id = %key_id, reason = ?reason, "key revoked");
        Ok(())
    }

    /// Checks all keys and updates expired ones.
    pub fn check_expirations(&self) -> Vec<RotationEvent> {
        let mut events = Vec::new();
        let mut keys = self.keys.write();

        for key in keys.values_mut() {
            if key.state == KeyState::Revoked {
                continue;
            }

            if key.is_expired() && key.state != KeyState::Expired {
                key.state = KeyState::Expired;
                events.push(RotationEvent {
                    event_type: RotationEventType::KeyExpired,
                    key_id: key.key_id.clone(),
                    timestamp: current_timestamp(),
                    details: None,
                });
            } else if key.should_warn(&self.policy) && key.state == KeyState::Active {
                events.push(RotationEvent {
                    event_type: RotationEventType::ExpiryWarning,
                    key_id: key.key_id.clone(),
                    timestamp: current_timestamp(),
                    details: key
                        .time_until_expiry()
                        .map(|d| format!("expires in {} seconds", d.as_secs())),
                });
            }
        }

        drop(keys);

        // Record events
        for event in &events {
            self.events.write().push(event.clone());
        }

        events
    }

    /// Gets recent rotation events.
    pub fn get_events(&self, limit: usize) -> Vec<RotationEvent> {
        let events = self.events.read();
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Emits a rotation event.
    fn emit_event(&self, event_type: RotationEventType, key_id: &str, details: Option<String>) {
        let event = RotationEvent {
            event_type,
            key_id: key_id.to_string(),
            timestamp: current_timestamp(),
            details,
        };

        self.events.write().push(event.clone());

        tracing::debug!(
            event_type = ?event.event_type,
            key_id = %key_id,
            "rotation event"
        );
    }

    /// Returns the current policy.
    pub fn policy(&self) -> &KeyRotationPolicy {
        &self.policy
    }

    /// Returns the number of managed keys.
    pub fn key_count(&self) -> usize {
        self.keys.read().len()
    }
}

/// Gets the current Unix timestamp.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generates a short UUID.
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = KeyRotationPolicy::default();

        assert_eq!(policy.max_age_secs, 90 * 24 * 60 * 60);
        assert_eq!(policy.overlap_period_secs, 7 * 24 * 60 * 60);
        assert!(policy.auto_rotate);
    }

    #[test]
    fn test_key_state_permissions() {
        assert!(KeyState::Active.can_sign());
        assert!(KeyState::Active.can_verify());

        assert!(!KeyState::Rotating.can_sign());
        assert!(KeyState::Rotating.can_verify());

        assert!(!KeyState::Deprecated.can_sign());
        assert!(KeyState::Deprecated.can_verify());

        assert!(!KeyState::Revoked.can_sign());
        assert!(!KeyState::Revoked.can_verify());

        assert!(!KeyState::Expired.can_sign());
        assert!(!KeyState::Expired.can_verify());
    }

    #[test]
    fn test_register_key() {
        let policy = KeyRotationPolicy::default();
        let manager = KeyManager::new(policy);

        let metadata = manager.register_key("test-key-1", "public_key_hex", false);

        assert_eq!(metadata.key_id, "test-key-1");
        assert_eq!(metadata.state, KeyState::Active);
        assert!(!metadata.hsm_backed);
        assert!(metadata.expires_at > metadata.created_at);
    }

    #[test]
    fn test_get_active_key() {
        let policy = KeyRotationPolicy::default();
        let manager = KeyManager::new(policy);

        manager.register_key("key-1", "pk1", false);

        let active = manager.get_active_key().unwrap();
        assert_eq!(active.key_id, "key-1");
        assert_eq!(active.state, KeyState::Active);
    }

    #[test]
    fn test_deprecate_key() {
        let policy = KeyRotationPolicy::default();
        let manager = KeyManager::new(policy);

        manager.register_key("key-1", "pk1", false);
        manager.deprecate_key("key-1").unwrap();

        let key = manager.get_key("key-1").unwrap();
        assert_eq!(key.state, KeyState::Deprecated);
        assert!(!key.state.can_sign());
        assert!(key.state.can_verify());
    }

    #[test]
    fn test_revoke_key() {
        let policy = KeyRotationPolicy::default();
        let manager = KeyManager::new(policy);

        manager.register_key("key-1", "pk1", false);
        manager.revoke_key("key-1", Some("compromised")).unwrap();

        let key = manager.get_key("key-1").unwrap();
        assert_eq!(key.state, KeyState::Revoked);
        assert!(!key.state.can_sign());
        assert!(!key.state.can_verify());
    }

    #[test]
    fn test_verification_keys() {
        let policy = KeyRotationPolicy::default();
        let manager = KeyManager::new(policy);

        manager.register_key("key-1", "pk1", false);
        manager.register_key("key-2", "pk2", false);
        manager.deprecate_key("key-1").unwrap();

        let verify_keys = manager.get_verification_keys();
        assert_eq!(verify_keys.len(), 2);
    }

    #[test]
    fn test_key_expiry_check() {
        // Use a very short expiry for testing
        let policy = KeyRotationPolicy {
            max_age_secs: 1,
            overlap_period_secs: 0,
            warn_before_secs: 2,
            auto_rotate: false,
        };
        let manager = KeyManager::new(policy);

        manager.register_key("key-1", "pk1", false);

        // Key should be close to expiry
        let key = manager.get_key("key-1").unwrap();
        assert!(key.should_warn(&manager.policy));
    }

    #[test]
    fn test_rotation_events() {
        let policy = KeyRotationPolicy::default();
        let manager = KeyManager::new(policy);

        manager.register_key("key-1", "pk1", false);
        manager.deprecate_key("key-1").unwrap();

        let events = manager.get_events(10);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, RotationEventType::KeyDeprecated);
        assert_eq!(events[1].event_type, RotationEventType::KeyGenerated);
    }
}
