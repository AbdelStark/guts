//! Error types for the security crate.

use thiserror::Error;

/// Result type alias for security operations.
pub type Result<T> = std::result::Result<T, SecurityError>;

/// Errors that can occur during security operations.
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Audit log entry not found.
    #[error("audit log entry not found: {0}")]
    AuditLogNotFound(String),

    /// Audit log is full or cannot accept more entries.
    #[error("audit log capacity exceeded")]
    AuditLogFull,

    /// Invalid cryptographic key.
    #[error("invalid key: {0}")]
    InvalidKey(String),

    /// Key rotation failed.
    #[error("key rotation failed: {0}")]
    RotationFailed(String),

    /// Key not found in the key store.
    #[error("key not found: {0}")]
    KeyNotFound(String),

    /// Key has expired and cannot be used.
    #[error("key expired: {0}")]
    KeyExpired(String),

    /// Secret not found.
    #[error("secret not found: {0}")]
    SecretNotFound(String),

    /// Secret storage error.
    #[error("secret storage error: {0}")]
    SecretStorageError(String),

    /// HSM communication error.
    #[error("HSM error: {0}")]
    HsmError(String),

    /// HSM not configured.
    #[error("HSM not configured")]
    HsmNotConfigured,

    /// Rate limit exceeded.
    #[error("rate limit exceeded for {resource}: retry after {retry_after} seconds")]
    RateLimitExceeded {
        /// The resource that was rate limited.
        resource: String,
        /// Seconds until the rate limit resets.
        retry_after: u64,
    },

    /// Suspicious activity detected.
    #[error("suspicious activity detected: {pattern}")]
    SuspiciousActivity {
        /// Description of the suspicious pattern.
        pattern: String,
        /// The actor that triggered the detection.
        actor: String,
    },

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Cryptographic operation failed.
    #[error("cryptographic error: {0}")]
    Crypto(String),

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

impl SecurityError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            SecurityError::AuditLogNotFound(_) => 404,
            SecurityError::AuditLogFull => 507,
            SecurityError::InvalidKey(_) => 400,
            SecurityError::RotationFailed(_) => 500,
            SecurityError::KeyNotFound(_) => 404,
            SecurityError::KeyExpired(_) => 401,
            SecurityError::SecretNotFound(_) => 404,
            SecurityError::SecretStorageError(_) => 500,
            SecurityError::HsmError(_) => 500,
            SecurityError::HsmNotConfigured => 503,
            SecurityError::RateLimitExceeded { .. } => 429,
            SecurityError::SuspiciousActivity { .. } => 403,
            SecurityError::Serialization(_) => 500,
            SecurityError::Io(_) => 500,
            SecurityError::Configuration(_) => 500,
            SecurityError::Crypto(_) => 500,
            SecurityError::PermissionDenied(_) => 403,
            SecurityError::InvalidInput(_) => 400,
        }
    }

    /// Returns whether this error indicates a client error (4xx).
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status_code())
    }

    /// Returns whether this error indicates a server error (5xx).
    pub fn is_server_error(&self) -> bool {
        self.status_code() >= 500
    }
}

impl From<std::io::Error> for SecurityError {
    fn from(err: std::io::Error) -> Self {
        SecurityError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for SecurityError {
    fn from(err: serde_json::Error) -> Self {
        SecurityError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            SecurityError::AuditLogNotFound("test".into()).status_code(),
            404
        );
        assert_eq!(
            SecurityError::RateLimitExceeded {
                resource: "api".into(),
                retry_after: 60
            }
            .status_code(),
            429
        );
        assert_eq!(
            SecurityError::PermissionDenied("test".into()).status_code(),
            403
        );
    }

    #[test]
    fn test_error_classification() {
        let client_err = SecurityError::InvalidInput("bad data".into());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        let server_err = SecurityError::HsmError("connection failed".into());
        assert!(!server_err.is_client_error());
        assert!(server_err.is_server_error());
    }

    #[test]
    fn test_error_display() {
        let err = SecurityError::RateLimitExceeded {
            resource: "api".into(),
            retry_after: 60,
        };
        assert!(err.to_string().contains("rate limit exceeded"));
        assert!(err.to_string().contains("60"));
    }
}
