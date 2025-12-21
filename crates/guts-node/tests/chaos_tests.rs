//! Chaos Testing for P2P Layer - Milestone 9 Phase 7
//!
//! This module provides chaos testing capabilities for the P2P networking layer:
//! - Message corruption simulation
//! - Truncation handling
//! - Byzantine behavior simulation
//! - Stress testing under chaos conditions

use bytes::Bytes;
use guts_p2p::{Message, ObjectData, RefUpdate, RepoAnnounce};
use guts_storage::{GitObject, ObjectId, ObjectType};

// ============================================================================
// Chaos Testing Configuration
// ============================================================================

/// Configuration for chaos testing scenarios
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Probability of message loss (0.0 - 1.0)
    pub message_loss_rate: f64,
    /// Probability of message corruption (0.0 - 1.0)
    pub corruption_rate: f64,
    /// Probability of message duplication (0.0 - 1.0)
    pub duplication_rate: f64,
    /// Maximum additional latency in milliseconds
    pub max_latency_jitter_ms: u64,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            message_loss_rate: 0.0,
            corruption_rate: 0.0,
            duplication_rate: 0.0,
            max_latency_jitter_ms: 0,
        }
    }
}

impl ChaosConfig {
    /// Creates a config with mild chaos
    pub fn mild() -> Self {
        Self {
            message_loss_rate: 0.01,
            corruption_rate: 0.001,
            duplication_rate: 0.01,
            max_latency_jitter_ms: 50,
        }
    }

    /// Creates a config with moderate chaos
    pub fn moderate() -> Self {
        Self {
            message_loss_rate: 0.05,
            corruption_rate: 0.01,
            duplication_rate: 0.02,
            max_latency_jitter_ms: 200,
        }
    }

    /// Creates a config with severe chaos
    pub fn severe() -> Self {
        Self {
            message_loss_rate: 0.1,
            corruption_rate: 0.02,
            duplication_rate: 0.05,
            max_latency_jitter_ms: 500,
        }
    }
}

// ============================================================================
// Message Corruption Utilities
// ============================================================================

/// Corrupts a message by flipping random bits
fn corrupt_message(data: &[u8], corruption_level: usize) -> Vec<u8> {
    let mut corrupted = data.to_vec();
    if corrupted.is_empty() {
        return corrupted;
    }

    // Flip random bits based on corruption level
    for i in 0..corruption_level.min(corrupted.len()) {
        let byte_idx = i % corrupted.len();
        let bit_idx = (i * 7) % 8;
        corrupted[byte_idx] ^= 1 << bit_idx;
    }

    corrupted
}

/// Truncates a message to simulate partial transmission
fn truncate_message(data: &[u8], keep_fraction: f64) -> Vec<u8> {
    let keep_bytes = ((data.len() as f64) * keep_fraction) as usize;
    data[..keep_bytes.max(1).min(data.len())].to_vec()
}

// ============================================================================
// Chaos Tests
// ============================================================================

/// Test that corrupted messages are handled gracefully
#[test]
fn test_corrupted_message_handling() {
    // Create a valid message
    let msg = RepoAnnounce {
        repo_key: "alice/test-repo".to_string(),
        object_ids: vec![
            ObjectId::from_bytes([1u8; 20]),
            ObjectId::from_bytes([2u8; 20]),
        ],
        refs: vec![(
            "refs/heads/main".to_string(),
            ObjectId::from_bytes([3u8; 20]),
        )],
    };

    let encoded = msg.encode();

    // Test various corruption levels
    for corruption_level in 1..=10 {
        let corrupted = corrupt_message(&encoded, corruption_level);

        // Should not panic, just return an error
        let result = Message::decode(&corrupted);
        // We expect errors for corrupted data, but no panics
        if result.is_err() {
            // This is expected
        }
    }
}

