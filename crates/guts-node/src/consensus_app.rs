//! Consensus application implementation.
//!
//! This module provides the implementation of the `ConsensusApplication` trait
//! that applies finalized transactions to the node's state (repositories,
//! collaboration, authentication, etc.).

use async_trait::async_trait;
use guts_auth::AuthStore;
use guts_collaboration::CollaborationStore;
use guts_consensus::{ConsensusApplication, ConsensusError, FinalizedBlock, Result, Transaction};
use guts_realtime::{EventHub, EventKind};
use guts_storage::RepoStore;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// The Guts application that applies consensus transactions to state.
pub struct GutsApplication {
    /// Repository store.
    repos: Arc<RepoStore>,

    /// Collaboration store (PRs, issues, comments).
    /// Used when applying collaboration-related transactions (CreateIssue, CreatePullRequest, etc.)
    #[allow(dead_code)]
    collaboration: Arc<CollaborationStore>,

    /// Auth store (orgs, teams, permissions).
    /// Used when applying auth-related transactions (CreateOrganization, etc.)
    #[allow(dead_code)]
    auth: Arc<AuthStore>,

    /// Real-time event hub for broadcasting updates.
    realtime: Arc<EventHub>,

    /// Current block height.
    height: RwLock<u64>,

    /// Current state root (hash of all state).
    state_root: RwLock<[u8; 32]>,
}

impl GutsApplication {
    /// Creates a new Guts application.
    pub fn new(
        repos: Arc<RepoStore>,
        collaboration: Arc<CollaborationStore>,
        auth: Arc<AuthStore>,
        realtime: Arc<EventHub>,
    ) -> Self {
        Self {
            repos,
            collaboration,
            auth,
            realtime,
            height: RwLock::new(0),
            state_root: RwLock::new([0u8; 32]),
        }
    }

    /// Applies a transaction to the state.
    fn apply_transaction(&self, tx: &Transaction) -> Result<()> {
        match tx {
            Transaction::CreateRepository {
                owner,
                name,
                description: _,
                default_branch: _,
                visibility: _,
                creator: _,
                signature: _,
            } => {
                // Create the repository
                match self.repos.create(name, owner) {
                    Ok(_repo) => {
                        info!(owner = %owner, name = %name, "Created repository via consensus");

                        // Emit event
                        let repo_key = format!("{}/{}", owner, name);
                        self.realtime.emit_event(
                            format!("repo:{}", repo_key),
                            EventKind::RepoCreated,
                            serde_json::json!({
                                "owner": owner,
                                "name": name,
                                "repository": repo_key
                            }),
                        );

                        Ok(())
                    }
                    Err(e) => {
                        warn!(owner = %owner, name = %name, error = %e, "Failed to create repository");
                        Err(ConsensusError::TransactionFailed(format!(
                            "create repository failed: {}",
                            e
                        )))
                    }
                }
            }

            Transaction::DeleteRepository {
                repo_key,
                deleter: _,
                signature: _,
            } => {
                // Note: RepoStore doesn't have a delete method yet
                // For now, just log the deletion
                info!(repo_key = %repo_key, "Repository deletion requested via consensus");

                // Emit event
                self.realtime.emit_event(
                    format!("repo:{}", repo_key),
                    EventKind::Push, // Use Push for now as placeholder
                    serde_json::json!({
                        "type": "repository_deleted",
                        "repository": repo_key
                    }),
                );

                Ok(())
            }

            Transaction::CreateIssue {
                repo_key,
                title,
                description,
                author,
                signer: _,
                signature: _,
            } => {
                // For now, log the issue creation
                // Full implementation would use collaboration.create_issue()
                info!(
                    repo_key = %repo_key,
                    title = %title,
                    author = %author,
                    "Issue creation requested via consensus"
                );

                // Emit event
                self.realtime.emit_event(
                    format!("repo:{}", repo_key),
                    EventKind::IssueOpened,
                    serde_json::json!({
                        "repository": repo_key,
                        "title": title,
                        "author": author,
                        "description": description
                    }),
                );

                Ok(())
            }

            Transaction::CreatePullRequest {
                repo_key,
                title,
                description: _,
                author,
                source_branch,
                target_branch,
                source_commit: _,
                target_commit: _,
                signer: _,
                signature: _,
            } => {
                // For now, log the PR creation
                info!(
                    repo_key = %repo_key,
                    title = %title,
                    author = %author,
                    source_branch = %source_branch,
                    target_branch = %target_branch,
                    "PR creation requested via consensus"
                );

                // Emit event
                self.realtime.emit_event(
                    format!("repo:{}", repo_key),
                    EventKind::PrOpened,
                    serde_json::json!({
                        "repository": repo_key,
                        "title": title,
                        "author": author,
                        "source_branch": source_branch,
                        "target_branch": target_branch
                    }),
                );

                Ok(())
            }

            Transaction::CreateOrganization {
                name,
                display_name,
                creator: _,
                signature: _,
            } => {
                // For now, log the org creation
                info!(
                    name = %name,
                    display_name = %display_name,
                    "Organization creation requested via consensus"
                );

                Ok(())
            }

            // For now, log unimplemented transaction types
            _ => {
                debug!(
                    kind = tx.kind(),
                    "Transaction type not yet fully implemented"
                );
                Ok(())
            }
        }
    }

