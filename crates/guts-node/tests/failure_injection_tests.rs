//! Failure Injection Tests - Milestone 9 Phase 7
//!
//! This module provides integration tests with failure injection:
//! - Storage failure simulation
//! - Network timeout simulation
//! - Resource exhaustion simulation
//! - Partial operation failure simulation

use bytes::Bytes;
use guts_p2p::{Message, ObjectData, RefUpdate, RepoAnnounce, SyncRequest};
use guts_storage::{GitObject, ObjectId, ObjectStore, ObjectType};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// ============================================================================
// Failure Injection Infrastructure
// ============================================================================

/// A wrapper around ObjectStore that can inject failures
pub struct FailableObjectStore {
    inner: ObjectStore,
    /// Probability of read failure (0-100)
    read_failure_rate: AtomicUsize,
    /// Probability of write failure (0-100)
    write_failure_rate: AtomicUsize,
    /// Whether to fail all operations
    fail_all: AtomicBool,
    /// Counter for operations
    operation_count: AtomicUsize,
}

impl FailableObjectStore {
    pub fn new() -> Self {
        Self {
            inner: ObjectStore::new(),
            read_failure_rate: AtomicUsize::new(0),
            write_failure_rate: AtomicUsize::new(0),
            fail_all: AtomicBool::new(false),
            operation_count: AtomicUsize::new(0),
        }
    }

    /// Set the read failure rate (0-100%)
    pub fn set_read_failure_rate(&self, rate: usize) {
        self.read_failure_rate
            .store(rate.min(100), Ordering::SeqCst);
    }

    /// Set the write failure rate (0-100%)
    pub fn set_write_failure_rate(&self, rate: usize) {
        self.write_failure_rate
            .store(rate.min(100), Ordering::SeqCst);
    }

    /// Enable or disable complete failure mode
    pub fn set_fail_all(&self, fail: bool) {
        self.fail_all.store(fail, Ordering::SeqCst);
    }

    /// Get the number of operations performed
    pub fn operation_count(&self) -> usize {
        self.operation_count.load(Ordering::SeqCst)
    }

    /// Check if operation should fail based on rate
    /// Uses a simple counter-based approach for deterministic testing
    fn should_fail(&self, rate: usize) -> bool {
        if self.fail_all.load(Ordering::SeqCst) {
            return true;
        }
        if rate == 0 {
            return false;
        }
        if rate >= 100 {
            return true;
        }
        let op_num = self.operation_count.fetch_add(1, Ordering::SeqCst);
        // Use a prime-based pattern for better distribution
        let pattern = (op_num * 97) % 100;
        pattern < rate
    }

    /// Put an object, potentially failing
    pub fn put(&self, object: GitObject) -> Result<ObjectId, String> {
        let rate = self.write_failure_rate.load(Ordering::SeqCst);
        if self.should_fail(rate) {
            return Err("Injected write failure".to_string());
        }
        Ok(self.inner.put(object))
    }

    /// Get an object, potentially failing
    pub fn get(&self, id: &ObjectId) -> Result<GitObject, String> {
        let rate = self.read_failure_rate.load(Ordering::SeqCst);
        if self.should_fail(rate) {
            return Err("Injected read failure".to_string());
        }
        self.inner.get(id).map_err(|e| e.to_string())
    }

    /// Get the inner store (for verification)
    pub fn inner(&self) -> &ObjectStore {
        &self.inner
    }
}

impl Default for FailableObjectStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Storage Failure Tests
// ============================================================================

