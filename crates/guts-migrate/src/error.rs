//! Error types for migration operations.

use thiserror::Error;

/// Migration-specific errors.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// Failed to authenticate with source platform.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Repository not found on source platform.
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    /// Failed to clone repository.
    #[error("Git clone failed: {0}")]
    GitCloneFailed(String),

    /// Failed to push to Guts.
    #[error("Git push failed: {0}")]
    GitPushFailed(String),

    /// API request failed.
    #[error("API request failed: {0}")]
    ApiError(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded, retry after {0} seconds")]
    RateLimitExceeded(u64),

    /// Network error.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Verification failed.
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Unsupported feature.
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Generic I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

/// Result type for migration operations.
pub type Result<T> = std::result::Result<T, MigrationError>;
