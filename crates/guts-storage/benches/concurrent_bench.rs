//! Concurrent access benchmarks for Guts storage.
//!
//! Measures performance under concurrent read/write workloads
//! to validate storage layer scalability.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use guts_storage::{GitObject, ObjectStore};
use std::sync::Arc;
use std::thread;

/// Benchmark concurrent reads
fn bench_concurrent_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_reads");

    for num_threads in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            num_threads,
            |b, &num_threads| {
                let store = Arc::new(ObjectStore::new());

                // Pre-populate with 1000 objects
                let ids: Vec<_> = (0..1000)
                    .map(|i| {
                        let obj = GitObject::blob(format!("object-{}", i).into_bytes());
                        store.put(obj)
                    })
                    .collect();
                let ids = Arc::new(ids);

                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|t| {
                            let store = Arc::clone(&store);
                            let ids = Arc::clone(&ids);
                            thread::spawn(move || {
                                for i in 0..100 {
                                    let idx = (t * 100 + i) % 1000;
                                    black_box(store.get(&ids[idx]).unwrap());
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent writes
fn bench_concurrent_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_writes");

    for num_threads in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let store = Arc::new(ObjectStore::new());

                    let handles: Vec<_> = (0..num_threads)
                        .map(|t| {
                            let store = Arc::clone(&store);
                            thread::spawn(move || {
                                for i in 0..100 {
                                    let obj =
                                        GitObject::blob(format!("object-{}-{}", t, i).into_bytes());
                                    black_box(store.put(obj));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark mixed read/write workload
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    // 80% reads, 20% writes - typical production workload
    for num_threads in [4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("80_20_threads", num_threads),
            num_threads,
            |b, &num_threads| {
                let store = Arc::new(ObjectStore::new());

                // Pre-populate
                let ids: Vec<_> = (0..1000)
                    .map(|i| {
                        let obj = GitObject::blob(format!("initial-{}", i).into_bytes());
                        store.put(obj)
                    })
                    .collect();
                let ids = Arc::new(parking_lot::RwLock::new(ids));

                b.iter(|| {
                    let reader_count = (num_threads * 4) / 5;
                    let writer_count = num_threads - reader_count;

                    let mut handles = Vec::new();

                    // Spawn readers
                    for _ in 0..reader_count {
                        let store = Arc::clone(&store);
                        let ids = Arc::clone(&ids);
                        handles.push(thread::spawn(move || {
                            for i in 0..100 {
                                let ids_read = ids.read();
                                let idx = i % ids_read.len();
                                black_box(store.get(&ids_read[idx]).unwrap());
                            }
                        }));
                    }

                    // Spawn writers
                    for t in 0..writer_count {
                        let store = Arc::clone(&store);
                        let ids = Arc::clone(&ids);
                        handles.push(thread::spawn(move || {
                            for i in 0..20 {
                                let obj = GitObject::blob(format!("new-{}-{}", t, i).into_bytes());
                                let id = store.put(obj);
                                ids.write().push(id);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark high contention scenario
fn bench_high_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("high_contention");

    // All threads accessing the same small set of objects
    for num_threads in [4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            num_threads,
            |b, &num_threads| {
                let store = Arc::new(ObjectStore::new());

                // Only 10 objects - high contention
                let ids: Vec<_> = (0..10)
                    .map(|i| {
                        let obj = GitObject::blob(format!("hot-object-{}", i).into_bytes());
                        store.put(obj)
                    })
                    .collect();
                let ids = Arc::new(ids);

                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let store = Arc::clone(&store);
                            let ids = Arc::clone(&ids);
                            thread::spawn(move || {
                                for i in 0..1000 {
                                    let idx = i % 10;
                                    black_box(store.get(&ids[idx]).unwrap());
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_concurrent_reads,
    bench_concurrent_writes,
    bench_mixed_workload,
    bench_high_contention,
);

criterion_main!(benches);