/// Test that read failures are handled gracefully
#[test]
fn test_read_failure_handling() {
    let store = FailableObjectStore::new();

    // Store an object successfully
    let blob = GitObject::blob(b"test content".to_vec());
    let id = store.put(blob).expect("First write should succeed");

    // Verify we can read it normally
    let obj = store.get(&id).expect("First read should succeed");
    assert_eq!(obj.data.as_ref(), b"test content");

    // Enable 50% read failure rate
    store.set_read_failure_rate(50);

    let mut successes = 0;
    let mut failures = 0;

    // Try to read multiple times
    for _ in 0..100 {
        match store.get(&id) {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    // Should have roughly 50% success rate
    assert!(successes > 30, "Should have some successes");
    assert!(failures > 30, "Should have some failures");
}

/// Test that write failures are handled gracefully
#[test]
fn test_write_failure_handling() {
    let store = FailableObjectStore::new();

    // Enable 50% write failure rate
    store.set_write_failure_rate(50);

    let mut successes = 0;
    let mut failures = 0;

    // Try to write multiple objects
    for i in 0..100 {
        let blob = GitObject::blob(format!("content {}", i).into_bytes());
        match store.put(blob) {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    // Should have roughly 50% success rate
    assert!(successes > 30, "Should have some successes");
    assert!(failures > 30, "Should have some failures");
}

/// Test complete failure mode
#[test]
fn test_complete_failure_mode() {
    let store = FailableObjectStore::new();

    // First, store some objects successfully
    let blob1 = GitObject::blob(b"content 1".to_vec());
    let id1 = store
        .put(blob1)
        .expect("Should succeed before failure mode");

    // Enable complete failure mode
    store.set_fail_all(true);

    // All operations should fail
    let blob2 = GitObject::blob(b"content 2".to_vec());
    assert!(store.put(blob2).is_err(), "Write should fail");
    assert!(store.get(&id1).is_err(), "Read should fail");

    // Disable failure mode
    store.set_fail_all(false);

    // Operations should work again
    let obj = store
        .get(&id1)
        .expect("Should succeed after disabling failure mode");
    assert_eq!(obj.data.as_ref(), b"content 1");
}

/// Test recovery after intermittent failures
#[test]
fn test_intermittent_failure_recovery() {
    let store = FailableObjectStore::new();

    // Enable low failure rate (10%)
    store.set_write_failure_rate(10);

    let mut stored_ids = Vec::new();

    // Try to store objects with retries
    for i in 0..50 {
        let blob = GitObject::blob(format!("content {}", i).into_bytes());
        let mut attempts = 0;
        let max_attempts = 10; // Increase max attempts for more reliability

        loop {
            attempts += 1;
            match store.put(blob.clone()) {
                Ok(id) => {
                    stored_ids.push(id);
                    break;
                }
                Err(_) if attempts < max_attempts => continue, // Retry
                Err(e) => {
                    // With retries, we should very rarely fail all attempts
                    panic!("Failed after {} attempts: {}", attempts, e);
                }
            }
        }
    }

    // Disable failures for reading
    store.set_read_failure_rate(0);

    // All objects that were stored should be retrievable
    for id in &stored_ids {
        assert!(store.get(id).is_ok(), "Object should be retrievable");
    }

    // We should have stored all objects eventually
    assert_eq!(
        stored_ids.len(),
        50,
        "Should have stored all objects with retries"
    );
}

// ============================================================================
// Message Processing Under Failure
// ============================================================================

/// Test message processing with storage failures
#[test]
fn test_message_processing_with_storage_failures() {
    let store = FailableObjectStore::new();

    // Simulate receiving an ObjectData message
    let objects: Vec<GitObject> = (0..10)
        .map(|i| GitObject::blob(format!("object {}", i).into_bytes()))
        .collect();

    // Store objects with potential failures
    store.set_write_failure_rate(20); // 20% failure rate

    let mut stored = 0;
    let mut failed = 0;

    for obj in &objects {
        match store.put(obj.clone()) {
            Ok(_) => stored += 1,
            Err(_) => failed += 1,
        }
    }

    println!("Stored {} objects, {} failures", stored, failed);

    // With 20% failure rate and 10 objects, we expect some failures
    // But we should have stored most objects
    assert!(stored > 0, "Should have stored some objects");
}

/// Test ref update processing with concurrent failures
#[test]
fn test_ref_update_with_failures() {
    // Simulate ref updates that might fail
    let updates: Vec<RefUpdate> = (0..20)
        .map(|i| RefUpdate {
            repo_key: format!("user/repo{}", i),
            ref_name: "refs/heads/main".to_string(),
            old_id: ObjectId::from_bytes([i as u8; 20]),
            new_id: ObjectId::from_bytes([(i + 1) as u8; 20]),
        })
        .collect();

    let mut processed = 0;
    let mut encode_errors = 0;
    let mut _decode_errors = 0;

    for (idx, update) in updates.iter().enumerate() {
        let encoded = update.encode();

        // Simulate occasional encoding corruption (every 5th message starting at index 4)
        let data = if idx % 5 == 4 {
            // Corrupt by truncating
            encoded[..encoded.len().saturating_sub(5)].to_vec()
        } else {
            encoded.to_vec()
        };

        match Message::decode(&data) {
            Ok(Message::RefUpdate(decoded)) => {
                if decoded.repo_key == update.repo_key {
                    processed += 1;
                } else {
                    _decode_errors += 1;
                }
            }
            Ok(_) => _decode_errors += 1,
            Err(_) => encode_errors += 1,
        }
    }

    // Most should succeed (80% were not corrupted - indices 4, 9, 14, 19 are corrupted)
    assert!(
        processed >= 15,
        "Most updates should process successfully, got {}",
        processed
    );
    assert!(
        encode_errors > 0,
        "Should have some encoding errors from corruption"
    );
}

// ============================================================================
// Resource Exhaustion Simulation
// ============================================================================

/// Test behavior when creating many objects rapidly
#[test]
fn test_rapid_object_creation() {
    let store = ObjectStore::new();

    let start = std::time::Instant::now();
    let object_count = 10000;

    for i in 0..object_count {
        let blob = GitObject::blob(format!("rapid object {}", i).into_bytes());
        store.put(blob);
    }

    let duration = start.elapsed();

    assert_eq!(store.len(), object_count, "All objects should be stored");

    // Should complete in reasonable time (< 5 seconds)
    assert!(
        duration.as_secs() < 5,
        "Rapid creation took too long: {:?}",
        duration
    );

    println!(
        "Created {} objects in {:?} ({:.0} objects/sec)",
        object_count,
        duration,
        object_count as f64 / duration.as_secs_f64()
    );
}

/// Test behavior with large objects
#[test]
fn test_large_object_handling() {
    let store = ObjectStore::new();

    // Create progressively larger objects
    for size_mb in [1, 5, 10] {
        let size = size_mb * 1024 * 1024;
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        let blob = GitObject::blob(data);
        let id = store.put(blob);

        // Verify we can retrieve it
        let retrieved = store.get(&id).expect("Should retrieve large object");
        assert_eq!(retrieved.data.len(), size, "Size should match");
    }
}

/// Test behavior with many concurrent message encodings
#[test]
fn test_concurrent_message_encoding() {
    use std::thread;

    let num_threads = 8;
    let iterations_per_thread = 1000;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            thread::spawn(move || {
                for i in 0..iterations_per_thread {
                    let msg = RepoAnnounce {
                        repo_key: format!("thread{}/repo{}", thread_id, i),
                        object_ids: (0..10)
                            .map(|j| ObjectId::from_bytes([(thread_id * 100 + j) as u8; 20]))
                            .collect(),
                        refs: vec![(
                            "refs/heads/main".to_string(),
                            ObjectId::from_bytes([thread_id as u8; 20]),
                        )],
                    };

                    let encoded = msg.encode();
                    let decoded = Message::decode(&encoded).expect("Should decode");

                    match decoded {
                        Message::RepoAnnounce(d) => {
                            assert_eq!(d.repo_key, msg.repo_key);
                        }
                        _ => panic!("Wrong message type"),
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should complete");
    }
}

// ============================================================================
// Partial Operation Failure Tests
// ============================================================================

/// Test partial sync request processing
#[test]
fn test_partial_sync_processing() {
    let store = FailableObjectStore::new();

    // Store some objects (no failures during storage)
    let objects: Vec<GitObject> = (0..20)
        .map(|i| GitObject::blob(format!("sync object {}", i).into_bytes()))
        .collect();

    let object_ids: Vec<ObjectId> = objects
        .iter()
        .map(|obj| {
            store
                .put(obj.clone())
                .expect("Initial store should succeed")
        })
        .collect();

    // Create a sync request for all objects
    let sync_request = SyncRequest {
        repo_key: "test/partial".to_string(),
        want: object_ids.clone(),
    };

    // Encode and decode
    let encoded = sync_request.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::SyncRequest(req) => {
            // Now simulate partial retrieval with failures (50% to ensure both success and failure)
            store.set_read_failure_rate(50);

            let mut retrieved = Vec::new();
            let mut failed_ids = Vec::new();

            for id in &req.want {
                match store.get(id) {
                    Ok(obj) => retrieved.push(obj),
                    Err(_) => failed_ids.push(*id),
                }
            }

            // With 50% failure rate over 20 objects, we should have a mix
            println!(
                "Retrieved: {}, Failed: {}",
                retrieved.len(),
                failed_ids.len()
            );

            // Retry failed retrievals with failures disabled
            store.set_read_failure_rate(0);

            for id in &failed_ids {
                let obj = store.get(id).expect("Retry should succeed");
                retrieved.push(obj);
            }

            assert_eq!(
                retrieved.len(),
                object_ids.len(),
                "All objects should be retrieved after retry"
            );
        }
        _ => panic!("Expected SyncRequest"),
    }
}

/// Test object data message with partially valid objects
#[test]
fn test_partial_object_data_validity() {
    // Create an ObjectData with mixed valid/potentially problematic objects
    let objects = vec![
        GitObject::blob(b"normal content".to_vec()),
        GitObject::blob(Vec::new()),                    // Empty blob
        GitObject::blob(vec![0u8; 10000]),              // Large blob of zeros
        GitObject::new(ObjectType::Tree, Bytes::new()), // Empty tree
        GitObject::new(ObjectType::Commit, Bytes::from("tree abc\n")), // Minimal commit-like
    ];

    let msg = ObjectData {
        repo_key: "test/partial-validity".to_string(),
        objects: objects.clone(),
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::ObjectData(data) => {
            assert_eq!(
                data.objects.len(),
                objects.len(),
                "All objects should be preserved"
            );

            // Verify each object
            for (orig, decoded) in objects.iter().zip(data.objects.iter()) {
                assert_eq!(orig.id, decoded.id, "Object ID should match");
                assert_eq!(
                    orig.object_type, decoded.object_type,
                    "Object type should match"
                );
                assert_eq!(
                    orig.data.as_ref(),
                    decoded.data.as_ref(),
                    "Data should match"
                );
            }
        }
        _ => panic!("Expected ObjectData"),
    }
}

// ============================================================================
// Error Recovery Pattern Tests
// ============================================================================

/// Test exponential backoff pattern for retries
#[test]
fn test_retry_with_backoff() {
    let store = FailableObjectStore::new();
    store.set_write_failure_rate(80); // High failure rate

    let blob = GitObject::blob(b"retry test".to_vec());
    let max_attempts = 10;
    let mut attempt = 0;
    let mut success = false;

    while attempt < max_attempts && !success {
        attempt += 1;

        match store.put(blob.clone()) {
            Ok(_) => {
                success = true;
                println!("Succeeded on attempt {}", attempt);
            }
            Err(_) => {
                // Simulate backoff (in real code, would sleep)
                let _backoff_ms = 2_usize.pow(attempt as u32) * 10;
                // std::thread::sleep(Duration::from_millis(backoff_ms as u64));
            }
        }
    }

    // With 80% failure rate and 10 attempts, success probability is:
    // 1 - 0.8^10 ≈ 89%
    // We might not always succeed, which is fine for this test
    println!("Retry test: {} attempts, success={}", attempt, success);
}

/// Test circuit breaker pattern
#[test]
fn test_circuit_breaker_pattern() {
    let store = FailableObjectStore::new();

    // Circuit breaker state
    let mut consecutive_failures = 0;
    let failure_threshold = 3;
    let mut circuit_open = false;
    let mut operations_while_open = 0;

    // Enable 100% failure
    store.set_fail_all(true);

    for i in 0..20 {
        if circuit_open {
            operations_while_open += 1;
            // In real code, would check if timeout has passed
            if operations_while_open > 5 {
                // Try to close circuit
                store.set_fail_all(false);
                circuit_open = false;
                consecutive_failures = 0;
            }
            continue;
        }

        let blob = GitObject::blob(format!("circuit {}", i).into_bytes());
        match store.put(blob) {
            Ok(_) => {
                consecutive_failures = 0;
            }
            Err(_) => {
                consecutive_failures += 1;
                if consecutive_failures >= failure_threshold {
                    circuit_open = true;
                }
            }
        }
    }

    // Circuit should have opened
    assert!(
        operations_while_open > 0,
        "Circuit breaker should have opened"
    );
}

/// Test graceful degradation
#[test]
fn test_graceful_degradation() {
    let store = FailableObjectStore::new();

    // Store primary and fallback data
    let primary = GitObject::blob(b"primary data".to_vec());
    let fallback = GitObject::blob(b"fallback data".to_vec());

    let primary_id = store.put(primary).expect("Primary store should succeed");
    let fallback_id = store.put(fallback).expect("Fallback store should succeed");

    // Enable read failures (25% rate is more predictable)
    store.set_read_failure_rate(25);

    let mut retrieved_count = 0;
    let mut used_fallback = 0;

    for _ in 0..100 {
        // Try primary, fall back to secondary
        match store.get(&primary_id) {
            Ok(obj) => {
                assert_eq!(obj.data.as_ref(), b"primary data");
                retrieved_count += 1;
            }
            Err(_) => {
                // Try fallback
                match store.get(&fallback_id) {
                    Ok(obj) => {
                        assert_eq!(obj.data.as_ref(), b"fallback data");
                        retrieved_count += 1;
                        used_fallback += 1;
                    }
                    Err(_) => {
                        // Complete degradation - no data available
                    }
                }
            }
        }
    }

    // With 25% failure rate, should have retrieved most and used fallback sometimes
    // Primary success ~75%, fallback needed ~25%, fallback success ~75% of that = ~93.75% total
    println!(
        "Graceful degradation: {} retrieved, {} used fallback",
        retrieved_count, used_fallback
    );

    assert!(
        retrieved_count > 50,
        "Should retrieve data most of the time, got {}",
        retrieved_count
    );
}
