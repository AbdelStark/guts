//! Object store and repository management.

use crate::{GitObject, ObjectId, ObjectType, RefStore, Reference, Result, StorageError};
use bytes::Bytes;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;

/// Content-addressed object store.
#[derive(Debug, Default)]
pub struct ObjectStore {
    /// Objects indexed by their SHA-1 hash.
    objects: RwLock<HashMap<ObjectId, GitObject>>,
}

impl ObjectStore {
    /// Creates a new empty object store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores an object and returns its ID.
    pub fn put(&self, object: GitObject) -> ObjectId {
        let id = object.id;
        self.objects.write().insert(id, object);
        id
    }

    /// Retrieves an object by ID.
    pub fn get(&self, id: &ObjectId) -> Result<GitObject> {
        self.objects
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::ObjectNotFound(id.to_hex()))
    }

    /// Checks if an object exists.
    pub fn contains(&self, id: &ObjectId) -> bool {
        self.objects.read().contains_key(id)
    }

    /// Returns the number of objects in the store.
    pub fn len(&self) -> usize {
        self.objects.read().len()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.objects.read().is_empty()
    }

    /// Lists all object IDs.
    pub fn list_objects(&self) -> Vec<ObjectId> {
        self.objects.read().keys().copied().collect()
    }

    /// Stores a blob and returns its ID.
    pub fn put_blob(&self, content: impl Into<Bytes>) -> ObjectId {
        self.put(GitObject::blob(content))
    }

    /// Compresses object data using zlib.
    pub fn compress(object: &GitObject) -> Result<Vec<u8>> {
        let header = format!("{} {}\0", object.object_type.as_str(), object.data.len());
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(header.as_bytes())
            .map_err(|e| StorageError::Compression(e.to_string()))?;
        encoder
            .write_all(&object.data)
            .map_err(|e| StorageError::Compression(e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| StorageError::Compression(e.to_string()))
    }

    /// Decompresses object data from zlib.
    pub fn decompress(compressed: &[u8]) -> Result<GitObject> {
        let mut decoder = ZlibDecoder::new(compressed);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| StorageError::Compression(e.to_string()))?;

        // Parse header: "type size\0data"
        let null_pos = decompressed.iter().position(|&b| b == 0).ok_or_else(|| {
            StorageError::InvalidObject("missing null byte in header".to_string())
        })?;

        let header = String::from_utf8_lossy(&decompressed[..null_pos]);
        let parts: Vec<&str> = header.split(' ').collect();
        if parts.len() != 2 {
            return Err(StorageError::InvalidObject(format!(
                "invalid header: {}",
                header
            )));
        }

        let object_type = ObjectType::parse(parts[0])?;
        let _size: usize = parts[1]
            .parse()
            .map_err(|_| StorageError::InvalidObject("invalid size".to_string()))?;

        let data = Bytes::from(decompressed[null_pos + 1..].to_vec());
        Ok(GitObject::new(object_type, data))
    }
}

/// A git repository with objects and references.
#[derive(Debug)]
pub struct Repository {
    /// Repository name.
    pub name: String,
    /// Repository owner (public key hex).
    pub owner: String,
    /// Object store.
    pub objects: Arc<ObjectStore>,
    /// Reference store.
    pub refs: Arc<RefStore>,
}

impl Repository {
    /// Creates a new empty repository.
    pub fn new(name: impl Into<String>, owner: impl Into<String>) -> Self {
        let refs = Arc::new(RefStore::new());
        // Initialize HEAD to point to main branch
        refs.set_symbolic("HEAD", "refs/heads/main");

        Self {
            name: name.into(),
            owner: owner.into(),
            objects: Arc::new(ObjectStore::new()),
            refs,
        }
    }

    /// Gets the current HEAD commit.
    pub fn head(&self) -> Result<ObjectId> {
        self.refs.resolve_head()
    }

    /// Gets the current branch name.
    pub fn current_branch(&self) -> Option<String> {
        self.refs.current_branch()
    }

