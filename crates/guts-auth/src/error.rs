//! Error types for the auth crate.

use thiserror::Error;

/// Errors that can occur in authorization operations.
#[derive(Debug, Error)]
pub enum AuthError {
    /// The requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// The user lacks permission for the operation.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// The resource already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Invalid input was provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Cannot remove the last owner of an organization.
    #[error("cannot remove last owner of organization")]
    LastOwner,

    /// Branch is protected and operation is not allowed.
    #[error("branch '{0}' is protected: {1}")]
    BranchProtected(String, String),

    /// Invalid webhook configuration.
    #[error("invalid webhook: {0}")]
    InvalidWebhook(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Result type for auth operations.
pub type Result<T> = std::result::Result<T, AuthError>;
