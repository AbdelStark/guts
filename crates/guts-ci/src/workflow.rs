//! Workflow definitions and parsing.

use crate::error::{CiError, Result};
use crate::job::JobDefinition;
use crate::trigger::Trigger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for a workflow.
pub type WorkflowId = String;

/// A complete workflow definition parsed from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier (set after parsing from path)
    #[serde(default)]
    pub id: WorkflowId,
    /// Workflow name
    pub name: String,
    /// Repository key (owner/name) (set after parsing)
    #[serde(default)]
    pub repo_key: String,
    /// Path to the workflow file (set after parsing)
    #[serde(default)]
    pub path: String,
    /// Triggers that activate this workflow
    #[serde(default, rename = "on")]
    pub triggers: WorkflowTriggers,
    /// Global environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Job definitions
    #[serde(default)]
    pub jobs: HashMap<String, JobDefinition>,
    /// Workflow-level concurrency settings
    #[serde(default)]
    pub concurrency: Option<ConcurrencyConfig>,
    /// Permissions for the workflow
    #[serde(default)]
    pub permissions: Option<PermissionsConfig>,
    /// Default settings for all jobs
    #[serde(default)]
    pub defaults: Option<DefaultsConfig>,
    /// When this workflow was created
    #[serde(default)]
    pub created_at: u64,
    /// When this workflow was last updated
    #[serde(default)]
    pub updated_at: u64,
}

/// Workflow triggers configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum WorkflowTriggers {
    /// Single trigger type (string shorthand)
    Single(String),
    /// List of trigger types
    List(Vec<String>),
    /// Full trigger configuration
    Full(HashMap<String, TriggerConfig>),
    /// No triggers
    #[default]
    None,
}

/// Configuration for a specific trigger type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TriggerConfig {
    /// Null/empty config (trigger on any)
    Empty,
    /// Branch/path filter config
    Filter(FilterConfig),
    /// Workflow dispatch with inputs
    Dispatch(DispatchConfig),
    /// Schedule with cron
    Schedule(Vec<ScheduleConfig>),
}

/// Branch and path filter configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterConfig {
    #[serde(default)]
    pub branches: Vec<String>,
    #[serde(default, rename = "branches-ignore")]
    pub branches_ignore: Vec<String>,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default, rename = "paths-ignore")]
    pub paths_ignore: Vec<String>,
    #[serde(default)]
    pub types: Vec<String>,
}

/// Workflow dispatch configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DispatchConfig {
    #[serde(default)]
    pub inputs: HashMap<String, InputConfig>,
}

/// Input parameter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default, rename = "type")]
    pub input_type: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
}

/// Schedule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub cron: String,
}

/// Concurrency configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConcurrencyConfig {
    /// Simple group name
    Simple(String),
    /// Full config
    Full {
        group: String,
        #[serde(default, rename = "cancel-in-progress")]
        cancel_in_progress: bool,
    },
}

/// Permissions configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermissionsConfig {
    /// Permission level for all scopes
    All(String),
    /// Per-scope permissions
    Scopes(HashMap<String, String>),
}

/// Default settings for jobs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultsConfig {
    #[serde(default)]
    pub run: Option<RunDefaults>,
}

/// Default settings for run steps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunDefaults {
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default, rename = "working-directory")]
    pub working_directory: Option<String>,
}

impl Workflow {
    /// Parse a workflow from YAML content.
    pub fn parse(yaml: &str, repo_key: &str, path: &str) -> Result<Self> {
        let mut workflow: Workflow =
            serde_yaml::from_str(yaml).map_err(|e| CiError::YamlParse(e.to_string()))?;

        // Generate ID from path
        workflow.id = path
            .trim_start_matches(".guts/workflows/")
            .trim_end_matches(".yml")
            .trim_end_matches(".yaml")
            .to_string();

        workflow.repo_key = repo_key.to_string();
        workflow.path = path.to_string();

        // Set timestamps
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if workflow.created_at == 0 {
            workflow.created_at = now;
        }
        workflow.updated_at = now;

        workflow.validate()?;
        Ok(workflow)
    }

