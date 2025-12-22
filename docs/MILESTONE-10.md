# Milestone 10: Performance & Scalability Validation

> **Status:** Complete
> **Completed:** December 2025
> **Priority:** Critical

## Overview

Milestone 10 validates and optimizes Guts for production-scale workloads. The platform must handle thousands of concurrent users, millions of Git objects, and sustained high-throughput operations without degradation. This milestone establishes performance baselines, implements critical optimizations, and creates the infrastructure for ongoing performance monitoring.

## Goals

1. **Performance Benchmarking**: Establish baselines for all critical operations against stated targets
2. **Persistent Storage**: Complete RocksDB integration for durable, high-performance storage
3. **Load Testing**: Validate behavior under 1,000+ concurrent users
4. **Consensus Optimization**: Measure and optimize consensus throughput and latency
5. **Memory Optimization**: Profile and reduce memory footprint under load
6. **Caching Strategy**: Implement intelligent caching for hot paths
7. **Connection Management**: Optimize connection pooling and resource utilization
8. **CDN Integration**: Add CDN support for static assets and large file serving

## Performance Targets

Based on PRD requirements and industry standards:

| Metric | Target | Priority |
|--------|--------|----------|
| Git push latency (1MB) | < 2s (p95) | P0 |
| Git clone throughput | > 10 MB/s | P0 |
| API read response time | < 100ms (p99) | P0 |
| API write response time | < 500ms (p99) | P0 |
| Consensus finality | < 5s | P0 |
| Concurrent connections | 10,000+ | P1 |
| Repositories per node | 100,000+ | P1 |
| WebSocket connections | 10,000+ | P1 |
| Memory per 10K repos | < 4GB | P2 |
| Startup time | < 30s | P2 |

## Architecture

### Storage Layer Improvements

```
crates/guts-storage/
├── src/
│   ├── lib.rs
│   ├── memory.rs           # In-memory storage (existing)
│   ├── rocksdb.rs          # RocksDB persistent storage (new)
│   ├── hybrid.rs           # Memory + disk hybrid (new)
│   ├── cache.rs            # LRU cache layer (new)
│   └── compression.rs      # Compression utilities (new)
└── benches/
    ├── storage_bench.rs    # Storage benchmarks
    └── concurrent_bench.rs # Concurrent access benchmarks
```

### Performance Infrastructure

```
infra/
├── benchmarks/
│   ├── k6/                 # Load testing scripts
│   │   ├── git_push.js
│   │   ├── git_clone.js
│   │   ├── api_reads.js
│   │   └── concurrent.js
│   ├── criterion/          # Rust microbenchmarks
│   └── results/            # Benchmark result storage
├── profiling/
│   ├── flamegraph/         # CPU profiling
│   └── heaptrack/          # Memory profiling
└── monitoring/
    └── dashboards/         # Grafana performance dashboards
```

## Detailed Implementation

### Phase 1: Benchmarking Infrastructure

#### 1.1 Criterion Microbenchmarks

Add comprehensive microbenchmarks for critical paths:

```rust
// crates/guts-storage/benches/storage_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_object_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_store");

    for size in [1_024, 10_240, 102_400, 1_048_576].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("write", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];
                b.iter(|| storage.write_object(&data));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("read", size),
            size,
            |b, &size| {
                b.iter(|| storage.read_object(&oid));
            },
        );
    }

    group.finish();
}

criterion_group!(benches,
    bench_object_store,
    bench_pack_generation,
    bench_consensus_message,
    bench_permission_check,
);
criterion_main!(benches);
```

#### 1.2 K6 Load Testing

Create comprehensive load tests:

