//! Replication protocol implementation.

use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use guts_storage::{ObjectId, Reference, Repository};
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use crate::message::{Message, ObjectData, RefUpdate, RepoAnnounce, SyncRequest};
use crate::{P2PError, Result};

/// Callback trait for sending messages to peers.
pub trait ReplicationHandler: Send + Sync + 'static {
    /// Send a message to all connected peers.
    fn broadcast(&self, message: Bytes);

    /// Send a message to a specific peer.
    fn send_to(&self, peer: &[u8], message: Bytes);
}

/// Repository replication protocol.
///
/// Handles incoming P2P messages and coordinates repository synchronization.
pub struct ReplicationProtocol {
    /// Repository store (owner/name -> Repository).
    repos: Arc<RwLock<HashMap<String, Arc<Repository>>>>,
    /// Message handler for sending responses.
    handler: Option<Arc<dyn ReplicationHandler>>,
}

impl ReplicationProtocol {
    /// Create a new replication protocol instance.
    pub fn new() -> Self {
        Self {
            repos: Arc::new(RwLock::new(HashMap::new())),
            handler: None,
        }
    }

    /// Set the message handler for sending responses.
    pub fn set_handler(&mut self, handler: Arc<dyn ReplicationHandler>) {
        self.handler = Some(handler);
    }

    /// Get the repository store reference.
    pub fn repos(&self) -> Arc<RwLock<HashMap<String, Arc<Repository>>>> {
        self.repos.clone()
    }

    /// Register a repository for replication.
    pub fn register_repo(&self, key: String, repo: Arc<Repository>) {
        self.repos.write().insert(key, repo);
    }

    /// Get a repository by key.
    pub fn get_repo(&self, key: &str) -> Option<Arc<Repository>> {
        self.repos.read().get(key).cloned()
    }

    /// Get or create a repository by key.
    pub fn get_or_create_repo(&self, key: &str) -> Arc<Repository> {
        {
            let repos = self.repos.read();
            if let Some(repo) = repos.get(key) {
                return repo.clone();
            }
        }

        let mut repos = self.repos.write();
        // Double-check after acquiring write lock
        if let Some(repo) = repos.get(key) {
            return repo.clone();
        }

        // Create new repo
        let parts: Vec<&str> = key.split('/').collect();
        let (owner, name) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("unknown", key)
        };

