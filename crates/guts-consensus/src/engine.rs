//! Consensus engine implementation.
//!
//! This module provides the core consensus engine that integrates with
//! commonware-consensus for BFT agreement.

use crate::block::{Block, BlockId, FinalizedBlock};
use crate::error::{ConsensusError, Result};
use crate::mempool::Mempool;
use crate::transaction::{
    SerializablePublicKey, SerializableSignature, Transaction, TransactionId,
};
use crate::validator::ValidatorSet;
use async_trait::async_trait;
use commonware_cryptography::Signer;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

/// Configuration for the consensus engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Target block time.
    pub block_time: Duration,

    /// Maximum transactions per block.
    pub max_txs_per_block: usize,

    /// Maximum block size in bytes.
    pub max_block_size: usize,

    /// View timeout multiplier.
    pub view_timeout_multiplier: f64,

    /// Enable consensus (false for single-node mode).
    pub consensus_enabled: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            block_time: Duration::from_millis(2000),
            max_txs_per_block: 1000,
            max_block_size: 10 * 1024 * 1024,
            view_timeout_multiplier: 2.0,
            consensus_enabled: true,
        }
    }
}

/// State of the consensus engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// Engine is starting up.
    Starting,
    /// Engine is syncing from peers.
    Syncing,
    /// Engine is actively participating in consensus.
    Active,
    /// Engine is a follower (non-validator).
    Following,
    /// Engine is stopped.
    Stopped,
}

/// Events emitted by the consensus engine.
#[derive(Debug, Clone)]
pub enum ConsensusEvent {
    /// A new block was proposed.
    BlockProposed {
        height: u64,
        producer: SerializablePublicKey,
        tx_count: usize,
    },
    /// A block was finalized.
    BlockFinalized {
        height: u64,
        block_id: BlockId,
        tx_count: usize,
    },
    /// View changed (new leader).
    ViewChanged {
        view: u64,
        leader: SerializablePublicKey,
    },
    /// Consensus state changed.
    StateChanged { old: EngineState, new: EngineState },
    /// Transaction was included in a block.
    TransactionIncluded {
        tx_id: TransactionId,
        block_height: u64,
    },
}

/// Application interface for the consensus engine.
///
/// This trait is implemented by the Guts node to handle finalized blocks
/// and provide state roots.
#[async_trait]
pub trait ConsensusApplication: Send + Sync {
    /// Called when a block is finalized.
    async fn on_block_finalized(&self, block: &FinalizedBlock) -> Result<()>;

    /// Computes the state root after applying transactions.
    async fn compute_state_root(&self, transactions: &[Transaction]) -> Result<[u8; 32]>;

    /// Verifies that a transaction is valid for inclusion.
    async fn verify_transaction(&self, transaction: &Transaction) -> Result<()>;

    /// Gets the current height.
    fn current_height(&self) -> u64;
}

/// The consensus engine.
pub struct ConsensusEngine {
    /// Engine configuration.
    config: EngineConfig,

    /// Our validator key (if we are a validator).
    validator_key: Option<commonware_cryptography::ed25519::PrivateKey>,

    /// Validator set.
    validators: Arc<RwLock<ValidatorSet>>,

    /// Transaction mempool.
    mempool: Arc<Mempool>,

    /// Finalized blocks by height.
    blocks: Arc<RwLock<HashMap<u64, FinalizedBlock>>>,

    /// Current engine state.
    state: Arc<RwLock<EngineState>>,

    /// Current view number.
    view: Arc<RwLock<u64>>,

    /// Latest finalized height.
    finalized_height: Arc<RwLock<u64>>,

    /// Event broadcaster.
    events: broadcast::Sender<ConsensusEvent>,

    /// Transaction submission channel.
    tx_sender: mpsc::Sender<Transaction>,

    /// Transaction receiver (owned by engine runner).
    tx_receiver: Arc<RwLock<Option<mpsc::Receiver<Transaction>>>>,
}