/// Test that truncated messages are handled gracefully
#[test]
fn test_truncated_message_handling() {
    let msg = RepoAnnounce {
        repo_key: "alice/test-repo".to_string(),
        object_ids: vec![ObjectId::from_bytes([1u8; 20])],
        refs: vec![],
    };

    let encoded = msg.encode();

    // Test various truncation levels
    for fraction in [0.1, 0.25, 0.5, 0.75, 0.9] {
        let truncated = truncate_message(&encoded, fraction);

        // Should not panic
        let result = Message::decode(&truncated);
        // Truncated messages should fail to decode
        assert!(
            result.is_err() || truncated.len() == encoded.len(),
            "Truncated message should fail to decode"
        );
    }
}

/// Test recovery from message corruption with retries
#[test]
fn test_corruption_recovery_with_retry() {
    let msg = ObjectData {
        repo_key: "test/repo".to_string(),
        objects: vec![GitObject::blob(b"test content".to_vec())],
    };

    let encoded = msg.encode();
    let mut successful_decodes = 0;
    let attempts = 100;

    for i in 0..attempts {
        // Sometimes corrupt, sometimes not
        let data = if i % 3 == 0 {
            corrupt_message(&encoded, 1)
        } else {
            encoded.to_vec()
        };

        if Message::decode(&data).is_ok() {
            successful_decodes += 1;
        }
    }

    // At least the uncorrupted messages should decode
    assert!(
        successful_decodes >= attempts * 2 / 3,
        "At least 2/3 of messages should decode correctly"
    );
}

/// Test behavior with empty messages
#[test]
fn test_empty_message_handling() {
    let result = Message::decode(&[]);
    assert!(result.is_err(), "Empty message should fail to decode");
}

/// Test behavior with single-byte messages
#[test]
fn test_single_byte_messages() {
    for byte in 0..=255 {
        let result = Message::decode(&[byte]);
        // Should not panic, may succeed or fail
        let _ = result;
    }
}

/// Test handling of messages with invalid type bytes
#[test]
fn test_invalid_message_type() {
    // Valid types are 1-4, test invalid types
    for invalid_type in [0, 5, 6, 100, 200, 255] {
        let data = vec![invalid_type, 0, 5]; // Invalid type + some data
        let result = Message::decode(&data);
        assert!(
            result.is_err(),
            "Invalid message type {} should fail",
            invalid_type
        );
    }
}

/// Test handling of oversized length fields
#[test]
fn test_oversized_length_fields() {
    // Create a message that claims to have a huge repo key
    let mut data = vec![1u8]; // RepoAnnounce type
    data.extend_from_slice(&[0xFF, 0xFF]); // Claim 65535 byte repo key
    data.extend_from_slice(b"short"); // But only provide 5 bytes

    let result = Message::decode(&data);
    assert!(result.is_err(), "Oversized length field should fail");
}

// ============================================================================
// Byzantine Behavior Tests
// ============================================================================

/// Test handling of malformed repo keys (Byzantine behavior)
#[test]
fn test_byzantine_malformed_repo_keys() {
    let malformed_keys = [
        "".to_string(),                   // Empty
        "/".to_string(),                  // Just slash
        "noowner".to_string(),            // No slash
        "owner/".to_string(),             // Empty repo name
        "/repo".to_string(),              // Empty owner
        "a".repeat(1000),                 // Very long
        "owner/repo\0hidden".to_string(), // Null byte
        "owner\n/repo".to_string(),       // Newline
        "öwner/répö".to_string(),         // Unicode
    ];

    for key in malformed_keys {
        let msg = RepoAnnounce {
            repo_key: key.clone(),
            object_ids: vec![],
            refs: vec![],
        };

        // Encoding should work
        let encoded = msg.encode();

        // Decoding should work (we accept any UTF-8 string)
        let result = Message::decode(&encoded);
        match result {
            Ok(Message::RepoAnnounce(decoded)) => {
                assert_eq!(decoded.repo_key, key);
            }
            Err(_) => {
                // Also acceptable - validation at decode time
            }
            _ => panic!("Unexpected message type"),
        }
    }
}