        let repo = Arc::new(Repository::new(name, owner));
        repos.insert(key.to_string(), repo.clone());
        repo
    }

    /// Handle an incoming message from a peer.
    ///
    /// Returns an optional response message.
    pub fn handle_message(&self, peer_id: &[u8], data: &[u8]) -> Result<Option<Message>> {
        let message = Message::decode(data)?;

        match message {
            Message::RepoAnnounce(announce) => self.handle_announce(peer_id, announce),
            Message::SyncRequest(request) => self.handle_sync_request(peer_id, request),
            Message::ObjectData(object_data) => self.handle_object_data(peer_id, object_data),
            Message::RefUpdate(ref_update) => self.handle_ref_update(peer_id, ref_update),
        }
    }

    /// Handle a repository announcement.
    fn handle_announce(&self, peer_id: &[u8], announce: RepoAnnounce) -> Result<Option<Message>> {
        info!(
            repo = %announce.repo_key,
            objects = announce.object_ids.len(),
            refs = announce.refs.len(),
            peer = %hex::encode(peer_id),
            "Received repo announce"
        );

        // Get or create the repository
        let repo = self.get_or_create_repo(&announce.repo_key);

        // Find objects we don't have
        let missing: Vec<ObjectId> = announce
            .object_ids
            .iter()
            .filter(|oid| !repo.objects.contains(oid))
            .copied()
            .collect();

        if missing.is_empty() {
            // We have all objects, just apply ref updates
            for (ref_name, oid) in announce.refs {
                repo.refs.set(&ref_name, oid);
            }
            debug!(repo = %announce.repo_key, "All objects already present");
            return Ok(None);
        }

        // Request missing objects
        info!(
            repo = %announce.repo_key,
            missing = missing.len(),
            "Requesting missing objects"
        );

        Ok(Some(Message::SyncRequest(SyncRequest {
            repo_key: announce.repo_key,
            want: missing,
        })))
    }

    /// Handle a sync request.
    fn handle_sync_request(&self, peer_id: &[u8], request: SyncRequest) -> Result<Option<Message>> {
        debug!(
            repo = %request.repo_key,
            want = request.want.len(),
            peer = %hex::encode(peer_id),
            "Received sync request"
        );

        let repo = self
            .get_repo(&request.repo_key)
            .ok_or_else(|| P2PError::RepoNotFound(request.repo_key.clone()))?;

        // Collect requested objects
        let mut objects = Vec::new();
        for oid in &request.want {
            match repo.objects.get(oid) {
                Ok(obj) => objects.push(obj),
                Err(e) => {
                    warn!(
                        object = %oid.to_hex(),
                        error = %e,
                        "Requested object not found"
                    );
                }
            }
        }

        if objects.is_empty() {
            debug!(repo = %request.repo_key, "No objects to send");
            return Ok(None);
        }

        info!(
            repo = %request.repo_key,
            objects = objects.len(),
            "Sending objects to peer"
        );

        Ok(Some(Message::ObjectData(ObjectData {
            repo_key: request.repo_key,
            objects,
        })))
    }

    /// Handle object data response.
    fn handle_object_data(
        &self,
        peer_id: &[u8],
        object_data: ObjectData,
    ) -> Result<Option<Message>> {
        info!(
            repo = %object_data.repo_key,
            objects = object_data.objects.len(),
            peer = %hex::encode(peer_id),
            "Received objects"
        );

        let repo = self.get_or_create_repo(&object_data.repo_key);

        // Store all received objects
        for obj in object_data.objects {
            let oid = repo.objects.put(obj);
            debug!(object = %oid.to_hex(), "Stored object");
        }

        Ok(None)
    }

    /// Handle a reference update.
    fn handle_ref_update(&self, peer_id: &[u8], ref_update: RefUpdate) -> Result<Option<Message>> {
        info!(
            repo = %ref_update.repo_key,
            ref_name = %ref_update.ref_name,
            old = %ref_update.old_id.to_hex(),
            new = %ref_update.new_id.to_hex(),
            peer = %hex::encode(peer_id),
            "Received ref update"
        );

        let repo = self.get_or_create_repo(&ref_update.repo_key);

        // Check if we have the target object
        let zero_id = ObjectId::from_bytes([0u8; 20]);
        if ref_update.new_id != zero_id && !repo.objects.contains(&ref_update.new_id) {
            // We don't have the target object, need to sync
            warn!(
                object = %ref_update.new_id.to_hex(),
                "Missing target object for ref update"
            );
            return Ok(Some(Message::SyncRequest(SyncRequest {
                repo_key: ref_update.repo_key,
                want: vec![ref_update.new_id],
            })));
        }

        // Apply the ref update
        if ref_update.new_id == zero_id {
            // Deletion
            let _ = repo.refs.delete(&ref_update.ref_name);
        } else {
            repo.refs.set(&ref_update.ref_name, ref_update.new_id);
        }

        Ok(None)
    }

    /// Broadcast a repository update to all peers.
    ///
    /// Called after a push to notify peers about new objects.
    pub fn broadcast_update(
        &self,
        repo_key: &str,
        new_objects: Vec<ObjectId>,
        refs: Vec<(String, ObjectId)>,
    ) {
        if let Some(handler) = &self.handler {
            let announce = RepoAnnounce {
                repo_key: repo_key.to_string(),
                object_ids: new_objects,
                refs,
            };
            handler.broadcast(announce.encode());
        }
    }

    /// Broadcast a reference update to all peers.
    pub fn broadcast_ref_update(
        &self,
        repo_key: &str,
        ref_name: &str,
        old_id: ObjectId,
        new_id: ObjectId,
    ) {
        if let Some(handler) = &self.handler {
            let update = RefUpdate {
                repo_key: repo_key.to_string(),
                ref_name: ref_name.to_string(),
                old_id,
                new_id,
            };
            handler.broadcast(update.encode());
        }
    }

    /// Get repository state summary for a given repo.
    pub fn get_repo_state(&self, key: &str) -> Option<RepoState> {
        let repos = self.repos.read();
        let repo = repos.get(key)?;

        let objects = repo.objects.list_objects();
        let refs: Vec<(String, ObjectId)> = repo
            .refs
            .list_all()
            .into_iter()
            .filter_map(|(name, reference)| match reference {
                Reference::Direct(oid) => Some((name, oid)),
                Reference::Symbolic(_) => None,
            })
            .collect();

        Some(RepoState { objects, refs })
    }
}

