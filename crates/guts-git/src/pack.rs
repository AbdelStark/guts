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

    #[test]
    fn test_pack_empty() {
        // Empty pack should still have valid header and checksum
        let builder = PackBuilder::new();
        let pack = builder.build().unwrap();

        // Should have header (12 bytes) + checksum (20 bytes)
        assert_eq!(pack.len(), 32);

        // Parse empty pack
        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let ids = parser.parse(&store).unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn test_pack_single_object() {
        let blob = GitObject::blob(b"single".to_vec());
        let id = blob.id;

        let mut builder = PackBuilder::new();
        builder.add(blob);
        let pack = builder.build().unwrap();

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let ids = parser.parse(&store).unwrap();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], id);
    }

    #[test]
    fn test_pack_all_object_types() {
        // Test blob, tree, commit, and tag
        let blob = GitObject::blob(b"blob content".to_vec());
        let tree = GitObject::new(ObjectType::Tree, b"tree content".to_vec());
        let commit = GitObject::new(ObjectType::Commit, b"commit content".to_vec());
        let tag = GitObject::new(ObjectType::Tag, b"tag content".to_vec());

        let ids: Vec<_> = [&blob, &tree, &commit, &tag].iter().map(|o| o.id).collect();

        let mut builder = PackBuilder::new();
        builder.add(blob);
        builder.add(tree);
        builder.add(commit);
        builder.add(tag);
        let pack = builder.build().unwrap();

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let parsed_ids = parser.parse(&store).unwrap();

        assert_eq!(parsed_ids.len(), 4);
        for id in &ids {
            assert!(parsed_ids.contains(id));
        }
    }

    #[test]
    fn test_pack_large_object() {
        // Test with a large object (1MB)
        let large_data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
        let blob = GitObject::blob(large_data.clone());
        let id = blob.id;

        let mut builder = PackBuilder::new();
        builder.add(blob);
        let pack = builder.build().unwrap();

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let ids = parser.parse(&store).unwrap();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], id);

        let obj = store.get(&id).unwrap();
        assert_eq!(obj.data.len(), large_data.len());
    }

    #[test]
    fn test_pack_invalid_signature() {
        let mut pack = vec![b'P', b'A', b'C', b'X']; // Wrong signature
        pack.extend_from_slice(&[0, 0, 0, 2]); // Version
        pack.extend_from_slice(&[0, 0, 0, 0]); // Object count
        pack.extend_from_slice(&[0u8; 20]); // Fake checksum

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let result = parser.parse(&store);
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_invalid_version() {
        let mut pack = b"PACK".to_vec();
        pack.extend_from_slice(&[0, 0, 0, 99]); // Invalid version
        pack.extend_from_slice(&[0, 0, 0, 0]); // Object count
        pack.extend_from_slice(&[0u8; 20]); // Fake checksum

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let result = parser.parse(&store);
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_too_small() {
        let pack = vec![0u8; 10]; // Too small for header

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let result = parser.parse(&store);
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_checksum_mismatch() {
        // Build a valid pack
        let blob = GitObject::blob(b"test".to_vec());
        let mut builder = PackBuilder::new();
        builder.add(blob);
        let mut pack = builder.build().unwrap();

        // Corrupt the checksum
        let len = pack.len();
        pack[len - 1] ^= 0xFF;

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let result = parser.parse(&store);
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_builder_default() {
        let builder = PackBuilder::default();
        let pack = builder.build().unwrap();
        assert!(!pack.is_empty());
    }

    #[test]
    fn test_pack_add_from_store() {
        let store = ObjectStore::new();
        let blob = GitObject::blob(b"stored".to_vec());
        let id = blob.id;
        store.put(blob);

        let mut builder = PackBuilder::new();
        builder.add_from_store(&store, &id).unwrap();
        let pack = builder.build().unwrap();

        let store2 = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let ids = parser.parse(&store2).unwrap();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], id);
    }

    #[test]
    fn test_pack_many_objects() {
        // Test with many small objects
        let mut builder = PackBuilder::new();
        let mut expected_ids = Vec::new();

        for i in 0..100 {
            let blob = GitObject::blob(format!("object {}", i).into_bytes());
            expected_ids.push(blob.id);
            builder.add(blob);
        }

        let pack = builder.build().unwrap();

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let ids = parser.parse(&store).unwrap();

        assert_eq!(ids.len(), 100);
        for id in &expected_ids {
            assert!(ids.contains(id));
        }
    }

    #[test]
    fn test_pack_binary_content() {
        // Test with binary content including null bytes
        let binary_data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let blob = GitObject::blob(binary_data.clone());
        let id = blob.id;

        let mut builder = PackBuilder::new();
        builder.add(blob);
        let pack = builder.build().unwrap();

        let store = ObjectStore::new();
        let mut parser = PackParser::new(&pack);
        let ids = parser.parse(&store).unwrap();

        let obj = store.get(&ids[0]).unwrap();
        assert_eq!(obj.data.as_ref(), binary_data.as_slice());
        assert_eq!(ids[0], id);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Pack roundtrip preserves blob content
        #[test]
        fn prop_pack_roundtrip_blob(data in prop::collection::vec(any::<u8>(), 0..10000)) {
            let blob = GitObject::blob(data.clone());
            let id = blob.id;

            let mut builder = PackBuilder::new();
            builder.add(blob);
            let pack = builder.build().unwrap();

            let store = ObjectStore::new();
            let mut parser = PackParser::new(&pack);
            let ids = parser.parse(&store).unwrap();

            prop_assert_eq!(ids.len(), 1);
            prop_assert_eq!(ids[0], id);

            let obj = store.get(&id).unwrap();
            prop_assert_eq!(obj.data.as_ref(), data.as_slice());
        }

        /// Property: Multiple unique objects roundtrip correctly
        #[test]
        fn prop_pack_roundtrip_multiple(
            blobs in prop::collection::vec(prop::collection::vec(any::<u8>(), 1..1000), 1..20)
        ) {
            // Ensure unique content to avoid duplicate object IDs
            let mut seen_ids = std::collections::HashSet::new();
            let objects: Vec<GitObject> = blobs.iter()
                .map(|data| GitObject::blob(data.clone()))
                .filter(|obj| seen_ids.insert(obj.id))
                .collect();

            if objects.is_empty() {
                return Ok(());
            }

            let expected_ids: Vec<ObjectId> = objects.iter().map(|o| o.id).collect();

            let mut builder = PackBuilder::new();
            for obj in objects {
                builder.add(obj);
            }
            let pack = builder.build().unwrap();

            let store = ObjectStore::new();
            let mut parser = PackParser::new(&pack);
            let ids = parser.parse(&store).unwrap();

            prop_assert_eq!(ids.len(), expected_ids.len());
            for id in &expected_ids {
                prop_assert!(ids.contains(id));
            }
        }

        /// Property: Invalid pack data doesn't panic
        #[test]
        fn prop_invalid_pack_no_panic(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let store = ObjectStore::new();
            let mut parser = PackParser::new(&data);
            // Should return error or Ok, but never panic
            let _ = parser.parse(&store);
        }

        /// Property: Corrupted checksum is detected
        #[test]
        fn prop_corrupted_checksum_detected(
            content in prop::collection::vec(any::<u8>(), 1..1000),
            corrupt_byte in 0u8..20
        ) {
            let blob = GitObject::blob(content);
            let mut builder = PackBuilder::new();
            builder.add(blob);
            let mut pack = builder.build().unwrap();

            // Corrupt the checksum (last 20 bytes)
            let len = pack.len();
            pack[len - 1 - (corrupt_byte as usize % 20)] ^= 0xFF;

            let store = ObjectStore::new();
            let mut parser = PackParser::new(&pack);
            let result = parser.parse(&store);
            prop_assert!(result.is_err());
        }
    }
}
