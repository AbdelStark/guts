//! Transaction mempool for pending transactions.
//!
//! The mempool holds transactions that have been submitted but not yet
//! included in a finalized block.

use crate::error::{ConsensusError, Result};
use crate::transaction::{Transaction, TransactionId};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Configuration for the mempool.
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum number of transactions in the mempool.
    pub max_transactions: usize,

    /// Maximum transaction age before eviction.
    pub max_transaction_age: Duration,

    /// Maximum transactions per block proposal.
    pub max_transactions_per_block: usize,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_transactions: 10_000,
            max_transaction_age: Duration::from_secs(600), // 10 minutes
            max_transactions_per_block: 1000,
        }
    }
}

/// Metadata about a pending transaction.
#[derive(Debug, Clone)]
struct PendingTransaction {
    /// The transaction.
    transaction: Transaction,

    /// When the transaction was added.
    added_at: Instant,

    /// Number of times this transaction has been proposed but not finalized.
    propose_count: u32,
}

/// The transaction mempool.
pub struct Mempool {
    /// Configuration.
    config: MempoolConfig,

    /// Pending transactions indexed by ID.
    transactions: RwLock<HashMap<TransactionId, PendingTransaction>>,

    /// Order of transaction arrival (for FIFO proposal).
    order: RwLock<VecDeque<TransactionId>>,
}

impl Mempool {
    /// Creates a new mempool with the given configuration.
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            config,
            transactions: RwLock::new(HashMap::new()),
            order: RwLock::new(VecDeque::new()),
        }
    }

    /// Creates a new mempool with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(MempoolConfig::default())
    }

    /// Adds a transaction to the mempool.
    pub fn add(&self, transaction: Transaction) -> Result<TransactionId> {
        let id = transaction.id();

        let mut txs = self.transactions.write();
        let mut order = self.order.write();

        // Check for duplicates
        if txs.contains_key(&id) {
            return Err(ConsensusError::DuplicateTransaction(id.to_hex()));
        }

        // Evict old transactions if at capacity
        while txs.len() >= self.config.max_transactions {
            if let Some(old_id) = order.pop_front() {
                txs.remove(&old_id);
                tracing::debug!(?old_id, "evicted transaction due to mempool capacity");
            } else {
                break;
            }
        }

        // Add the transaction
        let pending = PendingTransaction {
            transaction,
            added_at: Instant::now(),
            propose_count: 0,
        };

        txs.insert(id, pending);
        order.push_back(id);

        tracing::trace!(?id, "added transaction to mempool");

        Ok(id)
    }

    /// Gets a transaction by ID.
    pub fn get(&self, id: &TransactionId) -> Option<Transaction> {
        self.transactions
            .read()
            .get(id)
            .map(|p| p.transaction.clone())
    }

    /// Checks if a transaction exists in the mempool.
    pub fn contains(&self, id: &TransactionId) -> bool {
        self.transactions.read().contains_key(id)
    }

    /// Returns the number of pending transactions.
    pub fn len(&self) -> usize {
        self.transactions.read().len()
    }

    /// Returns true if the mempool is empty.
    pub fn is_empty(&self) -> bool {
        self.transactions.read().is_empty()
    }

    /// Removes a transaction from the mempool.
    pub fn remove(&self, id: &TransactionId) -> Option<Transaction> {
        let mut txs = self.transactions.write();
        let mut order = self.order.write();

        if let Some(pending) = txs.remove(id) {
            order.retain(|tx_id| tx_id != id);
            Some(pending.transaction)
        } else {
            None
        }
    }

    /// Removes multiple transactions from the mempool.
    pub fn remove_batch(&self, ids: &[TransactionId]) {
        let mut txs = self.transactions.write();
        let mut order = self.order.write();

        for id in ids {
            txs.remove(id);
        }

        order.retain(|tx_id| !ids.contains(tx_id));

        tracing::debug!(count = ids.len(), "removed batch from mempool");
    }

    /// Gets transactions for a block proposal.
    ///
    /// Returns up to `max_transactions_per_block` transactions in FIFO order.
    pub fn get_for_proposal(&self) -> Vec<Transaction> {
        let now = Instant::now();
        let mut txs = self.transactions.write();
        let order = self.order.read();

        let mut result = Vec::with_capacity(self.config.max_transactions_per_block);

        for id in order.iter() {
            if result.len() >= self.config.max_transactions_per_block {
                break;
            }

            if let Some(pending) = txs.get_mut(id) {
                // Skip if too old
                if now.duration_since(pending.added_at) > self.config.max_transaction_age {
                    continue;
                }

                pending.propose_count += 1;
                result.push(pending.transaction.clone());
            }
        }

        result
    }

    /// Reaps expired transactions from the mempool.
    pub fn reap_expired(&self) -> usize {
        let now = Instant::now();
        let mut txs = self.transactions.write();
        let mut order = self.order.write();

        let initial_len = txs.len();
        let expired_ids: Vec<_> = txs
            .iter()
            .filter(|(_, pending)| {
                now.duration_since(pending.added_at) > self.config.max_transaction_age
            })
            .map(|(id, _)| *id)
            .collect();

        for id in &expired_ids {
            txs.remove(id);
        }

        order.retain(|id| !expired_ids.contains(id));

        let removed = initial_len - txs.len();
        if removed > 0 {
            tracing::debug!(removed, "reaped expired transactions");
        }

        removed
    }

    /// Returns statistics about the mempool.
    pub fn stats(&self) -> MempoolStats {
        let txs = self.transactions.read();
        let now = Instant::now();

        let mut oldest_age = Duration::ZERO;
        let mut total_propose_count = 0u64;

        for pending in txs.values() {
            let age = now.duration_since(pending.added_at);
            if age > oldest_age {
                oldest_age = age;
            }
            total_propose_count += pending.propose_count as u64;
        }

        MempoolStats {
            transaction_count: txs.len(),
            oldest_transaction_age: oldest_age,
            average_propose_count: if txs.is_empty() {
                0.0
            } else {
                total_propose_count as f64 / txs.len() as f64
            },
        }
    }
}

