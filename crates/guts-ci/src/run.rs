//! Workflow run tracking and management.

use crate::trigger::TriggerContext;
use crate::workflow::WorkflowId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for a workflow run.
pub type RunId = String;

/// A unique identifier for a job run.
pub type JobRunId = String;

/// A single execution of a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    /// Unique identifier for this run
    pub id: RunId,
    /// ID of the workflow being run
    pub workflow_id: WorkflowId,
    /// Workflow name (for display)
    pub workflow_name: String,
    /// Repository key (owner/name)
    pub repo_key: String,
    /// Run number (sequential per workflow)
    pub number: u32,
    /// Current status
    pub status: RunStatus,
    /// Final conclusion (set when completed)
    pub conclusion: Option<Conclusion>,
    /// What triggered this run
    pub trigger: TriggerContext,
    /// Head commit SHA
    pub head_sha: String,
    /// Head branch (if applicable)
    pub head_branch: Option<String>,
    /// Job runs within this workflow run
    pub jobs: HashMap<String, JobRun>,
    /// When execution started
    pub started_at: Option<u64>,
    /// When execution completed
    pub completed_at: Option<u64>,
    /// When this run was created
    pub created_at: u64,
    /// URL to view this run (for status checks)
    pub html_url: Option<String>,
    /// Logs URL
    pub logs_url: Option<String>,
}

/// Status of a workflow or job run.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Waiting to be executed
    Queued,
    /// Waiting for dependencies
    Waiting,
    /// Currently executing
    InProgress,
    /// Execution completed
    Completed,
    /// Cancelled by user
    Cancelled,
}

impl RunStatus {
    /// Check if this status represents an active run.
    pub fn is_active(&self) -> bool {
        matches!(self, RunStatus::Queued | RunStatus::Waiting | RunStatus::InProgress)
    }

    /// Check if this status represents a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, RunStatus::Completed | RunStatus::Cancelled)
    }
}

/// Final conclusion of a workflow or job run.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
    /// All steps succeeded
    Success,
    /// One or more steps failed
    Failure,
    /// Run was cancelled
    Cancelled,
    /// Run was skipped
    Skipped,
    /// Run timed out
    TimedOut,
    /// Action required (manual approval)
    ActionRequired,
    /// Run encountered an error
    Error,
    /// Neutral conclusion (informational)
    Neutral,
}

impl Conclusion {
    /// Check if this is a successful conclusion.
    pub fn is_success(&self) -> bool {
        matches!(self, Conclusion::Success | Conclusion::Neutral | Conclusion::Skipped)
    }

    /// Check if this is a failure conclusion.
    pub fn is_failure(&self) -> bool {
        matches!(self, Conclusion::Failure | Conclusion::TimedOut | Conclusion::Error)
    }
}

impl WorkflowRun {
    /// Create a new workflow run.
    pub fn new(
        id: RunId,
        workflow_id: WorkflowId,
        workflow_name: String,
        repo_key: String,
        number: u32,
        trigger: TriggerContext,
        head_sha: String,
        head_branch: Option<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            workflow_id,
            workflow_name,
            repo_key,
            number,
            status: RunStatus::Queued,
            conclusion: None,
            trigger,
            head_sha,
            head_branch,
            jobs: HashMap::new(),
            started_at: None,
            completed_at: None,
            created_at: now,
            html_url: None,
            logs_url: None,
        }
    }

    /// Start the workflow run.
    pub fn start(&mut self) {
        self.status = RunStatus::InProgress;
        self.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Complete the workflow run with a conclusion.
    pub fn complete(&mut self, conclusion: Conclusion) {
        self.status = RunStatus::Completed;
        self.conclusion = Some(conclusion);
        self.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Cancel the workflow run.
    pub fn cancel(&mut self) {
        if self.status.is_active() {
            self.status = RunStatus::Cancelled;
            self.conclusion = Some(Conclusion::Cancelled);
            self.completed_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }
    }

    /// Calculate the overall conclusion based on job conclusions.
    pub fn calculate_conclusion(&self) -> Conclusion {
        let mut has_failure = false;
        let mut has_cancelled = false;
        let mut all_skipped = true;

        for job in self.jobs.values() {
            if let Some(conclusion) = job.conclusion {
                match conclusion {
                    Conclusion::Failure | Conclusion::TimedOut | Conclusion::Error => {
                        has_failure = true;
                        all_skipped = false;
                    }
                    Conclusion::Cancelled => {
                        has_cancelled = true;
                        all_skipped = false;
                    }
                    Conclusion::Success | Conclusion::Neutral => {
                        all_skipped = false;
                    }
                    Conclusion::Skipped | Conclusion::ActionRequired => {}
                }
            }
        }

        if has_failure {
            Conclusion::Failure
        } else if has_cancelled {
            Conclusion::Cancelled
        } else if all_skipped && !self.jobs.is_empty() {
            Conclusion::Skipped
        } else {
            Conclusion::Success
        }
    }

    /// Get the duration of the run in seconds.
    pub fn duration_seconds(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.saturating_sub(start)),
            (Some(start), None) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                Some(now.saturating_sub(start))
            }
            _ => None,
        }
    }
}

/// Execution state of a job within a workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRun {
    /// Unique identifier for this job run
    pub id: JobRunId,
    /// Job ID from the workflow definition
    pub job_id: String,
    /// Display name
    pub name: String,
    /// Current status
    pub status: RunStatus,
    /// Final conclusion
    pub conclusion: Option<Conclusion>,
    /// Step execution states
    pub steps: Vec<StepRun>,
    /// Runner that executed this job
    pub runner: Option<String>,
    /// When execution started
    pub started_at: Option<u64>,
    /// When execution completed
    pub completed_at: Option<u64>,
    /// Captured logs
    pub logs: Vec<LogEntry>,
}

