//! Git object storage for Guts.
//!
//! This crate provides content-addressed storage for git objects
//! (blobs, trees, commits) and reference management.

mod error;
mod object;
mod refs;
mod store;

pub use error::StorageError;
pub use object::{GitObject, ObjectId, ObjectType};
pub use refs::{RefStore, Reference};
pub use store::{ObjectStore, RepoStore, Repository};

/// Result type for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;