impl ConsensusEngine {
    /// Creates a new consensus engine.
    pub fn new(
        config: EngineConfig,
        validator_key: Option<commonware_cryptography::ed25519::PrivateKey>,
        validators: ValidatorSet,
        mempool: Arc<Mempool>,
    ) -> Self {
        let (events, _) = broadcast::channel(1024);
        let (tx_sender, tx_receiver) = mpsc::channel(10_000);

        Self {
            config,
            validator_key,
            validators: Arc::new(RwLock::new(validators)),
            mempool,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(EngineState::Starting)),
            view: Arc::new(RwLock::new(0)),
            finalized_height: Arc::new(RwLock::new(0)),
            events,
            tx_sender,
            tx_receiver: Arc::new(RwLock::new(Some(tx_receiver))),
        }
    }

    /// Returns a handle for submitting transactions.
    pub fn transaction_sender(&self) -> mpsc::Sender<Transaction> {
        self.tx_sender.clone()
    }

    /// Subscribes to consensus events.
    pub fn subscribe(&self) -> broadcast::Receiver<ConsensusEvent> {
        self.events.subscribe()
    }

    /// Returns the current engine state.
    pub fn state(&self) -> EngineState {
        *self.state.read()
    }

    /// Returns the current view.
    pub fn view(&self) -> u64 {
        *self.view.read()
    }

    /// Returns the latest finalized height.
    pub fn finalized_height(&self) -> u64 {
        *self.finalized_height.read()
    }

    /// Checks if we are the leader for the current view.
    pub fn is_leader(&self) -> bool {
        if let Some(ref key) = self.validator_key {
            let view = *self.view.read();
            let validators = self.validators.read();
            if let Some(leader) = validators.leader_for_view(view) {
                let our_pubkey = SerializablePublicKey::from_pubkey(&key.public_key());
                return leader.pubkey == our_pubkey;
            }
        }
        false
    }

    /// Gets the current leader.
    pub fn current_leader(&self) -> Option<SerializablePublicKey> {
        let view = *self.view.read();
        let validators = self.validators.read();
        validators.leader_for_view(view).map(|v| v.pubkey.clone())
    }

    /// Submits a transaction to the mempool.
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId> {
        // Add to mempool
        let id = self.mempool.add(tx.clone())?;

        // Send to engine for processing
        self.tx_sender
            .send(tx)
            .await
            .map_err(|e| ConsensusError::EngineError(e.to_string()))?;

        Ok(id)
    }

    /// Gets a finalized block by height.
    pub fn get_block(&self, height: u64) -> Option<FinalizedBlock> {
        self.blocks.read().get(&height).cloned()
    }

    /// Gets the validator set.
    pub fn validators(&self) -> Arc<RwLock<ValidatorSet>> {
        self.validators.clone()
    }

    /// Runs the consensus engine.
    ///
    /// This is the main event loop that drives consensus.
    pub async fn run<A: ConsensusApplication>(&self, app: Arc<A>) -> Result<()> {
        self.set_state(EngineState::Active);

        // Take ownership of the transaction receiver
        let mut tx_receiver = self
            .tx_receiver
            .write()
            .take()
            .ok_or_else(|| ConsensusError::EngineError("engine already running".into()))?;

        let block_time = self.config.block_time;
        let mut block_interval = tokio::time::interval(block_time);

        loop {
            tokio::select! {
                // Receive transactions
                Some(tx) = tx_receiver.recv() => {
                    self.handle_transaction(tx, &app).await?;
                }

                // Block proposal timer
                _ = block_interval.tick() => {
                    if self.is_leader() && self.config.consensus_enabled {
                        self.propose_block(&app).await?;
                    }
                }
            }
        }
    }

    /// Handles an incoming transaction.
    async fn handle_transaction<A: ConsensusApplication>(
        &self,
        tx: Transaction,
        app: &Arc<A>,
    ) -> Result<()> {
        // Verify the transaction is valid
        app.verify_transaction(&tx).await?;

        tracing::debug!(
            tx_id = %tx.id(),
            kind = tx.kind(),
            "transaction verified and pending"
        );

        Ok(())
    }

    /// Proposes a new block (called when we are the leader).
    async fn propose_block<A: ConsensusApplication>(&self, app: &Arc<A>) -> Result<()> {
        let validator_key = self
            .validator_key
            .as_ref()
            .ok_or_else(|| ConsensusError::EngineError("not a validator".into()))?;

        // Get transactions from mempool
        let transactions = self.mempool.get_for_proposal();

        if transactions.is_empty() {
            // Don't propose empty blocks
            return Ok(());
        }

        let height = app.current_height() + 1;
        let parent = self
            .blocks
            .read()
            .get(&(height - 1))
            .map(|b| b.id())
            .unwrap_or(BlockId::GENESIS_PARENT);

        // Compute state root
        let state_root = app.compute_state_root(&transactions).await?;

        // Create the block
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let producer = SerializablePublicKey::from_pubkey(&validator_key.public_key());

        let block = Block::new(
            height,
            parent,
            producer.clone(),
            timestamp,
            transactions.clone(),
            state_root,
        );

        let tx_count = block.tx_count();

        // Emit block proposed event
        let _ = self.events.send(ConsensusEvent::BlockProposed {
            height,
            producer,
            tx_count,
        });

        tracing::info!(
            height,
            tx_count,
            block_id = %block.id(),
            "proposed block"
        );

        // In single-node mode, immediately finalize
        if !self.config.consensus_enabled {
            self.finalize_block(block, 0, vec![], app).await?;
        }

        Ok(())
    }

    /// Finalizes a block after consensus is reached.
    async fn finalize_block<A: ConsensusApplication>(
        &self,
        block: Block,
        view: u64,
        signatures: Vec<(SerializablePublicKey, SerializableSignature)>,
        app: &Arc<A>,
    ) -> Result<()> {
        let height = block.height();
        let block_id = block.id();
        let tx_count = block.tx_count();

        // Create finalized block
        let finalized = FinalizedBlock::new(block, view, signatures);

        // Store the block
        self.blocks.write().insert(height, finalized.clone());

        // Update finalized height
        *self.finalized_height.write() = height;

        // Remove transactions from mempool
        let tx_ids: Vec<_> = finalized
            .block
            .transactions
            .iter()
            .map(|tx| tx.id())
            .collect();
        self.mempool.remove_batch(&tx_ids);

        // Notify application
        app.on_block_finalized(&finalized).await?;

        // Emit events
        let _ = self.events.send(ConsensusEvent::BlockFinalized {
            height,
            block_id,
            tx_count,
        });

        for tx in &finalized.block.transactions {
            let _ = self.events.send(ConsensusEvent::TransactionIncluded {
                tx_id: tx.id(),
                block_height: height,
            });
        }

        tracing::info!(
            height,
            tx_count,
            block_id = %block_id,
            "finalized block"
        );

        Ok(())
    }

    /// Sets the engine state and emits an event.
    fn set_state(&self, new_state: EngineState) {
        let old_state = {
            let mut state = self.state.write();
            let old = *state;
            *state = new_state;
            old
        };

        if old_state != new_state {
            let _ = self.events.send(ConsensusEvent::StateChanged {
                old: old_state,
                new: new_state,
            });
        }
    }

    /// Advances to the next view.
    pub fn advance_view(&self) {
        let new_view = {
            let mut view = self.view.write();
            *view += 1;
            *view
        };

        if let Some(leader) = self.current_leader() {
            let _ = self.events.send(ConsensusEvent::ViewChanged {
                view: new_view,
                leader,
            });
        }

        tracing::debug!(view = new_view, "advanced to new view");
    }

    /// Stops the engine.
    pub fn stop(&self) {
        self.set_state(EngineState::Stopped);
    }
}

