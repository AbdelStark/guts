//! # Input Validation Module
//!
//! Production-grade input validation for all API endpoints including:
//!
//! - Repository name validation
//! - Organization and team name validation
//! - Branch and tag name validation
//! - Path and content validation
//! - Request size limits
//!
//! ## Usage
//!
//! ```rust,no_run
//! use guts_node::validation::validate_name;
//!
//! // Validate a repository name
//! if let Err(e) = validate_name("my-repo") {
//!     println!("Invalid name: {}", e);
//! }
//! ```

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use validator::{ValidationError, ValidationErrors};

/// Regex for valid repository/organization names.
/// Must start with alphanumeric, can contain alphanumeric, hyphens, and underscores.
pub static NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]*$").expect("Invalid regex"));

/// Regex for valid branch/tag names.
/// Git reference names with common restrictions.
pub static REF_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9/_.-]*$").expect("Invalid regex"));

/// Reserved names that cannot be used for repositories or organizations.
pub static RESERVED_NAMES: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "api",
        "git",
        "admin",
        "administrator",
        "root",
        "system",
        "health",
        "metrics",
        "status",
        "settings",
        "help",
        "about",
        "login",
        "logout",
        "signup",
        "register",
        "new",
        "edit",
        "delete",
        "create",
        "update",
        "organizations",
        "orgs",
        "users",
        "teams",
        "repos",
        "repositories",
        "pulls",
        "issues",
        "commits",
        "branches",
        "tags",
        "releases",
        "actions",
        "workflows",
        "runs",
        "artifacts",
        "search",
        "explore",
        "trending",
        "notifications",
        "webhooks",
        "tokens",
        "keys",
        "ssh",
        "gpg",
    ]
});

/// Maximum lengths for various fields.
pub const MAX_NAME_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_TITLE_LENGTH: usize = 256;
pub const MAX_BODY_LENGTH: usize = 65536;
pub const MAX_PATH_LENGTH: usize = 4096;

/// Validation error response.
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    /// Error type.
    pub error: String,
    /// Human-readable message.
    pub message: String,
    /// Field-level error details.
    pub details: Vec<FieldError>,
}

/// Field-level validation error.
#[derive(Debug, Serialize)]
pub struct FieldError {
    /// Field name.
    pub field: String,
    /// Error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl IntoResponse for ValidationErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response()
    }
}

/// Convert ValidationErrors to our error response.
impl From<ValidationErrors> for ValidationErrorResponse {
    fn from(errors: ValidationErrors) -> Self {
        let details: Vec<FieldError> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| FieldError {
                    field: field.to_string(),
                    code: e.code.to_string(),
                    message: e
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Validation failed for field '{}'", field)),
                })
            })
            .collect();

        ValidationErrorResponse {
            error: "validation_error".to_string(),
            message: "Validation failed".to_string(),
            details,
        }
    }
}

/// Validate a repository or organization name.
pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    // Check length
    if name.is_empty() {
        let mut err = ValidationError::new("length");
        err.message = Some("Name cannot be empty".into());
        return Err(err);
    }

    if name.len() > MAX_NAME_LENGTH {
        let mut err = ValidationError::new("length");
        err.message = Some(format!("Name must be at most {} characters", MAX_NAME_LENGTH).into());
        return Err(err);
    }

    // Check pattern
    if !NAME_REGEX.is_match(name) {
        let mut err = ValidationError::new("pattern");
        err.message = Some(
            "Name must start with a letter or number and contain only letters, numbers, hyphens, and underscores".into()
        );
        return Err(err);
    }

    // Check reserved names
    if RESERVED_NAMES.contains(&name.to_lowercase().as_str()) {
        let mut err = ValidationError::new("reserved");
        err.message = Some("This name is reserved and cannot be used".into());
        return Err(err);
    }

    Ok(())
}

/// Validate a git reference name (branch/tag).
pub fn validate_ref_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        let mut err = ValidationError::new("length");
        err.message = Some("Reference name cannot be empty".into());
        return Err(err);
    }

    if name.len() > MAX_NAME_LENGTH {
        let mut err = ValidationError::new("length");
        err.message = Some(
            format!(
                "Reference name must be at most {} characters",
                MAX_NAME_LENGTH
            )
            .into(),
        );
        return Err(err);
    }

    if !REF_NAME_REGEX.is_match(name) {
        let mut err = ValidationError::new("pattern");
        err.message = Some("Invalid reference name format".into());
        return Err(err);
    }

    // Git-specific restrictions
    if name.contains("..") || name.starts_with('/') || name.ends_with('/') || name.ends_with('.') {
        let mut err = ValidationError::new("git_restriction");
        err.message = Some("Reference name contains invalid Git sequences".into());
        return Err(err);
    }

    Ok(())
}

/// Validate a file path.
pub fn validate_path(path: &str) -> Result<(), ValidationError> {
    if path.len() > MAX_PATH_LENGTH {
        let mut err = ValidationError::new("length");
        err.message = Some(format!("Path must be at most {} characters", MAX_PATH_LENGTH).into());
        return Err(err);
    }

    // Security checks
    if path.contains("..") {
        let mut err = ValidationError::new("security");
        err.message = Some("Path cannot contain '..' sequences".into());
        return Err(err);
    }

    if path.contains('\0') {
        let mut err = ValidationError::new("security");
        err.message = Some("Path cannot contain null bytes".into());
        return Err(err);
    }

    Ok(())
}

/// Request body size limit middleware.
pub async fn body_size_limit_middleware(
    request: Request,
    next: axum::middleware::Next,
) -> Response {
    // Get content length if available
    if let Some(content_length) = request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        // 50MB limit for most requests
        const MAX_BODY_SIZE: usize = 50 * 1024 * 1024;

        if content_length > MAX_BODY_SIZE {
            return Response::builder()
                .status(StatusCode::PAYLOAD_TOO_LARGE)
                .body(Body::from(
                    r#"{"error":"payload_too_large","message":"Request body exceeds maximum size"}"#,
                ))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .expect("Failed to build response")
                });
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        // Valid names
        assert!(validate_name("myrepo").is_ok());
        assert!(validate_name("my-repo").is_ok());
        assert!(validate_name("my_repo").is_ok());
        assert!(validate_name("MyRepo123").is_ok());
        assert!(validate_name("a").is_ok());

        // Invalid names
        assert!(validate_name("").is_err());
        assert!(validate_name("-myrepo").is_err());
        assert!(validate_name("_myrepo").is_err());
        assert!(validate_name("my repo").is_err());
        assert!(validate_name("my.repo").is_err());
        assert!(validate_name("api").is_err()); // reserved
        assert!(validate_name("admin").is_err()); // reserved
    }

    #[test]
    fn test_validate_ref_name() {
        // Valid refs
        assert!(validate_ref_name("main").is_ok());
        assert!(validate_ref_name("feature/test").is_ok());
        assert!(validate_ref_name("v1.0.0").is_ok());
        assert!(validate_ref_name("release-1.0").is_ok());

        // Invalid refs
        assert!(validate_ref_name("").is_err());
        assert!(validate_ref_name("..").is_err());
        assert!(validate_ref_name("/main").is_err());
        assert!(validate_ref_name("main/").is_err());
        assert!(validate_ref_name("main.").is_err());
    }

    #[test]
    fn test_validate_path() {
        // Valid paths
        assert!(validate_path("src/main.rs").is_ok());
        assert!(validate_path("README.md").is_ok());
        assert!(validate_path("path/to/file.txt").is_ok());

        // Invalid paths
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("path/../secret").is_err());
    }
}
