//! RocksDB persistent storage backend.
//!
//! Provides durable, high-performance storage using RocksDB.
//! Supports column families, batch operations, and configurable
//! compression and caching.

use crate::{GitObject, ObjectId, ObjectType, Result, StorageError};
use bytes::Bytes;
use rocksdb::{
    BlockBasedOptions, ColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded,
    Options, WriteBatch, WriteOptions, DB,
};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// RocksDB storage configuration.
#[derive(Debug, Clone)]
pub struct RocksDbConfig {
    /// Path to the database directory.
    pub path: std::path::PathBuf,

    /// Write buffer size in bytes.
    pub write_buffer_size: usize,

    /// Maximum number of write buffers.
    pub max_write_buffers: i32,

    /// Target file size for SST files.
    pub target_file_size: u64,

    /// Number of background compaction threads.
    pub background_jobs: i32,

    /// Enable write-ahead logging.
    pub wal_enabled: bool,

    /// Enable LZ4 compression.
    pub compression_enabled: bool,

    /// Block cache size in bytes.
    pub block_cache_size: usize,

    /// Bloom filter bits per key (0 to disable).
    pub bloom_filter_bits: i32,
}

impl Default for RocksDbConfig {
    fn default() -> Self {
        Self {
            path: std::path::PathBuf::from("./data/rocksdb"),
            write_buffer_size: 64 * 1024 * 1024, // 64 MB
            max_write_buffers: 3,
            target_file_size: 64 * 1024 * 1024, // 64 MB
            background_jobs: 4,
            wal_enabled: true,
            compression_enabled: true,
            block_cache_size: 128 * 1024 * 1024, // 128 MB
            bloom_filter_bits: 10,
        }
    }
}

/// Column family names.
const CF_OBJECTS: &str = "objects";
const CF_REFS: &str = "refs";
const CF_METADATA: &str = "metadata";

/// RocksDB persistent storage.
pub struct RocksDbStorage {
    /// The RocksDB instance.
    db: DBWithThreadMode<MultiThreaded>,

    /// Configuration.
    #[allow(dead_code)]
    config: RocksDbConfig,

    /// Statistics.
    stats: RocksDbStats,
}

/// RocksDB statistics.
#[derive(Debug, Default)]
struct RocksDbStats {
    reads: AtomicU64,
    writes: AtomicU64,
    deletes: AtomicU64,
    batch_writes: AtomicU64,
}

