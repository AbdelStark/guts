//! Storage for CI/CD workflows, runs, and related data.

use crate::error::{CiError, Result};
use crate::run::{RunId, WorkflowRun};
use crate::workflow::{Workflow, WorkflowId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// Storage for workflows.
#[derive(Debug, Default)]
pub struct WorkflowStore {
    /// Workflows by ID
    workflows: RwLock<HashMap<(String, WorkflowId), Workflow>>,
}

impl WorkflowStore {
    /// Create a new workflow store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a workflow.
    pub fn store(&self, workflow: Workflow) {
        let key = (workflow.repo_key.clone(), workflow.id.clone());
        let mut workflows = self.workflows.write();
        workflows.insert(key, workflow);
    }

    /// Get a workflow by ID.
    pub fn get(&self, repo_key: &str, workflow_id: &str) -> Option<Workflow> {
        let key = (repo_key.to_string(), workflow_id.to_string());
        let workflows = self.workflows.read();
        workflows.get(&key).cloned()
    }

    /// List workflows for a repository.
    pub fn list(&self, repo_key: &str) -> Vec<Workflow> {
        let workflows = self.workflows.read();
        workflows
            .values()
            .filter(|w| w.repo_key == repo_key)
            .cloned()
            .collect()
    }

    /// Delete a workflow.
    pub fn delete(&self, repo_key: &str, workflow_id: &str) -> Option<Workflow> {
        let key = (repo_key.to_string(), workflow_id.to_string());
        let mut workflows = self.workflows.write();
        workflows.remove(&key)
    }

    /// Get workflows that match a push event.
    pub fn get_matching_push(
        &self,
        repo_key: &str,
        branch: &str,
        changed_paths: &[String],
    ) -> Vec<Workflow> {
        let workflows = self.workflows.read();
        workflows
            .values()
            .filter(|w| w.repo_key == repo_key && w.matches_push(branch, changed_paths))
            .cloned()
            .collect()
    }

    /// Get workflows that match a PR event.
    pub fn get_matching_pr(
        &self,
        repo_key: &str,
        target_branch: &str,
        event_type: &crate::trigger::PrEventType,
        changed_paths: &[String],
    ) -> Vec<Workflow> {
        let workflows = self.workflows.read();
        workflows
            .values()
            .filter(|w| {
                w.repo_key == repo_key
                    && w.matches_pull_request(target_branch, event_type, changed_paths)
            })
            .cloned()
            .collect()
    }

    /// Get workflow count.
    pub fn count(&self) -> usize {
        self.workflows.read().len()
    }
}

/// Storage for workflow runs.
#[derive(Debug, Default)]
pub struct RunStore {
    /// Runs by ID
    runs: RwLock<HashMap<RunId, WorkflowRun>>,
    /// Index by (repo_key, workflow_id)
    by_workflow: RwLock<HashMap<(String, WorkflowId), Vec<RunId>>>,
    /// Index by repo_key
    by_repo: RwLock<HashMap<String, Vec<RunId>>>,
    /// Run number counters per workflow
    run_numbers: RwLock<HashMap<(String, WorkflowId), AtomicU32>>,
}

impl RunStore {
    /// Create a new run store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the next run number for a workflow.
    pub fn next_run_number(&self, repo_key: &str, workflow_id: &str) -> u32 {
        let key = (repo_key.to_string(), workflow_id.to_string());
        let mut counters = self.run_numbers.write();
        let counter = counters.entry(key).or_insert_with(|| AtomicU32::new(0));
        counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Store a workflow run.
    pub fn store(&self, run: WorkflowRun) {
        let id = run.id.clone();
        let workflow_key = (run.repo_key.clone(), run.workflow_id.clone());
        let repo_key = run.repo_key.clone();

        {
            let mut runs = self.runs.write();
            runs.insert(id.clone(), run);
        }
        {
            let mut by_workflow = self.by_workflow.write();
            by_workflow
                .entry(workflow_key)
                .or_default()
                .push(id.clone());
        }
        {
            let mut by_repo = self.by_repo.write();
            by_repo.entry(repo_key).or_default().push(id);
        }
    }

    /// Get a run by ID.
    pub fn get(&self, run_id: &str) -> Option<WorkflowRun> {
        let runs = self.runs.read();
        runs.get(run_id).cloned()
    }

    /// Get a run by ID, returning a Result.
    pub fn get_or_err(&self, run_id: &str) -> Result<WorkflowRun> {
        self.get(run_id)
            .ok_or_else(|| CiError::RunNotFound(run_id.to_string()))
    }

    /// Update a run.
    pub fn update(&self, run: WorkflowRun) -> Result<()> {
        let mut runs = self.runs.write();
        if !runs.contains_key(&run.id) {
            return Err(CiError::RunNotFound(run.id.clone()));
        }
        runs.insert(run.id.clone(), run);
        Ok(())
    }

    /// List runs for a workflow.
    pub fn list_by_workflow(
        &self,
        repo_key: &str,
        workflow_id: &str,
        limit: Option<usize>,
    ) -> Vec<WorkflowRun> {
        let key = (repo_key.to_string(), workflow_id.to_string());
        let run_ids = {
            let by_workflow = self.by_workflow.read();
            by_workflow.get(&key).cloned().unwrap_or_default()
        };

        let runs = self.runs.read();
        let mut result: Vec<_> = run_ids
            .iter()
            .filter_map(|id| runs.get(id).cloned())
            .collect();

        // Sort by created_at descending
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            result.truncate(limit);
        }

        result
    }

    /// List runs for a repository.
    pub fn list_by_repo(&self, repo_key: &str, limit: Option<usize>) -> Vec<WorkflowRun> {
        let run_ids = {
            let by_repo = self.by_repo.read();
            by_repo.get(repo_key).cloned().unwrap_or_default()
        };

        let runs = self.runs.read();
        let mut result: Vec<_> = run_ids
            .iter()
            .filter_map(|id| runs.get(id).cloned())
            .collect();

        // Sort by created_at descending
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            result.truncate(limit);
        }

        result
    }

    /// List active (in-progress) runs for a repository.
    pub fn list_active(&self, repo_key: &str) -> Vec<WorkflowRun> {
        let run_ids = {
            let by_repo = self.by_repo.read();
            by_repo.get(repo_key).cloned().unwrap_or_default()
        };

        let runs = self.runs.read();
        run_ids
            .iter()
            .filter_map(|id| runs.get(id).cloned())
            .filter(|r| r.status.is_active())
            .collect()
    }

    /// Delete a run.
    pub fn delete(&self, run_id: &str) -> Option<WorkflowRun> {
        let run = {
            let mut runs = self.runs.write();
            runs.remove(run_id)?
        };

        // Update indices
        {
            let key = (run.repo_key.clone(), run.workflow_id.clone());
            let mut by_workflow = self.by_workflow.write();
            if let Some(ids) = by_workflow.get_mut(&key) {
                ids.retain(|id| id != run_id);
            }
        }
        {
            let mut by_repo = self.by_repo.write();
            if let Some(ids) = by_repo.get_mut(&run.repo_key) {
                ids.retain(|id| id != run_id);
            }
        }

        Some(run)
    }

    /// Get run count.
    pub fn count(&self) -> usize {
        self.runs.read().len()
    }

    /// Get active run count.
    pub fn active_count(&self) -> usize {
        self.runs
            .read()
            .values()
            .filter(|r| r.status.is_active())
            .count()
    }
}