    /// Validate the workflow configuration.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(CiError::InvalidWorkflow("Workflow name is required".into()));
        }

        if self.jobs.is_empty() {
            return Err(CiError::InvalidWorkflow(
                "Workflow must have at least one job".into(),
            ));
        }

        // Validate job dependencies
        for (job_id, job) in &self.jobs {
            for dep in &job.needs {
                if !self.jobs.contains_key(dep) {
                    return Err(CiError::InvalidWorkflow(format!(
                        "Job '{}' depends on unknown job '{}'",
                        job_id, dep
                    )));
                }
            }

            if let Err(e) = job.validate() {
                return Err(CiError::InvalidWorkflow(format!(
                    "Job '{}' is invalid: {}",
                    job_id, e
                )));
            }
        }

        // Check for circular dependencies
        crate::job::resolve_job_order(&self.jobs)
            .map_err(|e| CiError::CircularDependency(e))?;

        Ok(())
    }

    /// Get triggers as a list of Trigger enums.
    pub fn get_triggers(&self) -> Vec<Trigger> {
        match &self.triggers {
            WorkflowTriggers::None => vec![],
            WorkflowTriggers::Single(s) => vec![self.trigger_from_string(s)],
            WorkflowTriggers::List(list) => {
                list.iter().map(|s| self.trigger_from_string(s)).collect()
            }
            WorkflowTriggers::Full(map) => map
                .iter()
                .map(|(name, config)| self.trigger_from_config(name, config))
                .collect(),
        }
    }

    fn trigger_from_string(&self, s: &str) -> Trigger {
        match s {
            "push" => Trigger::Push {
                branches: vec![],
                paths: vec![],
                branches_ignore: vec![],
                paths_ignore: vec![],
            },
            "pull_request" => Trigger::PullRequest {
                branches: vec![],
                types: vec![],
                paths: vec![],
            },
            "workflow_dispatch" => Trigger::WorkflowDispatch {
                inputs: HashMap::new(),
            },
            _ => Trigger::Push {
                branches: vec![],
                paths: vec![],
                branches_ignore: vec![],
                paths_ignore: vec![],
            },
        }
    }

    fn trigger_from_config(&self, name: &str, config: &TriggerConfig) -> Trigger {
        match (name, config) {
            ("push", TriggerConfig::Filter(f)) => Trigger::Push {
                branches: f.branches.clone(),
                paths: f.paths.clone(),
                branches_ignore: f.branches_ignore.clone(),
                paths_ignore: f.paths_ignore.clone(),
            },
            ("push", _) => Trigger::Push {
                branches: vec![],
                paths: vec![],
                branches_ignore: vec![],
                paths_ignore: vec![],
            },
            ("pull_request", TriggerConfig::Filter(f)) => Trigger::PullRequest {
                branches: f.branches.clone(),
                types: f
                    .types
                    .iter()
                    .filter_map(|t| serde_json::from_value(serde_json::json!(t)).ok())
                    .collect(),
                paths: f.paths.clone(),
            },
            ("pull_request", _) => Trigger::PullRequest {
                branches: vec![],
                types: vec![],
                paths: vec![],
            },
            ("workflow_dispatch", TriggerConfig::Dispatch(d)) => Trigger::WorkflowDispatch {
                inputs: d
                    .inputs
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            crate::trigger::InputDefinition {
                                description: v.description.clone(),
                                required: v.required,
                                default: v.default.clone(),
                                input_type: crate::trigger::InputType::String,
                                options: v.options.clone(),
                            },
                        )
                    })
                    .collect(),
            },
            ("schedule", TriggerConfig::Schedule(schedules)) => {
                if let Some(first) = schedules.first() {
                    Trigger::Schedule {
                        cron: first.cron.clone(),
                    }
                } else {
                    Trigger::Schedule {
                        cron: String::new(),
                    }
                }
            }
            _ => Trigger::WorkflowDispatch {
                inputs: HashMap::new(),
            },
        }
    }

    /// Check if this workflow should trigger on a push event.
    pub fn matches_push(&self, branch: &str, changed_paths: &[String]) -> bool {
        self.get_triggers()
            .iter()
            .any(|t| t.matches_push(branch, changed_paths))
    }

    /// Check if this workflow should trigger on a PR event.
    pub fn matches_pull_request(
        &self,
        target_branch: &str,
        event_type: &crate::trigger::PrEventType,
        changed_paths: &[String],
    ) -> bool {
        self.get_triggers()
            .iter()
            .any(|t| t.matches_pull_request(target_branch, event_type, changed_paths))
    }

    /// Check if this workflow can be manually triggered.
    pub fn allows_manual_trigger(&self) -> bool {
        self.get_triggers().iter().any(|t| t.is_workflow_dispatch())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_parsing() {
        let yaml = r#"
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    name: Build
    runs-on: default
    steps:
      - name: Checkout
        uses: checkout
      - name: Build
        run: cargo build
"#;

        let workflow = Workflow::parse(yaml, "alice/myrepo", ".guts/workflows/ci.yml").unwrap();
        assert_eq!(workflow.name, "CI");
        assert_eq!(workflow.repo_key, "alice/myrepo");
        assert_eq!(workflow.id, "ci");
        assert_eq!(workflow.jobs.len(), 1);
        assert!(workflow.jobs.contains_key("build"));
    }

    #[test]
    fn test_workflow_triggers() {
        let yaml = r#"
name: Test
on: push
jobs:
  test:
    steps:
      - run: echo test
"#;

        let workflow = Workflow::parse(yaml, "test/repo", ".guts/workflows/test.yml").unwrap();
        assert!(workflow.matches_push("main", &[]));
    }

    #[test]
    fn test_workflow_with_branch_filter() {
        let yaml = r#"
name: Test
on:
  push:
    branches: [main, develop]
jobs:
  test:
    steps:
      - run: echo test
"#;

        let workflow = Workflow::parse(yaml, "test/repo", ".guts/workflows/test.yml").unwrap();
        assert!(workflow.matches_push("main", &[]));
        assert!(workflow.matches_push("develop", &[]));
        assert!(!workflow.matches_push("feature/foo", &[]));
    }

    #[test]
    fn test_workflow_validation_empty_jobs() {
        let yaml = r#"
name: Empty
on: push
jobs: {}
"#;

        let result = Workflow::parse(yaml, "test/repo", ".guts/workflows/test.yml");
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_validation_missing_dependency() {
        let yaml = r#"
name: Bad Deps
on: push
jobs:
  build:
    needs: [setup]
    steps:
      - run: cargo build
"#;

        let result = Workflow::parse(yaml, "test/repo", ".guts/workflows/test.yml");
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_dispatch() {
        let yaml = r#"
name: Manual
on:
  workflow_dispatch:
    inputs:
      version:
        description: Version to deploy
        required: true
jobs:
  deploy:
    steps:
      - run: echo deploy
"#;

        let workflow = Workflow::parse(yaml, "test/repo", ".guts/workflows/test.yml").unwrap();
        assert!(workflow.allows_manual_trigger());
    }
}