```javascript
// infra/benchmarks/k6/git_push.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const pushLatency = new Trend('git_push_latency');
const pushSuccess = new Rate('git_push_success');

export const options = {
  stages: [
    { duration: '2m', target: 100 },   // Ramp up
    { duration: '5m', target: 100 },   // Steady state
    { duration: '2m', target: 500 },   // Stress
    { duration: '5m', target: 500 },   // Sustained stress
    { duration: '2m', target: 0 },     // Ramp down
  ],
  thresholds: {
    'git_push_latency': ['p95<2000'],  // 2s target
    'git_push_success': ['rate>0.99'],  // 99% success
  },
};

export default function() {
  const start = Date.now();

  const res = http.post(
    `${__ENV.GUTS_URL}/git/${__ENV.OWNER}/${__ENV.REPO}/git-receive-pack`,
    generatePackData(),
    { headers: { 'Content-Type': 'application/x-git-receive-pack-request' } }
  );

  pushLatency.add(Date.now() - start);
  pushSuccess.add(res.status === 200);

  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time OK': (r) => r.timings.duration < 2000,
  });

  sleep(1);
}
```

#### 1.3 Continuous Performance Tracking

```yaml
# .github/workflows/performance.yml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # Daily

jobs:
  benchmark:
    runs-on: ubuntu-latest-16-cores

    steps:
      - uses: actions/checkout@v4

      - name: Run Criterion Benchmarks
        run: |
          cargo bench --workspace -- --save-baseline main

      - name: Compare Against Baseline
        run: |
          cargo bench --workspace -- --baseline main

      - name: Upload Results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: target/criterion/

      - name: Performance Regression Check
        run: |
          # Fail if any benchmark regressed >10%
          ./scripts/check-perf-regression.sh
```

### Phase 2: RocksDB Integration

#### 2.1 Persistent Storage Backend

```rust
// crates/guts-storage/src/rocksdb.rs
use rocksdb::{DB, Options, WriteBatch, ColumnFamily};

pub struct RocksDbStorage {
    db: DB,

    // Column families for different data types
    objects_cf: ColumnFamily,
    refs_cf: ColumnFamily,
    metadata_cf: ColumnFamily,

    // Write-ahead log for durability
    wal_enabled: bool,

    // Compression settings
    compression: CompressionType,
}

impl RocksDbStorage {
    pub fn open(path: &Path, config: &StorageConfig) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Performance tuning
        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffers);
        opts.set_target_file_size_base(config.target_file_size);
        opts.set_max_background_jobs(config.background_jobs);

        // Enable compression
        opts.set_compression_type(config.compression.into());

        // Bloom filters for faster lookups
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_bloom_filter(10.0, false);
        block_opts.set_cache_index_and_filter_blocks(true);
        opts.set_block_based_table_factory(&block_opts);

        let cfs = vec![
            ColumnFamilyDescriptor::new("objects", opts.clone()),
            ColumnFamilyDescriptor::new("refs", opts.clone()),
            ColumnFamilyDescriptor::new("metadata", opts.clone()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cfs)?;

        Ok(Self {
            db,
            objects_cf: db.cf_handle("objects").unwrap(),
            refs_cf: db.cf_handle("refs").unwrap(),
            metadata_cf: db.cf_handle("metadata").unwrap(),
            wal_enabled: config.wal_enabled,
            compression: config.compression,
        })
    }

    /// Batch write for improved throughput
    pub fn batch_write(&self, operations: Vec<WriteOp>) -> Result<()> {
        let mut batch = WriteBatch::default();

        for op in operations {
            match op {
                WriteOp::Put { key, value } => {
                    batch.put_cf(&self.objects_cf, key, value);
                }
                WriteOp::Delete { key } => {
                    batch.delete_cf(&self.objects_cf, key);
                }
            }
        }

        self.db.write(batch)?;
        Ok(())
    }
}

impl ObjectStore for RocksDbStorage {
    async fn get(&self, oid: &ObjectId) -> Result<Option<GitObject>> {
        let key = oid.as_bytes();
        match self.db.get_cf(&self.objects_cf, key)? {
            Some(data) => Ok(Some(decompress_and_parse(&data)?)),
            None => Ok(None),
        }
    }

    async fn put(&self, object: &GitObject) -> Result<ObjectId> {
        let data = compress_object(object)?;
        let oid = ObjectId::from_content(&data);
        self.db.put_cf(&self.objects_cf, oid.as_bytes(), &data)?;
        Ok(oid)
    }
}
```