impl JobRun {
    /// Create a new job run.
    pub fn new(id: JobRunId, job_id: String, name: String, step_count: usize) -> Self {
        Self {
            id,
            job_id,
            name,
            status: RunStatus::Queued,
            conclusion: None,
            steps: (0..step_count)
                .map(|i| StepRun {
                    number: i as u32,
                    name: format!("Step {}", i + 1),
                    status: RunStatus::Queued,
                    conclusion: None,
                    started_at: None,
                    completed_at: None,
                })
                .collect(),
            runner: None,
            started_at: None,
            completed_at: None,
            logs: Vec::new(),
        }
    }

    /// Start the job run.
    pub fn start(&mut self, runner: Option<String>) {
        self.status = RunStatus::InProgress;
        self.runner = runner;
        self.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Complete the job run.
    pub fn complete(&mut self, conclusion: Conclusion) {
        self.status = RunStatus::Completed;
        self.conclusion = Some(conclusion);
        self.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Get the duration of the job in seconds.
    pub fn duration_seconds(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.saturating_sub(start)),
            (Some(start), None) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                Some(now.saturating_sub(start))
            }
            _ => None,
        }
    }

    /// Add a log entry.
    pub fn add_log(&mut self, step: Option<u32>, level: LogLevel, message: String) {
        self.logs.push(LogEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            step,
            level,
            message,
        });
    }
}

/// Execution state of a step within a job run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRun {
    /// Step number (0-indexed)
    pub number: u32,
    /// Display name
    pub name: String,
    /// Current status
    pub status: RunStatus,
    /// Final conclusion
    pub conclusion: Option<Conclusion>,
    /// When execution started
    pub started_at: Option<u64>,
    /// When execution completed
    pub completed_at: Option<u64>,
}

impl StepRun {
    /// Start the step.
    pub fn start(&mut self) {
        self.status = RunStatus::InProgress;
        self.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Complete the step.
    pub fn complete(&mut self, conclusion: Conclusion) {
        self.status = RunStatus::Completed;
        self.conclusion = Some(conclusion);
        self.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }
}

/// A log entry from job execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp in seconds since epoch
    pub timestamp: u64,
    /// Step number if applicable
    pub step: Option<u32>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
}

/// Log level for job execution logs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trigger::TriggerType;

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
    fn test_workflow_run_lifecycle() {
        let mut run = WorkflowRun::new(
            "run-1".to_string(),
            "ci".to_string(),
            "CI".to_string(),
            "alice/repo".to_string(),
            1,
            test_trigger_context(),
            "abc123".to_string(),
            Some("main".to_string()),
        );

        assert_eq!(run.status, RunStatus::Queued);
        assert!(run.started_at.is_none());

        run.start();
        assert_eq!(run.status, RunStatus::InProgress);
        assert!(run.started_at.is_some());

        run.complete(Conclusion::Success);
        assert_eq!(run.status, RunStatus::Completed);
        assert_eq!(run.conclusion, Some(Conclusion::Success));
        assert!(run.completed_at.is_some());
    }

    #[test]
    fn test_job_run_lifecycle() {
        let mut job = JobRun::new("job-1".to_string(), "build".to_string(), "Build".to_string(), 2);

        assert_eq!(job.status, RunStatus::Queued);
        assert_eq!(job.steps.len(), 2);

        job.start(Some("default".to_string()));
        assert_eq!(job.status, RunStatus::InProgress);
        assert_eq!(job.runner, Some("default".to_string()));

        job.complete(Conclusion::Success);
        assert_eq!(job.status, RunStatus::Completed);
        assert_eq!(job.conclusion, Some(Conclusion::Success));
    }

    #[test]
    fn test_run_cancellation() {
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

        run.start();
        run.cancel();
        assert_eq!(run.status, RunStatus::Cancelled);
        assert_eq!(run.conclusion, Some(Conclusion::Cancelled));
    }

    #[test]
    fn test_conclusion_calculation() {
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

        // Empty run is success
        assert_eq!(run.calculate_conclusion(), Conclusion::Success);

        // Add a successful job
        let mut job1 = JobRun::new("j1".to_string(), "build".to_string(), "Build".to_string(), 1);
        job1.complete(Conclusion::Success);
        run.jobs.insert("build".to_string(), job1);
        assert_eq!(run.calculate_conclusion(), Conclusion::Success);

        // Add a failed job
        let mut job2 = JobRun::new("j2".to_string(), "test".to_string(), "Test".to_string(), 1);
        job2.complete(Conclusion::Failure);
        run.jobs.insert("test".to_string(), job2);
        assert_eq!(run.calculate_conclusion(), Conclusion::Failure);
    }

    #[test]
    fn test_run_status_checks() {
        assert!(RunStatus::Queued.is_active());
        assert!(RunStatus::InProgress.is_active());
        assert!(!RunStatus::Completed.is_active());

        assert!(RunStatus::Completed.is_terminal());
        assert!(RunStatus::Cancelled.is_terminal());
        assert!(!RunStatus::InProgress.is_terminal());
    }

    #[test]
    fn test_conclusion_checks() {
        assert!(Conclusion::Success.is_success());
        assert!(Conclusion::Neutral.is_success());
        assert!(!Conclusion::Failure.is_success());

        assert!(Conclusion::Failure.is_failure());
        assert!(Conclusion::TimedOut.is_failure());
        assert!(!Conclusion::Success.is_failure());
    }
}
