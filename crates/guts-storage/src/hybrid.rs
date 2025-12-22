//! Hybrid storage with hot/cold tiering.
//!
//! Combines in-memory storage for hot data with persistent storage
//! for cold data, automatically migrating objects based on access patterns.

use crate::{CacheConfig, CachedStorage, GitObject, ObjectId, ObjectStore, Result, StorageError};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Configuration for hybrid storage.
#[derive(Debug, Clone)]
pub struct HybridConfig {
    /// Maximum number of objects in hot storage.
    pub hot_max_objects: usize,
    /// Maximum size in bytes for hot storage.
    pub hot_max_bytes: usize,
    /// Cache configuration for the warm layer.
    pub cache_config: CacheConfig,
    /// Threshold for promoting objects to hot storage.
    pub promote_threshold: u32,
    /// Interval for background migration (seconds).
    pub migration_interval_secs: u64,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            hot_max_objects: 10_000,
            hot_max_bytes: 512 * 1024 * 1024, // 512 MB
            cache_config: CacheConfig::default(),
            promote_threshold: 3,
            migration_interval_secs: 60,
        }
    }
}

/// Access tracking for objects.
#[derive(Debug, Default)]
struct AccessTracker {
    /// Access counts per object.
    counts: RwLock<std::collections::HashMap<ObjectId, u32>>,
    /// Total access count.
    total_accesses: AtomicU64,
}

impl AccessTracker {
    fn record_access(&self, id: &ObjectId) -> u32 {
        self.total_accesses.fetch_add(1, Ordering::Relaxed);
        let mut counts = self.counts.write();
        let count = counts.entry(*id).or_insert(0);
        *count += 1;
        *count
    }

    fn get_count(&self, id: &ObjectId) -> u32 {
        self.counts.read().get(id).copied().unwrap_or(0)
    }

    fn reset(&self, id: &ObjectId) {
        self.counts.write().remove(id);
    }
}

/// Hybrid storage combining hot in-memory and cold persistent storage.
pub struct HybridStorage<C> {
    /// Hot storage (in-memory, frequently accessed).
    hot: Arc<ObjectStore>,
    /// Cold storage (persistent).
    cold: Arc<C>,
    /// Cache layer on top of cold storage.
    cache: CachedStorage<Arc<C>>,
    /// Set of object IDs in hot storage.
    hot_objects: RwLock<HashSet<ObjectId>>,
    /// Current size of hot storage in bytes.
    hot_size: AtomicU64,
    /// Access tracker.
    tracker: AccessTracker,
    /// Configuration.
    config: HybridConfig,
    /// Statistics.
    stats: HybridStats,
}

/// Hybrid storage statistics.
#[derive(Debug, Default)]
struct HybridStats {
    hot_hits: AtomicU64,
    hot_misses: AtomicU64,
    promotions: AtomicU64,
    demotions: AtomicU64,
}

