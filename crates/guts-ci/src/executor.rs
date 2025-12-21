//! Job execution engine for CI/CD.

use crate::error::{CiError, Result};
use crate::job::JobDefinition;
use crate::run::{Conclusion, JobRun, LogEntry, LogLevel, RunStatus};
use crate::step::{BuiltinAction, Step, StepOutput};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Execution context for a job.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Repository key
    pub repo_key: String,
    /// Working directory
    pub work_dir: PathBuf,
    /// Commit SHA
    pub sha: String,
    /// Branch name
    pub branch: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Outputs from previous steps
    pub step_outputs: HashMap<String, StepOutput>,
    /// Outputs from dependency jobs
    pub job_outputs: HashMap<String, HashMap<String, String>>,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new(repo_key: String, work_dir: PathBuf, sha: String) -> Self {
        Self {
            repo_key,
            work_dir,
            sha,
            branch: None,
            env: HashMap::new(),
            step_outputs: HashMap::new(),
            job_outputs: HashMap::new(),
        }
    }

    /// Add environment variables.
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env.extend(env);
        self
    }

    /// Set the branch.
    pub fn with_branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self
    }

    /// Get all environment variables for a step.
    pub fn get_env(&self, step_env: &HashMap<String, String>) -> HashMap<String, String> {
        let mut env = self.env.clone();

        // Add built-in variables
        env.insert("GUTS_SHA".to_string(), self.sha.clone());
        env.insert("GUTS_REPOSITORY".to_string(), self.repo_key.clone());
        if let Some(ref branch) = self.branch {
            env.insert("GUTS_REF".to_string(), format!("refs/heads/{}", branch));
            env.insert("GUTS_BRANCH".to_string(), branch.clone());
        }
        env.insert(
            "GUTS_WORKSPACE".to_string(),
            self.work_dir.display().to_string(),
        );

        // Add step-specific env
        env.extend(step_env.clone());

        env
    }

    /// Store step output.
    pub fn set_step_output(&mut self, step_id: &str, output: StepOutput) {
        self.step_outputs.insert(step_id.to_string(), output);
    }
}

/// Result of executing a job.
#[derive(Debug)]
pub struct JobExecutionResult {
    /// Final job run state
    pub job_run: JobRun,
    /// Conclusion
    pub conclusion: Conclusion,
    /// Outputs from the job
    pub outputs: HashMap<String, String>,
}

/// Log sender for streaming logs.
pub type LogSender = mpsc::UnboundedSender<LogEntry>;

/// Job executor.
pub struct JobExecutor {
    /// Default timeout in seconds
    default_timeout_secs: u64,
    /// Maximum output size in bytes
    max_output_size: usize,
}

impl Default for JobExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl JobExecutor {
    /// Create a new job executor.
    pub fn new() -> Self {
        Self {
            default_timeout_secs: 60 * 60,     // 1 hour default
            max_output_size: 10 * 1024 * 1024, // 10MB max output
        }
    }

