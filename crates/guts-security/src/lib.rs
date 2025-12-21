//! # Guts Security Module
//!
//! Security utilities, audit logging, and key management for the Guts platform.
//!
//! ## Features
//!
//! - **Audit Logging**: Comprehensive security event tracking with tamper-evident logs
//! - **Key Rotation**: Automated key rotation with configurable policies
//! - **Secrets Management**: Secure storage and retrieval of secrets
//! - **HSM Integration**: Hardware Security Module support for high-security deployments
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    guts-security                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │ Audit       │  │ Key         │  │ Secrets             │  │
//! │  │ Logging     │  │ Rotation    │  │ Management          │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘  │
//! │                          │                                   │
//! │                          ▼                                   │
//! │              ┌─────────────────────┐                        │
//! │              │ HSM Interface       │                        │
//! │              │ (optional)          │                        │
//! │              └─────────────────────┘                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust,no_run
//! use guts_security::{AuditLog, AuditEvent, AuditEventType};
//!
//! // Create an audit log
//! let audit_log = AuditLog::new();
//!
//! // Record a security event
//! let event = AuditEvent::new(
//!     AuditEventType::Login,
//!     "user123",
//!     "session",
//!     "success",
//! );
//! audit_log.record(event);
//! ```

mod audit;
mod error;
mod hsm;
mod rate_limit;
mod rotation;
mod secrets;

pub use audit::{AuditEntry, AuditEvent, AuditEventType, AuditLog, AuditQuery, AuditQueryBuilder};
pub use error::{Result, SecurityError};
pub use hsm::{create_hsm_provider, HsmConfig, HsmProvider, MockHsmProvider, Pkcs11HsmProvider};
pub use rate_limit::{
    AdaptiveLimiter, EnhancedRateLimiter, RateLimitConfig, RequestContext, SuspiciousPattern,
};
pub use rotation::{KeyManager, KeyRotationPolicy, KeyState, RotationEvent};
pub use secrets::{
    EnvSecretsProvider, FileSecretsProvider, MemorySecretsProvider, SecretString, SecretsConfig,
    SecretsProvider,
};

/// Security namespace for domain separation in signatures
pub const SECURITY_NAMESPACE: &[u8] = b"_GUTS_SECURITY";

/// Current security module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_namespace_defined() {
        assert!(!SECURITY_NAMESPACE.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version_defined() {
        assert!(!VERSION.is_empty());
    }
}