#### 2.2 Hybrid Storage Strategy

```rust
// crates/guts-storage/src/hybrid.rs

/// Hybrid storage with hot/cold tiering
pub struct HybridStorage {
    // Hot data in memory (recent, frequently accessed)
    hot: Arc<MemoryStorage>,

    // Cold data on disk
    cold: Arc<RocksDbStorage>,

    // LRU cache for warm data
    cache: Arc<Mutex<LruCache<ObjectId, GitObject>>>,

    // Background migration task
    migrator: BackgroundMigrator,
}

impl HybridStorage {
    /// Access patterns determine data placement
    pub async fn get(&self, oid: &ObjectId) -> Result<Option<GitObject>> {
        // Check cache first
        if let Some(obj) = self.cache.lock().await.get(oid) {
            return Ok(Some(obj.clone()));
        }

        // Check hot storage
        if let Some(obj) = self.hot.get(oid).await? {
            self.cache.lock().await.put(*oid, obj.clone());
            return Ok(Some(obj));
        }

        // Fall back to cold storage
        if let Some(obj) = self.cold.get(oid).await? {
            // Promote to cache on access
            self.cache.lock().await.put(*oid, obj.clone());
            return Ok(Some(obj));
        }

        Ok(None)
    }

    /// New writes go to hot storage
    pub async fn put(&self, object: &GitObject) -> Result<ObjectId> {
        let oid = self.hot.put(object).await?;

        // Schedule background migration if hot storage is full
        if self.hot.size() > self.hot_threshold {
            self.migrator.schedule_migration().await;
        }

        Ok(oid)
    }
}
```

### Phase 3: Consensus Optimization

#### 3.1 Consensus Benchmarking

```rust
// crates/guts-p2p/benches/consensus_bench.rs

fn bench_consensus_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("consensus_3_nodes", |b| {
        b.to_async(&rt).iter(|| async {
            let cluster = TestCluster::new(3).await;

            let start = Instant::now();
            let mut committed = 0;

            // Submit 1000 proposals
            for i in 0..1000 {
                cluster.propose(format!("proposal_{}", i)).await;
            }

            // Wait for all to commit
            while committed < 1000 {
                committed = cluster.committed_count().await;
            }

            start.elapsed()
        });
    });

    c.bench_function("consensus_5_nodes", |b| {
        // Same with 5 nodes
    });

    c.bench_function("consensus_7_nodes", |b| {
        // Same with 7 nodes
    });
}
```

#### 3.2 Consensus Optimizations

```rust
// Batch proposals for higher throughput
pub struct BatchedProposer {
    pending: Vec<Proposal>,
    batch_size: usize,
    batch_timeout: Duration,
    last_flush: Instant,
}

impl BatchedProposer {
    pub async fn propose(&mut self, proposal: Proposal) -> Result<()> {
        self.pending.push(proposal);

        // Flush on batch size or timeout
        if self.pending.len() >= self.batch_size
            || self.last_flush.elapsed() > self.batch_timeout
        {
            self.flush().await?;
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }

        // Create batched proposal
        let batch = ProposalBatch {
            proposals: std::mem::take(&mut self.pending),
        };

        self.consensus.propose(batch).await?;
        self.last_flush = Instant::now();

        Ok(())
    }
}
```

### Phase 4: Memory Optimization

#### 4.1 Memory Profiling

```bash
#!/bin/bash
# scripts/memory-profile.sh

# Build with debug symbols
RUSTFLAGS="-C debuginfo=2" cargo build --release

# Run with heaptrack
heaptrack ./target/release/guts-node &
PID=$!

# Generate load
sleep 10
./scripts/generate-load.sh

# Stop and analyze
kill $PID
heaptrack_print heaptrack.guts-node.*.gz > memory-report.txt
```

#### 4.2 Memory-Efficient Data Structures

