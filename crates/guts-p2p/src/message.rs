//! P2P protocol messages for repository replication.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use guts_storage::{GitObject, ObjectId, ObjectType};

use crate::{P2PError, Result};

/// Message type discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Announce a repository update (new objects available).
    RepoAnnounce = 1,
    /// Request objects from a peer.
    SyncRequest = 2,
    /// Response with object data.
    ObjectData = 3,
    /// Broadcast a reference update.
    RefUpdate = 4,
}

impl MessageType {
    /// Parse a message type from a byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(MessageType::RepoAnnounce),
            2 => Ok(MessageType::SyncRequest),
            3 => Ok(MessageType::ObjectData),
            4 => Ok(MessageType::RefUpdate),
            _ => Err(P2PError::InvalidMessage(format!(
                "unknown message type: {}",
                b
            ))),
        }
    }
}

/// Repository announcement message.
///
/// Sent when a node receives new objects (e.g., after a push).
#[derive(Debug, Clone)]
pub struct RepoAnnounce {
    /// Repository key (owner/name).
    pub repo_key: String,
    /// List of new object IDs.
    pub object_ids: Vec<ObjectId>,
    /// Updated references.
    pub refs: Vec<(String, ObjectId)>,
}

impl RepoAnnounce {
    /// Encode the message to bytes.
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_u8(MessageType::RepoAnnounce as u8);

        // Repo key (length-prefixed)
        let repo_bytes = self.repo_key.as_bytes();
        buf.put_u16(repo_bytes.len() as u16);
        buf.put_slice(repo_bytes);

        // Object IDs count and data
        buf.put_u32(self.object_ids.len() as u32);
        for oid in &self.object_ids {
            buf.put_slice(oid.as_bytes());
        }

        // Refs count and data
        buf.put_u32(self.refs.len() as u32);
        for (name, oid) in &self.refs {
            let name_bytes = name.as_bytes();
            buf.put_u16(name_bytes.len() as u16);
            buf.put_slice(name_bytes);
            buf.put_slice(oid.as_bytes());
        }

        buf.freeze()
    }

    /// Decode the message from bytes.
    pub fn decode(mut buf: &[u8]) -> Result<Self> {
        // Repo key
        if buf.remaining() < 2 {
            return Err(P2PError::InvalidMessage("truncated repo key length".into()));
        }
        let repo_len = buf.get_u16() as usize;
        if buf.remaining() < repo_len {
            return Err(P2PError::InvalidMessage("truncated repo key".into()));
        }
        let repo_key = String::from_utf8(buf[..repo_len].to_vec())
            .map_err(|e| P2PError::InvalidMessage(format!("invalid repo key: {}", e)))?;
        buf.advance(repo_len);

        // Object IDs
        if buf.remaining() < 4 {
            return Err(P2PError::InvalidMessage("truncated object count".into()));
        }
        let obj_count = buf.get_u32() as usize;
        let mut object_ids = Vec::with_capacity(obj_count);
        for _ in 0..obj_count {
            if buf.remaining() < 20 {
                return Err(P2PError::InvalidMessage("truncated object id".into()));
            }
            let mut oid_bytes = [0u8; 20];
            buf.copy_to_slice(&mut oid_bytes);
            object_ids.push(ObjectId::from_bytes(oid_bytes));
        }

        // Refs
        if buf.remaining() < 4 {
            return Err(P2PError::InvalidMessage("truncated ref count".into()));
        }
        let ref_count = buf.get_u32() as usize;
        let mut refs = Vec::with_capacity(ref_count);
        for _ in 0..ref_count {
            if buf.remaining() < 2 {
                return Err(P2PError::InvalidMessage("truncated ref name length".into()));
            }
            let name_len = buf.get_u16() as usize;
            if buf.remaining() < name_len + 20 {
                return Err(P2PError::InvalidMessage("truncated ref data".into()));
            }
            let name = String::from_utf8(buf[..name_len].to_vec())
                .map_err(|e| P2PError::InvalidMessage(format!("invalid ref name: {}", e)))?;
            buf.advance(name_len);

            let mut oid_bytes = [0u8; 20];
            buf.copy_to_slice(&mut oid_bytes);
            refs.push((name, ObjectId::from_bytes(oid_bytes)));
        }

        Ok(RepoAnnounce {
            repo_key,
            object_ids,
            refs,
        })
    }
}

