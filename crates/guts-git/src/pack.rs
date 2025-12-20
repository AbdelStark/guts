//! Git pack file format implementation.
//!
//! Pack files are the format used by git for efficient object transfer.
//! See: https://git-scm.com/docs/pack-format

use crate::{GitError, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use guts_storage::{GitObject, ObjectId, ObjectStore, ObjectType};
use sha1::{Digest, Sha1};
use std::io::{Read, Write};

/// Magic bytes at the start of a pack file.
const PACK_SIGNATURE: &[u8; 4] = b"PACK";
/// Pack file version we support.
const PACK_VERSION: u32 = 2;

/// Builds a pack file from a set of objects.
pub struct PackBuilder {
    objects: Vec<GitObject>,
}

impl PackBuilder {
    /// Creates a new pack builder.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Adds an object to the pack.
    pub fn add(&mut self, object: GitObject) {
        self.objects.push(object);
    }

    /// Adds an object from the store by ID.
    pub fn add_from_store(&mut self, store: &ObjectStore, id: &ObjectId) -> Result<()> {
        let object = store.get(id)?;
        self.objects.push(object);
        Ok(())
    }

    /// Builds the pack file.
    pub fn build(self) -> Result<Vec<u8>> {
        let mut pack = Vec::new();

        // Write header
        pack.extend_from_slice(PACK_SIGNATURE);
        pack.extend_from_slice(&PACK_VERSION.to_be_bytes());
        pack.extend_from_slice(&(self.objects.len() as u32).to_be_bytes());

        // Write objects
        for object in &self.objects {
            Self::write_object(&mut pack, object)?;
        }

        // Compute and append checksum
        let mut hasher = Sha1::new();
        hasher.update(&pack);
        let checksum = hasher.finalize();
        pack.extend_from_slice(&checksum);

        Ok(pack)
    }

    /// Writes a single object entry.
    fn write_object(pack: &mut Vec<u8>, object: &GitObject) -> Result<()> {
        let obj_type = object.object_type.pack_type();
        let size = object.data.len();

        // Write type and size in variable-length encoding
        // First byte: (MSB=more bytes) (3 bits type) (4 bits size)
        let mut first_byte = (obj_type << 4) | ((size & 0x0F) as u8);
        let mut remaining_size = size >> 4;

        if remaining_size > 0 {
            first_byte |= 0x80; // More bytes follow
        }
        pack.push(first_byte);

        // Additional size bytes (7 bits each, MSB=continue)
        while remaining_size > 0 {
            let mut byte = (remaining_size & 0x7F) as u8;
            remaining_size >>= 7;
            if remaining_size > 0 {
                byte |= 0x80;
            }
            pack.push(byte);
        }

        // Compress and write data
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&object.data)
            .map_err(|e| GitError::InvalidPack(e.to_string()))?;
        let compressed = encoder
            .finish()
            .map_err(|e| GitError::InvalidPack(e.to_string()))?;
        pack.extend_from_slice(&compressed);

        Ok(())
    }
}

impl Default for PackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a pack file and extracts objects.
pub struct PackParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PackParser<'a> {
    /// Creates a new pack parser.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Parses the pack file and stores objects.
    pub fn parse(&mut self, store: &ObjectStore) -> Result<Vec<ObjectId>> {
        // Verify header
        if self.data.len() < 12 {
            return Err(GitError::InvalidPack("pack too small".to_string()));
        }

        if &self.data[0..4] != PACK_SIGNATURE {
            return Err(GitError::InvalidPack("invalid signature".to_string()));
        }

        let version = u32::from_be_bytes([self.data[4], self.data[5], self.data[6], self.data[7]]);
        if version != PACK_VERSION {
            return Err(GitError::InvalidPack(format!(
                "unsupported version: {}",
                version
            )));
        }

        let object_count =
            u32::from_be_bytes([self.data[8], self.data[9], self.data[10], self.data[11]]) as usize;

        self.pos = 12;

        // Parse objects
        let mut ids = Vec::with_capacity(object_count);
        for _ in 0..object_count {
            let id = self.parse_object(store)?;
            ids.push(id);
        }

        // Verify checksum (last 20 bytes)
        let checksum_start = self.data.len() - 20;
        let mut hasher = Sha1::new();
        hasher.update(&self.data[..checksum_start]);
        let computed = hasher.finalize();

        if computed.as_slice() != &self.data[checksum_start..] {
            return Err(GitError::InvalidPack("checksum mismatch".to_string()));
        }

        Ok(ids)
    }

    /// Parses a single object.
    fn parse_object(&mut self, store: &ObjectStore) -> Result<ObjectId> {
        if self.pos >= self.data.len() {
            return Err(GitError::InvalidPack("unexpected end of pack".to_string()));
        }

        // Read type and size
        let first_byte = self.data[self.pos];
        self.pos += 1;

        let obj_type_code = (first_byte >> 4) & 0x07;
        let mut size = (first_byte & 0x0F) as usize;
        let mut shift = 4;

        // Read remaining size bytes
        if first_byte & 0x80 != 0 {
            loop {
                if self.pos >= self.data.len() {
                    return Err(GitError::InvalidPack("unexpected end in size".to_string()));
                }
                let byte = self.data[self.pos];
                self.pos += 1;
                size |= ((byte & 0x7F) as usize) << shift;
                shift += 7;
                if byte & 0x80 == 0 {
                    break;
                }
            }
        }

        let object_type = ObjectType::from_pack_type(obj_type_code)?;

        // Decompress data
        let remaining = &self.data[self.pos..self.data.len() - 20]; // Exclude checksum
        let mut decoder = ZlibDecoder::new(remaining);
        let mut decompressed = vec![0u8; size];
        decoder
            .read_exact(&mut decompressed)
            .map_err(|e| GitError::InvalidPack(format!("decompression failed: {}", e)))?;

        // Update position based on how much was consumed
        let consumed = decoder.total_in() as usize;
        self.pos += consumed;

        // Create and store object
        let object = GitObject::new(object_type, decompressed);
        let id = object.id;
        store.put(object);

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_roundtrip() {
        let _store = ObjectStore::new();

        // Create some objects
        let blob1 = GitObject::blob(b"Hello, World!".to_vec());
        let blob2 = GitObject::blob(b"Goodbye, World!".to_vec());

        let id1 = blob1.id;
        let id2 = blob2.id;

        // Build pack
        let mut builder = PackBuilder::new();
        builder.add(blob1);
        builder.add(blob2);
        let pack = builder.build().unwrap();

        // Parse pack into a new store
        let store2 = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let ids = parser.parse(&store2).unwrap();

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));

        // Verify objects
        let obj1 = store2.get(&id1).unwrap();
        assert_eq!(obj1.data.as_ref(), b"Hello, World!");
    }
}
