//! End-to-end tests for the consensus layer.
//!
//! These tests verify:
//! - Consensus API endpoints work correctly
//! - Transaction submission and mempool operations
//! - Block finalization in single-node mode

use axum::{body::Body, http::Request};
use guts_auth::AuthStore;
use guts_ci::CiStore;
use guts_collaboration::CollaborationStore;
use guts_compat::CompatStore;
use guts_consensus::{
    ConsensusEngine, EngineConfig, Mempool, MempoolConfig, ValidatorConfig, ValidatorSet,
};
use guts_node::api::{create_router, AppState};
use guts_node::consensus_app::GutsApplication;
use guts_node::health::HealthState;
use guts_realtime::EventHub;
use guts_storage::RepoStore;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

/// Helper to extract JSON body from response
async fn json_body(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// Creates a test app state with consensus enabled.
fn create_test_state_with_consensus() -> (AppState, Arc<GutsApplication>) {
    let repos = Arc::new(RepoStore::new());
    let collaboration = Arc::new(CollaborationStore::new());
    let auth = Arc::new(AuthStore::new());
    let realtime = Arc::new(EventHub::new());
    let ci = Arc::new(CiStore::new());
    let compat = Arc::new(CompatStore::new());

    // Create mempool
    let mempool_config = MempoolConfig {
        max_transactions: 1000,
        max_transaction_age: Duration::from_secs(600),
        max_transactions_per_block: 100,
    };
    let mempool = Arc::new(Mempool::new(mempool_config));

    // Create consensus engine config (single-node mode)
    let engine_config = EngineConfig {
        block_time: Duration::from_millis(100), // Fast for tests
        max_txs_per_block: 100,
        max_block_size: 10 * 1024 * 1024,
        view_timeout_multiplier: 2.0,
        consensus_enabled: false, // Single-node mode for tests
    };

    // Create validator with test key
    use commonware_cryptography::PrivateKeyExt;
    let validator_key = commonware_cryptography::ed25519::PrivateKey::from_seed(42);
    let pubkey = guts_consensus::SerializablePublicKey::from_pubkey(
        &commonware_cryptography::Signer::public_key(&validator_key),
    );
    let validator = guts_consensus::Validator::new(
        pubkey,
        "test-validator",
        1,
        "127.0.0.1:9000".parse().unwrap(),
    );

    let validator_config = ValidatorConfig {
        min_validators: 0,
        max_validators: 100,
        quorum_threshold: 2.0 / 3.0,
        block_time_ms: 100,
    };
    let validators = ValidatorSet::new(vec![validator], 0, validator_config)
        .expect("Failed to create validators");

    // Create consensus engine
    let consensus = Arc::new(ConsensusEngine::new(
        engine_config,
        Some(validator_key),
        validators,
        mempool.clone(),
    ));

    // Create Guts application
    let guts_app = Arc::new(GutsApplication::new(
        repos.clone(),
        collaboration.clone(),
        auth.clone(),
        realtime.clone(),
    ));

    let state = AppState {
        repos,
        p2p: None,
        consensus: Some(consensus),
        mempool: Some(mempool),
        collaboration,
        auth,
        realtime,
        ci,
        compat,
    };

    (state, guts_app)
}

/// Creates a test app state without consensus.
fn create_test_state_without_consensus() -> AppState {
    AppState {
        repos: Arc::new(RepoStore::new()),
        p2p: None,
        consensus: None,
        mempool: None,
        collaboration: Arc::new(CollaborationStore::new()),
        auth: Arc::new(AuthStore::new()),
        realtime: Arc::new(EventHub::new()),
        ci: Arc::new(CiStore::new()),
        compat: Arc::new(CompatStore::new()),
    }
}

#[tokio::test]
async fn test_consensus_status_endpoint_enabled() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/consensus/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json = json_body(response).await;

    assert_eq!(json["enabled"], true);
    assert!(json["state"].is_string());
    assert!(json["view"].is_number());
    assert!(json["finalized_height"].is_number());
}

#[tokio::test]
async fn test_consensus_status_endpoint_disabled() {
    let state = create_test_state_without_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/consensus/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json = json_body(response).await;

    assert_eq!(json["enabled"], false);
    assert_eq!(json["state"], "Disabled");
}

#[tokio::test]
async fn test_validators_endpoint() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/consensus/validators")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json = json_body(response).await;

    assert_eq!(json["epoch"], 0);
    assert_eq!(json["validator_count"], 1);
    assert!(json["validators"].is_array());
    assert_eq!(json["validators"].as_array().unwrap().len(), 1);
    assert_eq!(json["validators"][0]["name"], "test-validator");
}

#[tokio::test]
async fn test_mempool_stats_endpoint() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/consensus/mempool")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json = json_body(response).await;

    assert_eq!(json["transaction_count"], 0);
    assert!(json["oldest_age_secs"].is_number());
}

#[tokio::test]
async fn test_blocks_endpoint_empty() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/consensus/blocks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let json = json_body(response).await;

    assert!(json.is_array());
    // No blocks finalized yet
}

