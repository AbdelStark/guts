//! Consensus message types for P2P communication.
//!
//! These messages are exchanged between validators during consensus rounds.
//! The protocol follows Simplex BFT with:
//! - 2 network hops for block proposal
//! - 3 network hops for finalization

use crate::block::{Block, BlockId, FinalizedBlock};
use crate::transaction::{SerializablePublicKey, SerializableSignature, Transaction};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Channel ID for consensus messages (high priority).
pub const CONSENSUS_CHANNEL: u64 = 0;

/// Channel ID for transaction broadcast.
pub const TRANSACTION_CHANNEL: u64 = 2;

/// Channel ID for block sync requests.
pub const SYNC_CHANNEL: u64 = 3;

/// Consensus message types exchanged between validators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Block proposal from the leader.
    Propose(ProposeMessage),

    /// Vote to notarize a block (step 1 of finalization).
    Notarize(NotarizeMessage),

    /// Vote when leader is unresponsive (timeout).
    Nullify(NullifyMessage),

    /// Vote to finalize a block (step 2 of finalization).
    Finalize(FinalizeMessage),

    /// Broadcast a new transaction.
    Transaction(TransactionMessage),

    /// Request missing blocks.
    SyncRequest(SyncRequestMessage),

    /// Response with requested blocks.
    SyncResponse(SyncResponseMessage),
}

impl ConsensusMessage {
    /// Encodes the message to bytes.
    pub fn encode(&self) -> Bytes {
        let json = serde_json::to_vec(self).expect("message serialization should not fail");
        Bytes::from(json)
    }

    /// Decodes a message from bytes.
    pub fn decode(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }

    /// Returns the message type as a string for logging.
    pub fn kind(&self) -> &'static str {
        match self {
            ConsensusMessage::Propose(_) => "propose",
            ConsensusMessage::Notarize(_) => "notarize",
            ConsensusMessage::Nullify(_) => "nullify",
            ConsensusMessage::Finalize(_) => "finalize",
            ConsensusMessage::Transaction(_) => "transaction",
            ConsensusMessage::SyncRequest(_) => "sync_request",
            ConsensusMessage::SyncResponse(_) => "sync_response",
        }
    }
}

/// Block proposal message from the leader.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeMessage {
    /// The view number for this proposal.
    pub view: u64,

    /// The proposed block.
    pub block: Block,

    /// Producer's public key.
    pub producer: SerializablePublicKey,

    /// Producer's signature over the block hash.
    pub signature: SerializableSignature,
}

impl ProposeMessage {
    /// Creates a new propose message.
    pub fn new(
        view: u64,
        block: Block,
        producer: SerializablePublicKey,
        signature: SerializableSignature,
    ) -> Self {
        Self {
            view,
            block,
            producer,
            signature,
        }
    }

    /// Returns the block ID.
    pub fn block_id(&self) -> BlockId {
        self.block.id()
    }
}

/// Vote to notarize a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotarizeMessage {
    /// The view number.
    pub view: u64,

    /// Block ID being voted for.
    pub block_id: BlockId,

    /// Voter's public key.
    pub voter: SerializablePublicKey,

    /// Voter's signature over (view, block_id).
    pub signature: SerializableSignature,
}

impl NotarizeMessage {
    /// Creates a new notarize message.
    pub fn new(
        view: u64,
        block_id: BlockId,
        voter: SerializablePublicKey,
        signature: SerializableSignature,
    ) -> Self {
        Self {
            view,
            block_id,
            voter,
            signature,
        }
    }

    /// Returns the data that should be signed for this vote.
    pub fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"NOTARIZE:");
        data.extend_from_slice(&self.view.to_le_bytes());
        data.extend_from_slice(self.block_id.as_bytes());
        data
    }
}

/// Nullify vote when leader is unresponsive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullifyMessage {
    /// The view number being nullified.
    pub view: u64,

    /// Voter's public key.
    pub voter: SerializablePublicKey,

    /// Voter's signature over (view, "NULLIFY").
    pub signature: SerializableSignature,
}

impl NullifyMessage {
    /// Creates a new nullify message.
    pub fn new(view: u64, voter: SerializablePublicKey, signature: SerializableSignature) -> Self {
        Self {
            view,
            voter,
            signature,
        }
    }

    /// Returns the data that should be signed for this vote.
    pub fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"NULLIFY:");
        data.extend_from_slice(&self.view.to_le_bytes());
        data
    }
}

/// Vote to finalize a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeMessage {
    /// The view number.
    pub view: u64,

    /// Block ID being finalized.
    pub block_id: BlockId,

    /// Voter's public key.
    pub voter: SerializablePublicKey,

    /// Voter's signature over (view, block_id, "FINALIZE").
    pub signature: SerializableSignature,
}

impl FinalizeMessage {
    /// Creates a new finalize message.
    pub fn new(
        view: u64,
        block_id: BlockId,
        voter: SerializablePublicKey,
        signature: SerializableSignature,
    ) -> Self {
        Self {
            view,
            block_id,
            voter,
            signature,
        }
    }

    /// Returns the data that should be signed for this vote.
    pub fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"FINALIZE:");
        data.extend_from_slice(&self.view.to_le_bytes());
        data.extend_from_slice(self.block_id.as_bytes());
        data
    }
}

/// Broadcast a new transaction to the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    /// The transaction.
    pub transaction: Transaction,
}

impl TransactionMessage {
    /// Creates a new transaction message.
    pub fn new(transaction: Transaction) -> Self {
        Self { transaction }
    }
}

