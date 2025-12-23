//! # Credentials Storage
//!
//! Stores user credentials including username, private key, and access token.

use serde::{Deserialize, Serialize};

use super::Identity;

/// User credentials for authentication.
///
/// Stored in the config file and loaded on app startup.
/// Contains everything needed to authenticate with the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// The user's username.
    pub username: String,

    /// Hex-encoded Ed25519 public key.
    ///
    /// This is stored for reference/display. The private key is only
    /// used during initial registration and not persisted.
    pub public_key_hex: String,

    /// Personal access token for API authentication.
    ///
    /// Format: `guts_<prefix>_<secret>`
    #[serde(default)]
    pub token: Option<String>,

    /// User ID from the server.
    #[serde(default)]
    pub user_id: Option<u64>,
}

impl Credentials {
    /// Create new credentials from a username and identity.
    ///
    /// The token and user_id are set after successful registration.
    #[must_use]
    pub fn new(username: String, identity: &Identity) -> Self {
        Self {
            username,
            public_key_hex: identity.public_key_hex(),
            token: None,
            user_id: None,
        }
    }

    /// Check if credentials have a valid token.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_new() {
        let identity = Identity::generate();
        let creds = Credentials::new("alice".to_string(), &identity);

        assert_eq!(creds.username, "alice");
        assert!(!creds.has_token());
        assert!(creds.user_id.is_none());
    }

    #[test]
    fn test_credentials_public_key() {
        let identity = Identity::generate();
        let expected = identity.public_key_hex();
        let creds = Credentials::new("charlie".to_string(), &identity);

        assert_eq!(creds.public_key_hex, expected);
    }

    #[test]
    fn test_credentials_serialization() {
        let identity = Identity::generate();
        let mut creds = Credentials::new("dave".to_string(), &identity);
        creds.token = Some("guts_abc12345_secretsecretsecretsecret32ch".to_string());
        creds.user_id = Some(42);

        let json = serde_json::to_string(&creds).unwrap();
        let restored: Credentials = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.username, "dave");
        assert_eq!(restored.token, creds.token);
        assert_eq!(restored.user_id, Some(42));
    }
}
