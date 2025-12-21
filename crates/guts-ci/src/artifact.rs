//! Artifact storage and management for CI/CD.

use crate::error::{CiError, Result};
use crate::run::RunId;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

/// A unique identifier for an artifact.
pub type ArtifactId = String;

/// Metadata for a build artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier
    pub id: ArtifactId,
    /// Run that created this artifact
    pub run_id: RunId,
    /// Repository key
    pub repo_key: String,
    /// Artifact name
    pub name: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// SHA-256 hash of content
    pub content_hash: String,
    /// MIME type
    pub content_type: String,
    /// When the artifact expires
    pub expires_at: Option<u64>,
    /// When this artifact was created
    pub created_at: u64,
}

impl Artifact {
    /// Create a new artifact.
    pub fn new(
        run_id: RunId,
        repo_key: String,
        name: String,
        content: &[u8],
        expires_in_days: Option<u32>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = format!("{:x}", hasher.finalize());

        let content_type = guess_content_type(&name);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            run_id,
            repo_key,
            name,
            size_bytes: content.len() as u64,
            content_hash: hash,
            content_type,
            expires_at: expires_in_days.map(|days| now + (days as u64 * 24 * 60 * 60)),
            created_at: now,
        }
    }

    /// Check if this artifact has expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }
}

