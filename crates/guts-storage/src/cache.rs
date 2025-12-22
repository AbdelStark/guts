//! LRU cache layer for storage backends.
//!
//! Provides a caching layer that can wrap any storage backend
//! to improve read performance for frequently accessed objects.

use crate::{GitObject, ObjectId, Result, StorageError};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};

/// Configuration for the cache layer.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of objects to cache.
    pub max_objects: usize,
    /// Maximum total size in bytes.
    pub max_size_bytes: usize,
    /// Whether to cache on write (write-through).
    pub write_through: bool,
    /// TTL for cached objects in seconds (0 = no expiry).
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_objects: 10_000,
            max_size_bytes: 256 * 1024 * 1024, // 256 MB
            write_through: true,
            ttl_seconds: 0,
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of evictions.
    pub evictions: u64,
    /// Current number of cached objects.
    pub size: usize,
    /// Current memory usage in bytes.
    pub memory_bytes: usize,
}

impl CacheStats {
    /// Returns the cache hit ratio.
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Cache metrics for monitoring.
#[derive(Debug, Default)]
pub struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl CacheMetrics {
    /// Creates new cache metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a cache hit.
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a cache miss.
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Records an eviction.
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns current metrics.
    pub fn snapshot(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            size: 0,
            memory_bytes: 0,
        }
    }
}

/// Cached storage wrapper.
///
/// Wraps any storage backend with an LRU cache for improved read performance.
pub struct CachedStorage<S> {
    /// The underlying storage backend.
    inner: S,
    /// LRU cache for objects.
    cache: Mutex<LruCache<ObjectId, GitObject>>,
    /// Current size in bytes.
    current_size: AtomicU64,
    /// Configuration.
    config: CacheConfig,
    /// Metrics.
    metrics: CacheMetrics,
}

impl<S> CachedStorage<S> {
    /// Creates a new cached storage wrapper.
    pub fn new(inner: S, config: CacheConfig) -> Self {
        let max_objects = NonZeroUsize::new(config.max_objects).unwrap_or(NonZeroUsize::MIN);
        Self {
            inner,
            cache: Mutex::new(LruCache::new(max_objects)),
            current_size: AtomicU64::new(0),
            config,
            metrics: CacheMetrics::new(),
        }
    }

    /// Creates a cached storage with default configuration.
    pub fn with_defaults(inner: S) -> Self {
        Self::new(inner, CacheConfig::default())
    }

    /// Returns the underlying storage.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Returns the cache configuration.
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Returns current cache statistics.
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock();
        let mut stats = self.metrics.snapshot();
        stats.size = cache.len();
        stats.memory_bytes = self.current_size.load(Ordering::Relaxed) as usize;
        stats
    }

    /// Clears the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.lock();
        cache.clear();
        self.current_size.store(0, Ordering::Relaxed);
    }

    /// Invalidates a specific object from the cache.
    pub fn invalidate(&self, id: &ObjectId) {
        let mut cache = self.cache.lock();
        if let Some(obj) = cache.pop(id) {
            let size = obj.data.len() as u64;
            self.current_size.fetch_sub(size, Ordering::Relaxed);
        }
    }

    /// Attempts to get an object from cache.
    fn cache_get(&self, id: &ObjectId) -> Option<GitObject> {
        let mut cache = self.cache.lock();
        cache.get(id).cloned()
    }

    /// Puts an object into the cache.
    fn cache_put(&self, object: &GitObject) {
        let size = object.data.len() as u64;

        // Check if we need to evict
        let mut cache = self.cache.lock();
        while self.current_size.load(Ordering::Relaxed) + size > self.config.max_size_bytes as u64 {
            if let Some((_, evicted)) = cache.pop_lru() {
                let evicted_size = evicted.data.len() as u64;
                self.current_size.fetch_sub(evicted_size, Ordering::Relaxed);
                self.metrics.record_eviction();
            } else {
                break;
            }
        }

        // Insert into cache
        if let Some(old) = cache.put(object.id, object.clone()) {
            let old_size = old.data.len() as u64;
            self.current_size.fetch_sub(old_size, Ordering::Relaxed);
        }
        self.current_size.fetch_add(size, Ordering::Relaxed);
    }
}

impl<S> CachedStorage<S>
where
    S: crate::traits::ObjectStoreBackend,
{
    /// Gets an object, checking cache first.
    pub fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
        // Check cache first
        if let Some(obj) = self.cache_get(id) {
            self.metrics.record_hit();
            return Ok(Some(obj));
        }

        self.metrics.record_miss();

        // Fetch from underlying storage
        let result = self.inner.get(id)?;

        // Cache the result if found
        if let Some(ref obj) = result {
            self.cache_put(obj);
        }

        Ok(result)
    }

    /// Puts an object, optionally caching it.
    pub fn put(&self, object: GitObject) -> Result<ObjectId> {
        let id = self.inner.put(object.clone())?;

        if self.config.write_through {
            self.cache_put(&object);
        }

        Ok(id)
    }

    /// Checks if an object exists.
    pub fn contains(&self, id: &ObjectId) -> Result<bool> {
        // Check cache first
        {
            let cache = self.cache.lock();
            if cache.contains(id) {
                return Ok(true);
            }
        }

        // Check underlying storage
        self.inner.contains(id)
    }

    /// Deletes an object.
    pub fn delete(&self, id: &ObjectId) -> Result<bool> {
        self.invalidate(id);
        self.inner.delete(id)
    }

    /// Returns the number of objects.
    pub fn len(&self) -> Result<usize> {
        self.inner.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> Result<bool> {
        self.inner.is_empty()
    }

    /// Lists all object IDs.
    pub fn list_objects(&self) -> Result<Vec<ObjectId>> {
        self.inner.list_objects()
    }

    /// Batch get with caching.
    pub fn batch_get(&self, ids: &[ObjectId]) -> Result<Vec<Option<GitObject>>> {
        let mut results = Vec::with_capacity(ids.len());
        let mut uncached = Vec::new();
        let mut uncached_indices = Vec::new();

        // Check cache first
        for (i, id) in ids.iter().enumerate() {
            if let Some(obj) = self.cache_get(id) {
                self.metrics.record_hit();
                results.push(Some(obj));
            } else {
                self.metrics.record_miss();
                uncached.push(*id);
                uncached_indices.push(i);
                results.push(None);
            }
        }

        // Fetch uncached from storage
        if !uncached.is_empty() {
            let fetched = self.inner.batch_get(&uncached)?;
            for (i, obj) in uncached_indices.into_iter().zip(fetched) {
                if let Some(ref o) = obj {
                    self.cache_put(o);
                }
                results[i] = obj;
            }
        }

        Ok(results)
    }
}

