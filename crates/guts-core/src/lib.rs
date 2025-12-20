//! # Guts Core
//!
//! Core types, traits, and error definitions for the Guts decentralized
//! code collaboration platform.
//!
//! This crate provides the foundational building blocks used throughout
//! the Guts ecosystem.
//!
//! ## Features
//!
//! - Common identifier types ([`RepositoryId`], [`CommitId`], [`ObjectId`])
//! - Core trait definitions
//! - Error types with rich context
//! - Timestamp and metadata types
//!
//! ## Example
//!
//! ```rust
//! use guts_core::{RepositoryId, Result};
//!
//! fn create_repository(name: &str) -> Result<RepositoryId> {
//!     // Implementation
//!     Ok(RepositoryId::generate())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod error;
pub mod id;
pub mod repository;
pub mod timestamp;
pub mod traits;

pub use error::{Error, Result};
pub use id::{CommitId, ObjectId, RepositoryId};
pub use repository::{Repository, RepositoryMetadata, Visibility};
pub use timestamp::Timestamp;
