//! Error types for identity operations.

use thiserror::Error;

/// Errors that can occur during identity operations.
#[derive(Debug, Error)]
pub enum IdentityError {
    /// The signature verification failed.
    #[error("signature verification failed")]
    InvalidSignature,

    /// The public key is malformed.
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    /// The secret key is malformed.
    #[error("invalid secret key")]
    InvalidSecretKey,

    /// Key generation failed.
    #[error("key generation failed: {0}")]
    KeyGeneration(String),
}

/// A specialized Result type for identity operations.
pub type Result<T> = std::result::Result<T, IdentityError>;