/// A no-op application for testing.
pub struct NoOpApplication {
    height: RwLock<u64>,
}

impl NoOpApplication {
    /// Creates a new no-op application.
    pub fn new() -> Self {
        Self {
            height: RwLock::new(0),
        }
    }
}

impl Default for NoOpApplication {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsensusApplication for NoOpApplication {
    async fn on_block_finalized(&self, block: &FinalizedBlock) -> Result<()> {
        *self.height.write() = block.height();
        Ok(())
    }

    async fn compute_state_root(&self, _transactions: &[Transaction]) -> Result<[u8; 32]> {
        Ok([0u8; 32])
    }

    async fn verify_transaction(&self, _transaction: &Transaction) -> Result<()> {
        Ok(())
    }

    fn current_height(&self) -> u64 {
        *self.height.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::generate_devnet_genesis;
    use crate::mempool::MempoolConfig;
    use commonware_cryptography::{PrivateKeyExt, Signer};

    fn test_tx(seed: u64) -> Transaction {
        use commonware_cryptography::ed25519;

        let key = ed25519::PrivateKey::from_seed(seed);
        let sig = key.sign(Some(b"_GUTS"), b"test");

        Transaction::CreateRepository {
            owner: "alice".into(),
            name: format!("repo-{}", seed),
            description: "A test".into(),
            default_branch: "main".into(),
            visibility: "public".into(),
            creator: SerializablePublicKey::from_pubkey(&key.public_key()),
            signature: SerializableSignature::from_signature(&sig),
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let genesis = generate_devnet_genesis(4);
        let validators = genesis.into_validator_set().unwrap();
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));

        let config = EngineConfig {
            consensus_enabled: false,
            ..Default::default()
        };

        let key = commonware_cryptography::ed25519::PrivateKey::from_seed(0);
        let engine = ConsensusEngine::new(config, Some(key), validators, mempool);

        assert_eq!(engine.state(), EngineState::Starting);
        assert_eq!(engine.view(), 0);
    }

    #[tokio::test]
    async fn test_transaction_submission() {
        let genesis = generate_devnet_genesis(4);
        let validators = genesis.into_validator_set().unwrap();
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));

        let config = EngineConfig {
            consensus_enabled: false,
            ..Default::default()
        };

        let key = commonware_cryptography::ed25519::PrivateKey::from_seed(0);
        let engine = ConsensusEngine::new(config, Some(key), validators, mempool.clone());

        let tx = test_tx(1);
        let id = engine.submit_transaction(tx).await.unwrap();

        assert!(mempool.contains(&id));
    }

    #[tokio::test]
    async fn test_leader_rotation() {
        let genesis = generate_devnet_genesis(4);
        let validators = genesis.into_validator_set().unwrap();
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));

        let config = EngineConfig::default();
        let key = commonware_cryptography::ed25519::PrivateKey::from_seed(0);
        let engine = ConsensusEngine::new(config, Some(key), validators, mempool);

        // View 0: validator 0 should be leader
        assert!(engine.is_leader());

        // Advance view
        engine.advance_view();
        assert_eq!(engine.view(), 1);

        // Now validator 1 should be leader (we are validator 0)
        assert!(!engine.is_leader());
    }
}