/// Combined CI/CD store.
#[derive(Debug, Default)]
pub struct CiStore {
    /// Workflow store
    pub workflows: WorkflowStore,
    /// Run store
    pub runs: RunStore,
    /// Artifact store
    pub artifacts: crate::artifact::ArtifactStore,
    /// Status check store
    pub statuses: crate::status::StatusStore,
}

impl CiStore {
    /// Create a new CI/CD store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get statistics.
    pub fn stats(&self) -> CiStats {
        CiStats {
            workflow_count: self.workflows.count(),
            run_count: self.runs.count(),
            active_run_count: self.runs.active_count(),
            artifact_count: self.artifacts.count(),
            artifact_size_bytes: self.artifacts.total_size(),
        }
    }
}

/// CI/CD statistics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CiStats {
    pub workflow_count: usize,
    pub run_count: usize,
    pub active_run_count: usize,
    pub artifact_count: usize,
    pub artifact_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trigger::{TriggerContext, TriggerType};

    fn test_workflow() -> Workflow {
        let yaml = r#"
name: Test
on: push
jobs:
  test:
    steps:
      - run: echo test
"#;
        Workflow::parse(yaml, "alice/repo", ".guts/workflows/test.yml").unwrap()
    }

    fn test_trigger_context() -> TriggerContext {
        TriggerContext {
            trigger_type: TriggerType::Push,
            actor: "alice".to_string(),
            ref_name: Some("refs/heads/main".to_string()),
            sha: "abc123".to_string(),
            base_sha: None,
            pr_number: None,
            inputs: HashMap::new(),
            event: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_workflow_store() {
        let store = WorkflowStore::new();

        let workflow = test_workflow();
        store.store(workflow.clone());

        let retrieved = store.get("alice/repo", "test").unwrap();
        assert_eq!(retrieved.name, "Test");

        let list = store.list("alice/repo");
        assert_eq!(list.len(), 1);

        store.delete("alice/repo", "test");
        assert!(store.get("alice/repo", "test").is_none());
    }

    #[test]
    fn test_run_store() {
        let store = RunStore::new();

        // Get run numbers
        let num1 = store.next_run_number("alice/repo", "ci");
        let num2 = store.next_run_number("alice/repo", "ci");
        assert_eq!(num1, 1);
        assert_eq!(num2, 2);

        // Store a run
        let run = WorkflowRun::new(
            "run-1".to_string(),
            "ci".to_string(),
            "CI".to_string(),
            "alice/repo".to_string(),
            num1,
            test_trigger_context(),
            "abc123".to_string(),
            Some("main".to_string()),
        );
        store.store(run);

        let retrieved = store.get("run-1").unwrap();
        assert_eq!(retrieved.workflow_id, "ci");

        let by_workflow = store.list_by_workflow("alice/repo", "ci", None);
        assert_eq!(by_workflow.len(), 1);

        let by_repo = store.list_by_repo("alice/repo", None);
        assert_eq!(by_repo.len(), 1);
    }

    #[test]
    fn test_run_update() {
        let store = RunStore::new();

        let mut run = WorkflowRun::new(
            "run-1".to_string(),
            "ci".to_string(),
            "CI".to_string(),
            "alice/repo".to_string(),
            1,
            test_trigger_context(),
            "abc123".to_string(),
            None,
        );
        store.store(run.clone());

        run.start();
        store.update(run).unwrap();

        let retrieved = store.get("run-1").unwrap();
        assert!(retrieved.started_at.is_some());
    }

    #[test]
    fn test_active_runs() {
        let store = RunStore::new();

        let mut run1 = WorkflowRun::new(
            "run-1".to_string(),
            "ci".to_string(),
            "CI".to_string(),
            "alice/repo".to_string(),
            1,
            test_trigger_context(),
            "abc123".to_string(),
            None,
        );
        run1.start();
        store.store(run1);

        let mut run2 = WorkflowRun::new(
            "run-2".to_string(),
            "ci".to_string(),
            "CI".to_string(),
            "alice/repo".to_string(),
            2,
            test_trigger_context(),
            "def456".to_string(),
            None,
        );
        run2.complete(crate::run::Conclusion::Success);
        store.store(run2);

        let active = store.list_active("alice/repo");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "run-1");
    }

    #[test]
    fn test_ci_store_stats() {
        let store = CiStore::new();

        store.workflows.store(test_workflow());
        store.runs.store(WorkflowRun::new(
            "run-1".to_string(),
            "ci".to_string(),
            "CI".to_string(),
            "alice/repo".to_string(),
            1,
            test_trigger_context(),
            "abc123".to_string(),
            None,
        ));

        let stats = store.stats();
        assert_eq!(stats.workflow_count, 1);
        assert_eq!(stats.run_count, 1);
    }
}