impl<C> HybridStorage<C>
where
    C: crate::traits::ObjectStoreBackend + Send + Sync + 'static,
{
    /// Creates a new hybrid storage.
    pub fn new(cold: C, config: HybridConfig) -> Self {
        let cold = Arc::new(cold);
        let cache = CachedStorage::new(Arc::clone(&cold), config.cache_config.clone());

        Self {
            hot: Arc::new(ObjectStore::new()),
            cold,
            cache,
            hot_objects: RwLock::new(HashSet::new()),
            hot_size: AtomicU64::new(0),
            tracker: AccessTracker::default(),
            config,
            stats: HybridStats::default(),
        }
    }

    /// Creates with default configuration.
    pub fn with_defaults(cold: C) -> Self {
        Self::new(cold, HybridConfig::default())
    }

    /// Gets an object, checking hot storage first.
    pub fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
        // Track access
        let access_count = self.tracker.record_access(id);

        // Check hot storage first
        if self.hot_objects.read().contains(id) {
            self.stats.hot_hits.fetch_add(1, Ordering::Relaxed);
            match self.hot.get(id) {
                Ok(obj) => return Ok(Some(obj)),
                Err(StorageError::ObjectNotFound(_)) => {
                    // Object was evicted, continue to cold
                }
                Err(e) => return Err(e),
            }
        }

        self.stats.hot_misses.fetch_add(1, Ordering::Relaxed);

        // Check cold storage (through cache)
        let result = self.cache.get(id)?;

        // Consider promotion if frequently accessed
        if let Some(ref obj) = result {
            if access_count >= self.config.promote_threshold {
                self.try_promote(obj.clone());
            }
        }

        Ok(result)
    }

    /// Puts an object (always goes to hot first, then cold).
    pub fn put(&self, object: GitObject) -> Result<ObjectId> {
        let size = object.data.len() as u64;
        let id = object.id;

        // Always write to cold for durability
        self.cold.put(object.clone())?;

        // Try to add to hot storage
        if self.can_add_to_hot(size) {
            self.hot.put(object);
            self.hot_objects.write().insert(id);
            self.hot_size.fetch_add(size, Ordering::Relaxed);
        }

        Ok(id)
    }

    /// Checks if an object exists.
    pub fn contains(&self, id: &ObjectId) -> Result<bool> {
        if self.hot_objects.read().contains(id) {
            return Ok(true);
        }
        self.cold.contains(id)
    }

    /// Deletes an object from all tiers.
    pub fn delete(&self, id: &ObjectId) -> Result<bool> {
        // Remove from hot
        if self.hot_objects.write().remove(id) {
            if let Ok(obj) = self.hot.get(id) {
                self.hot_size
                    .fetch_sub(obj.data.len() as u64, Ordering::Relaxed);
            }
        }

        // Remove from cache
        self.cache.invalidate(id);

        // Remove from cold
        self.cold.delete(id)
    }

    /// Returns the total number of objects.
    pub fn len(&self) -> Result<usize> {
        self.cold.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> Result<bool> {
        self.cold.is_empty()
    }

    /// Lists all object IDs.
    pub fn list_objects(&self) -> Result<Vec<ObjectId>> {
        self.cold.list_objects()
    }

    /// Flushes hot storage to cold.
    pub fn flush(&self) -> Result<()> {
        // Flush cold storage
        self.cold.flush()
    }

    /// Checks if we can add an object to hot storage.
    fn can_add_to_hot(&self, size: u64) -> bool {
        let current_size = self.hot_size.load(Ordering::Relaxed);
        let current_count = self.hot_objects.read().len();

        current_count < self.config.hot_max_objects
            && current_size + size <= self.config.hot_max_bytes as u64
    }

    /// Tries to promote an object to hot storage.
    fn try_promote(&self, object: GitObject) {
        let size = object.data.len() as u64;
        let id = object.id;

        // Evict if needed
        while !self.can_add_to_hot(size) {
            if !self.evict_one() {
                return; // Can't evict, give up
            }
        }

        self.hot.put(object);
        self.hot_objects.write().insert(id);
        self.hot_size.fetch_add(size, Ordering::Relaxed);
        self.stats.promotions.fetch_add(1, Ordering::Relaxed);
    }

    /// Evicts one object from hot storage (LRU based on access count).
    fn evict_one(&self) -> bool {
        let hot_objects = self.hot_objects.read();
        if hot_objects.is_empty() {
            return false;
        }

        // Find object with lowest access count
        let victim = hot_objects
            .iter()
            .min_by_key(|id| self.tracker.get_count(id))
            .copied();

        drop(hot_objects);

        if let Some(victim_id) = victim {
            if let Ok(obj) = self.hot.get(&victim_id) {
                let size = obj.data.len() as u64;
                self.hot_objects.write().remove(&victim_id);
                self.hot_size.fetch_sub(size, Ordering::Relaxed);
                self.tracker.reset(&victim_id);
                self.stats.demotions.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }

        false
    }

    /// Returns storage statistics.
    pub fn stats(&self) -> HybridStatsSnapshot {
        HybridStatsSnapshot {
            hot_objects: self.hot_objects.read().len(),
            hot_size_bytes: self.hot_size.load(Ordering::Relaxed),
            hot_hits: self.stats.hot_hits.load(Ordering::Relaxed),
            hot_misses: self.stats.hot_misses.load(Ordering::Relaxed),
            promotions: self.stats.promotions.load(Ordering::Relaxed),
            demotions: self.stats.demotions.load(Ordering::Relaxed),
            cache_stats: self.cache.stats(),
        }
    }
}

/// Snapshot of hybrid storage statistics.
#[derive(Debug, Clone)]
pub struct HybridStatsSnapshot {
    pub hot_objects: usize,
    pub hot_size_bytes: u64,
    pub hot_hits: u64,
    pub hot_misses: u64,
    pub promotions: u64,
    pub demotions: u64,
    pub cache_stats: crate::CacheStats,
}

impl HybridStatsSnapshot {
    /// Returns the hot storage hit ratio.
    pub fn hot_hit_ratio(&self) -> f64 {
        let total = self.hot_hits + self.hot_misses;
        if total == 0 {
            0.0
        } else {
            self.hot_hits as f64 / total as f64
        }
    }
}

// Implement ObjectStoreBackend for HybridStorage
impl<C> crate::traits::ObjectStoreBackend for HybridStorage<C>
where
    C: crate::traits::ObjectStoreBackend + Send + Sync + 'static,
{
    fn put(&self, object: GitObject) -> Result<ObjectId> {
        HybridStorage::put(self, object)
    }

    fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
        HybridStorage::get(self, id)
    }

    fn contains(&self, id: &ObjectId) -> Result<bool> {
        HybridStorage::contains(self, id)
    }

    fn delete(&self, id: &ObjectId) -> Result<bool> {
        HybridStorage::delete(self, id)
    }

    fn len(&self) -> Result<usize> {
        HybridStorage::len(self)
    }

    fn list_objects(&self) -> Result<Vec<ObjectId>> {
        HybridStorage::list_objects(self)
    }

    fn flush(&self) -> Result<()> {
        HybridStorage::flush(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ObjectStoreBackend;

    // Simple in-memory cold storage for testing
    struct MemoryCold {
        store: ObjectStore,
    }

    impl MemoryCold {
        fn new() -> Self {
            Self {
                store: ObjectStore::new(),
            }
        }
    }

    impl crate::traits::ObjectStoreBackend for MemoryCold {
        fn put(&self, object: GitObject) -> Result<ObjectId> {
            Ok(self.store.put(object))
        }

        fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
            match self.store.get(id) {
                Ok(obj) => Ok(Some(obj)),
                Err(StorageError::ObjectNotFound(_)) => Ok(None),
                Err(e) => Err(e),
            }
        }

        fn contains(&self, id: &ObjectId) -> Result<bool> {
            Ok(self.store.contains(id))
        }

        fn delete(&self, _id: &ObjectId) -> Result<bool> {
            Ok(false)
        }

        fn len(&self) -> Result<usize> {
            Ok(self.store.len())
        }

        fn list_objects(&self) -> Result<Vec<ObjectId>> {
            Ok(self.store.list_objects())
        }
    }

    #[test]
    fn test_hybrid_put_get() {
        let cold = MemoryCold::new();
        let hybrid = HybridStorage::with_defaults(cold);

        let obj = GitObject::blob(b"test data".to_vec());
        let id = hybrid.put(obj.clone()).unwrap();

        let retrieved = hybrid.get(&id).unwrap().unwrap();
        assert_eq!(retrieved.id, obj.id);
    }

    #[test]
    fn test_hot_storage_hit() {
        let cold = MemoryCold::new();
        let hybrid = HybridStorage::with_defaults(cold);

        let obj = GitObject::blob(b"hot data".to_vec());
        let id = hybrid.put(obj).unwrap();

        // Should be in hot storage
        assert!(hybrid.hot_objects.read().contains(&id));

        // Get should hit hot storage
        hybrid.get(&id).unwrap();

        let stats = hybrid.stats();
        assert_eq!(stats.hot_hits, 1);
    }

    #[test]
    fn test_promotion() {
        let config = HybridConfig {
            promote_threshold: 2,
            ..Default::default()
        };
        let cold = MemoryCold::new();
        let hybrid = HybridStorage::new(cold, config);

        // Put object directly in cold (bypass hybrid)
        let obj = GitObject::blob(b"promote me".to_vec());
        hybrid.cold.put(obj.clone()).unwrap();

        // First access - not promoted
        hybrid.get(&obj.id).unwrap();
        assert!(!hybrid.hot_objects.read().contains(&obj.id));

        // Second access - should be promoted
        hybrid.get(&obj.id).unwrap();
        assert!(hybrid.hot_objects.read().contains(&obj.id));

        let stats = hybrid.stats();
        assert_eq!(stats.promotions, 1);
    }

    #[test]
    fn test_eviction() {
        let config = HybridConfig {
            hot_max_objects: 2,
            hot_max_bytes: 30, // Small size to trigger eviction
            ..Default::default()
        };
        let cold = MemoryCold::new();
        let hybrid = HybridStorage::new(cold, config);

        // Add 3 objects (each ~15 bytes, so 3rd will exceed limit)
        for i in 0..3 {
            let obj = GitObject::blob(format!("data-{}-pad", i).into_bytes());
            hybrid.put(obj).unwrap();
        }

        // Should have limited capacity in hot storage
        assert!(hybrid.hot_objects.read().len() <= 2);
    }

    #[test]
    fn test_stats() {
        let cold = MemoryCold::new();
        let hybrid = HybridStorage::with_defaults(cold);

        let obj = GitObject::blob(b"test".to_vec());
        let id = hybrid.put(obj).unwrap();

        // Multiple accesses
        for _ in 0..5 {
            hybrid.get(&id).unwrap();
        }

        let stats = hybrid.stats();
        assert!(stats.hot_objects > 0);
        assert!(stats.hot_hits > 0);
    }
}