// Implement ObjectStoreBackend for CachedStorage
impl<S> crate::traits::ObjectStoreBackend for CachedStorage<S>
where
    S: crate::traits::ObjectStoreBackend,
{
    fn put(&self, object: GitObject) -> Result<ObjectId> {
        CachedStorage::put(self, object)
    }

    fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
        CachedStorage::get(self, id)
    }

    fn contains(&self, id: &ObjectId) -> Result<bool> {
        CachedStorage::contains(self, id)
    }

    fn delete(&self, id: &ObjectId) -> Result<bool> {
        CachedStorage::delete(self, id)
    }

    fn len(&self) -> Result<usize> {
        CachedStorage::len(self)
    }

    fn list_objects(&self) -> Result<Vec<ObjectId>> {
        CachedStorage::list_objects(self)
    }

    fn batch_get(&self, ids: &[ObjectId]) -> Result<Vec<Option<GitObject>>> {
        CachedStorage::batch_get(self, ids)
    }

    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ObjectStore;

    #[test]
    fn test_cache_hit() {
        let store = ObjectStore::new();
        let cached = CachedStorage::with_defaults(store);

        let obj = GitObject::blob(b"test data".to_vec());
        let id = cached.put(obj).unwrap();

        // First get should hit cache
        let result = cached.get(&id).unwrap();
        assert!(result.is_some());

        let stats = cached.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss_then_hit() {
        let store = ObjectStore::new();

        // Put directly in store
        let obj = GitObject::blob(b"test data".to_vec());
        let id = store.put(obj);

        let cached = CachedStorage::with_defaults(store);

        // First get - cache miss
        let result = cached.get(&id).unwrap();
        assert!(result.is_some());

        // Second get - cache hit
        let result = cached.get(&id).unwrap();
        assert!(result.is_some());

        let stats = cached.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let store = ObjectStore::new();
        let cached = CachedStorage::with_defaults(store);

        let obj = GitObject::blob(b"test data".to_vec());
        let id = cached.put(obj).unwrap();

        // Invalidate
        cached.invalidate(&id);

        // Next get should be a miss
        let _ = cached.get(&id).unwrap();

        let stats = cached.stats();
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            max_objects: 2,
            max_size_bytes: 50, // Small size to trigger eviction
            write_through: true,
            ttl_seconds: 0,
        };
        let store = ObjectStore::new();
        let cached = CachedStorage::new(store, config);

        // Add 3 objects (each ~20 bytes, so 3rd will exceed 50 byte limit)
        for i in 0..3 {
            let obj = GitObject::blob(format!("data-{}-padding", i).into_bytes());
            cached.put(obj).unwrap();
        }

        let stats = cached.stats();
        assert!(stats.size <= 2); // Should have evicted at least one
    }

    #[test]
    fn test_cache_clear() {
        let store = ObjectStore::new();
        let cached = CachedStorage::with_defaults(store);

        let obj = GitObject::blob(b"test data".to_vec());
        cached.put(obj).unwrap();

        cached.clear();

        let stats = cached.stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_hit_ratio() {
        let stats = CacheStats {
            hits: 8,
            misses: 2,
            evictions: 0,
            size: 10,
            memory_bytes: 1000,
        };
        assert!((stats.hit_ratio() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_hit_ratio_zero_total() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_ratio(), 0.0);
    }

    // Helper implementation for tests
    impl crate::traits::ObjectStoreBackend for ObjectStore {
        fn put(&self, object: GitObject) -> Result<ObjectId> {
            Ok(ObjectStore::put(self, object))
        }

        fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
            match ObjectStore::get(self, id) {
                Ok(obj) => Ok(Some(obj)),
                Err(StorageError::ObjectNotFound(_)) => Ok(None),
                Err(e) => Err(e),
            }
        }

        fn contains(&self, id: &ObjectId) -> Result<bool> {
            Ok(ObjectStore::contains(self, id))
        }

        fn delete(&self, _id: &ObjectId) -> Result<bool> {
            Ok(false) // Not implemented for basic ObjectStore
        }

        fn len(&self) -> Result<usize> {
            Ok(ObjectStore::len(self))
        }

        fn list_objects(&self) -> Result<Vec<ObjectId>> {
            Ok(ObjectStore::list_objects(self))
        }
    }
}
