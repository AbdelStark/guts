//! Error types for Guts core operations.

use thiserror::Error;

/// The main error type for Guts core operations.
#[derive(Debug, Error)]
pub enum Error {
    /// The requested resource was not found.
    #[error("not found: {resource_type} with id '{id}'")]
    NotFound {
        /// The type of resource that was not found.
        resource_type: &'static str,
        /// The identifier of the resource.
        id: String,
    },

    /// Permission was denied for the operation.
    #[error("permission denied: {reason}")]
    PermissionDenied {
        /// The reason for the denial.
        reason: String,
    },

    /// The provided input was invalid.
    #[error("invalid input: {field} - {message}")]
    InvalidInput {
        /// The field that was invalid.
        field: &'static str,
        /// A description of why the input was invalid.
        message: String,
    },

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization error occurred.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// An internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}

/// A specialized Result type for Guts operations.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Creates a new not found error.
    #[must_use]
    pub fn not_found(resource_type: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type,
            id: id.into(),
        }
    }

    /// Creates a new permission denied error.
    #[must_use]
    pub fn permission_denied(reason: impl Into<String>) -> Self {
        Self::PermissionDenied {
            reason: reason.into(),
        }
    }

    /// Creates a new invalid input error.
    #[must_use]
    pub fn invalid_input(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            field,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn error_not_found_display() {
        let err = Error::not_found("repository", "abc123");
        assert_eq!(
            err.to_string(),
            "not found: repository with id 'abc123'"
        );
    }

    #[test]
    fn error_permission_denied_display() {
        let err = Error::permission_denied("write access required");
        assert_eq!(
            err.to_string(),
            "permission denied: write access required"
        );
    }

    #[test]
    fn error_invalid_input_display() {
        let err = Error::invalid_input("name", "cannot be empty");
        assert_eq!(
            err.to_string(),
            "invalid input: name - cannot be empty"
        );
    }
}
