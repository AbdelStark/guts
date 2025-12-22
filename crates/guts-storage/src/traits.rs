//! Storage backend traits.
//!
//! Defines the interface that all storage backends must implement,
//! enabling pluggable storage strategies.

use crate::{GitObject, ObjectId, Result};
use std::sync::Arc;

/// Trait for object storage backends.
///
/// Implementations include in-memory, RocksDB, and hybrid storage.
pub trait ObjectStoreBackend: Send + Sync {
    /// Stores an object and returns its ID.
    fn put(&self, object: GitObject) -> Result<ObjectId>;

    /// Retrieves an object by ID.
    fn get(&self, id: &ObjectId) -> Result<Option<GitObject>>;

    /// Checks if an object exists.
    fn contains(&self, id: &ObjectId) -> Result<bool>;

    /// Deletes an object by ID.
    fn delete(&self, id: &ObjectId) -> Result<bool>;

    /// Returns the number of objects in the store.
    fn len(&self) -> Result<usize>;

    /// Returns true if the store is empty.
    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Lists all object IDs.
    fn list_objects(&self) -> Result<Vec<ObjectId>>;

    /// Batch put operation for improved throughput.
    fn batch_put(&self, objects: Vec<GitObject>) -> Result<Vec<ObjectId>> {
        objects.into_iter().map(|obj| self.put(obj)).collect()
    }

    /// Batch get operation.
    fn batch_get(&self, ids: &[ObjectId]) -> Result<Vec<Option<GitObject>>> {
        ids.iter().map(|id| self.get(id)).collect()
    }

    /// Flush any pending writes to durable storage.
    fn flush(&self) -> Result<()> {
        Ok(())
    }

    /// Compact the storage to reclaim space.
    fn compact(&self) -> Result<()> {
        Ok(())
    }
}

// Implement ObjectStoreBackend for Arc<T> where T: ObjectStoreBackend
impl<T: ObjectStoreBackend> ObjectStoreBackend for Arc<T> {
    fn put(&self, object: GitObject) -> Result<ObjectId> {
        (**self).put(object)
    }

    fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
        (**self).get(id)
    }

    fn contains(&self, id: &ObjectId) -> Result<bool> {
        (**self).contains(id)
    }

    fn delete(&self, id: &ObjectId) -> Result<bool> {
        (**self).delete(id)
    }

    fn len(&self) -> Result<usize> {
        (**self).len()
    }

    fn list_objects(&self) -> Result<Vec<ObjectId>> {
        (**self).list_objects()
    }

    fn batch_put(&self, objects: Vec<GitObject>) -> Result<Vec<ObjectId>> {
        (**self).batch_put(objects)
    }

    fn batch_get(&self, ids: &[ObjectId]) -> Result<Vec<Option<GitObject>>> {
        (**self).batch_get(ids)
    }

    fn flush(&self) -> Result<()> {
        (**self).flush()
    }

    fn compact(&self) -> Result<()> {
        (**self).compact()
    }
}

/// High-level storage backend trait with lifecycle management.
pub trait StorageBackend: ObjectStoreBackend {
    /// Opens or creates the storage at the given path.
    fn open(path: &std::path::Path) -> Result<Self>
    where
        Self: Sized;

    /// Closes the storage, flushing any pending data.
    fn close(&self) -> Result<()> {
        self.flush()
    }

    /// Returns storage statistics.
    fn stats(&self) -> StorageStats {
        StorageStats::default()
    }
}

/// Storage statistics.
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// Total number of objects.
    pub object_count: u64,
    /// Total size of all objects in bytes.
    pub total_size_bytes: u64,
    /// Size of storage on disk (if applicable).
    pub disk_size_bytes: Option<u64>,
    /// Number of read operations.
    pub reads: u64,
    /// Number of write operations.
    pub writes: u64,
    /// Cache hit ratio (if caching enabled).
    pub cache_hit_ratio: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockStorage;

    impl ObjectStoreBackend for MockStorage {
        fn put(&self, _object: GitObject) -> Result<ObjectId> {
            Ok(ObjectId::from_bytes([0u8; 20]))
        }

        fn get(&self, _id: &ObjectId) -> Result<Option<GitObject>> {
            Ok(None)
        }

        fn contains(&self, _id: &ObjectId) -> Result<bool> {
            Ok(false)
        }

        fn delete(&self, _id: &ObjectId) -> Result<bool> {
            Ok(false)
        }

        fn len(&self) -> Result<usize> {
            Ok(0)
        }

        fn list_objects(&self) -> Result<Vec<ObjectId>> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_is_empty_default() {
        let storage = MockStorage;
        assert!(storage.is_empty().unwrap());
    }
}