    /// Creates a new commit.
    pub fn commit(
        &self,
        tree_id: &ObjectId,
        message: &str,
        author: &str,
        committer: &str,
    ) -> Result<ObjectId> {
        // Get parent commits (current HEAD if it exists)
        let parents: Vec<ObjectId> = match self.head() {
            Ok(head) => vec![head],
            Err(_) => vec![], // First commit has no parents
        };

        // Create commit object
        let commit = GitObject::commit(tree_id, &parents, author, committer, message);
        let commit_id = self.objects.put(commit);

        // Update current branch
        if let Some(branch) = self.current_branch() {
            self.refs.set(&format!("refs/heads/{}", branch), commit_id);
        } else {
            // Detached HEAD - update HEAD directly
            self.refs.set("HEAD", commit_id);
        }

        Ok(commit_id)
    }

    /// Updates a reference.
    pub fn update_ref(&self, name: &str, target: ObjectId) {
        self.refs.set(name, target);
    }

    /// Lists all references.
    pub fn list_refs(&self) -> Vec<(String, Reference)> {
        self.refs.list_all()
    }
}

/// Global repository store.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct RepoStore {
    repos: RwLock<HashMap<String, Arc<Repository>>>,
}

#[allow(dead_code)]
impl RepoStore {
    /// Creates a new empty repository store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new repository.
    pub fn create(&self, name: &str, owner: &str) -> Result<Arc<Repository>> {
        let mut repos = self.repos.write();
        let key = format!("{}/{}", owner, name);

        if repos.contains_key(&key) {
            return Err(StorageError::RepoExists(key));
        }

        let repo = Arc::new(Repository::new(name, owner));
        repos.insert(key, repo.clone());
        Ok(repo)
    }

    /// Gets a repository by owner and name.
    pub fn get(&self, owner: &str, name: &str) -> Result<Arc<Repository>> {
        let key = format!("{}/{}", owner, name);
        self.repos
            .read()
            .get(&key)
            .cloned()
            .ok_or(StorageError::RepoNotFound(key))
    }

    /// Lists all repositories.
    pub fn list(&self) -> Vec<Arc<Repository>> {
        self.repos.read().values().cloned().collect()
    }

    /// Lists repositories by owner.
    pub fn list_by_owner(&self, owner: &str) -> Vec<Arc<Repository>> {
        let prefix = format!("{}/", owner);
        self.repos
            .read()
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(_, repo)| repo.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_store_roundtrip() {
        let store = ObjectStore::new();
        let blob = GitObject::blob(b"Hello, World!".to_vec());
        let id = blob.id;

        store.put(blob);

        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.data.as_ref(), b"Hello, World!");
    }

    #[test]
    fn test_object_compression_roundtrip() {
        let original = GitObject::blob(b"Hello, World!".to_vec());
        let compressed = ObjectStore::compress(&original).unwrap();
        let decompressed = ObjectStore::decompress(&compressed).unwrap();

        assert_eq!(original.id, decompressed.id);
        assert_eq!(original.object_type, decompressed.object_type);
        assert_eq!(original.data, decompressed.data);
    }

    #[test]
    fn test_repository_creation() {
        let repos = RepoStore::new();
        let repo = repos.create("test-repo", "alice").unwrap();

        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.owner, "alice");
        assert_eq!(repo.current_branch(), Some("main".to_string()));
    }

    #[test]
    fn test_repository_commit() {
        let repos = RepoStore::new();
        let repo = repos.create("test-repo", "alice").unwrap();

        // Create a blob
        let blob_id = repo.objects.put_blob(b"file content".to_vec());

        // Create a simple tree (just storing the blob ID for now)
        let tree_data = format!("100644 file.txt\0{}", hex::encode(blob_id.as_bytes()));
        let tree = GitObject::new(ObjectType::Tree, tree_data.into_bytes());
        let tree_id = repo.objects.put(tree);

        // Create commit
        let author = "Alice <alice@example.com> 1234567890 +0000";
        let commit_id = repo
            .commit(&tree_id, "Initial commit", author, author)
            .unwrap();

        // Verify HEAD points to the commit
        assert_eq!(repo.head().unwrap(), commit_id);
    }
}
