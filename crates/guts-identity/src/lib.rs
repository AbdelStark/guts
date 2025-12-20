//! # Guts Identity
//!
//! Cryptographic identity management for Guts using Ed25519 signatures.
//!
//! ## Example
//!
//! ```rust
//! use guts_identity::{Identity, Keypair};
//!
//! // Generate a new identity
//! let keypair = Keypair::generate();
//!
//! // Sign a message
//! let message = b"Hello, Guts!";
//! let signature = keypair.sign(message);
//!
//! // Verify the signature
//! assert!(keypair.verify(message, &signature).is_ok());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod keypair;
mod public_key;
mod signature;

pub use error::{IdentityError, Result};
pub use keypair::Keypair;
pub use public_key::PublicKey;
pub use signature::Signature;

/// A user identity consisting of a public key and metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Identity {
    /// The public key identifying this user.
    pub public_key: PublicKey,
    /// Optional username.
    pub username: Option<String>,
    /// Optional display name.
    pub display_name: Option<String>,
}

impl Identity {
    /// Creates a new identity from a public key.
    #[must_use]
    pub fn new(public_key: PublicKey) -> Self {
        Self {
            public_key,
            username: None,
            display_name: None,
        }
    }

    /// Creates an identity with a username.
    #[must_use]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Returns the identity's short ID (first 8 bytes of public key as hex).
    #[must_use]
    pub fn short_id(&self) -> String {
        self.public_key.short_id()
    }
}