    /// Execute a job.
    pub async fn execute_job(
        &self,
        job_id: &str,
        job_def: &JobDefinition,
        mut context: ExecutionContext,
        log_sender: Option<LogSender>,
    ) -> Result<JobExecutionResult> {
        let step_count = job_def.steps.len();
        let mut job_run = JobRun::new(
            uuid::Uuid::new_v4().to_string(),
            job_id.to_string(),
            job_def.display_name(job_id),
            step_count,
        );

        // Update step names from definition
        for (i, step) in job_def.steps.iter().enumerate() {
            if i < job_run.steps.len() {
                job_run.steps[i].name = step.name();
            }
        }

        job_run.start(Some("default".to_string()));
        self.log(
            &log_sender,
            None,
            LogLevel::Info,
            format!("Starting job: {}", job_def.display_name(job_id)),
        );

        // Add job-level environment
        context.env.extend(job_def.env.clone());

        let timeout_secs = (job_def.timeout_minutes as u64) * 60;
        let timeout_secs = if timeout_secs == 0 {
            self.default_timeout_secs
        } else {
            timeout_secs
        };

        let mut overall_success = true;
        let mut skip_remaining = false;

        for (step_idx, step) in job_def.steps.iter().enumerate() {
            let step_run = &mut job_run.steps[step_idx];
            step_run.name = step.name();

            if skip_remaining {
                step_run.status = RunStatus::Completed;
                step_run.conclusion = Some(Conclusion::Skipped);
                continue;
            }

            // Check condition (simplified - just check for "always()" or "failure()")
            let should_run = self.evaluate_condition(step.condition(), overall_success);
            if !should_run {
                step_run.status = RunStatus::Completed;
                step_run.conclusion = Some(Conclusion::Skipped);
                self.log(
                    &log_sender,
                    Some(step_idx as u32),
                    LogLevel::Info,
                    format!("Skipping step: {}", step.name()),
                );
                continue;
            }

            step_run.start();
            self.log(
                &log_sender,
                Some(step_idx as u32),
                LogLevel::Info,
                format!("Running step: {}", step.name()),
            );

            let step_timeout = step
                .timeout_minutes()
                .map(|m| (m as u64) * 60)
                .unwrap_or(timeout_secs);

            let result = tokio::time::timeout(
                std::time::Duration::from_secs(step_timeout),
                self.execute_step(step, &context, &log_sender, step_idx as u32),
            )
            .await;

            let conclusion = match result {
                Ok(Ok(output)) => {
                    if output.exit_code == 0 {
                        // Store output for step ID
                        if let Some(id) = step.id() {
                            context.set_step_output(id, output);
                        }
                        Conclusion::Success
                    } else {
                        self.log(
                            &log_sender,
                            Some(step_idx as u32),
                            LogLevel::Error,
                            format!("Step failed with exit code: {}", output.exit_code),
                        );
                        if step.continue_on_error() {
                            Conclusion::Neutral
                        } else {
                            overall_success = false;
                            Conclusion::Failure
                        }
                    }
                }
                Ok(Err(e)) => {
                    self.log(
                        &log_sender,
                        Some(step_idx as u32),
                        LogLevel::Error,
                        format!("Step error: {}", e),
                    );
                    if step.continue_on_error() {
                        Conclusion::Neutral
                    } else {
                        overall_success = false;
                        Conclusion::Error
                    }
                }
                Err(_) => {
                    self.log(
                        &log_sender,
                        Some(step_idx as u32),
                        LogLevel::Error,
                        "Step timed out".to_string(),
                    );
                    overall_success = false;
                    Conclusion::TimedOut
                }
            };

            step_run.complete(conclusion);

            // If step failed and not continue-on-error, skip remaining steps
            if !overall_success && !step.continue_on_error() {
                skip_remaining = true;
            }
        }

        let conclusion = if overall_success {
            Conclusion::Success
        } else {
            Conclusion::Failure
        };

        job_run.complete(conclusion);
        self.log(
            &log_sender,
            None,
            LogLevel::Info,
            format!("Job completed: {:?}", conclusion),
        );

        Ok(JobExecutionResult {
            job_run,
            conclusion,
            outputs: HashMap::new(),
        })
    }

    /// Execute a single step.
    async fn execute_step(
        &self,
        step: &Step,
        context: &ExecutionContext,
        log_sender: &Option<LogSender>,
        step_idx: u32,
    ) -> Result<StepOutput> {
        match step {
            Step::Run(run_step) => {
                self.execute_run_step(run_step, context, log_sender, step_idx)
                    .await
            }
            Step::Uses(uses_step) => {
                self.execute_uses_step(uses_step, context, log_sender, step_idx)
                    .await
            }
        }
    }

