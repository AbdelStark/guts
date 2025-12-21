//! Error types for the compatibility layer.

use thiserror::Error;

/// Result type for compatibility operations.
pub type Result<T> = std::result::Result<T, CompatError>;

/// Errors that can occur in the compatibility layer.
#[derive(Debug, Error)]
pub enum CompatError {
    /// User not found.
    #[error("user not found: {0}")]
    UserNotFound(String),

    /// Username already exists.
    #[error("username already exists: {0}")]
    UsernameExists(String),

    /// Invalid username format.
    #[error("invalid username: {0}")]
    InvalidUsername(String),

    /// Token not found.
    #[error("token not found")]
    TokenNotFound,

    /// Invalid token format.
    #[error("invalid token format")]
    InvalidTokenFormat,

    /// Token expired.
    #[error("token expired")]
    TokenExpired,

    /// Invalid token (hash mismatch).
    #[error("invalid token")]
    InvalidToken,

    /// Insufficient scope for operation.
    #[error("insufficient scope: requires {0:?}")]
    InsufficientScope(crate::token::TokenScope),

    /// SSH key not found.
    #[error("SSH key not found")]
    SshKeyNotFound,

    /// Invalid SSH key format.
    #[error("invalid SSH key format: {0}")]
    InvalidSshKey(String),

    /// SSH key already exists (duplicate fingerprint).
    #[error("SSH key already exists with fingerprint: {0}")]
    SshKeyExists(String),

    /// Release not found.
    #[error("release not found: {0}")]
    ReleaseNotFound(String),

    /// Release already exists (same tag).
    #[error("release already exists for tag: {0}")]
    ReleaseExists(String),

    /// Asset not found.
    #[error("asset not found: {0}")]
    AssetNotFound(String),

    /// Asset already exists.
    #[error("asset already exists: {0}")]
    AssetExists(String),

    /// Path not found in repository.
    #[error("path not found: {0}")]
    PathNotFound(String),

    /// Invalid ref (branch, tag, or SHA).
    #[error("invalid ref: {0}")]
    InvalidRef(String),

    /// Archive generation failed.
    #[error("archive generation failed: {0}")]
    ArchiveFailed(String),

    /// Rate limit exceeded.
    #[error("rate limit exceeded, resets at {0}")]
    RateLimitExceeded(u64),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Cryptographic operation failed.
    #[error("crypto error: {0}")]
    Crypto(String),
}

impl CompatError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::UserNotFound(_) => 404,
            Self::UsernameExists(_) => 409,
            Self::InvalidUsername(_) => 422,
            Self::TokenNotFound => 401,
            Self::InvalidTokenFormat => 401,
            Self::TokenExpired => 401,
            Self::InvalidToken => 401,
            Self::InsufficientScope(_) => 403,
            Self::SshKeyNotFound => 404,
            Self::InvalidSshKey(_) => 422,
            Self::SshKeyExists(_) => 409,
            Self::ReleaseNotFound(_) => 404,
            Self::ReleaseExists(_) => 409,
            Self::AssetNotFound(_) => 404,
            Self::AssetExists(_) => 409,
            Self::PathNotFound(_) => 404,
            Self::InvalidRef(_) => 422,
            Self::ArchiveFailed(_) => 500,
            Self::RateLimitExceeded(_) => 429,
            Self::Storage(_) => 500,
            Self::Crypto(_) => 500,
        }
    }

    /// Get the GitHub-compatible error message.
    pub fn github_message(&self) -> &str {
        match self {
            Self::UserNotFound(_) => "Not Found",
            Self::UsernameExists(_) => "Validation Failed",
            Self::InvalidUsername(_) => "Validation Failed",
            Self::TokenNotFound => "Bad credentials",
            Self::InvalidTokenFormat => "Bad credentials",
            Self::TokenExpired => "Bad credentials",
            Self::InvalidToken => "Bad credentials",
            Self::InsufficientScope(_) => "Forbidden",
            Self::SshKeyNotFound => "Not Found",
            Self::InvalidSshKey(_) => "Validation Failed",
            Self::SshKeyExists(_) => "Validation Failed",
            Self::ReleaseNotFound(_) => "Not Found",
            Self::ReleaseExists(_) => "Validation Failed",
            Self::AssetNotFound(_) => "Not Found",
            Self::AssetExists(_) => "Validation Failed",
            Self::PathNotFound(_) => "Not Found",
            Self::InvalidRef(_) => "Validation Failed",
            Self::ArchiveFailed(_) => "Server Error",
            Self::RateLimitExceeded(_) => "API rate limit exceeded",
            Self::Storage(_) => "Server Error",
            Self::Crypto(_) => "Server Error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(CompatError::UserNotFound("test".into()).status_code(), 404);
        assert_eq!(CompatError::TokenNotFound.status_code(), 401);
        assert_eq!(CompatError::RateLimitExceeded(0).status_code(), 429);
    }
}