    /// Computes a simple state root from repository count.
    fn compute_state_root_internal(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Hash repository state
        let repo_count = self.repos.list().len() as u64;
        hasher.update(repo_count.to_le_bytes());

        // Add current height
        hasher.update(self.height.read().to_le_bytes());

        let result = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(&result);
        root
    }
}

#[async_trait]
impl ConsensusApplication for GutsApplication {
    /// Called when a block is finalized.
    async fn on_block_finalized(&self, block: &FinalizedBlock) -> Result<()> {
        let height = block.height();
        let tx_count = block.block.tx_count();

        info!(
            height = height,
            tx_count = tx_count,
            block_id = %block.id(),
            "Applying finalized block"
        );

        // Apply each transaction in order
        for tx in &block.block.transactions {
            if let Err(e) = self.apply_transaction(tx) {
                error!(
                    tx_id = %tx.id(),
                    kind = tx.kind(),
                    error = %e,
                    "Failed to apply transaction"
                );
                // Continue with other transactions - failed transactions are logged but don't
                // halt block application. In production, transaction verification should
                // prevent invalid transactions from being included.
            }
        }

        // Update height
        *self.height.write() = height;

        // Update state root
        let new_root = self.compute_state_root_internal();
        *self.state_root.write() = new_root;

        debug!(
            height = height,
            state_root = hex::encode(new_root),
            "Block application complete"
        );

        Ok(())
    }

    /// Computes the state root after applying transactions.
    async fn compute_state_root(&self, _transactions: &[Transaction]) -> Result<[u8; 32]> {
        // For now, return the current state root
        // In production, we'd simulate applying transactions and return the resulting root
        Ok(*self.state_root.read())
    }

    /// Verifies that a transaction is valid for inclusion.
    async fn verify_transaction(&self, tx: &Transaction) -> Result<()> {
        match tx {
            Transaction::CreateRepository { owner, name, .. } => {
                // Check if repo already exists
                if self.repos.get(owner, name).is_ok() {
                    return Err(ConsensusError::TransactionFailed(format!(
                        "repository {}/{} already exists",
                        owner, name
                    )));
                }
            }
            Transaction::DeleteRepository { repo_key, .. } => {
                let parts: Vec<&str> = repo_key.split('/').collect();
                if parts.len() != 2 {
                    return Err(ConsensusError::TransactionFailed(
                        "invalid repo_key format".into(),
                    ));
                }
                // Check if repo exists
                if self.repos.get(parts[0], parts[1]).is_err() {
                    return Err(ConsensusError::TransactionFailed(format!(
                        "repository {} does not exist",
                        repo_key
                    )));
                }
            }
            _ => {
                // Other transaction types: basic validation
                // In production, we'd verify signatures, permissions, etc.
            }
        }

        Ok(())
    }

    /// Gets the current height.
    fn current_height(&self) -> u64 {
        *self.height.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> GutsApplication {
        GutsApplication::new(
            Arc::new(RepoStore::new()),
            Arc::new(CollaborationStore::new()),
            Arc::new(AuthStore::new()),
            Arc::new(EventHub::new()),
        )
    }

    #[test]
    fn test_guts_application_creation() {
        let app = test_app();
        assert_eq!(app.current_height(), 0);
    }

    #[tokio::test]
    async fn test_transaction_verification() {
        let app = test_app();

        // Create a test transaction
        use commonware_cryptography::{ed25519, PrivateKeyExt, Signer};
        use guts_consensus::{SerializablePublicKey, SerializableSignature};

        let key = ed25519::PrivateKey::from_seed(42);
        let sig = key.sign(Some(b"_GUTS"), b"test");

        let tx = Transaction::CreateRepository {
            owner: "alice".to_string(),
            name: "test-repo".to_string(),
            description: "A test repository".to_string(),
            default_branch: "main".to_string(),
            visibility: "public".to_string(),
            creator: SerializablePublicKey::from_pubkey(&key.public_key()),
            signature: SerializableSignature::from_signature(&sig),
        };

        // First creation should succeed
        let result = app.verify_transaction(&tx).await;
        assert!(result.is_ok());

        // Apply the transaction
        app.apply_transaction(&tx).unwrap();

        // Second verification should fail (repo exists)
        let result = app.verify_transaction(&tx).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_state_root_computation() {
        let app = test_app();

        // Initial state root should be computed
        let root1 = app.compute_state_root_internal();

        // Create a repository
        app.repos.create("test", "alice").unwrap();

        // State root should change
        let root2 = app.compute_state_root_internal();
        assert_ne!(root1, root2);
    }
}