/// Guess the content type based on file extension.
fn guess_content_type(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "tgz" => "application/gzip",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" | "log" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Storage for artifacts.
#[derive(Debug, Default)]
pub struct ArtifactStore {
    /// Artifact metadata by ID
    artifacts: RwLock<HashMap<ArtifactId, Artifact>>,
    /// Artifact content by hash
    content: RwLock<HashMap<String, Arc<Vec<u8>>>>,
    /// Index by run ID
    by_run: RwLock<HashMap<RunId, Vec<ArtifactId>>>,
    /// Index by repo key
    by_repo: RwLock<HashMap<String, Vec<ArtifactId>>>,
}

impl ArtifactStore {
    /// Create a new artifact store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Upload an artifact.
    pub fn upload(
        &self,
        run_id: RunId,
        repo_key: String,
        name: String,
        content: Vec<u8>,
        expires_in_days: Option<u32>,
    ) -> Result<Artifact> {
        let artifact = Artifact::new(
            run_id.clone(),
            repo_key.clone(),
            name,
            &content,
            expires_in_days,
        );

        // Store content (deduplicated by hash)
        {
            let mut content_store = self.content.write();
            content_store.insert(artifact.content_hash.clone(), Arc::new(content));
        }

        // Store metadata
        {
            let mut artifacts = self.artifacts.write();
            artifacts.insert(artifact.id.clone(), artifact.clone());
        }

        // Update indices
        {
            let mut by_run = self.by_run.write();
            by_run.entry(run_id).or_default().push(artifact.id.clone());
        }
        {
            let mut by_repo = self.by_repo.write();
            by_repo
                .entry(repo_key)
                .or_default()
                .push(artifact.id.clone());
        }

        Ok(artifact)
    }

    /// Download an artifact.
    pub fn download(&self, artifact_id: &str) -> Result<(Artifact, Arc<Vec<u8>>)> {
        let artifact = {
            let artifacts = self.artifacts.read();
            artifacts
                .get(artifact_id)
                .cloned()
                .ok_or_else(|| CiError::ArtifactNotFound(artifact_id.to_string()))?
        };

        if artifact.is_expired() {
            return Err(CiError::ArtifactNotFound(format!(
                "{} (expired)",
                artifact_id
            )));
        }

        let content = {
            let content_store = self.content.read();
            content_store
                .get(&artifact.content_hash)
                .cloned()
                .ok_or_else(|| CiError::Storage("Artifact content not found".into()))?
        };

        Ok((artifact, content))
    }

    /// Get artifact metadata.
    pub fn get(&self, artifact_id: &str) -> Result<Artifact> {
        let artifacts = self.artifacts.read();
        artifacts
            .get(artifact_id)
            .cloned()
            .ok_or_else(|| CiError::ArtifactNotFound(artifact_id.to_string()))
    }

    /// Get artifact by name within a run.
    pub fn get_by_name(&self, run_id: &str, name: &str) -> Result<Artifact> {
        let by_run = self.by_run.read();
        let artifact_ids = by_run
            .get(run_id)
            .ok_or_else(|| CiError::ArtifactNotFound(format!("{}/{}", run_id, name)))?;

        let artifacts = self.artifacts.read();
        for id in artifact_ids {
            if let Some(artifact) = artifacts.get(id) {
                if artifact.name == name {
                    return Ok(artifact.clone());
                }
            }
        }

        Err(CiError::ArtifactNotFound(format!("{}/{}", run_id, name)))
    }

    /// List artifacts for a run.
    pub fn list_by_run(&self, run_id: &str) -> Vec<Artifact> {
        let by_run = self.by_run.read();
        let artifact_ids = match by_run.get(run_id) {
            Some(ids) => ids.clone(),
            None => return Vec::new(),
        };
        drop(by_run);

        let artifacts = self.artifacts.read();
        artifact_ids
            .iter()
            .filter_map(|id| artifacts.get(id))
            .filter(|a| !a.is_expired())
            .cloned()
            .collect()
    }

    /// List artifacts for a repository.
    pub fn list_by_repo(&self, repo_key: &str, limit: Option<usize>) -> Vec<Artifact> {
        let by_repo = self.by_repo.read();
        let artifact_ids = match by_repo.get(repo_key) {
            Some(ids) => ids.clone(),
            None => return Vec::new(),
        };
        drop(by_repo);

        let artifacts = self.artifacts.read();
        let mut result: Vec<_> = artifact_ids
            .iter()
            .filter_map(|id| artifacts.get(id))
            .filter(|a| !a.is_expired())
            .cloned()
            .collect();

        // Sort by created_at descending
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            result.truncate(limit);
        }

        result
    }

    /// Delete an artifact.
    pub fn delete(&self, artifact_id: &str) -> Result<()> {
        let artifact = {
            let mut artifacts = self.artifacts.write();
            artifacts
                .remove(artifact_id)
                .ok_or_else(|| CiError::ArtifactNotFound(artifact_id.to_string()))?
        };

        // Update indices
        {
            let mut by_run = self.by_run.write();
            if let Some(ids) = by_run.get_mut(&artifact.run_id) {
                ids.retain(|id| id != artifact_id);
            }
        }
        {
            let mut by_repo = self.by_repo.write();
            if let Some(ids) = by_repo.get_mut(&artifact.repo_key) {
                ids.retain(|id| id != artifact_id);
            }
        }

        // Note: Content is not deleted as it may be shared by other artifacts
        // A garbage collection pass would clean up orphaned content

        Ok(())
    }

    /// Delete all artifacts for a run.
    pub fn delete_by_run(&self, run_id: &str) -> Result<usize> {
        let artifact_ids = {
            let by_run = self.by_run.read();
            by_run.get(run_id).cloned().unwrap_or_default()
        };

        let count = artifact_ids.len();
        for id in artifact_ids {
            let _ = self.delete(&id);
        }

        Ok(count)
    }

    /// Clean up expired artifacts.
    pub fn cleanup_expired(&self) -> usize {
        let expired: Vec<_> = {
            let artifacts = self.artifacts.read();
            artifacts
                .values()
                .filter(|a| a.is_expired())
                .map(|a| a.id.clone())
                .collect()
        };

        let count = expired.len();
        for id in expired {
            let _ = self.delete(&id);
        }

        count
    }

    /// Get total storage size in bytes.
    pub fn total_size(&self) -> u64 {
        let artifacts = self.artifacts.read();
        artifacts.values().map(|a| a.size_bytes).sum()
    }

    /// Get artifact count.
    pub fn count(&self) -> usize {
        self.artifacts.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_creation() {
        let artifact = Artifact::new(
            "run-1".to_string(),
            "alice/repo".to_string(),
            "build.zip".to_string(),
            b"test content",
            Some(30),
        );

        assert!(!artifact.id.is_empty());
        assert_eq!(artifact.run_id, "run-1");
        assert_eq!(artifact.name, "build.zip");
        assert_eq!(artifact.size_bytes, 12);
        assert_eq!(artifact.content_type, "application/zip");
        assert!(artifact.expires_at.is_some());
        assert!(!artifact.is_expired());
    }

    #[test]
    fn test_artifact_upload_download() {
        let store = ArtifactStore::new();

        let artifact = store
            .upload(
                "run-1".to_string(),
                "alice/repo".to_string(),
                "output.txt".to_string(),
                b"hello world".to_vec(),
                None,
            )
            .unwrap();

        let (downloaded, content) = store.download(&artifact.id).unwrap();
        assert_eq!(downloaded.name, "output.txt");
        assert_eq!(content.as_ref(), b"hello world");
    }

    #[test]
    fn test_artifact_listing() {
        let store = ArtifactStore::new();

        store
            .upload(
                "run-1".to_string(),
                "alice/repo".to_string(),
                "a.txt".to_string(),
                b"a".to_vec(),
                None,
            )
            .unwrap();
        store
            .upload(
                "run-1".to_string(),
                "alice/repo".to_string(),
                "b.txt".to_string(),
                b"b".to_vec(),
                None,
            )
            .unwrap();
        store
            .upload(
                "run-2".to_string(),
                "alice/repo".to_string(),
                "c.txt".to_string(),
                b"c".to_vec(),
                None,
            )
            .unwrap();

        let by_run = store.list_by_run("run-1");
        assert_eq!(by_run.len(), 2);

        let by_repo = store.list_by_repo("alice/repo", None);
        assert_eq!(by_repo.len(), 3);
    }

    #[test]
    fn test_artifact_deletion() {
        let store = ArtifactStore::new();

        let artifact = store
            .upload(
                "run-1".to_string(),
                "alice/repo".to_string(),
                "test.txt".to_string(),
                b"test".to_vec(),
                None,
            )
            .unwrap();

        assert!(store.get(&artifact.id).is_ok());

        store.delete(&artifact.id).unwrap();
        assert!(store.get(&artifact.id).is_err());
    }

    #[test]
    fn test_content_deduplication() {
        let store = ArtifactStore::new();

        let content = b"same content".to_vec();
        store
            .upload(
                "run-1".to_string(),
                "alice/repo".to_string(),
                "a.txt".to_string(),
                content.clone(),
                None,
            )
            .unwrap();
        store
            .upload(
                "run-2".to_string(),
                "alice/repo".to_string(),
                "b.txt".to_string(),
                content,
                None,
            )
            .unwrap();

        // Both artifacts share the same content
        let content_store = store.content.read();
        assert_eq!(content_store.len(), 1);
    }

    #[test]
    fn test_content_type_guessing() {
        assert_eq!(guess_content_type("file.zip"), "application/zip");
        assert_eq!(guess_content_type("file.json"), "application/json");
        assert_eq!(guess_content_type("file.txt"), "text/plain");
        assert_eq!(guess_content_type("file.png"), "image/png");
        assert_eq!(
            guess_content_type("file.unknown"),
            "application/octet-stream"
        );
    }
}