/// Statistics about the mempool.
#[derive(Debug, Clone)]
pub struct MempoolStats {
    /// Number of pending transactions.
    pub transaction_count: usize,

    /// Age of the oldest transaction.
    pub oldest_transaction_age: Duration,

    /// Average number of times transactions have been proposed.
    pub average_propose_count: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{SerializablePublicKey, SerializableSignature};
    use commonware_cryptography::{ed25519, PrivateKeyExt, Signer};

    fn test_tx(seed: u64) -> Transaction {
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

    #[test]
    fn test_mempool_add_and_get() {
        let mempool = Mempool::with_defaults();
        let tx = test_tx(1);
        let id = tx.id();

        let result = mempool.add(tx.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);

        let retrieved = mempool.get(&id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_mempool_duplicate() {
        let mempool = Mempool::with_defaults();
        let tx = test_tx(1);

        assert!(mempool.add(tx.clone()).is_ok());
        assert!(matches!(
            mempool.add(tx),
            Err(ConsensusError::DuplicateTransaction(_))
        ));
    }

    #[test]
    fn test_mempool_remove() {
        let mempool = Mempool::with_defaults();
        let tx = test_tx(1);
        let id = mempool.add(tx).unwrap();

        assert!(mempool.contains(&id));
        assert!(mempool.remove(&id).is_some());
        assert!(!mempool.contains(&id));
    }

    #[test]
    fn test_mempool_capacity() {
        let config = MempoolConfig {
            max_transactions: 3,
            ..Default::default()
        };
        let mempool = Mempool::new(config);

        for i in 1..=5 {
            mempool.add(test_tx(i)).unwrap();
        }

        // Should have evicted first 2
        assert_eq!(mempool.len(), 3);
    }

    #[test]
    fn test_mempool_get_for_proposal() {
        let config = MempoolConfig {
            max_transactions_per_block: 2,
            ..Default::default()
        };
        let mempool = Mempool::new(config);

        for i in 1..=5 {
            mempool.add(test_tx(i)).unwrap();
        }

        let proposal = mempool.get_for_proposal();
        assert_eq!(proposal.len(), 2);
    }

    #[test]
    fn test_mempool_stats() {
        let mempool = Mempool::with_defaults();

        for i in 1..=3 {
            mempool.add(test_tx(i)).unwrap();
        }

        let stats = mempool.stats();
        assert_eq!(stats.transaction_count, 3);
    }

    #[test]
    fn test_mempool_remove_batch() {
        let mempool = Mempool::with_defaults();
        let mut ids = Vec::new();

        for i in 1..=5 {
            let id = mempool.add(test_tx(i)).unwrap();
            ids.push(id);
        }

        mempool.remove_batch(&ids[0..3]);
        assert_eq!(mempool.len(), 2);
    }
}
