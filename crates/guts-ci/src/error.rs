//! Error types for the CI/CD system.

use thiserror::Error;

/// Errors that can occur in the CI/CD system.
#[derive(Error, Debug)]
pub enum CiError {
    /// Workflow not found
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    /// Workflow run not found
    #[error("Workflow run not found: {0}")]
    RunNotFound(String),

    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(String),

    /// Artifact not found
    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    /// Invalid workflow configuration
    #[error("Invalid workflow configuration: {0}")]
    InvalidWorkflow(String),

    /// Invalid trigger configuration
    #[error("Invalid trigger configuration: {0}")]
    InvalidTrigger(String),

    /// Job execution failed
    #[error("Job execution failed: {0}")]
    ExecutionFailed(String),

    /// Step execution failed
    #[error("Step execution failed: {0}")]
    StepFailed(String),

    /// Timeout exceeded
    #[error("Execution timeout exceeded: {0}")]
    Timeout(String),

    /// Circular dependency detected
    #[error("Circular dependency detected in jobs: {0}")]
    CircularDependency(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlParse(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Run is not cancellable
    #[error("Run cannot be cancelled: {0}")]
    NotCancellable(String),

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
}

/// Result type for CI operations.
pub type Result<T> = std::result::Result<T, CiError>;
