//! Git reference management.

use crate::{ObjectId, Result, StorageError};
use parking_lot::RwLock;
use std::collections::HashMap;

/// A git reference (branch, tag, or symbolic ref).
#[derive(Debug, Clone)]
pub enum Reference {
    /// Direct reference to an object.
    Direct(ObjectId),
    /// Symbolic reference (e.g., HEAD -> refs/heads/main).
    Symbolic(String),
}

impl Reference {
    /// Resolves a symbolic reference to a direct object ID.
    pub fn resolve(&self, store: &RefStore) -> Result<ObjectId> {
        match self {
            Self::Direct(id) => Ok(*id),
            Self::Symbolic(target) => {
                let target_ref = store.get(target)?;
                target_ref.resolve(store)
            }
        }
    }

    /// Returns the object ID if this is a direct reference.
    pub fn as_direct(&self) -> Option<ObjectId> {
        match self {
            Self::Direct(id) => Some(*id),
            Self::Symbolic(_) => None,
        }
    }
}

/// Thread-safe reference store.
#[derive(Debug, Default)]
pub struct RefStore {
    refs: RwLock<HashMap<String, Reference>>,
}

impl RefStore {
    /// Creates a new empty reference store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a reference by name.
    pub fn get(&self, name: &str) -> Result<Reference> {
        self.refs
            .read()
            .get(name)
            .cloned()
            .ok_or_else(|| StorageError::RefNotFound(name.to_string()))
    }

    /// Sets a reference to point to an object.
    pub fn set(&self, name: &str, target: ObjectId) {
        self.refs
            .write()
            .insert(name.to_string(), Reference::Direct(target));
    }

    /// Sets a symbolic reference.
    pub fn set_symbolic(&self, name: &str, target: &str) {
        self.refs
            .write()
            .insert(name.to_string(), Reference::Symbolic(target.to_string()));
    }

