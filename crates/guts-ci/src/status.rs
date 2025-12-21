//! Status checks for commits, integrated with branch protection.

use crate::run::Conclusion;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for a status check.
pub type StatusCheckId = String;

/// Status check for a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCheck {
    /// Unique identifier
    pub id: StatusCheckId,
    /// Repository key (owner/name)
    pub repo_key: String,
    /// Commit SHA
    pub sha: String,
    /// Context name (e.g., "CI / Build", "CI / Test")
    pub context: String,
    /// Current state
    pub state: CheckState,
    /// Short description
    pub description: Option<String>,
    /// URL for more details
    pub target_url: Option<String>,
    /// Avatar URL of the creator
    pub avatar_url: Option<String>,
    /// Who created this check
    pub creator: Option<String>,
    /// When this check was created
    pub created_at: u64,
    /// When this check was last updated
    pub updated_at: u64,
}

/// State of a status check.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckState {
    /// Check is pending/in progress
    Pending,
    /// Check passed
    Success,
    /// Check failed
    Failure,
    /// Check encountered an error
    Error,
}

impl CheckState {
    /// Check if this state is successful.
    pub fn is_success(&self) -> bool {
        matches!(self, CheckState::Success)
    }

    /// Check if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        !matches!(self, CheckState::Pending)
    }
}

impl From<Conclusion> for CheckState {
    fn from(conclusion: Conclusion) -> Self {
        match conclusion {
            Conclusion::Success | Conclusion::Neutral | Conclusion::Skipped => CheckState::Success,
            Conclusion::Failure | Conclusion::TimedOut => CheckState::Failure,
            Conclusion::Cancelled => CheckState::Failure, // Treat cancelled as failure for checks
            Conclusion::ActionRequired => CheckState::Pending,
            Conclusion::Error => CheckState::Error,
        }
    }
}

impl StatusCheck {
    /// Create a new status check.
    pub fn new(repo_key: String, sha: String, context: String, state: CheckState) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_key,
            sha,
            context,
            state,
            description: None,
            target_url: None,
            avatar_url: None,
            creator: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the target URL.
    pub fn with_target_url(mut self, url: impl Into<String>) -> Self {
        self.target_url = Some(url.into());
        self
    }

    /// Set the creator.
    pub fn with_creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// Update the state.
    pub fn update_state(&mut self, state: CheckState, description: Option<String>) {
        self.state = state;
        if let Some(desc) = description {
            self.description = Some(desc);
        }
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

/// Combined status for a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedStatus {
    /// Repository key
    pub repo_key: String,
    /// Commit SHA
    pub sha: String,
    /// Overall state
    pub state: CheckState,
    /// Total number of status checks
    pub total_count: usize,
    /// Individual statuses
    pub statuses: Vec<StatusCheck>,
}

impl CombinedStatus {
    /// Calculate the combined state from individual statuses.
    pub fn calculate_state(statuses: &[StatusCheck]) -> CheckState {
        if statuses.is_empty() {
            return CheckState::Success; // No checks = success
        }

        let mut has_pending = false;
        let mut has_failure = false;
        let mut has_error = false;

        for status in statuses {
            match status.state {
                CheckState::Pending => has_pending = true,
                CheckState::Failure => has_failure = true,
                CheckState::Error => has_error = true,
                CheckState::Success => {}
            }
        }

        if has_error {
            CheckState::Error
        } else if has_failure {
            CheckState::Failure
        } else if has_pending {
            CheckState::Pending
        } else {
            CheckState::Success
        }
    }
}

/// Storage for status checks.
#[derive(Debug, Default)]
pub struct StatusStore {
    /// Status checks by ID
    checks: RwLock<HashMap<StatusCheckId, StatusCheck>>,
    /// Index by (repo_key, sha)
    by_commit: RwLock<HashMap<(String, String), Vec<StatusCheckId>>>,
    /// Index by (repo_key, sha, context) - for upsert
    by_context: RwLock<HashMap<(String, String, String), StatusCheckId>>,
}

impl StatusStore {
    /// Create a new status store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create or update a status check.
    pub fn create_or_update(&self, mut check: StatusCheck) -> StatusCheck {
        let key = (
            check.repo_key.clone(),
            check.sha.clone(),
            check.context.clone(),
        );

        // Check if we already have a check for this context
        let existing_id = {
            let by_context = self.by_context.read();
            by_context.get(&key).cloned()
        };

        if let Some(existing_id) = existing_id {
            // Update existing check
            let mut checks = self.checks.write();
            if let Some(existing) = checks.get_mut(&existing_id) {
                existing.state = check.state;
                existing.description = check.description.clone();
                existing.target_url = check.target_url.clone();
                existing.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                return existing.clone();
            }
        }

        // Create new check
        check.id = uuid::Uuid::new_v4().to_string();
        let id = check.id.clone();
        let commit_key = (check.repo_key.clone(), check.sha.clone());

        {
            let mut checks = self.checks.write();
            checks.insert(id.clone(), check.clone());
        }
        {
            let mut by_commit = self.by_commit.write();
            by_commit.entry(commit_key).or_default().push(id.clone());
        }
        {
            let mut by_context = self.by_context.write();
            by_context.insert(key, id);
        }

        check
    }

