//! Storage benchmarks for Guts.
//!
//! Benchmarks critical storage operations including:
//! - Object read/write at various sizes
//! - Compression/decompression performance
//! - Repository operations
//! - Reference operations

use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use guts_storage::{GitObject, ObjectStore, ObjectType, RepoStore};

/// Generate test data of specified size
fn generate_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Benchmark object store write operations
fn bench_object_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_store_write");

    // Test various object sizes: 1KB, 10KB, 100KB, 1MB
    for size in [1_024, 10_240, 102_400, 1_048_576].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("put", size), size, |b, &size| {
            let store = ObjectStore::new();
            let data = generate_data(size);

            b.iter(|| {
                let obj = GitObject::blob(data.clone());
                black_box(store.put(obj))
            });
        });
    }

    group.finish();
}

/// Benchmark object store read operations
fn bench_object_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_store_read");

    for size in [1_024, 10_240, 102_400, 1_048_576].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("get", size), size, |b, &size| {
            let store = ObjectStore::new();
            let data = generate_data(size);
            let obj = GitObject::blob(data);
            let id = store.put(obj);

            b.iter(|| black_box(store.get(&id).unwrap()));
        });
    }

    group.finish();
}

/// Benchmark object compression
fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    for size in [1_024, 10_240, 102_400, 1_048_576].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        // Compress benchmark
        group.bench_with_input(BenchmarkId::new("compress", size), size, |b, &size| {
            let data = generate_data(size);
            let obj = GitObject::blob(data);

            b.iter(|| black_box(ObjectStore::compress(&obj).unwrap()));
        });

        // Decompress benchmark
        group.bench_with_input(BenchmarkId::new("decompress", size), size, |b, &size| {
            let data = generate_data(size);
            let obj = GitObject::blob(data);
            let compressed = ObjectStore::compress(&obj).unwrap();

            b.iter(|| black_box(ObjectStore::decompress(&compressed).unwrap()));
        });
    }

    group.finish();
}

/// Benchmark object hashing
fn bench_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashing");

    for size in [1_024, 10_240, 102_400, 1_048_576].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("sha1", size), size, |b, &size| {
            let data = generate_data(size);

            b.iter(|| {
                let obj = GitObject::new(ObjectType::Blob, Bytes::from(data.clone()));
                black_box(obj.id)
            });
        });
    }

    group.finish();
}

/// Benchmark repository operations
fn bench_repository(c: &mut Criterion) {
    let mut group = c.benchmark_group("repository");

    // Repository creation
    group.bench_function("create", |b| {
        let store = RepoStore::new();
        let mut counter = 0u64;

        b.iter(|| {
            counter += 1;
            black_box(store.create(&format!("repo-{}", counter), "owner").unwrap())
        });
    });

    // Repository lookup
    group.bench_function("get", |b| {
        let store = RepoStore::new();
        store.create("test-repo", "owner").unwrap();

        b.iter(|| black_box(store.get("owner", "test-repo").unwrap()));
    });

    // List repositories
    group.bench_function("list_100_repos", |b| {
        let store = RepoStore::new();
        for i in 0..100 {
            store.create(&format!("repo-{}", i), "owner").unwrap();
        }

        b.iter(|| black_box(store.list()));
    });

    group.finish();
}

/// Benchmark object lookup with various store sizes
fn bench_lookup_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_scaling");

    for count in [100, 1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("get_from_n_objects", count),
            count,
            |b, &count| {
                let store = ObjectStore::new();

                // Pre-populate store
                let mut ids = Vec::with_capacity(count);
                for i in 0..count {
                    let data = format!("object-{}", i);
                    let obj = GitObject::blob(data.into_bytes());
                    ids.push(store.put(obj));
                }

                // Lookup random objects
                let mut idx = 0;
                b.iter(|| {
                    idx = (idx + 1) % count;
                    black_box(store.get(&ids[idx]).unwrap())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch operations
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_put", batch_size),
            batch_size,
            |b, &batch_size| {
                let store = ObjectStore::new();
                let objects: Vec<_> = (0..batch_size)
                    .map(|i| GitObject::blob(format!("blob-{}", i).into_bytes()))
                    .collect();

                b.iter(|| {
                    for obj in objects.clone() {
                        store.put(obj);
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_object_write,
    bench_object_read,
    bench_compression,
    bench_hashing,
    bench_repository,
    bench_lookup_scaling,
    bench_batch_operations,
);

criterion_main!(benches);