    /// Deletes a reference.
    pub fn delete(&self, name: &str) -> Result<()> {
        self.refs
            .write()
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| StorageError::RefNotFound(name.to_string()))
    }

    /// Lists all references with a given prefix.
    pub fn list(&self, prefix: &str) -> Vec<(String, Reference)> {
        self.refs
            .read()
            .iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(name, refr)| (name.clone(), refr.clone()))
            .collect()
    }

    /// Lists all references.
    pub fn list_all(&self) -> Vec<(String, Reference)> {
        self.refs
            .read()
            .iter()
            .map(|(name, refr)| (name.clone(), refr.clone()))
            .collect()
    }

    /// Resolves HEAD to find the current branch and commit.
    pub fn resolve_head(&self) -> Result<ObjectId> {
        let head = self.get("HEAD")?;
        match head {
            Reference::Direct(id) => Ok(id),
            Reference::Symbolic(target) => {
                let target_ref = self.get(&target)?;
                match target_ref {
                    Reference::Direct(id) => Ok(id),
                    Reference::Symbolic(_) => Err(StorageError::InvalidRef(
                        "deeply nested symbolic refs not supported".to_string(),
                    )),
                }
            }
        }
    }

    /// Gets the current branch name (if HEAD is symbolic).
    pub fn current_branch(&self) -> Option<String> {
        match self.get("HEAD").ok()? {
            Reference::Symbolic(target) => {
                target.strip_prefix("refs/heads/").map(|s| s.to_string())
            }
            Reference::Direct(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_store_basic() {
        let store = RefStore::new();
        let id = ObjectId::from_hex("a94a8fe5ccb19ba61c4c0873d391e987982fbbd3").unwrap();

        store.set("refs/heads/main", id);
        store.set_symbolic("HEAD", "refs/heads/main");

        assert_eq!(store.current_branch(), Some("main".to_string()));

        let resolved = store.resolve_head().unwrap();
        assert_eq!(resolved.to_hex(), id.to_hex());
    }

    #[test]
    fn test_ref_listing() {
        let store = RefStore::new();
        let id = ObjectId::from_hex("a94a8fe5ccb19ba61c4c0873d391e987982fbbd3").unwrap();

        store.set("refs/heads/main", id);
        store.set("refs/heads/feature", id);
        store.set("refs/tags/v1.0", id);

        let heads = store.list("refs/heads/");
        assert_eq!(heads.len(), 2);

        let tags = store.list("refs/tags/");
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn test_ref_store_get_not_found() {
        let store = RefStore::new();
        let result = store.get("refs/heads/nonexistent");
        assert!(matches!(result, Err(StorageError::RefNotFound(_))));
    }

    #[test]
    fn test_ref_store_delete() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("refs/heads/feature", id);
        assert!(store.get("refs/heads/feature").is_ok());

        store.delete("refs/heads/feature").unwrap();
        assert!(store.get("refs/heads/feature").is_err());
    }

    #[test]
    fn test_ref_store_delete_not_found() {
        let store = RefStore::new();
        let result = store.delete("refs/heads/nonexistent");
        assert!(matches!(result, Err(StorageError::RefNotFound(_))));
    }

    #[test]
    fn test_ref_store_list_all() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("refs/heads/main", id);
        store.set("refs/heads/feature", id);
        store.set("refs/tags/v1.0", id);
        store.set_symbolic("HEAD", "refs/heads/main");

        let all = store.list_all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_reference_direct() {
        let id = ObjectId::from_bytes([1u8; 20]);
        let reference = Reference::Direct(id);

        assert_eq!(reference.as_direct(), Some(id));
    }

    #[test]
    fn test_reference_symbolic() {
        let reference = Reference::Symbolic("refs/heads/main".to_string());

        assert!(reference.as_direct().is_none());
    }

    #[test]
    fn test_reference_resolve_direct() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        let reference = Reference::Direct(id);
        let resolved = reference.resolve(&store).unwrap();

        assert_eq!(resolved, id);
    }

    #[test]
    fn test_reference_resolve_symbolic() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("refs/heads/main", id);

        let reference = Reference::Symbolic("refs/heads/main".to_string());
        let resolved = reference.resolve(&store).unwrap();

        assert_eq!(resolved, id);
    }

    #[test]
    fn test_resolve_head_direct() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("HEAD", id);

        let resolved = store.resolve_head().unwrap();
        assert_eq!(resolved, id);
    }

    #[test]
    fn test_resolve_head_symbolic() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("refs/heads/main", id);
        store.set_symbolic("HEAD", "refs/heads/main");

        let resolved = store.resolve_head().unwrap();
        assert_eq!(resolved, id);
    }

    #[test]
    fn test_resolve_head_not_found() {
        let store = RefStore::new();
        let result = store.resolve_head();
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_head_dangling_symbolic() {
        let store = RefStore::new();
        store.set_symbolic("HEAD", "refs/heads/nonexistent");

        let result = store.resolve_head();
        assert!(result.is_err());
    }

    #[test]
    fn test_current_branch_with_direct_head() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("HEAD", id);

        assert!(store.current_branch().is_none());
    }

    #[test]
    fn test_current_branch_feature() {
        let store = RefStore::new();
        store.set_symbolic("HEAD", "refs/heads/feature-branch");

        assert_eq!(store.current_branch(), Some("feature-branch".to_string()));
    }

    #[test]
    fn test_ref_update() {
        let store = RefStore::new();
        let id1 = ObjectId::from_bytes([1u8; 20]);
        let id2 = ObjectId::from_bytes([2u8; 20]);

        store.set("refs/heads/main", id1);
        assert_eq!(store.get("refs/heads/main").unwrap().as_direct(), Some(id1));

        store.set("refs/heads/main", id2);
        assert_eq!(store.get("refs/heads/main").unwrap().as_direct(), Some(id2));
    }

    #[test]
    fn test_ref_list_empty_prefix() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("refs/heads/main", id);
        store.set("refs/tags/v1", id);

        let all = store.list("");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_ref_list_no_matches() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("refs/heads/main", id);

        let remotes = store.list("refs/remotes/");
        assert!(remotes.is_empty());
    }

    #[test]
    fn test_reference_clone() {
        let id = ObjectId::from_bytes([1u8; 20]);
        let original = Reference::Direct(id);
        let cloned = original.clone();

        assert_eq!(original.as_direct(), cloned.as_direct());
    }

    #[test]
    fn test_symbolic_reference_update() {
        let store = RefStore::new();
        let id = ObjectId::from_bytes([1u8; 20]);

        store.set("refs/heads/main", id);
        store.set("refs/heads/feature", id);
        store.set_symbolic("HEAD", "refs/heads/main");

        assert_eq!(store.current_branch(), Some("main".to_string()));

        store.set_symbolic("HEAD", "refs/heads/feature");

        assert_eq!(store.current_branch(), Some("feature".to_string()));
    }

    #[test]
    fn test_ref_store_default() {
        let store: RefStore = Default::default();
        assert!(store.list_all().is_empty());
    }

    #[test]
    fn test_current_branch_non_heads() {
        let store = RefStore::new();
        // HEAD pointing to something that's not under refs/heads/
        store.set_symbolic("HEAD", "refs/remotes/origin/main");

        assert!(store.current_branch().is_none());
    }
}
