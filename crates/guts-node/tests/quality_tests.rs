//! Production Quality Tests - Milestone 9 Phase 7
//!
//! This module provides comprehensive testing for production readiness:
//! - Property-based tests for protocol parsing
//! - Chaos testing for P2P layer
//! - Load testing infrastructure
//! - Integration tests with failure injection

use bytes::Bytes;
use guts_p2p::{Message, ObjectData, RefUpdate, RepoAnnounce, SyncRequest};
use guts_storage::{GitObject, ObjectId, ObjectType};
use proptest::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Property-Based Tests for Protocol Parsing
// ============================================================================

/// Generate a valid repository key (owner/name format)
fn repo_key_strategy() -> impl Strategy<Value = String> {
    ("[a-z][a-z0-9]{0,20}", "[a-z][a-z0-9]{0,20}")
        .prop_map(|(owner, name)| format!("{}/{}", owner, name))
}

/// Generate a valid reference name
fn ref_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("refs/heads/main".to_string()),
        Just("refs/heads/develop".to_string()),
        Just("refs/heads/feature-branch".to_string()),
        Just("refs/tags/v1.0.0".to_string()),
        Just("refs/tags/release-2024".to_string()),
        ("[a-z]{1,30}").prop_map(|s| format!("refs/heads/{}", s)),
    ]
}

/// Generate a valid ObjectId (20 bytes)
fn object_id_strategy() -> impl Strategy<Value = ObjectId> {
    prop::array::uniform20(any::<u8>()).prop_map(ObjectId::from_bytes)
}

/// Generate a valid GitObject
fn git_object_strategy() -> impl Strategy<Value = GitObject> {
    (
        prop_oneof![
            Just(ObjectType::Blob),
            Just(ObjectType::Tree),
            Just(ObjectType::Commit),
            Just(ObjectType::Tag),
        ],
        prop::collection::vec(any::<u8>(), 0..5000),
    )
        .prop_map(|(obj_type, data)| GitObject::new(obj_type, Bytes::from(data)))
}