    /// Execute a run step (shell command).
    async fn execute_run_step(
        &self,
        step: &crate::step::RunStep,
        context: &ExecutionContext,
        log_sender: &Option<LogSender>,
        step_idx: u32,
    ) -> Result<StepOutput> {
        let shell = step.shell.as_deref().unwrap_or("sh");
        let work_dir = step
            .working_directory
            .as_ref()
            .map(|d| context.work_dir.join(d))
            .unwrap_or_else(|| context.work_dir.clone());

        let env = context.get_env(&step.env);

        debug!("Executing command: {}", step.run);

        let mut cmd = Command::new(shell);
        cmd.arg("-c")
            .arg(&step.run)
            .current_dir(&work_dir)
            .envs(&env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| CiError::ExecutionFailed(e.to_string()))?;

        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut stdout_output = String::new();
        let mut stderr_output = String::new();

        // Stream output
        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            if stdout_output.len() + line.len() < self.max_output_size {
                                stdout_output.push_str(&line);
                                stdout_output.push('\n');
                            }
                            self.log(log_sender, Some(step_idx), LogLevel::Info, line);
                        }
                        Ok(None) => break,
                        Err(e) => {
                            warn!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            if stderr_output.len() + line.len() < self.max_output_size {
                                stderr_output.push_str(&line);
                                stderr_output.push('\n');
                            }
                            self.log(log_sender, Some(step_idx), LogLevel::Warning, line);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn!("Error reading stderr: {}", e);
                        }
                    }
                }
            }
        }

        // Collect remaining stderr
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            if stderr_output.len() + line.len() < self.max_output_size {
                stderr_output.push_str(&line);
                stderr_output.push('\n');
            }
            self.log(log_sender, Some(step_idx), LogLevel::Warning, line);
        }

        let status = child
            .wait()
            .await
            .map_err(|e| CiError::ExecutionFailed(e.to_string()))?;
        let exit_code = status.code().unwrap_or(-1);

        // Parse output commands (simplified)
        let outputs = self.parse_output_commands(&stdout_output);

        Ok(StepOutput {
            outputs,
            exit_code,
            stdout: stdout_output,
            stderr: stderr_output,
        })
    }

    /// Execute a uses step (built-in action).
    async fn execute_uses_step(
        &self,
        step: &crate::step::UsesStep,
        context: &ExecutionContext,
        log_sender: &Option<LogSender>,
        step_idx: u32,
    ) -> Result<StepOutput> {
        let action = BuiltinAction::from(step.uses.as_str());

        match action {
            BuiltinAction::Checkout => {
                self.log(
                    log_sender,
                    Some(step_idx),
                    LogLevel::Info,
                    "Checking out repository...".to_string(),
                );
                // In a real implementation, this would clone/checkout the repo
                // For now, we just verify the work directory exists
                if !context.work_dir.exists() {
                    return Err(CiError::ExecutionFailed(
                        "Work directory does not exist".into(),
                    ));
                }
                self.log(
                    log_sender,
                    Some(step_idx),
                    LogLevel::Info,
                    format!("Checked out at {}", context.sha),
                );
                Ok(StepOutput::default())
            }
            BuiltinAction::Cache => {
                self.log(
                    log_sender,
                    Some(step_idx),
                    LogLevel::Info,
                    "Cache action (no-op in MVP)".to_string(),
                );
                Ok(StepOutput::default())
            }
            BuiltinAction::UploadArtifact => {
                self.log(
                    log_sender,
                    Some(step_idx),
                    LogLevel::Info,
                    "Upload artifact action (no-op in MVP)".to_string(),
                );
                Ok(StepOutput::default())
            }
            BuiltinAction::DownloadArtifact => {
                self.log(
                    log_sender,
                    Some(step_idx),
                    LogLevel::Info,
                    "Download artifact action (no-op in MVP)".to_string(),
                );
                Ok(StepOutput::default())
            }
            BuiltinAction::SetupRust => {
                self.log(
                    log_sender,
                    Some(step_idx),
                    LogLevel::Info,
                    "Rust is already available".to_string(),
                );
                Ok(StepOutput::default())
            }
            BuiltinAction::Unknown(name) => {
                self.log(
                    log_sender,
                    Some(step_idx),
                    LogLevel::Warning,
                    format!("Unknown action: {}", name),
                );
                Err(CiError::ExecutionFailed(format!(
                    "Unknown action: {}",
                    name
                )))
            }
        }
    }

    /// Evaluate a step condition.
    fn evaluate_condition(&self, condition: Option<&str>, previous_success: bool) -> bool {
        match condition {
            None => previous_success, // Default: run if previous steps succeeded
            Some("always()") => true,
            Some("success()") => previous_success,
            Some("failure()") => !previous_success,
            Some("cancelled()") => false, // We don't support cancellation checks yet
            Some(_) => previous_success,  // Unknown condition, default to previous success
        }
    }

    /// Parse output commands from stdout.
    fn parse_output_commands(&self, output: &str) -> HashMap<String, String> {
        let mut outputs = HashMap::new();

        for line in output.lines() {
            // Support GitHub-style output format: ::set-output name=key::value
            if line.starts_with("::set-output ") {
                if let Some(rest) = line.strip_prefix("::set-output ") {
                    if let Some((name_part, value)) = rest.split_once("::") {
                        if let Some((_, key)) = name_part.split_once("name=") {
                            outputs.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
            // Also support: GUTS_OUTPUT_key=value
            if line.starts_with("GUTS_OUTPUT_") {
                if let Some((key, value)) = line
                    .strip_prefix("GUTS_OUTPUT_")
                    .and_then(|s| s.split_once('='))
                {
                    outputs.insert(key.to_string(), value.to_string());
                }
            }
        }

        outputs
    }

    /// Send a log message.
    fn log(&self, sender: &Option<LogSender>, step: Option<u32>, level: LogLevel, message: String) {
        if let Some(sender) = sender {
            let entry = LogEntry {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                step,
                level,
                message: message.clone(),
            };
            let _ = sender.send(entry);
        }

        // Also log via tracing
        match level {
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warning => warn!("{}", message),
            LogLevel::Error => error!("{}", message),
        }
    }
}

/// Execute a workflow run.
pub async fn execute_workflow(
    workflow: &crate::workflow::Workflow,
    context: ExecutionContext,
    log_sender: Option<LogSender>,
) -> Result<HashMap<String, JobExecutionResult>> {
    let executor = JobExecutor::new();
    let mut results = HashMap::new();
    let mut completed_jobs: Vec<String> = Vec::new();
    let mut job_outputs: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Resolve job order
    let job_order =
        crate::job::resolve_job_order(&workflow.jobs).map_err(CiError::CircularDependency)?;

    for job_id in job_order {
        let job_def = match workflow.jobs.get(&job_id) {
            Some(def) => def,
            None => continue,
        };

        // Check if dependencies are satisfied
        let deps_ok = job_def.needs.iter().all(|dep| {
            results
                .get(dep)
                .map(|r: &JobExecutionResult| r.conclusion.is_success())
                .unwrap_or(false)
        });

        if !deps_ok && !job_def.needs.is_empty() {
            // Skip this job - dependencies failed
            let mut job_run = JobRun::new(
                uuid::Uuid::new_v4().to_string(),
                job_id.clone(),
                job_def.display_name(&job_id),
                job_def.steps.len(),
            );
            job_run.status = RunStatus::Completed;
            job_run.conclusion = Some(Conclusion::Skipped);

            results.insert(
                job_id.clone(),
                JobExecutionResult {
                    job_run,
                    conclusion: Conclusion::Skipped,
                    outputs: HashMap::new(),
                },
            );
            continue;
        }

        // Create context for this job
        let mut job_context = context.clone();
        job_context.job_outputs = job_outputs.clone();
        job_context.env.extend(workflow.env.clone());

        let result = executor
            .execute_job(&job_id, job_def, job_context, log_sender.clone())
            .await?;

        // Store outputs for dependent jobs
        job_outputs.insert(job_id.clone(), result.outputs.clone());
        completed_jobs.push(job_id.clone());
        results.insert(job_id, result);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::RunStep;

    #[test]
    fn test_execution_context() {
        let ctx = ExecutionContext::new(
            "alice/repo".to_string(),
            PathBuf::from("/tmp/work"),
            "abc123".to_string(),
        )
        .with_branch(Some("main".to_string()));

        let env = ctx.get_env(&HashMap::new());
        assert_eq!(env.get("GUTS_SHA"), Some(&"abc123".to_string()));
        assert_eq!(env.get("GUTS_BRANCH"), Some(&"main".to_string()));
    }

    #[test]
    fn test_output_parsing() {
        let executor = JobExecutor::new();

        let output =
            "::set-output name=version::1.0.0\nGUTS_OUTPUT_build=success\nsome other output";
        let outputs = executor.parse_output_commands(output);

        assert_eq!(outputs.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(outputs.get("build"), Some(&"success".to_string()));
    }

    #[test]
    fn test_condition_evaluation() {
        let executor = JobExecutor::new();

        assert!(executor.evaluate_condition(None, true));
        assert!(!executor.evaluate_condition(None, false));
        assert!(executor.evaluate_condition(Some("always()"), false));
        assert!(executor.evaluate_condition(Some("success()"), true));
        assert!(!executor.evaluate_condition(Some("success()"), false));
        assert!(executor.evaluate_condition(Some("failure()"), false));
    }

    #[tokio::test]
    async fn test_run_step_execution() {
        let executor = JobExecutor::new();
        let context = ExecutionContext::new(
            "test/repo".to_string(),
            std::env::temp_dir(),
            "abc123".to_string(),
        );

        let step = RunStep {
            name: Some("Echo test".to_string()),
            run: "echo 'hello world'".to_string(),
            working_directory: None,
            env: HashMap::new(),
            shell: None,
            condition: None,
            continue_on_error: false,
            timeout_minutes: None,
            id: None,
        };

        let result = executor
            .execute_run_step(&step, &context, &None, 0)
            .await
            .unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello world"));
    }

    #[tokio::test]
    async fn test_failing_step() {
        let executor = JobExecutor::new();
        let context = ExecutionContext::new(
            "test/repo".to_string(),
            std::env::temp_dir(),
            "abc123".to_string(),
        );

        let step = RunStep {
            name: Some("Failing step".to_string()),
            run: "exit 1".to_string(),
            working_directory: None,
            env: HashMap::new(),
            shell: None,
            condition: None,
            continue_on_error: false,
            timeout_minutes: None,
            id: None,
        };

        let result = executor
            .execute_run_step(&step, &context, &None, 0)
            .await
            .unwrap();
        assert_eq!(result.exit_code, 1);
    }
}
