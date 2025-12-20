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
}
