//! # API Errors
//!
//! Error types for API operations.

use thiserror::Error;

/// Errors that can occur during API operations.
#[derive(Error, Debug)]
pub enum ApiError {
    /// Network or HTTP error.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Node returned an error response.
    #[error("node error: {status} - {message}")]
    NodeError {
        /// HTTP status code.
        status: u16,
        /// Error message from the node.
        message: String,
    },

    /// Failed to deserialize response.
    #[error("invalid response format: {0}")]
    InvalidResponse(String),
}

/// Result type for API operations.
pub type ApiResult<T> = Result<T, ApiError>;