/// Request missing blocks from a peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequestMessage {
    /// Starting height (exclusive).
    pub from_height: u64,

    /// Ending height (inclusive).
    pub to_height: u64,

    /// Requestor's public key.
    pub requestor: SerializablePublicKey,
}

impl SyncRequestMessage {
    /// Creates a new sync request.
    pub fn new(from_height: u64, to_height: u64, requestor: SerializablePublicKey) -> Self {
        Self {
            from_height,
            to_height,
            requestor,
        }
    }
}

/// Response with requested blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponseMessage {
    /// The finalized blocks.
    pub blocks: Vec<FinalizedBlock>,

    /// Responder's public key.
    pub responder: SerializablePublicKey,
}

impl SyncResponseMessage {
    /// Creates a new sync response.
    pub fn new(blocks: Vec<FinalizedBlock>, responder: SerializablePublicKey) -> Self {
        Self { blocks, responder }
    }
}

/// Vote collection for tracking quorum.
#[derive(Debug, Clone, Default)]
pub struct VoteCollector {
    /// Notarize votes by block ID.
    notarize_votes: std::collections::HashMap<BlockId, Vec<NotarizeMessage>>,

    /// Finalize votes by block ID.
    finalize_votes: std::collections::HashMap<BlockId, Vec<FinalizeMessage>>,

    /// Nullify votes by view.
    nullify_votes: std::collections::HashMap<u64, Vec<NullifyMessage>>,
}

impl VoteCollector {
    /// Creates a new vote collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a notarize vote.
    pub fn add_notarize(&mut self, vote: NotarizeMessage) {
        self.notarize_votes
            .entry(vote.block_id)
            .or_default()
            .push(vote);
    }

    /// Adds a finalize vote.
    pub fn add_finalize(&mut self, vote: FinalizeMessage) {
        self.finalize_votes
            .entry(vote.block_id)
            .or_default()
            .push(vote);
    }

    /// Adds a nullify vote.
    pub fn add_nullify(&mut self, vote: NullifyMessage) {
        self.nullify_votes.entry(vote.view).or_default().push(vote);
    }

    /// Gets notarize votes for a block.
    pub fn get_notarize_votes(&self, block_id: &BlockId) -> &[NotarizeMessage] {
        self.notarize_votes
            .get(block_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Gets finalize votes for a block.
    pub fn get_finalize_votes(&self, block_id: &BlockId) -> &[FinalizeMessage] {
        self.finalize_votes
            .get(block_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Gets nullify votes for a view.
    pub fn get_nullify_votes(&self, view: u64) -> &[NullifyMessage] {
        self.nullify_votes
            .get(&view)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the number of notarize votes for a block.
    pub fn notarize_count(&self, block_id: &BlockId) -> usize {
        self.notarize_votes
            .get(block_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Returns the number of finalize votes for a block.
    pub fn finalize_count(&self, block_id: &BlockId) -> usize {
        self.finalize_votes
            .get(block_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Returns the number of nullify votes for a view.
    pub fn nullify_count(&self, view: u64) -> usize {
        self.nullify_votes.get(&view).map(|v| v.len()).unwrap_or(0)
    }

    /// Clears votes for a given view (after finalization or nullification).
    pub fn clear_view(&mut self, view: u64) {
        self.nullify_votes.remove(&view);
        // Note: block votes are kept for proof of finalization
    }

    /// Clears all votes (used when syncing).
    pub fn clear_all(&mut self) {
        self.notarize_votes.clear();
        self.finalize_votes.clear();
        self.nullify_votes.clear();
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
    fn test_consensus_message_roundtrip() {
        let (pk, sig) = test_keypair();
        let block_id = BlockId::from_bytes([1u8; 32]);

        let msg = ConsensusMessage::Notarize(NotarizeMessage {
            view: 10,
            block_id,
            voter: pk,
            signature: sig,
        });

        let encoded = msg.encode();
        let decoded = ConsensusMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.kind(), "notarize");
        if let ConsensusMessage::Notarize(n) = decoded {
            assert_eq!(n.view, 10);
            assert_eq!(n.block_id, block_id);
        } else {
            panic!("unexpected message type");
        }
    }

    #[test]
    fn test_vote_collector() {
        let (pk, sig) = test_keypair();
        let block_id = BlockId::from_bytes([1u8; 32]);

        let mut collector = VoteCollector::new();

        let vote = NotarizeMessage {
            view: 1,
            block_id,
            voter: pk.clone(),
            signature: sig.clone(),
        };

        collector.add_notarize(vote);
        assert_eq!(collector.notarize_count(&block_id), 1);

        let finalize_vote = FinalizeMessage {
            view: 1,
            block_id,
            voter: pk,
            signature: sig,
        };

        collector.add_finalize(finalize_vote);
        assert_eq!(collector.finalize_count(&block_id), 1);
    }

    #[test]
    fn test_signing_data() {
        let (pk, sig) = test_keypair();
        let block_id = BlockId::from_bytes([1u8; 32]);

        let notarize = NotarizeMessage::new(5, block_id, pk.clone(), sig.clone());
        let data = notarize.signing_data();
        assert!(data.starts_with(b"NOTARIZE:"));

        let nullify = NullifyMessage::new(5, pk.clone(), sig.clone());
        let data = nullify.signing_data();
        assert!(data.starts_with(b"NULLIFY:"));

        let finalize = FinalizeMessage::new(5, block_id, pk, sig);
        let data = finalize.signing_data();
        assert!(data.starts_with(b"FINALIZE:"));
    }
}
