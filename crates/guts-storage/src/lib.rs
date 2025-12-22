//! Git object storage for Guts.
//!
//! This crate provides content-addressed storage for git objects
//! (blobs, trees, commits) and reference management.
//!
//! # Features
//!
//! - `memory` - In-memory storage backend (default)
//! - `rocksdb-backend` - RocksDB persistent storage backend
//! - `full` - All features enabled
//!
//! # Storage Backends
//!
//! The crate supports multiple storage backends:
//!
//! - [`ObjectStore`] - In-memory storage (fast but volatile)
//! - [`RocksDbStorage`] - Persistent storage using RocksDB (requires `rocksdb-backend` feature)
//! - [`CachedStorage`] - LRU cache wrapper for any storage backend
//! - [`HybridStorage`] - Hot/cold tiering with automatic migration

mod cache;
mod compression;
mod error;
mod hybrid;
mod object;
mod pool;
mod refs;
#[cfg(feature = "rocksdb-backend")]
mod rocksdb;
mod store;
mod traits;

pub use cache::{CacheConfig, CacheMetrics, CacheStats, CachedStorage};
pub use compression::{CompressionLevel, CompressionStats};
pub use error::StorageError;
pub use hybrid::{HybridConfig, HybridStatsSnapshot, HybridStorage};
pub use object::{GitObject, ObjectId, ObjectType};
pub use pool::{ObjectPool, PooledBuffer, IO_BUFFER_POOL, PACK_BUFFER_POOL};
pub use refs::{RefStore, Reference};
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::{RocksDbConfig, RocksDbStorage};
pub use store::{ObjectStore, RepoStore, Repository};
pub use traits::{ObjectStoreBackend, StorageBackend, StorageStats};

/// Result type for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;
