//! Consensus API endpoints.
//!
//! This module provides HTTP endpoints for interacting with the consensus layer:
//!
//! - **Status**: Get consensus engine state, current view, leader info
//! - **Transactions**: Submit transactions and query pending transactions
//! - **Blocks**: Query finalized blocks by height or hash
//! - **Validators**: Get validator set information
//!
//! ## Endpoint Overview
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/consensus/status` | Consensus engine status |
//! | GET | `/api/consensus/blocks` | List recent finalized blocks |
//! | GET | `/api/consensus/blocks/{height}` | Get block by height |
//! | GET | `/api/consensus/validators` | Current validator set |
//! | GET | `/api/consensus/mempool` | Mempool statistics |
//! | POST | `/api/consensus/transactions` | Submit a transaction |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use guts_consensus::{SerializablePublicKey, SerializableSignature, Transaction};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

/// Consensus status response.
#[derive(Serialize)]
pub struct ConsensusStatusResponse {
    /// Whether consensus is enabled.
    pub enabled: bool,
    /// Current engine state.
    pub state: String,
    /// Current view number.
    pub view: u64,
    /// Latest finalized block height.
    pub finalized_height: u64,
    /// Current leader public key (hex).
    pub current_leader: Option<String>,
    /// Whether this node is the current leader.
    pub is_leader: bool,
    /// Number of pending transactions in mempool.
    pub pending_transactions: usize,
}

/// Block info response.
#[derive(Serialize)]
pub struct BlockInfoResponse {
    /// Block height.
    pub height: u64,
    /// Block ID (hash) in hex.
    pub block_id: String,
    /// Parent block ID in hex.
    pub parent_id: String,
    /// Block producer public key in hex.
    pub producer: String,
    /// Block timestamp (unix milliseconds).
    pub timestamp: u64,
    /// Number of transactions in block.
    pub tx_count: usize,
    /// Transaction root hash in hex.
    pub tx_root: String,
    /// State root hash in hex.
    pub state_root: String,
    /// View number when finalized.
    pub view: u64,
    /// Number of validator signatures.
    pub signature_count: usize,
}

/// Validator info response.
#[derive(Serialize)]
pub struct ValidatorInfoResponse {
    /// Validator name.
    pub name: String,
    /// Public key in hex.
    pub pubkey: String,
    /// Voting weight.
    pub weight: u64,
    /// Network address.
    pub addr: String,
    /// Whether validator is active.
    pub active: bool,
}

/// Validator set response.
#[derive(Serialize)]
pub struct ValidatorSetResponse {
    /// Current epoch.
    pub epoch: u64,
    /// Total weight.
    pub total_weight: u64,
    /// Quorum weight required.
    pub quorum_weight: u64,
    /// Number of validators.
    pub validator_count: usize,
    /// List of validators.
    pub validators: Vec<ValidatorInfoResponse>,
}

/// Mempool statistics response.
#[derive(Serialize)]
pub struct MempoolStatsResponse {
    /// Number of pending transactions.
    pub transaction_count: usize,
    /// Oldest transaction age in seconds.
    pub oldest_age_secs: f64,
    /// Average number of times transactions have been proposed.
    pub average_propose_count: f64,
}

/// Transaction submission request.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum SubmitTransactionRequest {
    /// Create a new repository.
    CreateRepository {
        owner: String,
        name: String,
        description: String,
        default_branch: String,
        visibility: String,
        creator_pubkey: String,
        signature: String,
    },
    /// Create an issue.
    CreateIssue {
        repo_key: String,
        title: String,
        body: String,
        author: String,
        creator_pubkey: String,
        signature: String,
    },
    /// Create a pull request.
    CreatePullRequest {
        repo_key: String,
        title: String,
        description: String,
        author: String,
        source_branch: String,
        target_branch: String,
        source_commit: String,
        target_commit: String,
        creator_pubkey: String,
        signature: String,
    },
}

/// Transaction submission response.
#[derive(Serialize)]
pub struct SubmitTransactionResponse {
    /// Transaction ID (hash) in hex.
    pub transaction_id: String,
    /// Whether the transaction was accepted.
    pub accepted: bool,
    /// Optional error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Creates the consensus API router.
pub fn consensus_routes() -> Router<AppState> {
    Router::new()
        .route("/api/consensus/status", get(get_consensus_status))
        .route("/api/consensus/blocks", get(list_recent_blocks))
        .route("/api/consensus/blocks/{height}", get(get_block_by_height))
        .route("/api/consensus/validators", get(get_validators))
        .route("/api/consensus/mempool", get(get_mempool_stats))
        .route("/api/consensus/transactions", post(submit_transaction))
}

/// Get consensus engine status.
async fn get_consensus_status(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref consensus) = state.consensus {
        let current_leader = consensus.current_leader().map(|pk| pk.to_hex());
        let pending_transactions = state.mempool.as_ref().map(|m| m.len()).unwrap_or(0);

        Json(ConsensusStatusResponse {
            enabled: true,
            state: format!("{:?}", consensus.state()),
            view: consensus.view(),
            finalized_height: consensus.finalized_height(),
            current_leader,
            is_leader: consensus.is_leader(),
            pending_transactions,
        })
    } else {
        Json(ConsensusStatusResponse {
            enabled: false,
            state: "Disabled".to_string(),
            view: 0,
            finalized_height: 0,
            current_leader: None,
            is_leader: false,
            pending_transactions: 0,
        })
    }
}

/// List recent finalized blocks.
async fn list_recent_blocks(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref consensus) = state.consensus {
        let finalized_height = consensus.finalized_height();
        let start_height = finalized_height.saturating_sub(9); // Last 10 blocks

        let mut blocks = Vec::new();
        for height in start_height..=finalized_height {
            if let Some(block) = consensus.get_block(height) {
                blocks.push(BlockInfoResponse {
                    height: block.height(),
                    block_id: block.id().to_hex(),
                    parent_id: block.block.parent().to_hex(),
                    producer: block.block.header.producer.to_hex(),
                    timestamp: block.block.timestamp(),
                    tx_count: block.block.tx_count(),
                    tx_root: hex::encode(block.block.header.tx_root),
                    state_root: hex::encode(block.block.header.state_root),
                    view: block.view,
                    signature_count: block.signature_count(),
                });
            }
        }

        (StatusCode::OK, Json(blocks))
    } else {
        (StatusCode::OK, Json(Vec::<BlockInfoResponse>::new()))
    }
}

/// Get a block by height.
async fn get_block_by_height(
    State(state): State<AppState>,
    Path(height): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(ref consensus) = state.consensus {
        if let Some(block) = consensus.get_block(height) {
            Ok(Json(BlockInfoResponse {
                height: block.height(),
                block_id: block.id().to_hex(),
                parent_id: block.block.parent().to_hex(),
                producer: block.block.header.producer.to_hex(),
                timestamp: block.block.timestamp(),
                tx_count: block.block.tx_count(),
                tx_root: hex::encode(block.block.header.tx_root),
                state_root: hex::encode(block.block.header.state_root),
                view: block.view,
                signature_count: block.signature_count(),
            }))
        } else {
            Err((
                StatusCode::NOT_FOUND,
                format!("Block at height {} not found", height),
            ))
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Consensus is not enabled".to_string(),
        ))
    }
}

/// Get current validator set.
async fn get_validators(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref consensus) = state.consensus {
        let validators_lock = consensus.validators();
        let validators = validators_lock.read();

        let validator_list: Vec<ValidatorInfoResponse> = validators
            .validators()
            .iter()
            .map(|v| ValidatorInfoResponse {
                name: v.name.clone(),
                pubkey: v.pubkey.to_hex(),
                weight: v.weight,
                addr: v.addr.to_string(),
                active: v.active,
            })
            .collect();

        Json(ValidatorSetResponse {
            epoch: validators.epoch(),
            total_weight: validators.total_weight(),
            quorum_weight: validators.quorum_weight(),
            validator_count: validators.len(),
            validators: validator_list,
        })
    } else {
        Json(ValidatorSetResponse {
            epoch: 0,
            total_weight: 0,
            quorum_weight: 0,
            validator_count: 0,
            validators: Vec::new(),
        })
    }
}

/// Get mempool statistics.
async fn get_mempool_stats(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref mempool) = state.mempool {
        let stats = mempool.stats();
        Json(MempoolStatsResponse {
            transaction_count: stats.transaction_count,
            oldest_age_secs: stats.oldest_transaction_age.as_secs_f64(),
            average_propose_count: stats.average_propose_count,
        })
    } else {
        Json(MempoolStatsResponse {
            transaction_count: 0,
            oldest_age_secs: 0.0,
            average_propose_count: 0.0,
        })
    }
}

/// Submit a transaction to the mempool.
async fn submit_transaction(
    State(state): State<AppState>,
    Json(req): Json<SubmitTransactionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<SubmitTransactionResponse>)> {
    // Convert request to transaction
    let transaction = match req {
        SubmitTransactionRequest::CreateRepository {
            owner,
            name,
            description,
            default_branch,
            visibility,
            creator_pubkey,
            signature,
        } => Transaction::CreateRepository {
            owner,
            name,
            description,
            default_branch,
            visibility,
            creator: SerializablePublicKey::from_hex(&creator_pubkey),
            signature: SerializableSignature::from_hex(&signature),
        },
        SubmitTransactionRequest::CreateIssue {
            repo_key,
            title,
            body,
            author,
            creator_pubkey,
            signature,
        } => Transaction::CreateIssue {
            repo_key,
            title,
            description: body,
            author,
            signer: SerializablePublicKey::from_hex(&creator_pubkey),
            signature: SerializableSignature::from_hex(&signature),
        },
        SubmitTransactionRequest::CreatePullRequest {
            repo_key,
            title,
            description,
            author,
            source_branch,
            target_branch,
            source_commit,
            target_commit,
            creator_pubkey,
            signature,
        } => {
            // Parse source_commit and target_commit as ObjectId (20-byte hex)
            let source_oid = guts_storage::ObjectId::from_hex(&source_commit).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(SubmitTransactionResponse {
                        transaction_id: String::new(),
                        accepted: false,
                        error: Some("Invalid source_commit hex".to_string()),
                    }),
                )
            })?;
            let target_oid = guts_storage::ObjectId::from_hex(&target_commit).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(SubmitTransactionResponse {
                        transaction_id: String::new(),
                        accepted: false,
                        error: Some("Invalid target_commit hex".to_string()),
                    }),
                )
            })?;

            Transaction::CreatePullRequest {
                repo_key,
                title,
                description,
                author,
                source_branch,
                target_branch,
                source_commit: source_oid,
                target_commit: target_oid,
                signer: SerializablePublicKey::from_hex(&creator_pubkey),
                signature: SerializableSignature::from_hex(&signature),
            }
        }
    };

    // Submit to consensus or mempool
    if let Some(ref consensus) = state.consensus {
        match consensus.submit_transaction(transaction.clone()).await {
            Ok(id) => Ok((
                StatusCode::ACCEPTED,
                Json(SubmitTransactionResponse {
                    transaction_id: id.to_hex(),
                    accepted: true,
                    error: None,
                }),
            )),
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(SubmitTransactionResponse {
                    transaction_id: String::new(),
                    accepted: false,
                    error: Some(e.to_string()),
                }),
            )),
        }
    } else if let Some(ref mempool) = state.mempool {
        match mempool.add(transaction) {
            Ok(id) => Ok((
                StatusCode::ACCEPTED,
                Json(SubmitTransactionResponse {
                    transaction_id: id.to_hex(),
                    accepted: true,
                    error: None,
                }),
            )),
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(SubmitTransactionResponse {
                    transaction_id: String::new(),
                    accepted: false,
                    error: Some(e.to_string()),
                }),
            )),
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(SubmitTransactionResponse {
                transaction_id: String::new(),
                accepted: false,
                error: Some("Consensus and mempool are not enabled".to_string()),
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_status_response_serialization() {
        let response = ConsensusStatusResponse {
            enabled: true,
            state: "Active".to_string(),
            view: 10,
            finalized_height: 100,
            current_leader: Some("abc123".to_string()),
            is_leader: false,
            pending_transactions: 5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"view\":10"));
    }

    #[test]
    fn test_block_info_response_serialization() {
        let response = BlockInfoResponse {
            height: 42,
            block_id: "abc".to_string(),
            parent_id: "def".to_string(),
            producer: "xyz".to_string(),
            timestamp: 1234567890,
            tx_count: 10,
            tx_root: "root".to_string(),
            state_root: "state".to_string(),
            view: 5,
            signature_count: 3,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"height\":42"));
        assert!(json.contains("\"tx_count\":10"));
    }
}