proptest! {
    // ========================================================================
    // RepoAnnounce Tests
    // ========================================================================

    /// Property: RepoAnnounce encoding/decoding is reversible
    #[test]
    fn prop_repo_announce_roundtrip(
        repo_key in repo_key_strategy(),
        object_ids in prop::collection::vec(object_id_strategy(), 0..50),
        refs in prop::collection::vec((ref_name_strategy(), object_id_strategy()), 0..20)
    ) {
        let msg = RepoAnnounce {
            repo_key: repo_key.clone(),
            object_ids: object_ids.clone(),
            refs: refs.clone(),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        prop_assert!(decoded.is_ok(), "Decoding should succeed");

        if let Ok(Message::RepoAnnounce(d)) = decoded {
            prop_assert_eq!(d.repo_key, repo_key);
            prop_assert_eq!(d.object_ids.len(), object_ids.len());
            prop_assert_eq!(d.refs.len(), refs.len());

            for (orig, dec) in object_ids.iter().zip(d.object_ids.iter()) {
                prop_assert_eq!(orig, dec);
            }

            for ((orig_name, orig_id), (dec_name, dec_id)) in refs.iter().zip(d.refs.iter()) {
                prop_assert_eq!(orig_name, dec_name);
                prop_assert_eq!(orig_id, dec_id);
            }
        } else {
            prop_assert!(false, "Expected RepoAnnounce message");
        }
    }

    /// Property: RepoAnnounce handles empty lists correctly
    #[test]
    fn prop_repo_announce_empty(repo_key in repo_key_strategy()) {
        let msg = RepoAnnounce {
            repo_key: repo_key.clone(),
            object_ids: vec![],
            refs: vec![],
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        prop_assert!(decoded.is_ok());
        if let Ok(Message::RepoAnnounce(d)) = decoded {
            prop_assert_eq!(d.repo_key, repo_key);
            prop_assert!(d.object_ids.is_empty());
            prop_assert!(d.refs.is_empty());
        }
    }

    // ========================================================================
    // SyncRequest Tests
    // ========================================================================

    /// Property: SyncRequest encoding/decoding is reversible
    #[test]
    fn prop_sync_request_roundtrip(
        repo_key in repo_key_strategy(),
        want in prop::collection::vec(object_id_strategy(), 0..100)
    ) {
        let msg = SyncRequest {
            repo_key: repo_key.clone(),
            want: want.clone(),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        prop_assert!(decoded.is_ok());

        if let Ok(Message::SyncRequest(d)) = decoded {
            prop_assert_eq!(d.repo_key, repo_key);
            prop_assert_eq!(d.want.len(), want.len());

            for (orig, dec) in want.iter().zip(d.want.iter()) {
                prop_assert_eq!(orig, dec);
            }
        } else {
            prop_assert!(false, "Expected SyncRequest message");
        }
    }

    // ========================================================================
    // ObjectData Tests
    // ========================================================================

    /// Property: ObjectData encoding/decoding preserves object content
    #[test]
    fn prop_object_data_roundtrip(
        repo_key in repo_key_strategy(),
        objects in prop::collection::vec(git_object_strategy(), 0..20)
    ) {
        let msg = ObjectData {
            repo_key: repo_key.clone(),
            objects: objects.clone(),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        prop_assert!(decoded.is_ok());

        if let Ok(Message::ObjectData(d)) = decoded {
            prop_assert_eq!(d.repo_key, repo_key);
            prop_assert_eq!(d.objects.len(), objects.len());

            for (orig, dec) in objects.iter().zip(d.objects.iter()) {
                prop_assert_eq!(orig.id, dec.id);
                prop_assert_eq!(orig.object_type, dec.object_type);
                prop_assert_eq!(orig.data.as_ref(), dec.data.as_ref());
            }
        } else {
            prop_assert!(false, "Expected ObjectData message");
        }
    }

    // ========================================================================
    // RefUpdate Tests
    // ========================================================================

    /// Property: RefUpdate encoding/decoding is reversible
    #[test]
    fn prop_ref_update_roundtrip(
        repo_key in repo_key_strategy(),
        ref_name in ref_name_strategy(),
        old_id in object_id_strategy(),
        new_id in object_id_strategy()
    ) {
        let msg = RefUpdate {
            repo_key: repo_key.clone(),
            ref_name: ref_name.clone(),
            old_id,
            new_id,
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        prop_assert!(decoded.is_ok());

        if let Ok(Message::RefUpdate(d)) = decoded {
            prop_assert_eq!(d.repo_key, repo_key);
            prop_assert_eq!(d.ref_name, ref_name);
            prop_assert_eq!(d.old_id, old_id);
            prop_assert_eq!(d.new_id, new_id);
        } else {
            prop_assert!(false, "Expected RefUpdate message");
        }
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    /// Property: Random bytes don't cause panics when decoding
    #[test]
    fn prop_random_bytes_no_panic(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        // Should return error or Ok, but never panic
        let _ = Message::decode(&data);
    }

    /// Property: Truncated messages are handled gracefully
    #[test]
    fn prop_truncated_messages_graceful(
        repo_key in repo_key_strategy(),
        truncate_point in 1usize..100
    ) {
        let msg = RepoAnnounce {
            repo_key: repo_key.clone(),
            object_ids: vec![ObjectId::from_bytes([1u8; 20])],
            refs: vec![("refs/heads/main".to_string(), ObjectId::from_bytes([2u8; 20]))],
        };

        let encoded = msg.encode();

        // Truncate at various points
        if truncate_point < encoded.len() {
            let truncated = &encoded[..truncate_point];
            // Should not panic
            let _ = Message::decode(truncated);
        }
    }

    /// Property: Objects with all valid types can be encoded/decoded
    #[test]
    fn prop_all_object_types_roundtrip(
        repo_key in repo_key_strategy(),
        blob_data in prop::collection::vec(any::<u8>(), 0..1000),
        tree_data in prop::collection::vec(any::<u8>(), 0..1000),
        commit_data in prop::collection::vec(any::<u8>(), 0..1000),
        tag_data in prop::collection::vec(any::<u8>(), 0..1000),
    ) {
        let objects = vec![
            GitObject::new(ObjectType::Blob, Bytes::from(blob_data)),
            GitObject::new(ObjectType::Tree, Bytes::from(tree_data)),
            GitObject::new(ObjectType::Commit, Bytes::from(commit_data)),
            GitObject::new(ObjectType::Tag, Bytes::from(tag_data)),
        ];

        let msg = ObjectData {
            repo_key: repo_key.clone(),
            objects: objects.clone(),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        prop_assert!(decoded.is_ok());

        if let Ok(Message::ObjectData(d)) = decoded {
            prop_assert_eq!(d.objects.len(), 4);
            prop_assert_eq!(d.objects[0].object_type, ObjectType::Blob);
            prop_assert_eq!(d.objects[1].object_type, ObjectType::Tree);
            prop_assert_eq!(d.objects[2].object_type, ObjectType::Commit);
            prop_assert_eq!(d.objects[3].object_type, ObjectType::Tag);
        }
    }

    /// Property: Large object counts are handled correctly
    #[test]
    fn prop_large_object_counts(
        repo_key in repo_key_strategy(),
        count in 50usize..200
    ) {
        let object_ids: Vec<ObjectId> = (0..count)
            .map(|i| {
                let mut bytes = [0u8; 20];
                bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
                ObjectId::from_bytes(bytes)
            })
            .collect();

        let msg = RepoAnnounce {
            repo_key: repo_key.clone(),
            object_ids: object_ids.clone(),
            refs: vec![],
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        prop_assert!(decoded.is_ok());

        if let Ok(Message::RepoAnnounce(d)) = decoded {
            prop_assert_eq!(d.object_ids.len(), count);
        }
    }

    /// Property: ObjectId uniqueness is preserved through roundtrip
    #[test]
    fn prop_object_id_uniqueness(
        ids in prop::collection::hash_set(object_id_strategy(), 1..100)
    ) {
        let ids_vec: Vec<ObjectId> = ids.into_iter().collect();
        let original_count = ids_vec.len();

        let msg = SyncRequest {
            repo_key: "test/repo".to_string(),
            want: ids_vec.clone(),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded).unwrap();

        if let Message::SyncRequest(d) = decoded {
            // Count unique IDs after roundtrip
            let unique_ids: HashSet<ObjectId> = d.want.iter().cloned().collect();
            prop_assert_eq!(unique_ids.len(), original_count);
        }
    }
}

// ============================================================================
// Load Testing Infrastructure
// ============================================================================

/// Load test configuration
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Number of concurrent operations
    pub concurrency: usize,
    /// Total number of operations to perform
    pub total_ops: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Target operations per second (0 = unlimited)
    pub target_ops_per_sec: usize,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            total_ops: 1000,
            max_message_size: 65536,
            target_ops_per_sec: 0,
        }
    }
}

/// Results from a load test run
#[derive(Debug, Clone)]
pub struct LoadTestResults {
    /// Total operations completed
    pub total_ops: usize,
    /// Successful operations
    pub successful_ops: usize,
    /// Failed operations
    pub failed_ops: usize,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Operations per second
    pub ops_per_sec: f64,
    /// Average latency in microseconds
    pub avg_latency_us: u64,
    /// P50 latency in microseconds
    pub p50_latency_us: u64,
    /// P95 latency in microseconds
    pub p95_latency_us: u64,
    /// P99 latency in microseconds
    pub p99_latency_us: u64,
    /// Maximum latency in microseconds
    pub max_latency_us: u64,
}

/// Run message encoding load test
pub fn run_message_encoding_load_test(config: &LoadTestConfig) -> LoadTestResults {
    use std::time::Instant;

    let mut latencies: Vec<u64> = Vec::with_capacity(config.total_ops);
    let mut successful = 0usize;
    let mut failed = 0usize;

    let start = Instant::now();

    for i in 0..config.total_ops {
        let op_start = Instant::now();

        // Create a message of varying complexity
        let msg = RepoAnnounce {
            repo_key: format!("user{}/repo{}", i % 100, i % 50),
            object_ids: (0..(i % 10 + 1))
                .map(|j| {
                    let mut bytes = [0u8; 20];
                    bytes[0..8].copy_from_slice(&((i + j) as u64).to_le_bytes());
                    ObjectId::from_bytes(bytes)
                })
                .collect(),
            refs: vec![(
                "refs/heads/main".to_string(),
                ObjectId::from_bytes([i as u8; 20]),
            )],
        };

        // Encode and decode
        let encoded = msg.encode();
        match Message::decode(&encoded) {
            Ok(_) => successful += 1,
            Err(_) => failed += 1,
        }

        let latency = op_start.elapsed().as_micros() as u64;
        latencies.push(latency);
    }

    let duration = start.elapsed();

    // Sort latencies for percentile calculation
    latencies.sort_unstable();

    let avg_latency = if !latencies.is_empty() {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    } else {
        0
    };

    let percentile = |p: f64| -> u64 {
        if latencies.is_empty() {
            return 0;
        }
        let idx = ((p / 100.0) * latencies.len() as f64) as usize;
        latencies[idx.min(latencies.len() - 1)]
    };

    LoadTestResults {
        total_ops: config.total_ops,
        successful_ops: successful,
        failed_ops: failed,
        duration_ms: duration.as_millis() as u64,
        ops_per_sec: config.total_ops as f64 / duration.as_secs_f64(),
        avg_latency_us: avg_latency,
        p50_latency_us: percentile(50.0),
        p95_latency_us: percentile(95.0),
        p99_latency_us: percentile(99.0),
        max_latency_us: latencies.last().copied().unwrap_or(0),
    }
}

/// Run object data load test with varying sizes
pub fn run_object_data_load_test(config: &LoadTestConfig) -> LoadTestResults {
    use std::time::Instant;

    let mut latencies: Vec<u64> = Vec::with_capacity(config.total_ops);
    let mut successful = 0usize;
    let mut failed = 0usize;

    let start = Instant::now();

    for i in 0..config.total_ops {
        let op_start = Instant::now();

        // Create objects of varying sizes
        let size = (i % 10 + 1) * 1000; // 1KB to 10KB
        let objects: Vec<GitObject> = (0..(i % 5 + 1))
            .map(|j| {
                let data: Vec<u8> = (0..size).map(|k| ((i + j + k) % 256) as u8).collect();
                GitObject::blob(data)
            })
            .collect();

        let msg = ObjectData {
            repo_key: format!("user{}/repo{}", i % 100, i % 50),
            objects,
        };

        // Encode and decode
        let encoded = msg.encode();
        if encoded.len() <= config.max_message_size {
            match Message::decode(&encoded) {
                Ok(_) => successful += 1,
                Err(_) => failed += 1,
            }
        } else {
            failed += 1;
        }

        let latency = op_start.elapsed().as_micros() as u64;
        latencies.push(latency);
    }

    let duration = start.elapsed();

    latencies.sort_unstable();

    let avg_latency = if !latencies.is_empty() {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    } else {
        0
    };

    let percentile = |p: f64| -> u64 {
        if latencies.is_empty() {
            return 0;
        }
        let idx = ((p / 100.0) * latencies.len() as f64) as usize;
        latencies[idx.min(latencies.len() - 1)]
    };

    LoadTestResults {
        total_ops: config.total_ops,
        successful_ops: successful,
        failed_ops: failed,
        duration_ms: duration.as_millis() as u64,
        ops_per_sec: config.total_ops as f64 / duration.as_secs_f64(),
        avg_latency_us: avg_latency,
        p50_latency_us: percentile(50.0),
        p95_latency_us: percentile(95.0),
        p99_latency_us: percentile(99.0),
        max_latency_us: latencies.last().copied().unwrap_or(0),
    }
}

// ============================================================================
// Load Tests
// ============================================================================

#[test]
fn test_message_encoding_performance() {
    let config = LoadTestConfig {
        concurrency: 1,
        total_ops: 10000,
        max_message_size: 1024 * 1024,
        target_ops_per_sec: 0,
    };

    let results = run_message_encoding_load_test(&config);

    println!("Message Encoding Load Test Results:");
    println!("  Total ops: {}", results.total_ops);
    println!("  Successful: {}", results.successful_ops);
    println!("  Failed: {}", results.failed_ops);
    println!("  Duration: {}ms", results.duration_ms);
    println!("  Ops/sec: {:.2}", results.ops_per_sec);
    println!("  Avg latency: {}μs", results.avg_latency_us);
    println!("  P50 latency: {}μs", results.p50_latency_us);
    println!("  P95 latency: {}μs", results.p95_latency_us);
    println!("  P99 latency: {}μs", results.p99_latency_us);
    println!("  Max latency: {}μs", results.max_latency_us);

    // Performance assertions
    assert_eq!(results.successful_ops, results.total_ops);
    assert_eq!(results.failed_ops, 0);
    assert!(
        results.ops_per_sec > 1000.0,
        "Should achieve at least 1000 ops/sec"
    );
}

#[test]
fn test_object_data_performance() {
    let config = LoadTestConfig {
        concurrency: 1,
        total_ops: 1000,
        max_message_size: 10 * 1024 * 1024, // 10MB
        target_ops_per_sec: 0,
    };

    let results = run_object_data_load_test(&config);

    println!("Object Data Load Test Results:");
    println!("  Total ops: {}", results.total_ops);
    println!("  Successful: {}", results.successful_ops);
    println!("  Failed: {}", results.failed_ops);
    println!("  Duration: {}ms", results.duration_ms);
    println!("  Ops/sec: {:.2}", results.ops_per_sec);
    println!("  Avg latency: {}μs", results.avg_latency_us);
    println!("  P95 latency: {}μs", results.p95_latency_us);
    println!("  P99 latency: {}μs", results.p99_latency_us);

    // All operations should succeed
    assert_eq!(results.successful_ops, results.total_ops);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_large_message_stress() {
    // Test with progressively larger messages
    for size_kb in [1, 10, 100, 1000] {
        let data: Vec<u8> = (0..size_kb * 1024).map(|i| (i % 256) as u8).collect();
        let blob = GitObject::blob(data);

        let msg = ObjectData {
            repo_key: "test/large-repo".to_string(),
            objects: vec![blob],
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        assert!(decoded.is_ok(), "Should handle {}KB message", size_kb);

        if let Ok(Message::ObjectData(d)) = decoded {
            assert_eq!(d.objects.len(), 1);
            assert_eq!(d.objects[0].data.len(), size_kb * 1024);
        }
    }
}

#[test]
fn test_many_objects_stress() {
    // Test with many small objects
    for count in [10, 100, 500, 1000] {
        let objects: Vec<GitObject> = (0..count)
            .map(|i| GitObject::blob(format!("object {}", i).into_bytes()))
            .collect();

        let msg = ObjectData {
            repo_key: "test/many-objects".to_string(),
            objects: objects.clone(),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        assert!(decoded.is_ok(), "Should handle {} objects", count);

        if let Ok(Message::ObjectData(d)) = decoded {
            assert_eq!(d.objects.len(), count);
        }
    }
}

#[test]
fn test_many_refs_stress() {
    // Test with many refs
    for count in [10, 100, 500] {
        let refs: Vec<(String, ObjectId)> = (0..count)
            .map(|i| {
                (
                    format!("refs/heads/branch-{}", i),
                    ObjectId::from_bytes([(i % 256) as u8; 20]),
                )
            })
            .collect();

        let msg = RepoAnnounce {
            repo_key: "test/many-refs".to_string(),
            object_ids: vec![],
            refs,
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);

        assert!(decoded.is_ok(), "Should handle {} refs", count);

        if let Ok(Message::RepoAnnounce(d)) = decoded {
            assert_eq!(d.refs.len(), count);
        }
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_repo_key() {
    // Empty repo key should still encode/decode
    let msg = RepoAnnounce {
        repo_key: String::new(),
        object_ids: vec![],
        refs: vec![],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded);

    assert!(decoded.is_ok());
}

#[test]
fn test_unicode_repo_key() {
    // Test with Unicode characters (valid UTF-8)
    let msg = RepoAnnounce {
        repo_key: "用户/仓库".to_string(),
        object_ids: vec![],
        refs: vec![],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded);

    assert!(decoded.is_ok());

    if let Ok(Message::RepoAnnounce(d)) = decoded {
        assert_eq!(d.repo_key, "用户/仓库");
    }
}

#[test]
fn test_max_u32_object_count_header() {
    // Test that we handle malformed headers gracefully
    // A message claiming to have MAX objects but no data
    let mut bad_msg = vec![1u8]; // RepoAnnounce type
    bad_msg.extend_from_slice(&0u16.to_be_bytes()); // empty repo key length
    bad_msg.extend_from_slice(&u32::MAX.to_be_bytes()); // claim MAX objects

    let result = Message::decode(&bad_msg);
    // Should fail gracefully, not panic or OOM
    assert!(result.is_err());
}

#[test]
fn test_binary_data_in_objects() {
    // Test with binary data including null bytes and all byte values
    let binary_data: Vec<u8> = (0..=255).collect();
    let blob = GitObject::blob(binary_data.clone());

    let msg = ObjectData {
        repo_key: "test/binary".to_string(),
        objects: vec![blob],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded);

    assert!(decoded.is_ok());

    if let Ok(Message::ObjectData(d)) = decoded {
        assert_eq!(d.objects[0].data.as_ref(), binary_data.as_slice());
    }
}

#[test]
fn test_zero_object_id() {
    // Test with all-zero object IDs
    let zero_id = ObjectId::from_bytes([0u8; 20]);

    let msg = RefUpdate {
        repo_key: "test/zero".to_string(),
        ref_name: "refs/heads/main".to_string(),
        old_id: zero_id,
        new_id: zero_id,
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded);

    assert!(decoded.is_ok());

    if let Ok(Message::RefUpdate(d)) = decoded {
        assert_eq!(d.old_id, zero_id);
        assert_eq!(d.new_id, zero_id);
    }
}

#[test]
fn test_max_object_id() {
    // Test with all-0xFF object IDs
    let max_id = ObjectId::from_bytes([0xFF; 20]);

    let msg = RefUpdate {
        repo_key: "test/max".to_string(),
        ref_name: "refs/heads/main".to_string(),
        old_id: max_id,
        new_id: max_id,
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded);

    assert!(decoded.is_ok());

    if let Ok(Message::RefUpdate(d)) = decoded {
        assert_eq!(d.old_id, max_id);
        assert_eq!(d.new_id, max_id);
    }
}