/// Test handling of duplicate object IDs
#[test]
fn test_duplicate_object_ids() {
    let dup_id = ObjectId::from_bytes([42u8; 20]);

    let msg = RepoAnnounce {
        repo_key: "test/dups".to_string(),
        object_ids: vec![dup_id, dup_id, dup_id], // Same ID three times
        refs: vec![],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::RepoAnnounce(announce) => {
            // Should preserve duplicates (storage layer handles dedup)
            assert_eq!(announce.object_ids.len(), 3);
            assert!(announce.object_ids.iter().all(|id| *id == dup_id));
        }
        _ => panic!("Expected RepoAnnounce"),
    }
}

/// Test handling of conflicting refs
#[test]
fn test_conflicting_refs() {
    let id1 = ObjectId::from_bytes([1u8; 20]);
    let id2 = ObjectId::from_bytes([2u8; 20]);

    let msg = RepoAnnounce {
        repo_key: "test/conflicts".to_string(),
        object_ids: vec![],
        refs: vec![
            ("refs/heads/main".to_string(), id1),
            ("refs/heads/main".to_string(), id2), // Same ref, different ID
        ],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::RepoAnnounce(announce) => {
            // Protocol preserves duplicates; resolution is at higher layer
            assert_eq!(announce.refs.len(), 2);
        }
        _ => panic!("Expected RepoAnnounce"),
    }
}

