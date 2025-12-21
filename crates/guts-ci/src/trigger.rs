//! Trigger types for workflow execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What triggers a workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    /// Triggered on push to matching branches
    Push {
        /// Branch patterns to match (e.g., "main", "feature/*")
        #[serde(default)]
        branches: Vec<String>,
        /// Path patterns to match (e.g., "src/**", "*.rs")
        #[serde(default)]
        paths: Vec<String>,
        /// Branch patterns to ignore
        #[serde(default)]
        branches_ignore: Vec<String>,
        /// Path patterns to ignore
        #[serde(default)]
        paths_ignore: Vec<String>,
    },
    /// Triggered on pull request events
    PullRequest {
        /// Branch patterns to match
        #[serde(default)]
        branches: Vec<String>,
        /// PR event types
        #[serde(default)]
        types: Vec<PrEventType>,
        /// Path patterns to match
        #[serde(default)]
        paths: Vec<String>,
    },
    /// Triggered on a schedule (cron)
    Schedule {
        /// Cron expression
        cron: String,
    },
    /// Manual trigger via API or UI
    WorkflowDispatch {
        /// Input definitions for manual trigger
        #[serde(default)]
        inputs: HashMap<String, InputDefinition>,
    },
    /// Triggered by another workflow
    WorkflowCall {
        /// Input definitions
        #[serde(default)]
        inputs: HashMap<String, InputDefinition>,
        /// Secrets required
        #[serde(default)]
        secrets: Vec<String>,
    },
}

/// PR event types that can trigger workflows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PrEventType {
    Opened,
    Closed,
    Reopened,
    #[default]
    Synchronize,
    Edited,
    ReadyForReview,
    Labeled,
    Unlabeled,
    ReviewRequested,
    ReviewRequestRemoved,
}

/// Definition for a workflow input parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputDefinition {
    /// Description of the input
    #[serde(default)]
    pub description: String,
    /// Whether the input is required
    #[serde(default)]
    pub required: bool,
    /// Default value
    #[serde(default)]
    pub default: Option<String>,
    /// Input type
    #[serde(default, rename = "type")]
    pub input_type: InputType,
    /// Allowed values for choice type
    #[serde(default)]
    pub options: Vec<String>,
}

/// Type of input parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    #[default]
    String,
    Boolean,
    Choice,
    Environment,
}

/// Context about what triggered a workflow run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerContext {
    /// The trigger type
    pub trigger_type: TriggerType,
    /// The actor who triggered it
    pub actor: String,
    /// Reference that triggered (e.g., "refs/heads/main")
    pub ref_name: Option<String>,
    /// SHA of the commit
    pub sha: String,
    /// Base SHA for PRs
    pub base_sha: Option<String>,
    /// PR number if triggered by PR
    pub pr_number: Option<u32>,
    /// Input values for manual triggers
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    /// Event payload
    #[serde(default)]
    pub event: serde_json::Value,
}

/// Simplified trigger type for context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    Push,
    PullRequest,
    Schedule,
    WorkflowDispatch,
    WorkflowCall,
    Manual,
}

