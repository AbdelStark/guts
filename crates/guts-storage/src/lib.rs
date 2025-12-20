//! # Guts Storage
//!
//! Content-addressed storage layer for Guts.
//!
//! Provides a trait-based abstraction for storing and retrieving content-addressed
//! blobs, with an in-memory implementation for testing.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod backend;
mod error;
mod hash;

pub use backend::{MemoryBackend, StorageBackend};
pub use error::{Result, StorageError};
pub use hash::ContentHash;

use async_trait::async_trait;
use bytes::Bytes;

/// The main storage interface for content-addressed data.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Stores a blob and returns its content hash.
    async fn put(&self, data: Bytes) -> Result<ContentHash>;

    /// Retrieves a blob by its content hash.
    async fn get(&self, hash: &ContentHash) -> Result<Option<Bytes>>;

    /// Checks if a blob exists.
    async fn exists(&self, hash: &ContentHash) -> Result<bool>;

    /// Deletes a blob by its content hash.
    async fn delete(&self, hash: &ContentHash) -> Result<bool>;

    /// Returns the total number of stored blobs.
    async fn count(&self) -> Result<usize>;
}

/// A content-addressed storage instance backed by an in-memory store.
pub struct ContentStore<B: StorageBackend> {
    backend: B,
}

impl<B: StorageBackend> ContentStore<B> {
    /// Creates a new content store with the given backend.
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> Storage for ContentStore<B> {
    async fn put(&self, data: Bytes) -> Result<ContentHash> {
        let hash = ContentHash::compute(&data);
        self.backend.write(&hash, data).await?;
        Ok(hash)
    }

    async fn get(&self, hash: &ContentHash) -> Result<Option<Bytes>> {
        self.backend.read(hash).await
    }

    async fn exists(&self, hash: &ContentHash) -> Result<bool> {
        self.backend.exists(hash).await
    }

    async fn delete(&self, hash: &ContentHash) -> Result<bool> {
        self.backend.delete(hash).await
    }

    async fn count(&self) -> Result<usize> {
        self.backend.count().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn content_store_roundtrip() {
        let store = ContentStore::new(MemoryBackend::new());
        let data = Bytes::from("Hello, Guts!");

        let hash = store.put(data.clone()).await.unwrap();
        let retrieved = store.get(&hash).await.unwrap();

        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn content_store_deduplication() {
        let store = ContentStore::new(MemoryBackend::new());
        let data = Bytes::from("duplicate");

        let hash1 = store.put(data.clone()).await.unwrap();
        let hash2 = store.put(data).await.unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(store.count().await.unwrap(), 1);
    }
}
