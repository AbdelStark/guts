//! # Guts API
//!
//! HTTP and gRPC API server for Guts nodes.
//!
//! This crate provides the REST API for interacting with Guts nodes,
//! including repository management, identity, and Git operations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod handlers;
mod router;
mod types;

pub use error::{ApiError, Result};
pub use router::create_router;
pub use types::{ApiResponse, HealthResponse};

/// Default API port.
pub const DEFAULT_API_PORT: u16 = 8080;

/// API version.
pub const API_VERSION: &str = "v1";
