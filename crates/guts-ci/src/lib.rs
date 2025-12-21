//! # Guts CI/CD
//!
//! CI/CD pipeline support for the Guts code collaboration platform.
//!
//! This crate provides decentralized continuous integration and deployment
//! pipelines, enabling automated build, test, and deployment workflows.
//!
//! ## Features
//!
//! - **Workflow Configuration**: YAML-based pipeline definitions
//! - **Job Execution**: Isolated job execution with step-by-step processing
//! - **Status Checks**: Integration with branch protection
//! - **Artifact Management**: Store and retrieve build artifacts
//! - **Trigger Matching**: Push, PR, schedule, and manual triggers
//!
//! ## Workflow Example
//!
//! ```yaml
//! name: CI
//!
//! on:
//!   push:
//!     branches: [main]
//!   pull_request:
//!     branches: [main]
//!
//! jobs:
//!   build:
//!     name: Build
//!     runs-on: default
//!     steps:
//!       - name: Checkout
//!         uses: checkout
//!       - name: Build
//!         run: cargo build --workspace
//!       - name: Test
//!         run: cargo test --workspace
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use guts_ci::{Workflow, CiStore, WorkflowRun, TriggerContext, TriggerType};
//! use std::collections::HashMap;
//!
//! // Parse a workflow
//! let yaml = r#"
//! name: CI
//! on: push
//! jobs:
//!   test:
//!     steps:
//!       - run: echo "Hello"
//! "#;
//! let workflow = Workflow::parse(yaml, "alice/repo", ".guts/workflows/ci.yml").unwrap();
//!
//! // Store the workflow
//! let store = CiStore::new();
//! store.workflows.store(workflow.clone());
//!
//! // Check if it matches a push event
//! assert!(workflow.matches_push("main", &[]));
//!
//! // Create a workflow run
//! let trigger = TriggerContext {
//!     trigger_type: TriggerType::Push,
//!     actor: "alice".to_string(),
//!     ref_name: Some("refs/heads/main".to_string()),
//!     sha: "abc123def".to_string(),
//!     base_sha: None,
//!     pr_number: None,
//!     inputs: HashMap::new(),
//!     event: serde_json::Value::Null,
//! };
//!
//! let run = WorkflowRun::new(
//!     uuid::Uuid::new_v4().to_string(),
//!     workflow.id.clone(),
//!     workflow.name.clone(),
//!     "alice/repo".to_string(),
//!     1,
//!     trigger,
//!     "abc123def".to_string(),
//!     Some("main".to_string()),
//! );
//! store.runs.store(run);
//! ```
//!
//! ## Architecture
//!
//! ```text
//! Push/PR Event
//!      │
//!      ▼
//! ┌───────────────────┐
//! │  Trigger Matcher  │
//! │ (match workflows) │
//! └─────────┬─────────┘
//!           │
//!           ▼
//! ┌───────────────────┐
//! │   WorkflowRun     │
//! │    Created        │
//! └─────────┬─────────┘
//!           │
//!           ▼
//! ┌───────────────────┐
//! │   Job Executor    │
//! │  ┌─────────────┐  │
//! │  │  Step 1     │  │
//! │  │  Step 2     │  │
//! │  │  Step 3     │  │
//! │  └─────────────┘  │
//! └─────────┬─────────┘
//!           │
//!           ▼
//! ┌───────────────────┐
//! │  Status Check     │
//! │    Updated        │
//! └───────────────────┘
//! ```

pub mod artifact;
pub mod error;
pub mod executor;
pub mod job;
pub mod run;
pub mod status;
pub mod step;
pub mod store;
pub mod trigger;
pub mod workflow;

// Re-export main types
pub use artifact::{Artifact, ArtifactId, ArtifactStore};
pub use error::{CiError, Result};
pub use executor::{ExecutionContext, JobExecutionResult, JobExecutor, LogSender};
pub use job::{
    get_ready_jobs, resolve_job_order, ConcurrencySettings, JobDefinition, JobId, JobStrategy,
    ServiceDefinition,
};
pub use run::{
    Conclusion, JobRun, JobRunId, LogEntry, LogLevel, RunId, RunStatus, StepRun, WorkflowRun,
};
pub use status::{
    check_required_statuses, CheckState, CombinedStatus, StatusCheck, StatusCheckId, StatusStore,
};
pub use step::{BuiltinAction, RunStep, Step, StepOutput, UsesStep};
pub use store::{CiStats, CiStore, RunStore, WorkflowStore};
pub use trigger::{InputDefinition, InputType, PrEventType, Trigger, TriggerContext, TriggerType};
pub use workflow::{Workflow, WorkflowId, WorkflowTriggers};

/// Version of the CI/CD system.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a status check context from a workflow run and job.
pub fn make_check_context(workflow_name: &str, job_name: &str) -> String {
    format!("{} / {}", workflow_name, job_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_public_api() {
        // Verify main types are accessible
        let store = CiStore::new();
        assert_eq!(store.workflows.count(), 0);
        assert_eq!(store.runs.count(), 0);
    }

    #[test]
    fn test_workflow_parsing() {
        let yaml = r#"
name: CI
on: push
jobs:
  build:
    steps:
      - run: cargo build
"#;
        let workflow = Workflow::parse(yaml, "alice/repo", ".guts/workflows/ci.yml").unwrap();
        assert_eq!(workflow.name, "CI");
        assert!(workflow.matches_push("main", &[]));
    }

    #[test]
    fn test_full_flow() {
        let store = CiStore::new();

        // Create workflow
        let yaml = r#"
name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    steps:
      - run: cargo build
"#;
        let workflow = Workflow::parse(yaml, "alice/repo", ".guts/workflows/ci.yml").unwrap();
        store.workflows.store(workflow.clone());

        // Check trigger matching
        assert!(workflow.matches_push("main", &[]));
        assert!(!workflow.matches_push("feature/x", &[]));

        // Create run
        let trigger = TriggerContext {
            trigger_type: TriggerType::Push,
            actor: "alice".to_string(),
            ref_name: Some("refs/heads/main".to_string()),
            sha: "abc123".to_string(),
            base_sha: None,
            pr_number: None,
            inputs: HashMap::new(),
            event: serde_json::Value::Null,
        };

        let run_number = store.runs.next_run_number("alice/repo", "ci");
        let mut run = WorkflowRun::new(
            "run-1".to_string(),
            workflow.id.clone(),
            workflow.name.clone(),
            "alice/repo".to_string(),
            run_number,
            trigger,
            "abc123".to_string(),
            Some("main".to_string()),
        );

        // Start run
        run.start();
        store.runs.store(run.clone());

        // Create status check
        let check = StatusCheck::new(
            "alice/repo".to_string(),
            "abc123".to_string(),
            make_check_context(&workflow.name, "build"),
            CheckState::Pending,
        );
        store.statuses.create_or_update(check);

        // Verify state
        assert_eq!(store.runs.active_count(), 1);

        let combined = store.statuses.get_combined_status("alice/repo", "abc123");
        assert_eq!(combined.state, CheckState::Pending);
    }

    #[test]
    fn test_make_check_context() {
        assert_eq!(make_check_context("CI", "Build"), "CI / Build");
        assert_eq!(make_check_context("Deploy", "Production"), "Deploy / Production");
    }
}
