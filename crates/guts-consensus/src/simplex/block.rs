//! Simplex consensus block type.
//!
//! This module provides the block type used by the Simplex BFT consensus engine.
//! The block implements all required traits for commonware-consensus integration.

use bytes::{Buf, BufMut};
use commonware_codec::{varint::UInt, EncodeSize, Error as CodecError, Read, ReadExt, Write};
use commonware_cryptography::{sha256::Digest, Committable, Digestible, Hasher, Sha256};

/// A block in the Simplex consensus chain.
///
/// This block type is designed for the Guts decentralized code collaboration
/// platform and follows the Simplex BFT consensus protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimplexBlock {
    /// The parent block's digest.
    pub parent: Digest,

    /// The height of the block in the blockchain.
    pub height: u64,

    /// The timestamp of the block (in milliseconds since the Unix epoch).
    pub timestamp: u64,

    /// State root after applying transactions in this block.
    pub state_root: [u8; 32],

    /// Number of transactions in this block.
    pub tx_count: u32,

    /// Serialized transaction data (for lightweight blocks, actual data synced separately).
    pub tx_root: [u8; 32],

    /// Pre-computed digest of the block.
    digest: Digest,
}

impl SimplexBlock {
    /// Computes the digest of a block from its components.
    fn compute_digest(
        parent: &Digest,
        height: u64,
        timestamp: u64,
        state_root: &[u8; 32],
        tx_count: u32,
        tx_root: &[u8; 32],
    ) -> Digest {
        let mut hasher = Sha256::new();
        hasher.update(parent);
        hasher.update(&height.to_be_bytes());
        hasher.update(&timestamp.to_be_bytes());
        hasher.update(state_root);
        hasher.update(&tx_count.to_be_bytes());
        hasher.update(tx_root);
        hasher.finalize()
    }

    /// Creates a new block.
    pub fn new(
        parent: Digest,
        height: u64,
        timestamp: u64,
        state_root: [u8; 32],
        tx_count: u32,
        tx_root: [u8; 32],
    ) -> Self {
        let digest =
            Self::compute_digest(&parent, height, timestamp, &state_root, tx_count, &tx_root);
        Self {
            parent,
            height,
            timestamp,
            state_root,
            tx_count,
            tx_root,
            digest,
        }
    }

    /// Creates a genesis block.
    pub fn genesis() -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"guts-genesis");
        let genesis_parent = hasher.finalize();

        Self::new(genesis_parent, 0, 0, [0u8; 32], 0, [0u8; 32])
    }
}

impl Write for SimplexBlock {
    fn write(&self, writer: &mut impl BufMut) {
        self.parent.write(writer);
        UInt(self.height).write(writer);
        UInt(self.timestamp).write(writer);
        writer.put_slice(&self.state_root);
        UInt(self.tx_count as u64).write(writer);
        writer.put_slice(&self.tx_root);
    }
}

impl Read for SimplexBlock {
    type Cfg = ();

    fn read_cfg(reader: &mut impl Buf, _: &Self::Cfg) -> Result<Self, CodecError> {
        let parent = Digest::read(reader)?;
        let height = UInt::read(reader)?.into();
        let timestamp = UInt::read(reader)?.into();

        let mut state_root = [0u8; 32];
        if reader.remaining() < 32 {
            return Err(CodecError::EndOfBuffer);
        }
        reader.copy_to_slice(&mut state_root);

        let tx_count: u64 = UInt::read(reader)?.into();

        let mut tx_root = [0u8; 32];
        if reader.remaining() < 32 {
            return Err(CodecError::EndOfBuffer);
        }
        reader.copy_to_slice(&mut tx_root);

        let digest = Self::compute_digest(
            &parent,
            height,
            timestamp,
            &state_root,
            tx_count as u32,
            &tx_root,
        );
        Ok(Self {
            parent,
            height,
            timestamp,
            state_root,
            tx_count: tx_count as u32,
            tx_root,
            digest,
        })
    }
}

impl EncodeSize for SimplexBlock {
    fn encode_size(&self) -> usize {
        self.parent.encode_size()
            + UInt(self.height).encode_size()
            + UInt(self.timestamp).encode_size()
            + 32 // state_root
            + UInt(self.tx_count as u64).encode_size()
            + 32 // tx_root
    }
}

impl Digestible for SimplexBlock {
    type Digest = Digest;

    fn digest(&self) -> Digest {
        self.digest
    }
}

impl Committable for SimplexBlock {
    type Commitment = Digest;

    fn commitment(&self) -> Digest {
        self.digest
    }
}

impl commonware_consensus::Block for SimplexBlock {
    fn parent(&self) -> Digest {
        self.parent
    }

    fn height(&self) -> u64 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_codec::Encode;

    /// Create a test parent digest from bytes.
    fn test_parent() -> Digest {
        let genesis = SimplexBlock::genesis();
        genesis.digest()
    }

    #[test]
    fn test_genesis_block() {
        let genesis = SimplexBlock::genesis();
        assert_eq!(genesis.height, 0);
        assert_eq!(genesis.timestamp, 0);
        assert_eq!(genesis.tx_count, 0);
    }

    #[test]
    fn test_block_serialization() {
        let block = SimplexBlock::new(test_parent(), 1, 1234567890, [1u8; 32], 5, [2u8; 32]);

        // Encode
        let encoded = block.encode();

        // Decode
        let decoded = SimplexBlock::read(&mut encoded.as_ref()).unwrap();

        assert_eq!(block.height, decoded.height);
        assert_eq!(block.timestamp, decoded.timestamp);
        assert_eq!(block.state_root, decoded.state_root);
        assert_eq!(block.tx_count, decoded.tx_count);
        assert_eq!(block.digest(), decoded.digest());
    }

    #[test]
    fn test_block_digest_consistency() {
        let parent = test_parent();
        let block1 = SimplexBlock::new(parent, 1, 1234567890, [1u8; 32], 5, [2u8; 32]);

        let block2 = SimplexBlock::new(parent, 1, 1234567890, [1u8; 32], 5, [2u8; 32]);

        assert_eq!(block1.digest(), block2.digest());
    }

    #[test]
    fn test_different_blocks_different_digests() {
        let parent = test_parent();
        let block1 = SimplexBlock::new(parent, 1, 1234567890, [1u8; 32], 5, [2u8; 32]);

        let block2 = SimplexBlock::new(
            parent, 2, // Different height
            1234567890, [1u8; 32], 5, [2u8; 32],
        );

        assert_ne!(block1.digest(), block2.digest());
    }
}