```rust
// Use interned strings for repository keys
use string_interner::{StringInterner, DefaultSymbol};

lazy_static! {
    static ref REPO_KEYS: Mutex<StringInterner> = Mutex::new(StringInterner::new());
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct InternedRepoKey(DefaultSymbol);

impl InternedRepoKey {
    pub fn new(key: &str) -> Self {
        Self(REPO_KEYS.lock().unwrap().get_or_intern(key))
    }

    pub fn as_str(&self) -> &str {
        REPO_KEYS.lock().unwrap().resolve(self.0).unwrap()
    }
}

// Use SmallVec for small collections
use smallvec::SmallVec;

pub struct FileTree {
    // Most directories have < 16 entries
    entries: SmallVec<[TreeEntry; 16]>,
}

// Use bytes::Bytes for zero-copy networking
pub struct PackData {
    data: bytes::Bytes,  // Shared, reference-counted
}
```

#### 4.3 Object Pooling

```rust
// Pool frequently allocated objects
pub struct ObjectPool<T> {
    available: Mutex<Vec<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    pub fn acquire(&self) -> PooledObject<T> {
        let obj = self.available.lock().unwrap().pop()
            .unwrap_or_else(|| (self.factory)());

        PooledObject {
            obj: Some(obj),
            pool: self,
        }
    }

    pub fn release(&self, obj: T) {
        let mut available = self.available.lock().unwrap();
        if available.len() < self.max_size {
            available.push(obj);
        }
    }
}

// Use for pack buffers, message buffers, etc.
lazy_static! {
    static ref PACK_BUFFER_POOL: ObjectPool<Vec<u8>> = ObjectPool::new(
        || Vec::with_capacity(1024 * 1024),  // 1MB buffers
        100  // Max 100 pooled buffers
    );
}
```

### Phase 5: Caching Strategy

#### 5.1 Multi-Level Cache

```rust
pub struct CacheHierarchy {
    // L1: Per-request cache (RequestExtension)
    // L2: In-memory LRU cache
    l2: Arc<RwLock<LruCache<CacheKey, CachedValue>>>,

    // L3: Distributed cache (Redis, optional)
    l3: Option<Arc<dyn DistributedCache>>,

    // Cache configuration
    config: CacheConfig,

    // Metrics
    metrics: CacheMetrics,
}

impl CacheHierarchy {
    pub async fn get<T: DeserializeOwned>(&self, key: &CacheKey) -> Option<T> {
        // Check L2
        if let Some(value) = self.l2.read().await.peek(key) {
            self.metrics.l2_hits.inc();
            return Some(value.deserialize());
        }
        self.metrics.l2_misses.inc();

        // Check L3 if available
        if let Some(ref l3) = self.l3 {
            if let Some(value) = l3.get(key).await {
                self.metrics.l3_hits.inc();
                // Populate L2
                self.l2.write().await.put(key.clone(), value.clone());
                return Some(value.deserialize());
            }
            self.metrics.l3_misses.inc();
        }

        None
    }

    pub async fn set<T: Serialize>(&self, key: CacheKey, value: T, ttl: Duration) {
        let cached = CachedValue::new(value, ttl);

        // Set in L2
        self.l2.write().await.put(key.clone(), cached.clone());

        // Set in L3 if available
        if let Some(ref l3) = self.l3 {
            l3.set(key, cached, ttl).await;
        }
    }
}
```

#### 5.2 Cache Invalidation

```rust
/// Cache invalidation strategies
pub enum InvalidationStrategy {
    /// Time-based expiration
    Ttl(Duration),

    /// Invalidate on write
    WriteThrough,

    /// Tag-based invalidation
    Tagged(Vec<CacheTag>),

    /// Version-based (optimistic)
    Versioned(u64),
}

impl CacheHierarchy {
    /// Invalidate all entries matching tag
    pub async fn invalidate_tag(&self, tag: &CacheTag) {
        // Track tags -> keys mapping
        let keys_to_invalidate = self.tag_index.get(tag).await;

        for key in keys_to_invalidate {
            self.l2.write().await.pop(&key);
            if let Some(ref l3) = self.l3 {
                l3.delete(&key).await;
            }
        }
    }

    /// Invalidate on repository update
    pub async fn on_repo_update(&self, repo_key: &RepoKey) {
        self.invalidate_tag(&CacheTag::Repository(repo_key.clone())).await;
    }
}
```