impl Trigger {
    /// Check if this trigger matches a push event.
    pub fn matches_push(&self, branch: &str, changed_paths: &[String]) -> bool {
        match self {
            Trigger::Push {
                branches,
                paths,
                branches_ignore,
                paths_ignore,
            } => {
                // Check branch ignore patterns first
                if Self::matches_patterns(branch, branches_ignore) {
                    return false;
                }

                // Check branch patterns (empty means all branches)
                if !branches.is_empty() && !Self::matches_patterns(branch, branches) {
                    return false;
                }

                // Check path patterns
                if !paths.is_empty() {
                    let any_path_matches = changed_paths
                        .iter()
                        .any(|p| Self::matches_patterns(p, paths));
                    if !any_path_matches {
                        return false;
                    }
                }

                // Check path ignore patterns
                if !paths_ignore.is_empty() && !changed_paths.is_empty() {
                    let all_paths_ignored = changed_paths
                        .iter()
                        .all(|p| Self::matches_patterns(p, paths_ignore));
                    if all_paths_ignored {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    /// Check if this trigger matches a PR event.
    pub fn matches_pull_request(
        &self,
        target_branch: &str,
        event_type: &PrEventType,
        changed_paths: &[String],
    ) -> bool {
        match self {
            Trigger::PullRequest {
                branches,
                types,
                paths,
            } => {
                // Check branch patterns (empty means all branches)
                if !branches.is_empty() && !Self::matches_patterns(target_branch, branches) {
                    return false;
                }

                // Check event types (empty means default types)
                if !types.is_empty() && !types.contains(event_type) {
                    return false;
                }

                // Check path patterns
                if !paths.is_empty() {
                    let any_path_matches = changed_paths
                        .iter()
                        .any(|p| Self::matches_patterns(p, paths));
                    if !any_path_matches {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    /// Check if this trigger allows manual dispatch.
    pub fn is_workflow_dispatch(&self) -> bool {
        matches!(self, Trigger::WorkflowDispatch { .. })
    }

    /// Simple glob-like pattern matching.
    fn matches_patterns(value: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if Self::matches_pattern(value, pattern) {
                return true;
            }
        }
        false
    }

    /// Match a single pattern against a value.
    fn matches_pattern(value: &str, pattern: &str) -> bool {
        if pattern == "*" || pattern == "**" {
            return true;
        }

        if pattern.contains('*') {
            // Handle **/*.ext patterns (any file with extension in any directory)
            if pattern.starts_with("**/") && pattern.contains('.') {
                let suffix = &pattern[3..]; // e.g., "*.rs"
                if let Some(ext) = suffix.strip_prefix('*') {
                    // Pattern like **/*.rs
                    // e.g., ".rs"
                    return value.ends_with(ext);
                }
                // Pattern like **/foo.rs
                return value.ends_with(suffix) || value.contains(&format!("/{suffix}"));
            }
            // Handle prefix/** patterns
            if let Some(prefix) = pattern.strip_suffix("/**") {
                return value.starts_with(prefix);
            }
            // Simple glob matching (single *)
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let (prefix, suffix) = (parts[0], parts[1]);
                return value.starts_with(prefix) && value.ends_with(suffix);
            }
        }

        value == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_trigger_matching() {
        let trigger = Trigger::Push {
            branches: vec!["main".to_string(), "develop".to_string()],
            paths: vec![],
            branches_ignore: vec![],
            paths_ignore: vec![],
        };

        assert!(trigger.matches_push("main", &[]));
        assert!(trigger.matches_push("develop", &[]));
        assert!(!trigger.matches_push("feature/foo", &[]));
    }

    #[test]
    fn test_push_trigger_with_paths() {
        let trigger = Trigger::Push {
            branches: vec!["main".to_string()],
            paths: vec!["src/**".to_string()],
            branches_ignore: vec![],
            paths_ignore: vec![],
        };

        assert!(trigger.matches_push("main", &["src/lib.rs".to_string()]));
        assert!(!trigger.matches_push("main", &["docs/README.md".to_string()]));
    }

    #[test]
    fn test_pr_trigger_matching() {
        let trigger = Trigger::PullRequest {
            branches: vec!["main".to_string()],
            types: vec![PrEventType::Opened, PrEventType::Synchronize],
            paths: vec![],
        };

        assert!(trigger.matches_pull_request("main", &PrEventType::Opened, &[]));
        assert!(!trigger.matches_pull_request("main", &PrEventType::Closed, &[]));
        assert!(!trigger.matches_pull_request("develop", &PrEventType::Opened, &[]));
    }

    #[test]
    fn test_pattern_matching() {
        assert!(Trigger::matches_pattern("main", "main"));
        assert!(Trigger::matches_pattern("main", "*"));
        assert!(Trigger::matches_pattern("feature/foo", "feature/*"));
        assert!(Trigger::matches_pattern("src/lib.rs", "src/**"));
        assert!(Trigger::matches_pattern("deep/nested/file.rs", "**/*.rs"));
    }
}