/// Test handling of extremely large object counts
#[test]
fn test_large_object_count() {
    // Create message with many objects
    let object_ids: Vec<ObjectId> = (0..10000)
        .map(|i| {
            let mut bytes = [0u8; 20];
            bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            ObjectId::from_bytes(bytes)
        })
        .collect();

    let msg = RepoAnnounce {
        repo_key: "test/large".to_string(),
        object_ids: object_ids.clone(),
        refs: vec![],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode large message");

    match decoded {
        Message::RepoAnnounce(announce) => {
            assert_eq!(announce.object_ids.len(), 10000);
        }
        _ => panic!("Expected RepoAnnounce"),
    }
}

// ============================================================================
// Stress Tests Under Chaos
// ============================================================================

/// Stress test message encoding/decoding under simulated chaos
#[test]
fn test_message_encoding_stress() {
    let iterations = 1000;
    let mut successes = 0usize;
    let mut failures = 0usize;

    for i in 0..iterations {
        // Create message of varying complexity
        let obj_count = (i % 100) + 1;
        let object_ids: Vec<ObjectId> = (0..obj_count)
            .map(|j| {
                let mut bytes = [0u8; 20];
                bytes[0..8].copy_from_slice(&((i * 1000 + j) as u64).to_le_bytes());
                ObjectId::from_bytes(bytes)
            })
            .collect();

        let ref_count = (i % 10) + 1;
        let refs: Vec<(String, ObjectId)> = (0..ref_count)
            .map(|j| {
                (
                    format!("refs/heads/branch-{}", j),
                    object_ids[j % object_ids.len()],
                )
            })
            .collect();

        let msg = RepoAnnounce {
            repo_key: format!("user{}/repo{}", i % 50, i % 100),
            object_ids,
            refs,
        };

        let encoded = msg.encode();

        // Randomly corrupt some messages (10%)
        let data = if i % 10 == 0 {
            corrupt_message(&encoded, 1)
        } else {
            encoded.to_vec()
        };

        match Message::decode(&data) {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    // At least 85% should succeed (some are intentionally corrupted)
    let success_rate = successes as f64 / iterations as f64;
    assert!(
        success_rate >= 0.85,
        "Success rate {} is too low",
        success_rate
    );

    println!(
        "Stress test: {} successes, {} failures ({:.1}% success rate)",
        successes,
        failures,
        success_rate * 100.0
    );
}

/// Test concurrent message processing (simulated)
#[test]
fn test_concurrent_message_processing() {
    use std::thread;

    let num_threads = 4;
    let messages_per_thread = 250;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            thread::spawn(move || {
                let mut successes = 0;
                for i in 0..messages_per_thread {
                    let msg = ObjectData {
                        repo_key: format!("thread{}/repo{}", thread_id, i),
                        objects: vec![GitObject::blob(
                            format!("content from thread {} iteration {}", thread_id, i)
                                .into_bytes(),
                        )],
                    };

                    let encoded = msg.encode();
                    if Message::decode(&encoded).is_ok() {
                        successes += 1;
                    }
                }
                successes
            })
        })
        .collect();

    let total_successes: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

    assert_eq!(
        total_successes,
        num_threads * messages_per_thread,
        "All concurrent operations should succeed"
    );
}

// ============================================================================
// RefUpdate Edge Cases
// ============================================================================

/// Test RefUpdate with zero object IDs
#[test]
fn test_ref_update_zero_ids() {
    let zero_id = ObjectId::from_bytes([0u8; 20]);

    let msg = RefUpdate {
        repo_key: "test/zero".to_string(),
        ref_name: "refs/heads/main".to_string(),
        old_id: zero_id,
        new_id: zero_id,
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::RefUpdate(update) => {
            assert_eq!(update.old_id, zero_id);
            assert_eq!(update.new_id, zero_id);
        }
        _ => panic!("Expected RefUpdate"),
    }
}

/// Test RefUpdate with max object IDs
#[test]
fn test_ref_update_max_ids() {
    let max_id = ObjectId::from_bytes([0xFF; 20]);

    let msg = RefUpdate {
        repo_key: "test/max".to_string(),
        ref_name: "refs/heads/main".to_string(),
        old_id: max_id,
        new_id: max_id,
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::RefUpdate(update) => {
            assert_eq!(update.old_id, max_id);
            assert_eq!(update.new_id, max_id);
        }
        _ => panic!("Expected RefUpdate"),
    }
}

/// Test all object types in ObjectData
#[test]
fn test_all_object_types() {
    let objects = vec![
        GitObject::new(ObjectType::Blob, Bytes::from("blob content")),
        GitObject::new(ObjectType::Tree, Bytes::from("tree content")),
        GitObject::new(ObjectType::Commit, Bytes::from("commit content")),
        GitObject::new(ObjectType::Tag, Bytes::from("tag content")),
    ];

    let msg = ObjectData {
        repo_key: "test/types".to_string(),
        objects: objects.clone(),
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::ObjectData(data) => {
            assert_eq!(data.objects.len(), 4);
            assert_eq!(data.objects[0].object_type, ObjectType::Blob);
            assert_eq!(data.objects[1].object_type, ObjectType::Tree);
            assert_eq!(data.objects[2].object_type, ObjectType::Commit);
            assert_eq!(data.objects[3].object_type, ObjectType::Tag);
        }
        _ => panic!("Expected ObjectData"),
    }
}

/// Test empty ObjectData
#[test]
fn test_empty_object_data() {
    let msg = ObjectData {
        repo_key: "test/empty".to_string(),
        objects: vec![],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::ObjectData(data) => {
            assert!(data.objects.is_empty());
        }
        _ => panic!("Expected ObjectData"),
    }
}

/// Test binary content in objects
#[test]
fn test_binary_object_content() {
    // All possible byte values
    let binary_data: Vec<u8> = (0..=255).collect();
    let blob = GitObject::blob(binary_data.clone());

    let msg = ObjectData {
        repo_key: "test/binary".to_string(),
        objects: vec![blob],
    };

    let encoded = msg.encode();
    let decoded = Message::decode(&encoded).expect("Should decode");

    match decoded {
        Message::ObjectData(data) => {
            assert_eq!(data.objects[0].data.as_ref(), binary_data.as_slice());
        }
        _ => panic!("Expected ObjectData"),
    }
}
