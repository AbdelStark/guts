//! Storage backend implementations.

use crate::{ContentHash, Result};
use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::HashMap;

/// A trait for storage backends.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Writes data to storage.
    async fn write(&self, hash: &ContentHash, data: Bytes) -> Result<()>;

    /// Reads data from storage.
    async fn read(&self, hash: &ContentHash) -> Result<Option<Bytes>>;

    /// Checks if data exists in storage.
    async fn exists(&self, hash: &ContentHash) -> Result<bool>;

    /// Deletes data from storage.
    async fn delete(&self, hash: &ContentHash) -> Result<bool>;

    /// Returns the count of stored items.
    async fn count(&self) -> Result<usize>;
}

/// An in-memory storage backend for testing.
pub struct MemoryBackend {
    data: RwLock<HashMap<ContentHash, Bytes>>,
}

impl MemoryBackend {
    /// Creates a new in-memory backend.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn write(&self, hash: &ContentHash, data: Bytes) -> Result<()> {
        self.data.write().insert(*hash, data);
        Ok(())
    }

    async fn read(&self, hash: &ContentHash) -> Result<Option<Bytes>> {
        Ok(self.data.read().get(hash).cloned())
    }

    async fn exists(&self, hash: &ContentHash) -> Result<bool> {
        Ok(self.data.read().contains_key(hash))
    }

    async fn delete(&self, hash: &ContentHash) -> Result<bool> {
        Ok(self.data.write().remove(hash).is_some())
    }

    async fn count(&self) -> Result<usize> {
        Ok(self.data.read().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_backend_write_read() {
        let backend = MemoryBackend::new();
        let data = Bytes::from("test data");
        let hash = ContentHash::compute(&data);

        backend.write(&hash, data.clone()).await.unwrap();
        let read = backend.read(&hash).await.unwrap();

        assert_eq!(read, Some(data));
    }

    #[tokio::test]
    async fn memory_backend_delete() {
        let backend = MemoryBackend::new();
        let data = Bytes::from("to delete");
        let hash = ContentHash::compute(&data);

        backend.write(&hash, data).await.unwrap();
        assert!(backend.exists(&hash).await.unwrap());

        assert!(backend.delete(&hash).await.unwrap());
        assert!(!backend.exists(&hash).await.unwrap());
    }
}
