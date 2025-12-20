//! API request and response types.

use serde::{Deserialize, Serialize};

/// A generic API response wrapper.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful.
    pub success: bool,
    /// The response data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error message if unsuccessful.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Creates a successful response.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Creates an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: String,
    /// Service version.
    pub version: String,
    /// Node ID.
    pub node_id: Option<String>,
}

/// Repository info response.
#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository ID.
    pub id: String,
    /// Repository name.
    pub name: String,
    /// Owner.
    pub owner: String,
    /// Description.
    pub description: Option<String>,
    /// Default branch.
    pub default_branch: String,
    /// Created timestamp.
    pub created_at: String,
}

/// Create repository request.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    /// Repository name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Whether the repository is private.
    pub private: Option<bool>,
}
