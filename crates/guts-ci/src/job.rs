//! Job definitions for CI/CD workflows.

use crate::step::Step;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for a job definition.
pub type JobId = String;

/// A job within a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobDefinition {
    /// Display name for the job
    #[serde(default)]
    pub name: Option<String>,

    /// Runner label (e.g., "default", "ubuntu-latest")
    #[serde(default = "default_runner", rename = "runs-on")]
    pub runs_on: String,

    /// Dependencies on other jobs (job IDs)
    #[serde(default)]
    pub needs: Vec<String>,

    /// Environment variables for all steps
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Steps to execute
    #[serde(default)]
    pub steps: Vec<Step>,

    /// Timeout in minutes
    #[serde(default = "default_timeout", rename = "timeout-minutes")]
    pub timeout_minutes: u32,

    /// Continue workflow even if this job fails
    #[serde(default, rename = "continue-on-error")]
    pub continue_on_error: bool,

    /// Condition for running this job
    #[serde(default, rename = "if")]
    pub condition: Option<String>,

    /// Strategy for matrix builds (future)
    #[serde(default)]
    pub strategy: Option<JobStrategy>,

    /// Services to run alongside the job (future)
    #[serde(default)]
    pub services: HashMap<String, ServiceDefinition>,

    /// Concurrency settings
    #[serde(default)]
    pub concurrency: Option<ConcurrencySettings>,

    /// Outputs from this job
    #[serde(default)]
    pub outputs: HashMap<String, String>,
}

fn default_runner() -> String {
    "default".to_string()
}

fn default_timeout() -> u32 {
    60 // 60 minutes default
}

impl Default for JobDefinition {
    fn default() -> Self {
        Self {
            name: None,
            runs_on: default_runner(),
            needs: Vec::new(),
            env: HashMap::new(),
            steps: Vec::new(),
            timeout_minutes: default_timeout(),
            continue_on_error: false,
            condition: None,
            strategy: None,
            services: HashMap::new(),
            concurrency: None,
            outputs: HashMap::new(),
        }
    }
}

impl JobDefinition {
    /// Get the display name for this job.
    pub fn display_name(&self, job_id: &str) -> String {
        self.name.clone().unwrap_or_else(|| job_id.to_string())
    }

    /// Validate the job definition.
    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("Job must have at least one step".to_string());
        }

        for (idx, step) in self.steps.iter().enumerate() {
            match step {
                Step::Run(run_step) => {
                    if run_step.run.trim().is_empty() {
                        return Err(format!("Step {} has empty run command", idx + 1));
                    }
                }
                Step::Uses(uses_step) => {
                    if uses_step.uses.trim().is_empty() {
                        return Err(format!("Step {} has empty uses action", idx + 1));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all job IDs this job depends on.
    pub fn dependencies(&self) -> &[String] {
        &self.needs
    }
}

/// Strategy for matrix builds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobStrategy {
    /// Matrix configuration
    #[serde(default)]
    pub matrix: HashMap<String, Vec<serde_json::Value>>,
    /// Fail fast on first failure
    #[serde(default = "default_fail_fast", rename = "fail-fast")]
    pub fail_fast: bool,
    /// Maximum parallel jobs
    #[serde(default, rename = "max-parallel")]
    pub max_parallel: Option<u32>,
}

fn default_fail_fast() -> bool {
    true
}

/// Service container definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceDefinition {
    /// Docker image
    pub image: String,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Port mappings
    #[serde(default)]
    pub ports: Vec<String>,
    /// Volume mappings
    #[serde(default)]
    pub volumes: Vec<String>,
    /// Health check options
    #[serde(default)]
    pub options: Option<String>,
}

/// Concurrency settings for a job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConcurrencySettings {
    /// Concurrency group
    pub group: String,
    /// Cancel in-progress runs
    #[serde(default, rename = "cancel-in-progress")]
    pub cancel_in_progress: bool,
}

/// Resolve job execution order based on dependencies.
pub fn resolve_job_order(jobs: &HashMap<String, JobDefinition>) -> Result<Vec<String>, String> {
    let mut order = Vec::new();
    let mut visited = HashMap::new();
    let mut temp_mark = HashMap::new();

    for job_id in jobs.keys() {
        if !visited.contains_key(job_id) {
            visit_job(job_id, jobs, &mut visited, &mut temp_mark, &mut order)?;
        }
    }

    Ok(order)
}