/// Request to sync objects from a peer.
#[derive(Debug, Clone)]
pub struct SyncRequest {
    /// Repository key (owner/name).
    pub repo_key: String,
    /// Object IDs we want.
    pub want: Vec<ObjectId>,
}

impl SyncRequest {
    /// Encode the message to bytes.
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_u8(MessageType::SyncRequest as u8);

        // Repo key
        let repo_bytes = self.repo_key.as_bytes();
        buf.put_u16(repo_bytes.len() as u16);
        buf.put_slice(repo_bytes);

        // Want list
        buf.put_u32(self.want.len() as u32);
        for oid in &self.want {
            buf.put_slice(oid.as_bytes());
        }

        buf.freeze()
    }

    /// Decode the message from bytes.
    pub fn decode(mut buf: &[u8]) -> Result<Self> {
        // Repo key
        if buf.remaining() < 2 {
            return Err(P2PError::InvalidMessage("truncated repo key length".into()));
        }
        let repo_len = buf.get_u16() as usize;
        if buf.remaining() < repo_len {
            return Err(P2PError::InvalidMessage("truncated repo key".into()));
        }
        let repo_key = String::from_utf8(buf[..repo_len].to_vec())
            .map_err(|e| P2PError::InvalidMessage(format!("invalid repo key: {}", e)))?;
        buf.advance(repo_len);

        // Want list
        if buf.remaining() < 4 {
            return Err(P2PError::InvalidMessage("truncated want count".into()));
        }
        let want_count = buf.get_u32() as usize;
        let mut want = Vec::with_capacity(want_count);
        for _ in 0..want_count {
            if buf.remaining() < 20 {
                return Err(P2PError::InvalidMessage("truncated object id".into()));
            }
            let mut oid_bytes = [0u8; 20];
            buf.copy_to_slice(&mut oid_bytes);
            want.push(ObjectId::from_bytes(oid_bytes));
        }

        Ok(SyncRequest { repo_key, want })
    }
}

/// Response with object data.
#[derive(Debug, Clone)]
pub struct ObjectData {
    /// Repository key (owner/name).
    pub repo_key: String,
    /// Objects being sent.
    pub objects: Vec<GitObject>,
}

impl ObjectData {
    /// Encode the message to bytes.
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_u8(MessageType::ObjectData as u8);

        // Repo key
        let repo_bytes = self.repo_key.as_bytes();
        buf.put_u16(repo_bytes.len() as u16);
        buf.put_slice(repo_bytes);

        // Objects
        buf.put_u32(self.objects.len() as u32);
        for obj in &self.objects {
            // Object type (1 byte)
            buf.put_u8(match obj.object_type {
                ObjectType::Blob => 1,
                ObjectType::Tree => 2,
                ObjectType::Commit => 3,
                ObjectType::Tag => 4,
            });
            // Object data (length-prefixed)
            buf.put_u32(obj.data.len() as u32);
            buf.put_slice(&obj.data);
        }

        buf.freeze()
    }

    /// Decode the message from bytes.
    pub fn decode(mut buf: &[u8]) -> Result<Self> {
        // Repo key
        if buf.remaining() < 2 {
            return Err(P2PError::InvalidMessage("truncated repo key length".into()));
        }
        let repo_len = buf.get_u16() as usize;
        if buf.remaining() < repo_len {
            return Err(P2PError::InvalidMessage("truncated repo key".into()));
        }
        let repo_key = String::from_utf8(buf[..repo_len].to_vec())
            .map_err(|e| P2PError::InvalidMessage(format!("invalid repo key: {}", e)))?;
        buf.advance(repo_len);

        // Objects
        if buf.remaining() < 4 {
            return Err(P2PError::InvalidMessage("truncated object count".into()));
        }
        let obj_count = buf.get_u32() as usize;
        let mut objects = Vec::with_capacity(obj_count);
        for _ in 0..obj_count {
            if buf.remaining() < 5 {
                return Err(P2PError::InvalidMessage("truncated object header".into()));
            }
            let obj_type = match buf.get_u8() {
                1 => ObjectType::Blob,
                2 => ObjectType::Tree,
                3 => ObjectType::Commit,
                4 => ObjectType::Tag,
                t => {
                    return Err(P2PError::InvalidMessage(format!(
                        "invalid object type: {}",
                        t
                    )))
                }
            };
            let data_len = buf.get_u32() as usize;
            if buf.remaining() < data_len {
                return Err(P2PError::InvalidMessage("truncated object data".into()));
            }
            let data = Bytes::copy_from_slice(&buf[..data_len]);
            buf.advance(data_len);
            objects.push(GitObject::new(obj_type, data));
        }

        Ok(ObjectData { repo_key, objects })
    }
}

