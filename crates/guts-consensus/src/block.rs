//! Consensus block structure.
//!
//! Blocks contain ordered transactions and are the unit of consensus.

use crate::transaction::{
    SerializablePublicKey, SerializableSignature, Transaction, TransactionId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A unique block identifier (SHA-256 hash of the block header).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId([u8; 32]);

impl BlockId {
    /// The genesis block ID (all zeros).
    pub const GENESIS_PARENT: Self = Self([0u8; 32]);

    /// Creates a block ID from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the hex representation.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Creates a block ID from a hex string.
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl Default for BlockId {
    fn default() -> Self {
        Self::GENESIS_PARENT
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// A block header containing metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    /// Block height (0 = genesis).
    pub height: u64,

    /// Parent block hash.
    pub parent: BlockId,

    /// Block producer (validator public key).
    pub producer: SerializablePublicKey,

    /// Timestamp (unix milliseconds).
    pub timestamp: u64,

    /// Merkle root of transactions.
    pub tx_root: [u8; 32],

    /// State root after applying all transactions.
    pub state_root: [u8; 32],

    /// Number of transactions in this block.
    pub tx_count: u32,
}

impl BlockHeader {
    /// Computes the block ID from the header.
    pub fn id(&self) -> BlockId {
        let bytes = serde_json::to_vec(self).expect("header serialization should not fail");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        BlockId(id)
    }
}

/// A full block containing header and transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header.
    pub header: BlockHeader,

    /// Ordered transactions in this block.
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Creates a new block.
    pub fn new(
        height: u64,
        parent: BlockId,
        producer: SerializablePublicKey,
        timestamp: u64,
        transactions: Vec<Transaction>,
        state_root: [u8; 32],
    ) -> Self {
        let tx_root = Self::compute_tx_root(&transactions);
        let tx_count = transactions.len() as u32;

        let header = BlockHeader {
            height,
            parent,
            producer,
            timestamp,
            tx_root,
            state_root,
            tx_count,
        };

        Self {
            header,
            transactions,
        }
    }

    /// Creates the genesis block.
    pub fn genesis(producer: SerializablePublicKey) -> Self {
        Self::new(0, BlockId::GENESIS_PARENT, producer, 0, vec![], [0u8; 32])
    }

    /// Returns the block ID.
    pub fn id(&self) -> BlockId {
        self.header.id()
    }

    /// Returns the block height.
    pub fn height(&self) -> u64 {
        self.header.height
    }

    /// Returns the parent block ID.
    pub fn parent(&self) -> BlockId {
        self.header.parent
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> u64 {
        self.header.timestamp
    }

    /// Returns the number of transactions.
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }

    /// Computes the Merkle root of transactions.
    fn compute_tx_root(transactions: &[Transaction]) -> [u8; 32] {
        if transactions.is_empty() {
            return [0u8; 32];
        }

        // Simple Merkle tree: hash pairs of transaction IDs
        let mut hashes: Vec<[u8; 32]> = transactions.iter().map(|tx| *tx.id().as_bytes()).collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::with_capacity(hashes.len().div_ceil(2));

            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate last hash if odd
                }
                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                next_level.push(hash);
            }

            hashes = next_level;
        }

        hashes[0]
    }

    /// Verifies the transaction root matches.
    pub fn verify_tx_root(&self) -> bool {
        let computed = Self::compute_tx_root(&self.transactions);
        computed == self.header.tx_root
    }

    /// Returns an iterator over transaction IDs.
    pub fn transaction_ids(&self) -> impl Iterator<Item = TransactionId> + '_ {
        self.transactions.iter().map(|tx| tx.id())
    }
}

/// Block with consensus metadata (votes, signatures).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizedBlock {
    /// The block.
    pub block: Block,

    /// View number when finalized.
    pub view: u64,

    /// Validator signatures (notarization).
    pub signatures: Vec<(SerializablePublicKey, SerializableSignature)>,
}

impl FinalizedBlock {
    /// Creates a new finalized block.
    pub fn new(
        block: Block,
        view: u64,
        signatures: Vec<(SerializablePublicKey, SerializableSignature)>,
    ) -> Self {
        Self {
            block,
            view,
            signatures,
        }
    }

    /// Returns the block ID.
    pub fn id(&self) -> BlockId {
        self.block.id()
    }

    /// Returns the block height.
    pub fn height(&self) -> u64 {
        self.block.height()
    }

    /// Returns the number of signatures.
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::{ed25519, PrivateKeyExt, Signer};

    fn test_keypair() -> (SerializablePublicKey, SerializableSignature) {
        let key = ed25519::PrivateKey::from_seed(42);
        let sig = key.sign(Some(b"_GUTS"), b"test");
        (
            SerializablePublicKey::from_pubkey(&key.public_key()),
            SerializableSignature::from_signature(&sig),
        )
    }

    #[test]
    fn test_block_id_roundtrip() {
        let bytes = [0xab; 32];
        let id = BlockId::from_bytes(bytes);
        assert_eq!(id.as_bytes(), &bytes);

        let hex = id.to_hex();
        let parsed = BlockId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_genesis_block() {
        let (producer, _) = test_keypair();
        let genesis = Block::genesis(producer);

        assert_eq!(genesis.height(), 0);
        assert_eq!(genesis.parent(), BlockId::GENESIS_PARENT);
        assert_eq!(genesis.tx_count(), 0);
        assert!(genesis.verify_tx_root());
    }

    #[test]
    fn test_block_with_transactions() {
        let (producer, signature) = test_keypair();

        let tx = Transaction::CreateRepository {
            owner: "alice".into(),
            name: "test".into(),
            description: "A test".into(),
            default_branch: "main".into(),
            visibility: "public".into(),
            creator: producer.clone(),
            signature: signature.clone(),
        };

        let block = Block::new(
            1,
            BlockId::GENESIS_PARENT,
            producer,
            12345,
            vec![tx],
            [0u8; 32],
        );

        assert_eq!(block.height(), 1);
        assert_eq!(block.tx_count(), 1);
        assert!(block.verify_tx_root());
    }

    #[test]
    fn test_block_id_unique() {
        let (producer, _) = test_keypair();

        let block1 = Block::new(
            1,
            BlockId::GENESIS_PARENT,
            producer.clone(),
            12345,
            vec![],
            [0u8; 32],
        );

        let block2 = Block::new(
            2, // Different height
            BlockId::GENESIS_PARENT,
            producer,
            12345,
            vec![],
            [0u8; 32],
        );

        assert_ne!(block1.id(), block2.id());
    }

    #[test]
    fn test_finalized_block() {
        let (producer, sig) = test_keypair();
        let block = Block::genesis(producer.clone());
        let signatures = vec![(producer, sig)];

        let finalized = FinalizedBlock::new(block, 1, signatures);

        assert_eq!(finalized.height(), 0);
        assert_eq!(finalized.signature_count(), 1);
        assert_eq!(finalized.view, 1);
    }
}