    /// Get a status check by ID.
    pub fn get(&self, id: &str) -> Option<StatusCheck> {
        let checks = self.checks.read();
        checks.get(id).cloned()
    }

    /// Get the combined status for a commit.
    pub fn get_combined_status(&self, repo_key: &str, sha: &str) -> CombinedStatus {
        let key = (repo_key.to_string(), sha.to_string());

        let check_ids = {
            let by_commit = self.by_commit.read();
            by_commit.get(&key).cloned().unwrap_or_default()
        };

        let checks = self.checks.read();
        let statuses: Vec<_> = check_ids
            .iter()
            .filter_map(|id| checks.get(id).cloned())
            .collect();

        let state = CombinedStatus::calculate_state(&statuses);

        CombinedStatus {
            repo_key: repo_key.to_string(),
            sha: sha.to_string(),
            state,
            total_count: statuses.len(),
            statuses,
        }
    }

    /// List all status checks for a commit.
    pub fn list_for_commit(&self, repo_key: &str, sha: &str) -> Vec<StatusCheck> {
        let key = (repo_key.to_string(), sha.to_string());

        let check_ids = {
            let by_commit = self.by_commit.read();
            by_commit.get(&key).cloned().unwrap_or_default()
        };

        let checks = self.checks.read();
        check_ids
            .iter()
            .filter_map(|id| checks.get(id).cloned())
            .collect()
    }

    /// Delete all status checks for a commit.
    pub fn delete_for_commit(&self, repo_key: &str, sha: &str) -> usize {
        let key = (repo_key.to_string(), sha.to_string());

        let check_ids = {
            let mut by_commit = self.by_commit.write();
            by_commit.remove(&key).unwrap_or_default()
        };

        let count = check_ids.len();

        let mut checks = self.checks.write();
        let mut by_context = self.by_context.write();

        for id in check_ids {
            if let Some(check) = checks.remove(&id) {
                let context_key = (check.repo_key, check.sha, check.context);
                by_context.remove(&context_key);
            }
        }

        count
    }

    /// Get count of checks for a repository.
    pub fn count_for_repo(&self, repo_key: &str) -> usize {
        let checks = self.checks.read();
        checks.values().filter(|c| c.repo_key == repo_key).count()
    }
}

/// Check if all required status checks have passed.
pub fn check_required_statuses(
    status_store: &StatusStore,
    repo_key: &str,
    sha: &str,
    required_contexts: &[String],
) -> (bool, Vec<String>) {
    let combined = status_store.get_combined_status(repo_key, sha);
    let mut missing = Vec::new();
    let mut failed = Vec::new();

    for context in required_contexts {
        let status = combined.statuses.iter().find(|s| &s.context == context);
        match status {
            None => missing.push(context.clone()),
            Some(s) if !s.state.is_success() => failed.push(context.clone()),
            _ => {}
        }
    }

    let all_passed = missing.is_empty() && failed.is_empty();
    let issues: Vec<_> = missing
        .into_iter()
        .map(|c| format!("missing: {}", c))
        .chain(failed.into_iter().map(|c| format!("failed: {}", c)))
        .collect();

    (all_passed, issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_check_creation() {
        let check = StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            "CI / Build".to_string(),
            CheckState::Pending,
        );

        assert!(!check.id.is_empty());
        assert_eq!(check.repo_key, "alice/repo");
        assert_eq!(check.sha, "abc123");
        assert_eq!(check.context, "CI / Build");
        assert_eq!(check.state, CheckState::Pending);
    }

    #[test]
    fn test_status_check_state_update() {
        let mut check = StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            "CI".to_string(),
            CheckState::Pending,
        );

        check.update_state(CheckState::Success, Some("All tests passed".to_string()));
        assert_eq!(check.state, CheckState::Success);
        assert_eq!(check.description, Some("All tests passed".to_string()));
    }