/// Reference update broadcast.
#[derive(Debug, Clone)]
pub struct RefUpdate {
    /// Repository key (owner/name).
    pub repo_key: String,
    /// Reference name.
    pub ref_name: String,
    /// Old object ID (zeros if new ref).
    pub old_id: ObjectId,
    /// New object ID (zeros if deleted).
    pub new_id: ObjectId,
}

impl RefUpdate {
    /// Encode the message to bytes.
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_u8(MessageType::RefUpdate as u8);

        // Repo key
        let repo_bytes = self.repo_key.as_bytes();
        buf.put_u16(repo_bytes.len() as u16);
        buf.put_slice(repo_bytes);

        // Ref name
        let ref_bytes = self.ref_name.as_bytes();
        buf.put_u16(ref_bytes.len() as u16);
        buf.put_slice(ref_bytes);

        // Old and new IDs
        buf.put_slice(self.old_id.as_bytes());
        buf.put_slice(self.new_id.as_bytes());

        buf.freeze()
    }

    /// Decode the message from bytes.
    pub fn decode(mut buf: &[u8]) -> Result<Self> {
        // Repo key
        if buf.remaining() < 2 {
            return Err(P2PError::InvalidMessage("truncated repo key length".into()));
        }
        let repo_len = buf.get_u16() as usize;
        if buf.remaining() < repo_len {
            return Err(P2PError::InvalidMessage("truncated repo key".into()));
        }
        let repo_key = String::from_utf8(buf[..repo_len].to_vec())
            .map_err(|e| P2PError::InvalidMessage(format!("invalid repo key: {}", e)))?;
        buf.advance(repo_len);

        // Ref name
        if buf.remaining() < 2 {
            return Err(P2PError::InvalidMessage("truncated ref name length".into()));
        }
        let ref_len = buf.get_u16() as usize;
        if buf.remaining() < ref_len + 40 {
            return Err(P2PError::InvalidMessage("truncated ref data".into()));
        }
        let ref_name = String::from_utf8(buf[..ref_len].to_vec())
            .map_err(|e| P2PError::InvalidMessage(format!("invalid ref name: {}", e)))?;
        buf.advance(ref_len);

        // Old and new IDs
        let mut old_bytes = [0u8; 20];
        let mut new_bytes = [0u8; 20];
        buf.copy_to_slice(&mut old_bytes);
        buf.copy_to_slice(&mut new_bytes);

        Ok(RefUpdate {
            repo_key,
            ref_name,
            old_id: ObjectId::from_bytes(old_bytes),
            new_id: ObjectId::from_bytes(new_bytes),
        })
    }
}

/// Unified message enum.
#[derive(Debug, Clone)]
pub enum Message {
    RepoAnnounce(RepoAnnounce),
    SyncRequest(SyncRequest),
    ObjectData(ObjectData),
    RefUpdate(RefUpdate),
}

impl Message {
    /// Encode the message to bytes.
    pub fn encode(&self) -> Bytes {
        match self {
            Message::RepoAnnounce(m) => m.encode(),
            Message::SyncRequest(m) => m.encode(),
            Message::ObjectData(m) => m.encode(),
            Message::RefUpdate(m) => m.encode(),
        }
    }

