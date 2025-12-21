//! Step definitions for CI/CD jobs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single step in a job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Step {
    /// Run a shell command
    Run(RunStep),
    /// Use a built-in or external action
    Uses(UsesStep),
}

/// A step that runs shell commands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunStep {
    /// Optional name for the step
    #[serde(default)]
    pub name: Option<String>,

    /// The command to run
    pub run: String,

    /// Working directory for the command
    #[serde(default)]
    pub working_directory: Option<String>,

    /// Environment variables for this step
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Shell to use (e.g., "bash", "sh", "python")
    #[serde(default)]
    pub shell: Option<String>,

    /// Condition for running this step
    #[serde(default, rename = "if")]
    pub condition: Option<String>,

    /// Continue on error
    #[serde(default)]
    pub continue_on_error: bool,

    /// Timeout in minutes
    #[serde(default)]
    pub timeout_minutes: Option<u32>,

    /// Step ID for referencing outputs
    #[serde(default)]
    pub id: Option<String>,
}

/// A step that uses a built-in or external action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsesStep {
    /// Optional name for the step
    #[serde(default)]
    pub name: Option<String>,

    /// Action to use (e.g., "checkout", "cache@v1")
    pub uses: String,

    /// Input parameters for the action
    #[serde(default)]
    pub with: HashMap<String, serde_json::Value>,

    /// Environment variables for this step
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Condition for running this step
    #[serde(default, rename = "if")]
    pub condition: Option<String>,

    /// Continue on error
    #[serde(default)]
    pub continue_on_error: bool,

    /// Timeout in minutes
    #[serde(default)]
    pub timeout_minutes: Option<u32>,

    /// Step ID for referencing outputs
    #[serde(default)]
    pub id: Option<String>,
}

impl Step {
    /// Get the display name for this step.
    pub fn name(&self) -> String {
        match self {
            Step::Run(step) => step
                .name
                .clone()
                .unwrap_or_else(|| truncate_command(&step.run)),
            Step::Uses(step) => step
                .name
                .clone()
                .unwrap_or_else(|| format!("Use {}", step.uses)),
        }
    }

    /// Get the step ID if set.
    pub fn id(&self) -> Option<&str> {
        match self {
            Step::Run(step) => step.id.as_deref(),
            Step::Uses(step) => step.id.as_deref(),
        }
    }

    /// Check if this step should continue on error.
    pub fn continue_on_error(&self) -> bool {
        match self {
            Step::Run(step) => step.continue_on_error,
            Step::Uses(step) => step.continue_on_error,
        }
    }

    /// Get the timeout in minutes, if set.
    pub fn timeout_minutes(&self) -> Option<u32> {
        match self {
            Step::Run(step) => step.timeout_minutes,
            Step::Uses(step) => step.timeout_minutes,
        }
    }

    /// Get the condition expression, if set.
    pub fn condition(&self) -> Option<&str> {
        match self {
            Step::Run(step) => step.condition.as_deref(),
            Step::Uses(step) => step.condition.as_deref(),
        }
    }

    /// Get environment variables for this step.
    pub fn env(&self) -> &HashMap<String, String> {
        match self {
            Step::Run(step) => &step.env,
            Step::Uses(step) => &step.env,
        }
    }
}

/// Truncate a command for display purposes.
fn truncate_command(cmd: &str) -> String {
    let first_line = cmd.lines().next().unwrap_or(cmd);
    if first_line.len() > 50 {
        format!("{}...", &first_line[..47])
    } else {
        first_line.to_string()
    }
}

/// Output from a step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    /// Output key-value pairs
    pub outputs: HashMap<String, String>,
    /// Exit code
    pub exit_code: i32,
    /// Stdout
    pub stdout: String,
    /// Stderr
    pub stderr: String,
}

impl Default for StepOutput {
    fn default() -> Self {
        Self {
            outputs: HashMap::new(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

/// Built-in actions available by default.
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinAction {
    /// Checkout the repository
    Checkout,
    /// Cache dependencies
    Cache,
    /// Upload artifacts
    UploadArtifact,
    /// Download artifacts
    DownloadArtifact,
    /// Setup Rust toolchain
    SetupRust,
    /// Unknown action
    Unknown(String),
}

impl From<&str> for BuiltinAction {
    fn from(s: &str) -> Self {
        // Remove version suffix if present
        let action = s.split('@').next().unwrap_or(s);
        match action {
            "checkout" => BuiltinAction::Checkout,
            "cache" => BuiltinAction::Cache,
            "upload-artifact" => BuiltinAction::UploadArtifact,
            "download-artifact" => BuiltinAction::DownloadArtifact,
            "setup-rust" => BuiltinAction::SetupRust,
            other => BuiltinAction::Unknown(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_step_parsing() {
        let yaml = r#"
name: Build
run: cargo build --workspace
env:
  RUST_BACKTRACE: "1"
"#;
        let step: RunStep = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.name, Some("Build".to_string()));
        assert_eq!(step.run, "cargo build --workspace");
        assert_eq!(step.env.get("RUST_BACKTRACE"), Some(&"1".to_string()));
    }

    #[test]
    fn test_uses_step_parsing() {
        let yaml = r#"
name: Checkout
uses: checkout
with:
  fetch-depth: 1
"#;
        let step: UsesStep = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.name, Some("Checkout".to_string()));
        assert_eq!(step.uses, "checkout");
        assert_eq!(
            step.with.get("fetch-depth"),
            Some(&serde_json::json!(1))
        );
    }

    #[test]
    fn test_builtin_action_parsing() {
        assert_eq!(BuiltinAction::from("checkout"), BuiltinAction::Checkout);
        assert_eq!(BuiltinAction::from("checkout@v1"), BuiltinAction::Checkout);
        assert_eq!(BuiltinAction::from("cache"), BuiltinAction::Cache);
        assert!(matches!(
            BuiltinAction::from("custom-action"),
            BuiltinAction::Unknown(_)
        ));
    }

    #[test]
    fn test_step_name() {
        let run_step = Step::Run(RunStep {
            name: Some("Build".to_string()),
            run: "cargo build".to_string(),
            working_directory: None,
            env: HashMap::new(),
            shell: None,
            condition: None,
            continue_on_error: false,
            timeout_minutes: None,
            id: None,
        });
        assert_eq!(run_step.name(), "Build");

        let run_step_no_name = Step::Run(RunStep {
            name: None,
            run: "cargo build --workspace".to_string(),
            working_directory: None,
            env: HashMap::new(),
            shell: None,
            condition: None,
            continue_on_error: false,
            timeout_minutes: None,
            id: None,
        });
        assert_eq!(run_step_no_name.name(), "cargo build --workspace");
    }
}