fn visit_job(
    job_id: &str,
    jobs: &HashMap<String, JobDefinition>,
    visited: &mut HashMap<String, bool>,
    temp_mark: &mut HashMap<String, bool>,
    order: &mut Vec<String>,
) -> Result<(), String> {
    if temp_mark.get(job_id).copied().unwrap_or(false) {
        return Err(format!(
            "Circular dependency detected involving job: {}",
            job_id
        ));
    }

    if !visited.contains_key(job_id) {
        temp_mark.insert(job_id.to_string(), true);

        if let Some(job) = jobs.get(job_id) {
            for dep in &job.needs {
                if !jobs.contains_key(dep) {
                    return Err(format!("Job '{}' depends on unknown job '{}'", job_id, dep));
                }
                visit_job(dep, jobs, visited, temp_mark, order)?;
            }
        }

        temp_mark.insert(job_id.to_string(), false);
        visited.insert(job_id.to_string(), true);
        order.push(job_id.to_string());
    }

    Ok(())
}

/// Get jobs that can run in parallel (have no unmet dependencies).
pub fn get_ready_jobs(
    jobs: &HashMap<String, JobDefinition>,
    completed_jobs: &[String],
) -> Vec<String> {
    jobs.iter()
        .filter(|(id, job)| {
            !completed_jobs.contains(id) && job.needs.iter().all(|dep| completed_jobs.contains(dep))
        })
        .map(|(id, _)| id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_definition_parsing() {
        let yaml = r#"
name: Build
runs-on: default
needs: [setup]
timeout-minutes: 30
steps:
  - name: Build
    run: cargo build
"#;
        let job: JobDefinition = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(job.name, Some("Build".to_string()));
        assert_eq!(job.runs_on, "default");
        assert_eq!(job.needs, vec!["setup"]);
        assert_eq!(job.timeout_minutes, 30);
        assert_eq!(job.steps.len(), 1);
    }

    #[test]
    fn test_job_order_resolution() {
        let mut jobs = HashMap::new();
        jobs.insert(
            "build".to_string(),
            JobDefinition {
                needs: vec!["setup".to_string()],
                ..Default::default()
            },
        );
        jobs.insert("setup".to_string(), JobDefinition::default());
        jobs.insert(
            "test".to_string(),
            JobDefinition {
                needs: vec!["build".to_string()],
                ..Default::default()
            },
        );

        let order = resolve_job_order(&jobs).unwrap();
        let setup_idx = order.iter().position(|j| j == "setup").unwrap();
        let build_idx = order.iter().position(|j| j == "build").unwrap();
        let test_idx = order.iter().position(|j| j == "test").unwrap();

        assert!(setup_idx < build_idx);
        assert!(build_idx < test_idx);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut jobs = HashMap::new();
        jobs.insert(
            "a".to_string(),
            JobDefinition {
                needs: vec!["b".to_string()],
                ..Default::default()
            },
        );
        jobs.insert(
            "b".to_string(),
            JobDefinition {
                needs: vec!["a".to_string()],
                ..Default::default()
            },
        );

        let result = resolve_job_order(&jobs);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[test]
    fn test_get_ready_jobs() {
        let mut jobs = HashMap::new();
        jobs.insert("setup".to_string(), JobDefinition::default());
        jobs.insert(
            "build".to_string(),
            JobDefinition {
                needs: vec!["setup".to_string()],
                ..Default::default()
            },
        );
        jobs.insert(
            "test".to_string(),
            JobDefinition {
                needs: vec!["build".to_string()],
                ..Default::default()
            },
        );

        // Initially only setup can run
        let ready = get_ready_jobs(&jobs, &[]);
        assert_eq!(ready, vec!["setup"]);

        // After setup, build can run
        let ready = get_ready_jobs(&jobs, &["setup".to_string()]);
        assert_eq!(ready, vec!["build"]);

        // After build, test can run
        let ready = get_ready_jobs(&jobs, &["setup".to_string(), "build".to_string()]);
        assert_eq!(ready, vec!["test"]);
    }

    #[test]
    fn test_job_validation() {
        let job = JobDefinition::default();
        assert!(job.validate().is_err());

        let job_with_steps = JobDefinition {
            steps: vec![Step::Run(crate::step::RunStep {
                name: None,
                run: "echo hello".to_string(),
                working_directory: None,
                env: HashMap::new(),
                shell: None,
                condition: None,
                continue_on_error: false,
                timeout_minutes: None,
                id: None,
            })],
            ..Default::default()
        };
        assert!(job_with_steps.validate().is_ok());
    }
}
