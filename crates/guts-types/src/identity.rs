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
}