    #[test]
    fn test_combined_status_calculation() {
        // All success
        let statuses = vec![
            StatusCheck::new(
                "r".to_string(),
                "s".to_string(),
                "a".to_string(),
                CheckState::Success,
            ),
            StatusCheck::new(
                "r".to_string(),
                "s".to_string(),
                "b".to_string(),
                CheckState::Success,
            ),
        ];
        assert_eq!(
            CombinedStatus::calculate_state(&statuses),
            CheckState::Success
        );

        // One pending
        let statuses = vec![
            StatusCheck::new(
                "r".to_string(),
                "s".to_string(),
                "a".to_string(),
                CheckState::Success,
            ),
            StatusCheck::new(
                "r".to_string(),
                "s".to_string(),
                "b".to_string(),
                CheckState::Pending,
            ),
        ];
        assert_eq!(
            CombinedStatus::calculate_state(&statuses),
            CheckState::Pending
        );

        // One failure
        let statuses = vec![
            StatusCheck::new(
                "r".to_string(),
                "s".to_string(),
                "a".to_string(),
                CheckState::Success,
            ),
            StatusCheck::new(
                "r".to_string(),
                "s".to_string(),
                "b".to_string(),
                CheckState::Failure,
            ),
        ];
        assert_eq!(
            CombinedStatus::calculate_state(&statuses),
            CheckState::Failure
        );

        // Empty = success
        assert_eq!(CombinedStatus::calculate_state(&[]), CheckState::Success);
    }

    #[test]
    fn test_status_store_create_and_get() {
        let store = StatusStore::new();

        let check = StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            "CI".to_string(),
            CheckState::Pending,
        );

        let created = store.create_or_update(check);
        let retrieved = store.get(&created.id).unwrap();

        assert_eq!(retrieved.context, "CI");
        assert_eq!(retrieved.state, CheckState::Pending);
    }

    #[test]
    fn test_status_store_upsert() {
        let store = StatusStore::new();

        // Create initial check
        let check1 = StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            "CI".to_string(),
            CheckState::Pending,
        );
        let created1 = store.create_or_update(check1);

        // Update same context
        let mut check2 = StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            "CI".to_string(),
            CheckState::Success,
        );
        check2.description = Some("All tests passed".to_string());
        let created2 = store.create_or_update(check2);

        // Should be the same check, just updated
        assert_eq!(created1.id, created2.id);
        assert_eq!(created2.state, CheckState::Success);
        assert_eq!(created2.description, Some("All tests passed".to_string()));

        // Only one check should exist
        let combined = store.get_combined_status("alice/repo", "abc123");
        assert_eq!(combined.total_count, 1);
    }

    #[test]
    fn test_required_status_check() {
        let store = StatusStore::new();

        store.create_or_update(StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            "CI / Build".to_string(),
            CheckState::Success,
        ));
        store.create_or_update(StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            "CI / Test".to_string(),
            CheckState::Failure,
        ));

        let required = vec![
            "CI / Build".to_string(),
            "CI / Test".to_string(),
            "CI / Lint".to_string(),
        ];
        let (passed, issues) = check_required_statuses(&store, "alice/repo", "abc123", &required);

        assert!(!passed);
        assert!(issues.iter().any(|i| i.contains("missing: CI / Lint")));
        assert!(issues.iter().any(|i| i.contains("failed: CI / Test")));
    }

    #[test]
    fn test_conclusion_to_check_state() {
        assert_eq!(CheckState::from(Conclusion::Success), CheckState::Success);
        assert_eq!(CheckState::from(Conclusion::Failure), CheckState::Failure);
        assert_eq!(CheckState::from(Conclusion::Error), CheckState::Error);
        assert_eq!(CheckState::from(Conclusion::Cancelled), CheckState::Failure);
        assert_eq!(CheckState::from(Conclusion::TimedOut), CheckState::Failure);
    }
}