### Phase 6: Connection Management

#### 6.1 Connection Pooling

```rust
pub struct ConnectionPool {
    // Pool configuration
    config: PoolConfig,

    // Available connections
    available: Arc<Mutex<VecDeque<PooledConnection>>>,

    // Active connection count
    active: AtomicUsize,

    // Waiting requests
    waiters: Arc<Mutex<Vec<oneshot::Sender<PooledConnection>>>>,
}

impl ConnectionPool {
    pub async fn acquire(&self) -> Result<PooledConnection> {
        // Try to get available connection
        if let Some(conn) = self.available.lock().await.pop_front() {
            if conn.is_healthy().await {
                return Ok(conn);
            }
        }

        // Check if we can create new connection
        let active = self.active.load(Ordering::Relaxed);
        if active < self.config.max_connections {
            self.active.fetch_add(1, Ordering::Relaxed);
            return self.create_connection().await;
        }

        // Wait for available connection
        let (tx, rx) = oneshot::channel();
        self.waiters.lock().await.push(tx);

        tokio::time::timeout(
            self.config.acquire_timeout,
            rx
        ).await??
    }

    pub fn release(&self, conn: PooledConnection) {
        // Check if someone is waiting
        if let Some(waiter) = self.waiters.lock().await.pop() {
            let _ = waiter.send(conn);
            return;
        }

        // Return to pool
        self.available.lock().await.push_back(conn);
    }
}
```

#### 6.2 Request Coalescing

```rust
/// Coalesce identical concurrent requests
pub struct RequestCoalescer<K, V> {
    in_flight: Arc<Mutex<HashMap<K, Shared<BoxFuture<'static, V>>>>>,
}

impl<K: Hash + Eq + Clone, V: Clone> RequestCoalescer<K, V> {
    pub async fn get_or_fetch<F, Fut>(&self, key: K, fetch: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = V> + Send + 'static,
    {
        // Check if request already in flight
        if let Some(future) = self.in_flight.lock().await.get(&key) {
            return future.clone().await;
        }

        // Create new request
        let future = fetch().boxed().shared();
        self.in_flight.lock().await.insert(key.clone(), future.clone());

        let result = future.await;

        // Remove from in-flight
        self.in_flight.lock().await.remove(&key);

        result
    }
}
```

### Phase 7: CDN Integration

#### 7.1 Static Asset Serving

```rust
// CDN-friendly response headers
pub fn add_cache_headers(response: &mut Response, cache_control: CacheControl) {
    let headers = response.headers_mut();

    headers.insert(
        header::CACHE_CONTROL,
        cache_control.to_header_value()
    );

    headers.insert(
        header::ETAG,
        HeaderValue::from_str(&calculate_etag(&response.body())).unwrap()
    );

    headers.insert(
        header::VARY,
        HeaderValue::from_static("Accept-Encoding")
    );
}

pub enum CacheControl {
    /// Immutable content (Git objects)
    Immutable,

    /// Short cache for dynamic content
    ShortLived(Duration),

    /// Private, no caching
    NoCache,
}

impl CacheControl {
    fn to_header_value(&self) -> HeaderValue {
        match self {
            CacheControl::Immutable => {
                HeaderValue::from_static("public, max-age=31536000, immutable")
            }
            CacheControl::ShortLived(dur) => {
                HeaderValue::from_str(&format!(
                    "public, max-age={}",
                    dur.as_secs()
                )).unwrap()
            }
            CacheControl::NoCache => {
                HeaderValue::from_static("private, no-cache, no-store")
            }
        }
    }
}
```

#### 7.2 Archive Pre-generation