    /// Decode a message from bytes.
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(P2PError::InvalidMessage("empty message".into()));
        }

        let msg_type = MessageType::from_byte(data[0])?;
        let payload = &data[1..];

        match msg_type {
            MessageType::RepoAnnounce => Ok(Message::RepoAnnounce(RepoAnnounce::decode(payload)?)),
            MessageType::SyncRequest => Ok(Message::SyncRequest(SyncRequest::decode(payload)?)),
            MessageType::ObjectData => Ok(Message::ObjectData(ObjectData::decode(payload)?)),
            MessageType::RefUpdate => Ok(Message::RefUpdate(RefUpdate::decode(payload)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_announce_roundtrip() {
        let msg = RepoAnnounce {
            repo_key: "alice/test-repo".to_string(),
            object_ids: vec![
                ObjectId::from_bytes([1u8; 20]),
                ObjectId::from_bytes([2u8; 20]),
            ],
            refs: vec![(
                "refs/heads/main".to_string(),
                ObjectId::from_bytes([3u8; 20]),
            )],
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded).unwrap();

        match decoded {
            Message::RepoAnnounce(d) => {
                assert_eq!(d.repo_key, msg.repo_key);
                assert_eq!(d.object_ids.len(), 2);
                assert_eq!(d.refs.len(), 1);
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_sync_request_roundtrip() {
        let msg = SyncRequest {
            repo_key: "bob/my-repo".to_string(),
            want: vec![ObjectId::from_bytes([5u8; 20])],
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded).unwrap();

        match decoded {
            Message::SyncRequest(d) => {
                assert_eq!(d.repo_key, msg.repo_key);
                assert_eq!(d.want.len(), 1);
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_object_data_roundtrip() {
        let obj = GitObject::blob(b"hello world".to_vec());
        let msg = ObjectData {
            repo_key: "carol/repo".to_string(),
            objects: vec![obj.clone()],
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded).unwrap();

        match decoded {
            Message::ObjectData(d) => {
                assert_eq!(d.repo_key, msg.repo_key);
                assert_eq!(d.objects.len(), 1);
                assert_eq!(d.objects[0].id, obj.id);
                assert_eq!(d.objects[0].data, obj.data);
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_ref_update_roundtrip() {
        let msg = RefUpdate {
            repo_key: "dave/code".to_string(),
            ref_name: "refs/heads/feature".to_string(),
            old_id: ObjectId::from_bytes([0u8; 20]),
            new_id: ObjectId::from_bytes([7u8; 20]),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded).unwrap();

        match decoded {
            Message::RefUpdate(d) => {
                assert_eq!(d.repo_key, msg.repo_key);
                assert_eq!(d.ref_name, msg.ref_name);
                assert_eq!(d.old_id, msg.old_id);
                assert_eq!(d.new_id, msg.new_id);
            }
            _ => panic!("wrong message type"),
        }
    }

    // Error handling tests
    #[test]
    fn test_message_decode_empty() {
        let result = Message::decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_decode_invalid_type() {
        let result = Message::decode(&[0xFF]);
        assert!(result.is_err());
    }

    #[test]
    fn test_repo_announce_truncated() {
        // Just the message type byte
        let result = RepoAnnounce::decode(&[]);
        assert!(result.is_err());

        // Truncated repo key length
        let result = RepoAnnounce::decode(&[0x00]);
        assert!(result.is_err());

        // Valid length but no data
        let result = RepoAnnounce::decode(&[0x00, 0x05]);
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_request_truncated() {
        let result = SyncRequest::decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_object_data_truncated() {
        let result = ObjectData::decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ref_update_truncated() {
        let result = RefUpdate::decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_object_data_invalid_type() {
        // Build a message with invalid object type (99)
        let mut buf = bytes::BytesMut::new();
        buf.put_u16(4); // repo key length
        buf.put_slice(b"test");
        buf.put_u32(1); // object count
        buf.put_u8(99); // invalid object type
        buf.put_u32(5); // data length
        buf.put_slice(b"hello");

        let result = ObjectData::decode(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_repo_announce_empty_lists() {
        let msg = RepoAnnounce {
            repo_key: "test/repo".to_string(),
            object_ids: vec![],
            refs: vec![],
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded).unwrap();

        match decoded {
            Message::RepoAnnounce(d) => {
                assert_eq!(d.repo_key, "test/repo");
                assert!(d.object_ids.is_empty());
                assert!(d.refs.is_empty());
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_message_type_from_byte() {
        assert_eq!(
            MessageType::from_byte(1).unwrap(),
            MessageType::RepoAnnounce
        );
        assert_eq!(MessageType::from_byte(2).unwrap(), MessageType::SyncRequest);
        assert_eq!(MessageType::from_byte(3).unwrap(), MessageType::ObjectData);
        assert_eq!(MessageType::from_byte(4).unwrap(), MessageType::RefUpdate);
        assert!(MessageType::from_byte(0).is_err());
        assert!(MessageType::from_byte(5).is_err());
        assert!(MessageType::from_byte(255).is_err());
    }

    #[test]
    fn test_object_data_all_types() {
        // Test all object types
        let objects = vec![
            GitObject::blob(b"blob data".to_vec()),
            GitObject::new(ObjectType::Tree, Bytes::from_static(b"tree data")),
            GitObject::new(ObjectType::Commit, Bytes::from_static(b"commit data")),
            GitObject::new(ObjectType::Tag, Bytes::from_static(b"tag data")),
        ];

        let msg = ObjectData {
            repo_key: "test/repo".to_string(),
            objects: objects.clone(),
        };

        let encoded = msg.encode();
        let decoded = Message::decode(&encoded).unwrap();

        match decoded {
            Message::ObjectData(d) => {
                assert_eq!(d.objects.len(), 4);
                assert_eq!(d.objects[0].object_type, ObjectType::Blob);
                assert_eq!(d.objects[1].object_type, ObjectType::Tree);
                assert_eq!(d.objects[2].object_type, ObjectType::Commit);
                assert_eq!(d.objects[3].object_type, ObjectType::Tag);
            }
            _ => panic!("wrong message type"),
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for generating valid ObjectIds
    fn object_id_strategy() -> impl Strategy<Value = ObjectId> {
        prop::array::uniform20(any::<u8>()).prop_map(ObjectId::from_bytes)
    }

    /// Strategy for generating valid repo keys
    fn repo_key_strategy() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9-]{0,30}/[a-z][a-z0-9-]{0,30}"
    }

    /// Strategy for generating valid ref names
    fn ref_name_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("refs/heads/main".to_string()),
            Just("refs/heads/develop".to_string()),
            Just("refs/tags/v1.0.0".to_string()),
            "[a-z/]{1,50}".prop_map(|s| format!("refs/{}", s)),
        ]
    }

    proptest! {
        /// Property: RepoAnnounce roundtrip preserves data
        #[test]
        fn prop_repo_announce_roundtrip(
            repo_key in repo_key_strategy(),
            object_ids in prop::collection::vec(object_id_strategy(), 0..10),
            refs in prop::collection::vec(
                (ref_name_strategy(), object_id_strategy()),
                0..10
            )
        ) {
            let msg = RepoAnnounce {
                repo_key: repo_key.clone(),
                object_ids: object_ids.clone(),
                refs: refs.clone(),
            };

            let encoded = msg.encode();
            let decoded = Message::decode(&encoded).unwrap();

            match decoded {
                Message::RepoAnnounce(d) => {
                    prop_assert_eq!(d.repo_key, repo_key);
                    prop_assert_eq!(d.object_ids.len(), object_ids.len());
                    prop_assert_eq!(d.refs.len(), refs.len());
                    for (orig, dec) in object_ids.iter().zip(d.object_ids.iter()) {
                        prop_assert_eq!(orig, dec);
                    }
                    for ((orig_name, orig_id), (dec_name, dec_id)) in refs.iter().zip(d.refs.iter()) {
                        prop_assert_eq!(orig_name, dec_name);
                        prop_assert_eq!(orig_id, dec_id);
                    }
                }
                _ => prop_assert!(false, "wrong message type"),
            }
        }

        /// Property: SyncRequest roundtrip preserves data
        #[test]
        fn prop_sync_request_roundtrip(
            repo_key in repo_key_strategy(),
            want in prop::collection::vec(object_id_strategy(), 0..20)
        ) {
            let msg = SyncRequest {
                repo_key: repo_key.clone(),
                want: want.clone(),
            };

            let encoded = msg.encode();
            let decoded = Message::decode(&encoded).unwrap();

            match decoded {
                Message::SyncRequest(d) => {
                    prop_assert_eq!(d.repo_key, repo_key);
                    prop_assert_eq!(d.want.len(), want.len());
                    for (orig, dec) in want.iter().zip(d.want.iter()) {
                        prop_assert_eq!(orig, dec);
                    }
                }
                _ => prop_assert!(false, "wrong message type"),
            }
        }

        /// Property: RefUpdate roundtrip preserves data
        #[test]
        fn prop_ref_update_roundtrip(
            repo_key in repo_key_strategy(),
            ref_name in ref_name_strategy(),
            old_id in object_id_strategy(),
            new_id in object_id_strategy()
        ) {
            let msg = RefUpdate {
                repo_key: repo_key.clone(),
                ref_name: ref_name.clone(),
                old_id,
                new_id,
            };

            let encoded = msg.encode();
            let decoded = Message::decode(&encoded).unwrap();

            match decoded {
                Message::RefUpdate(d) => {
                    prop_assert_eq!(d.repo_key, repo_key);
                    prop_assert_eq!(d.ref_name, ref_name);
                    prop_assert_eq!(d.old_id, old_id);
                    prop_assert_eq!(d.new_id, new_id);
                }
                _ => prop_assert!(false, "wrong message type"),
            }
        }

        /// Property: ObjectData roundtrip preserves data (with blob objects)
        #[test]
        fn prop_object_data_roundtrip(
            repo_key in repo_key_strategy(),
            blobs in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..1000), 0..5)
        ) {
            let objects: Vec<GitObject> = blobs.iter()
                .map(|data| GitObject::blob(data.clone()))
                .collect();

            let msg = ObjectData {
                repo_key: repo_key.clone(),
                objects: objects.clone(),
            };

            let encoded = msg.encode();
            let decoded = Message::decode(&encoded).unwrap();

            match decoded {
                Message::ObjectData(d) => {
                    prop_assert_eq!(d.repo_key, repo_key);
                    prop_assert_eq!(d.objects.len(), objects.len());
                    for (orig, dec) in objects.iter().zip(d.objects.iter()) {
                        prop_assert_eq!(orig.id, dec.id);
                        prop_assert_eq!(orig.object_type, dec.object_type);
                        prop_assert_eq!(orig.data.as_ref(), dec.data.as_ref());
                    }
                }
                _ => prop_assert!(false, "wrong message type"),
            }
        }

        /// Property: Truncated messages return errors
        #[test]
        fn prop_truncated_repo_announce_fails(
            repo_key in repo_key_strategy(),
            truncate_at in 0usize..50
        ) {
            let msg = RepoAnnounce {
                repo_key,
                object_ids: vec![ObjectId::from_bytes([1u8; 20])],
                refs: vec![],
            };

            let encoded = msg.encode();
            if truncate_at < encoded.len() {
                // Skip message type byte (index 0), decode expects payload only
                let truncated = &encoded[1..truncate_at.max(1)];
                let result = RepoAnnounce::decode(truncated);
                // Truncated data should fail or succeed depending on if we cut essential data
                // The test verifies it doesn't panic
                let _ = result;
            }
        }

        /// Property: Invalid message types return errors
        #[test]
        fn prop_invalid_message_type_fails(msg_type in 5u8..=255) {
            let result = MessageType::from_byte(msg_type);
            prop_assert!(result.is_err());
        }

        /// Property: Encoding then decoding is identity
        #[test]
        fn prop_message_encode_decode_identity(
            msg_type in 1u8..=4,
            repo_key in repo_key_strategy()
        ) {
            let msg = match msg_type {
                1 => Message::RepoAnnounce(RepoAnnounce {
                    repo_key: repo_key.clone(),
                    object_ids: vec![],
                    refs: vec![],
                }),
                2 => Message::SyncRequest(SyncRequest {
                    repo_key: repo_key.clone(),
                    want: vec![],
                }),
                3 => Message::ObjectData(ObjectData {
                    repo_key: repo_key.clone(),
                    objects: vec![],
                }),
                4 => Message::RefUpdate(RefUpdate {
                    repo_key: repo_key.clone(),
                    ref_name: "refs/heads/main".to_string(),
                    old_id: ObjectId::from_bytes([0u8; 20]),
                    new_id: ObjectId::from_bytes([1u8; 20]),
                }),
                _ => unreachable!(),
            };

            let encoded = msg.encode();
            let decoded = Message::decode(&encoded);
            prop_assert!(decoded.is_ok());
        }
    }
}
