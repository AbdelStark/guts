//! Identity types for Guts using Ed25519 signatures.

use commonware_cryptography::ed25519;
use serde::{Deserialize, Serialize};

/// Ed25519 public key used for peer identity.
pub type PublicKey = ed25519::PublicKey;

/// Ed25519 signature.
pub type Signature = ed25519::Signature;

/// A user identity in the Guts network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// The user's public key (hex encoded).
    pub public_key: String,
    /// Optional username.
    pub username: Option<String>,
    /// Optional display name.
    pub display_name: Option<String>,
}

impl Identity {
    /// Creates a new identity from a public key.
    pub fn new(public_key: impl Into<String>) -> Self {
        Self {
            public_key: public_key.into(),
            username: None,
            display_name: None,
        }
    }

    /// Sets the username.
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation() {
        let identity = Identity::new("abc123")
            .with_username("alice")
            .with_display_name("Alice");

        assert_eq!(identity.public_key, "abc123");
        assert_eq!(identity.username, Some("alice".to_string()));
        assert_eq!(identity.display_name, Some("Alice".to_string()));
    }

    #[test]
    fn test_identity_without_optional_fields() {
        let identity = Identity::new("pubkey123");

        assert_eq!(identity.public_key, "pubkey123");
        assert!(identity.username.is_none());
        assert!(identity.display_name.is_none());
    }

    #[test]
    fn test_identity_with_only_username() {
        let identity = Identity::new("key").with_username("user");

        assert_eq!(identity.username, Some("user".to_string()));
        assert!(identity.display_name.is_none());
    }

    #[test]
    fn test_identity_with_only_display_name() {
        let identity = Identity::new("key").with_display_name("Display Name");

        assert!(identity.username.is_none());
        assert_eq!(identity.display_name, Some("Display Name".to_string()));
    }

    #[test]
    fn test_identity_accepts_string_types() {
        // Test with String
        let identity1 = Identity::new(String::from("key1"));
        assert_eq!(identity1.public_key, "key1");

        // Test with &str
        let identity2 = Identity::new("key2");
        assert_eq!(identity2.public_key, "key2");
    }

    #[test]
    fn test_identity_serialization() {
        let identity = Identity::new("abc123")
            .with_username("alice")
            .with_display_name("Alice");

        let json = serde_json::to_string(&identity).unwrap();
        let parsed: Identity = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.public_key, identity.public_key);
        assert_eq!(parsed.username, identity.username);
        assert_eq!(parsed.display_name, identity.display_name);
    }

    #[test]
    fn test_identity_clone() {
        let original = Identity::new("key").with_username("user");
        let cloned = original.clone();

        assert_eq!(cloned.public_key, original.public_key);
        assert_eq!(cloned.username, original.username);
    }

    #[test]
    fn test_identity_empty_strings() {
        // Empty strings should be accepted (validation is at higher layer)
        let identity = Identity::new("").with_username("").with_display_name("");

        assert_eq!(identity.public_key, "");
        assert_eq!(identity.username, Some(String::new()));
        assert_eq!(identity.display_name, Some(String::new()));
    }

    #[test]
    fn test_identity_unicode() {
        let identity = Identity::new("key")
            .with_username("用户")
            .with_display_name("名前 🎉");

        assert_eq!(identity.username, Some("用户".to_string()));
        assert_eq!(identity.display_name, Some("名前 🎉".to_string()));
    }
}