impl Default for ReplicationProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Repository state summary.
#[derive(Debug, Clone)]
pub struct RepoState {
    /// All object IDs in the repository.
    pub objects: Vec<ObjectId>,
    /// All direct references (name -> target).
    pub refs: Vec<(String, ObjectId)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use guts_storage::GitObject;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockHandler {
        broadcast_count: AtomicUsize,
        messages: RwLock<Vec<Bytes>>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                broadcast_count: AtomicUsize::new(0),
                messages: RwLock::new(Vec::new()),
            }
        }
    }

    impl ReplicationHandler for MockHandler {
        fn broadcast(&self, message: Bytes) {
            self.broadcast_count.fetch_add(1, Ordering::SeqCst);
            self.messages.write().push(message);
        }

        fn send_to(&self, _peer: &[u8], message: Bytes) {
            self.messages.write().push(message);
        }
    }

    #[test]
    fn test_protocol_register_repo() {
        let protocol = ReplicationProtocol::new();
        let repo = Arc::new(Repository::new("test", "alice"));
        protocol.register_repo("alice/test".to_string(), repo.clone());

        let retrieved = protocol.get_repo("alice/test").unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.owner, "alice");
    }

    #[test]
    fn test_protocol_handle_announce() {
        let protocol = ReplicationProtocol::new();

        // Create announce with unknown objects
        let announce = RepoAnnounce {
            repo_key: "bob/repo".to_string(),
            object_ids: vec![ObjectId::from_bytes([1u8; 20])],
            refs: vec![],
        };

        let peer_id = [0u8; 32];
        let result = protocol
            .handle_message(&peer_id, &announce.encode())
            .unwrap();

        // Should request the missing object
        match result {
            Some(Message::SyncRequest(req)) => {
                assert_eq!(req.repo_key, "bob/repo");
                assert_eq!(req.want.len(), 1);
            }
            _ => panic!("expected sync request"),
        }
    }

    #[test]
    fn test_protocol_handle_sync_request() {
        let protocol = ReplicationProtocol::new();

        // Create a repo with an object
        let repo = Arc::new(Repository::new("repo", "alice"));
        let obj = GitObject::blob(b"hello".to_vec());
        let oid = repo.objects.put(obj);
        protocol.register_repo("alice/repo".to_string(), repo);

        // Request that object
        let request = SyncRequest {
            repo_key: "alice/repo".to_string(),
            want: vec![oid],
        };

        let peer_id = [0u8; 32];
        let result = protocol
            .handle_message(&peer_id, &request.encode())
            .unwrap();

        // Should return the object
        match result {
            Some(Message::ObjectData(data)) => {
                assert_eq!(data.repo_key, "alice/repo");
                assert_eq!(data.objects.len(), 1);
                assert_eq!(data.objects[0].id, oid);
            }
            _ => panic!("expected object data"),
        }
    }

    #[test]
    fn test_protocol_handle_object_data() {
        let protocol = ReplicationProtocol::new();

        let obj = GitObject::blob(b"world".to_vec());
        let oid = obj.id;

        let object_data = ObjectData {
            repo_key: "carol/code".to_string(),
            objects: vec![obj],
        };

        let peer_id = [0u8; 32];
        let result = protocol
            .handle_message(&peer_id, &object_data.encode())
            .unwrap();

        assert!(result.is_none());

        // Verify object was stored
        let repo = protocol.get_repo("carol/code").unwrap();
        assert!(repo.objects.contains(&oid));
    }

    #[test]
    fn test_protocol_broadcast() {
        let mut protocol = ReplicationProtocol::new();
        let handler = Arc::new(MockHandler::new());
        protocol.set_handler(handler.clone());

        protocol.broadcast_update("test/repo", vec![ObjectId::from_bytes([1u8; 20])], vec![]);

        assert_eq!(handler.broadcast_count.load(Ordering::SeqCst), 1);
        assert_eq!(handler.messages.read().len(), 1);
    }
}