impl RocksDbStorage {
    /// Opens or creates a RocksDB database.
    pub fn open(config: RocksDbConfig) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Performance tuning
        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffers);
        opts.set_target_file_size_base(config.target_file_size);
        opts.increase_parallelism(config.background_jobs);
        opts.set_max_background_jobs(config.background_jobs);

        // Compression
        if config.compression_enabled {
            opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        }

        // Block-based options
        let mut block_opts = BlockBasedOptions::default();
        if config.bloom_filter_bits > 0 {
            block_opts.set_bloom_filter(config.bloom_filter_bits as f64, false);
        }
        block_opts.set_cache_index_and_filter_blocks(true);
        opts.set_block_based_table_factory(&block_opts);

        // Column family options
        let cf_opts = opts.clone();

        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_OBJECTS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_REFS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_METADATA, cf_opts),
        ];

        let db = DB::open_cf_descriptors(&opts, &config.path, cfs)
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?;

        Ok(Self {
            db,
            config,
            stats: RocksDbStats::default(),
        })
    }

    /// Opens with default configuration.
    pub fn open_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open(RocksDbConfig {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        })
    }

    /// Gets the objects column family.
    fn objects_cf(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_OBJECTS).unwrap()
    }

    /// Gets the refs column family.
    fn refs_cf(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_REFS).unwrap()
    }

    /// Gets the metadata column family.
    #[allow(dead_code)]
    fn metadata_cf(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_METADATA).unwrap()
    }

    /// Stores a git object.
    pub fn put(&self, object: GitObject) -> Result<ObjectId> {
        let id = object.id;
        let data = Self::serialize_object(&object)?;

        self.db
            .put_cf(self.objects_cf(), id.as_bytes(), &data)
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?;

        self.stats.writes.fetch_add(1, Ordering::Relaxed);
        Ok(id)
    }

    /// Retrieves a git object by ID.
    pub fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
        self.stats.reads.fetch_add(1, Ordering::Relaxed);

        match self
            .db
            .get_cf(self.objects_cf(), id.as_bytes())
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?
        {
            Some(data) => Ok(Some(Self::deserialize_object(&data)?)),
            None => Ok(None),
        }
    }

    /// Checks if an object exists.
    pub fn contains(&self, id: &ObjectId) -> Result<bool> {
        self.stats.reads.fetch_add(1, Ordering::Relaxed);

        self.db
            .get_pinned_cf(self.objects_cf(), id.as_bytes())
            .map(|opt| opt.is_some())
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))
    }

    /// Deletes an object.
    pub fn delete(&self, id: &ObjectId) -> Result<bool> {
        let existed = self.contains(id)?;

        self.db
            .delete_cf(self.objects_cf(), id.as_bytes())
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?;

        self.stats.deletes.fetch_add(1, Ordering::Relaxed);
        Ok(existed)
    }

    /// Batch write operation.
    pub fn batch_put(&self, objects: Vec<GitObject>) -> Result<Vec<ObjectId>> {
        let mut batch = WriteBatch::default();
        let mut ids = Vec::with_capacity(objects.len());

        for object in objects {
            let id = object.id;
            let data = Self::serialize_object(&object)?;
            batch.put_cf(self.objects_cf(), id.as_bytes(), &data);
            ids.push(id);
        }

        let mut write_opts = WriteOptions::default();
        write_opts.set_sync(false);

        self.db
            .write_opt(batch, &write_opts)
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?;

        self.stats.batch_writes.fetch_add(1, Ordering::Relaxed);
        self.stats
            .writes
            .fetch_add(ids.len() as u64, Ordering::Relaxed);

        Ok(ids)
    }

    /// Stores a reference.
    pub fn set_ref(&self, name: &str, target: &ObjectId) -> Result<()> {
        self.db
            .put_cf(self.refs_cf(), name.as_bytes(), target.as_bytes())
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))
    }

    /// Gets a reference.
    pub fn get_ref(&self, name: &str) -> Result<Option<ObjectId>> {
        match self
            .db
            .get_cf(self.refs_cf(), name.as_bytes())
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?
        {
            Some(data) => {
                if data.len() != 20 {
                    return Err(StorageError::InvalidRef("invalid ref target".to_string()));
                }
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&data);
                Ok(Some(ObjectId::from_bytes(bytes)))
            }
            None => Ok(None),
        }
    }

    /// Deletes a reference.
    pub fn delete_ref(&self, name: &str) -> Result<()> {
        self.db
            .delete_cf(self.refs_cf(), name.as_bytes())
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))
    }

    /// Lists all references.
    pub fn list_refs(&self) -> Result<Vec<(String, ObjectId)>> {
        let mut refs = Vec::new();

        let iter = self
            .db
            .iterator_cf(self.refs_cf(), rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, value) =
                item.map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?;

            let name = String::from_utf8_lossy(&key).to_string();
            if value.len() == 20 {
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&value);
                refs.push((name, ObjectId::from_bytes(bytes)));
            }
        }

        Ok(refs)
    }

    /// Flushes all pending writes.
    pub fn flush(&self) -> Result<()> {
        self.db
            .flush()
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))
    }

    /// Compacts the database.
    pub fn compact(&self) -> Result<()> {
        self.db
            .compact_range_cf(self.objects_cf(), None::<&[u8]>, None::<&[u8]>);
        self.db
            .compact_range_cf(self.refs_cf(), None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }

    /// Returns the number of objects (approximate).
    pub fn len(&self) -> Result<usize> {
        // RocksDB doesn't have an efficient count, so we iterate
        let iter = self
            .db
            .iterator_cf(self.objects_cf(), rocksdb::IteratorMode::Start);
        Ok(iter.count())
    }

    /// Returns true if the database is empty.
    pub fn is_empty(&self) -> Result<bool> {
        let mut iter = self
            .db
            .iterator_cf(self.objects_cf(), rocksdb::IteratorMode::Start);
        Ok(iter.next().is_none())
    }

    /// Lists all object IDs.
    pub fn list_objects(&self) -> Result<Vec<ObjectId>> {
        let mut ids = Vec::new();

        let iter = self
            .db
            .iterator_cf(self.objects_cf(), rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, _) =
                item.map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))?;

            if key.len() == 20 {
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&key);
                ids.push(ObjectId::from_bytes(bytes));
            }
        }

        Ok(ids)
    }

    /// Serializes a git object for storage.
    fn serialize_object(object: &GitObject) -> Result<Vec<u8>> {
        // Format: type_byte | data
        let type_byte = object.object_type.pack_type();
        let mut buf = Vec::with_capacity(1 + object.data.len());
        buf.push(type_byte);
        buf.extend_from_slice(&object.data);
        Ok(buf)
    }

    /// Deserializes a git object from storage.
    fn deserialize_object(data: &[u8]) -> Result<GitObject> {
        if data.is_empty() {
            return Err(StorageError::InvalidObject("empty data".to_string()));
        }

        let type_byte = data[0];
        let object_type = ObjectType::from_pack_type(type_byte)?;
        let object_data = Bytes::copy_from_slice(&data[1..]);

        Ok(GitObject::new(object_type, object_data))
    }

    /// Returns storage statistics.
    pub fn stats(&self) -> RocksDbStatsSnapshot {
        RocksDbStatsSnapshot {
            reads: self.stats.reads.load(Ordering::Relaxed),
            writes: self.stats.writes.load(Ordering::Relaxed),
            deletes: self.stats.deletes.load(Ordering::Relaxed),
            batch_writes: self.stats.batch_writes.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of RocksDB statistics.
#[derive(Debug, Clone)]
pub struct RocksDbStatsSnapshot {
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub batch_writes: u64,
}

// Implement the ObjectStoreBackend trait
impl crate::traits::ObjectStoreBackend for RocksDbStorage {
    fn put(&self, object: GitObject) -> Result<ObjectId> {
        RocksDbStorage::put(self, object)
    }

    fn get(&self, id: &ObjectId) -> Result<Option<GitObject>> {
        RocksDbStorage::get(self, id)
    }

    fn contains(&self, id: &ObjectId) -> Result<bool> {
        RocksDbStorage::contains(self, id)
    }

    fn delete(&self, id: &ObjectId) -> Result<bool> {
        RocksDbStorage::delete(self, id)
    }

    fn len(&self) -> Result<usize> {
        RocksDbStorage::len(self)
    }

    fn list_objects(&self) -> Result<Vec<ObjectId>> {
        RocksDbStorage::list_objects(self)
    }

    fn batch_put(&self, objects: Vec<GitObject>) -> Result<Vec<ObjectId>> {
        RocksDbStorage::batch_put(self, objects)
    }

    fn flush(&self) -> Result<()> {
        RocksDbStorage::flush(self)
    }

    fn compact(&self) -> Result<()> {
        RocksDbStorage::compact(self)
    }
}

impl crate::traits::StorageBackend for RocksDbStorage {
    fn open(path: &Path) -> Result<Self> {
        RocksDbStorage::open_default(path)
    }

    fn stats(&self) -> crate::traits::StorageStats {
        let stats = self.stats();
        crate::traits::StorageStats {
            object_count: stats.writes.saturating_sub(stats.deletes),
            reads: stats.reads,
            writes: stats.writes,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (RocksDbStorage, TempDir) {
        let dir = TempDir::new().unwrap();
        let storage = RocksDbStorage::open_default(dir.path()).unwrap();
        (storage, dir)
    }

    #[test]
    fn test_put_get() {
        let (storage, _dir) = create_test_db();

        let obj = GitObject::blob(b"hello world".to_vec());
        let id = storage.put(obj.clone()).unwrap();

        let retrieved = storage.get(&id).unwrap().unwrap();
        assert_eq!(retrieved.id, obj.id);
        assert_eq!(retrieved.data, obj.data);
    }

    #[test]
    fn test_contains() {
        let (storage, _dir) = create_test_db();

        let obj = GitObject::blob(b"test".to_vec());
        let id = storage.put(obj).unwrap();

        assert!(storage.contains(&id).unwrap());

        let missing_id = ObjectId::from_bytes([0u8; 20]);
        assert!(!storage.contains(&missing_id).unwrap());
    }

    #[test]
    fn test_delete() {
        let (storage, _dir) = create_test_db();

        let obj = GitObject::blob(b"test".to_vec());
        let id = storage.put(obj).unwrap();

        assert!(storage.delete(&id).unwrap());
        assert!(!storage.contains(&id).unwrap());
    }

    #[test]
    fn test_batch_put() {
        let (storage, _dir) = create_test_db();

        let objects: Vec<_> = (0..100)
            .map(|i| GitObject::blob(format!("blob-{}", i).into_bytes()))
            .collect();

        let ids = storage.batch_put(objects.clone()).unwrap();
        assert_eq!(ids.len(), 100);

        for (id, obj) in ids.iter().zip(objects.iter()) {
            assert_eq!(*id, obj.id);
            assert!(storage.contains(id).unwrap());
        }
    }

    #[test]
    fn test_refs() {
        let (storage, _dir) = create_test_db();

        let obj = GitObject::blob(b"test".to_vec());
        let id = storage.put(obj).unwrap();

        storage.set_ref("refs/heads/main", &id).unwrap();

        let retrieved = storage.get_ref("refs/heads/main").unwrap().unwrap();
        assert_eq!(retrieved, id);

        let refs = storage.list_refs().unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0, "refs/heads/main");

        storage.delete_ref("refs/heads/main").unwrap();
        assert!(storage.get_ref("refs/heads/main").unwrap().is_none());
    }

    #[test]
    fn test_list_objects() {
        let (storage, _dir) = create_test_db();

        let objects: Vec<_> = (0..10)
            .map(|i| GitObject::blob(format!("blob-{}", i).into_bytes()))
            .collect();

        let expected_ids: Vec<_> = objects.iter().map(|o| o.id).collect();

        for obj in objects {
            storage.put(obj).unwrap();
        }

        let listed = storage.list_objects().unwrap();
        assert_eq!(listed.len(), 10);

        for id in &expected_ids {
            assert!(listed.contains(id));
        }
    }

    #[test]
    fn test_flush() {
        let (storage, _dir) = create_test_db();

        let obj = GitObject::blob(b"test".to_vec());
        storage.put(obj).unwrap();

        storage.flush().unwrap();
    }

    #[test]
    fn test_compact() {
        let (storage, _dir) = create_test_db();

        // Add and delete some objects
        for i in 0..100 {
            let obj = GitObject::blob(format!("blob-{}", i).into_bytes());
            let id = storage.put(obj).unwrap();
            if i % 2 == 0 {
                storage.delete(&id).unwrap();
            }
        }

        storage.compact().unwrap();
    }

    #[test]
    fn test_stats() {
        let (storage, _dir) = create_test_db();

        let obj = GitObject::blob(b"test".to_vec());
        let id = storage.put(obj).unwrap();

        storage.get(&id).unwrap();
        storage.get(&id).unwrap();

        let stats = storage.stats();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.reads, 2);
    }
}