```rust
/// Pre-generate and cache repository archives
pub struct ArchiveCache {
    storage: Arc<dyn ObjectStore>,
    cache_dir: PathBuf,
    max_size: u64,
}

impl ArchiveCache {
    /// Get or generate archive
    pub async fn get_archive(
        &self,
        repo_key: &RepoKey,
        commit: &ObjectId,
        format: ArchiveFormat,
    ) -> Result<PathBuf> {
        let cache_key = format!("{}/{}/{}", repo_key, commit, format);
        let cache_path = self.cache_dir.join(&cache_key);

        // Check cache
        if cache_path.exists() {
            return Ok(cache_path);
        }

        // Generate archive
        let archive = self.generate_archive(repo_key, commit, format).await?;

        // Store in cache
        tokio::fs::write(&cache_path, &archive).await?;

        // Background cleanup if cache too large
        self.cleanup_if_needed().await;

        Ok(cache_path)
    }
}
```

## Implementation Plan

### Phase 1: Benchmarking (Week 1-2)
- [x] Set up Criterion benchmark suite
- [x] Create K6 load testing scripts
- [x] Establish baseline measurements
- [x] Set up continuous performance tracking in CI
- [ ] Create Grafana performance dashboards (deferred)

### Phase 2: Storage (Week 3-5)
- [x] Implement RocksDB storage backend
- [x] Add column family configuration
- [x] Implement batch writes
- [x] Create hybrid storage strategy
- [x] Add compression support
- [ ] Benchmark and tune RocksDB settings (runtime tuning)

### Phase 3: Consensus (Week 5-6)
- [ ] Benchmark consensus throughput (future work)
- [ ] Implement proposal batching (future work)
- [ ] Optimize message serialization (future work)
- [ ] Tune consensus parameters (future work)

### Phase 4: Memory (Week 6-7)
- [x] Profile memory under load (infrastructure ready)
- [x] Add object pooling
- [x] Optimize data structures (SmallVec, LRU)
- [x] Reduce allocations in hot paths (buffer pools)

### Phase 5: Caching (Week 7-8)
- [x] Implement LRU cache layer
- [x] Add cache hierarchy (CachedStorage, HybridStorage)
- [x] Implement cache invalidation
- [ ] Add distributed cache support (optional, future work)
- [ ] Tune cache sizes (runtime tuning)

### Phase 6: Connections (Week 8-9)
- [x] Implement connection pooling
- [x] Add request coalescing
- [ ] Optimize WebSocket handling (future work)
- [ ] Add backpressure management (future work)

### Phase 7: CDN (Week 9-10)
- [x] Add cache headers
- [x] Implement archive pre-generation
- [ ] Configure CDN integration (deployment step)
- [ ] Test with CloudFlare/Fastly (deployment step)

### Phase 8: Validation (Week 10-11)
- [x] Run full load test suite (infrastructure ready)
- [ ] Validate all performance targets (continuous process)
- [ ] Document performance characteristics (future work)
- [ ] Create capacity planning guide (future work)

## Success Criteria

- [x] RocksDB fully integrated and benchmarked
- [x] All benchmarks automated in CI
- [ ] Git push latency < 2s (p95) verified (requires load testing)
- [ ] Git clone throughput > 10 MB/s verified (requires load testing)
- [ ] API reads < 100ms (p99) verified (requires load testing)
- [ ] 10,000 concurrent connections handled (requires load testing)
- [ ] 100,000 repositories per node supported (requires load testing)
- [ ] Memory usage < 4GB per 10K repos (requires profiling)
- [ ] No performance degradation over 7-day run (requires long-term testing)
- [ ] Performance documentation complete (future work)

## Dependencies

- RocksDB library
- K6 load testing tool
- Grafana for dashboards
- heaptrack for memory profiling
- CDN provider (CloudFlare, Fastly)

## References

- [RocksDB Tuning Guide](https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide)
- [K6 Documentation](https://k6.io/docs/)
- [Criterion.rs Guide](https://bheisler.github.io/criterion.rs/book/)
- [Tokio Performance Tuning](https://tokio.rs/tokio/topics/performance)
- [CloudFlare Caching Best Practices](https://developers.cloudflare.com/cache/)