#[tokio::test]
async fn test_block_by_height_not_found() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/consensus/blocks/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_transaction_submission() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    // Create a test transaction
    use commonware_cryptography::{PrivateKeyExt, Signer};
    let key = commonware_cryptography::ed25519::PrivateKey::from_seed(100);
    let sig = key.sign(Some(b"_GUTS"), b"test-transaction");

    let tx_request = json!({
        "type": "CreateRepository",
        "owner": "alice",
        "name": "test-repo",
        "description": "A test repository",
        "default_branch": "main",
        "visibility": "public",
        "creator_pubkey": hex::encode(key.public_key().as_ref()),
        "signature": hex::encode(sig.as_ref())
    });

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consensus/transactions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&tx_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 202);

    let json = json_body(response).await;

    assert_eq!(json["accepted"], true);
    assert!(json["transaction_id"].is_string());
    assert!(!json["transaction_id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_transaction_submission_without_consensus() {
    let state = create_test_state_without_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    let tx_request = json!({
        "type": "CreateRepository",
        "owner": "alice",
        "name": "test-repo",
        "description": "A test repository",
        "default_branch": "main",
        "visibility": "public",
        "creator_pubkey": "0000000000000000000000000000000000000000000000000000000000000000",
        "signature": "0000000000000000000000000000000000000000000000000000000000000000"
    });

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consensus/transactions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&tx_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail because consensus and mempool are not enabled
    assert_eq!(response.status(), 503);
}

#[tokio::test]
async fn test_create_issue_transaction() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();
    let router = create_router(state, health);

    use commonware_cryptography::{PrivateKeyExt, Signer};
    let key = commonware_cryptography::ed25519::PrivateKey::from_seed(101);
    let sig = key.sign(Some(b"_GUTS"), b"test-issue");

    let tx_request = json!({
        "type": "CreateIssue",
        "repo_key": "alice/test-repo",
        "title": "Test Issue",
        "body": "This is a test issue",
        "author": "alice",
        "creator_pubkey": hex::encode(key.public_key().as_ref()),
        "signature": hex::encode(sig.as_ref())
    });

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consensus/transactions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&tx_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 202);

    let json = json_body(response).await;

    assert_eq!(json["accepted"], true);
}

#[tokio::test]
async fn test_mempool_transaction_count_increases() {
    let (state, _app) = create_test_state_with_consensus();
    let health = HealthState::new();

    // Get initial mempool count
    let initial_count = state.mempool.as_ref().unwrap().len();
    assert_eq!(initial_count, 0);

    // Submit a transaction
    use commonware_cryptography::{PrivateKeyExt, Signer};
    let key = commonware_cryptography::ed25519::PrivateKey::from_seed(102);
    let sig = key.sign(Some(b"_GUTS"), b"test");

    let tx = guts_consensus::Transaction::CreateRepository {
        owner: "bob".to_string(),
        name: "mempool-test".to_string(),
        description: "Test".to_string(),
        default_branch: "main".to_string(),
        visibility: "public".to_string(),
        creator: guts_consensus::SerializablePublicKey::from_pubkey(&key.public_key()),
        signature: guts_consensus::SerializableSignature::from_signature(&sig),
    };

    state
        .consensus
        .as_ref()
        .unwrap()
        .submit_transaction(tx)
        .await
        .unwrap();

    // Verify mempool count increased
    let new_count = state.mempool.as_ref().unwrap().len();
    assert_eq!(new_count, 1);

    // Get router and check mempool endpoint
    let router = create_router(state, health);
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/consensus/mempool")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = json_body(response).await;

    assert_eq!(json["transaction_count"], 1);
}

#[tokio::test]
async fn test_guts_application_state_root() {
    let repos = Arc::new(RepoStore::new());
    let collaboration = Arc::new(CollaborationStore::new());
    let auth = Arc::new(AuthStore::new());
    let realtime = Arc::new(EventHub::new());

    let app = GutsApplication::new(repos.clone(), collaboration, auth, realtime);

    // Initial height should be 0
    use guts_consensus::ConsensusApplication;
    assert_eq!(app.current_height(), 0);

    // Compute initial state root
    let root1 = app.compute_state_root(&[]).await.unwrap();

    // Create a repository
    repos.create("test-repo", "alice").unwrap();

    // State root should still be the same (we haven't applied a block)
    let root2 = app.compute_state_root(&[]).await.unwrap();
    assert_eq!(root1, root2);
}

#[tokio::test]
async fn test_guts_application_verify_transaction() {
    let repos = Arc::new(RepoStore::new());
    let collaboration = Arc::new(CollaborationStore::new());
    let auth = Arc::new(AuthStore::new());
    let realtime = Arc::new(EventHub::new());

    let app = GutsApplication::new(repos.clone(), collaboration, auth, realtime);

    use commonware_cryptography::{PrivateKeyExt, Signer};
    let key = commonware_cryptography::ed25519::PrivateKey::from_seed(103);
    let sig = key.sign(Some(b"_GUTS"), b"test");

    // Create a transaction
    let tx = guts_consensus::Transaction::CreateRepository {
        owner: "alice".to_string(),
        name: "verify-test".to_string(),
        description: "Test".to_string(),
        default_branch: "main".to_string(),
        visibility: "public".to_string(),
        creator: guts_consensus::SerializablePublicKey::from_pubkey(&key.public_key()),
        signature: guts_consensus::SerializableSignature::from_signature(&sig),
    };

    // First verification should succeed
    use guts_consensus::ConsensusApplication;
    let result = app.verify_transaction(&tx).await;
    assert!(result.is_ok());

    // Create the repo
    repos.create("verify-test", "alice").unwrap();

    // Second verification should fail (repo exists)
    let result = app.verify_transaction(&tx).await;
    assert!(result.is_err());
}
