//! # CI/CD API
//!
//! This module provides the CI/CD API endpoints for managing workflows, runs,
//! artifacts, and status checks.
//!
//! ## Endpoint Overview
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/repos/{owner}/{name}/workflows` | List workflows |
//! | POST | `/api/repos/{owner}/{name}/workflows` | Create/update workflow |
//! | GET | `/api/repos/{owner}/{name}/workflows/{id}` | Get workflow |
//! | DELETE | `/api/repos/{owner}/{name}/workflows/{id}` | Delete workflow |
//! | GET | `/api/repos/{owner}/{name}/runs` | List runs |
//! | POST | `/api/repos/{owner}/{name}/runs` | Trigger manual run |
//! | GET | `/api/repos/{owner}/{name}/runs/{id}` | Get run details |
//! | POST | `/api/repos/{owner}/{name}/runs/{id}/cancel` | Cancel run |
//! | GET | `/api/repos/{owner}/{name}/runs/{id}/jobs` | List jobs in run |
//! | GET | `/api/repos/{owner}/{name}/runs/{id}/jobs/{job}/logs` | Get job logs |
//! | GET | `/api/repos/{owner}/{name}/runs/{id}/artifacts` | List artifacts |
//! | POST | `/api/repos/{owner}/{name}/runs/{id}/artifacts` | Upload artifact |
//! | GET | `/api/repos/{owner}/{name}/runs/{id}/artifacts/{name}` | Download artifact |
//! | GET | `/api/repos/{owner}/{name}/commits/{sha}/status` | Get combined status |
//! | GET | `/api/repos/{owner}/{name}/commits/{sha}/statuses` | List all statuses |
//! | POST | `/api/repos/{owner}/{name}/commits/{sha}/statuses` | Create status check |

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use guts_ci::{
    Artifact, CheckState, CiStore, StatusCheck, TriggerContext, TriggerType, Workflow,
    WorkflowRun,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// CI/CD error type.
#[derive(Debug, thiserror::Error)]
pub enum CiApiError {
    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),
    #[error("run not found: {0}")]
    RunNotFound(String),
    #[error("artifact not found: {0}")]
    ArtifactNotFound(String),
    #[error("invalid workflow: {0}")]
    InvalidWorkflow(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("ci error: {0}")]
    CiError(#[from] guts_ci::CiError),
}

impl IntoResponse for CiApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            CiApiError::WorkflowNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            CiApiError::RunNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            CiApiError::ArtifactNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            CiApiError::InvalidWorkflow(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            CiApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            CiApiError::CiError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// CI/CD state shared across handlers.
#[derive(Clone)]
pub struct CiState {
    /// CI/CD store.
    pub ci: Arc<CiStore>,
}

// ==================== Request/Response Types ====================

/// Request to create/update a workflow.
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    /// Path to the workflow file (e.g., ".guts/workflows/ci.yml")
    pub path: String,
    /// YAML content of the workflow
    pub content: String,
}

/// Response for workflow info.
#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<&Workflow> for WorkflowResponse {
    fn from(w: &Workflow) -> Self {
        Self {
            id: w.id.clone(),
            name: w.name.clone(),
            path: w.path.clone(),
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}

/// Request to trigger a manual workflow run.
#[derive(Debug, Deserialize)]
pub struct TriggerRunRequest {
    /// Workflow ID to run
    pub workflow_id: String,
    /// Branch or ref to run on
    #[serde(default)]
    pub ref_name: Option<String>,
    /// Input parameters for workflow_dispatch
    #[serde(default)]
    pub inputs: HashMap<String, String>,
}

/// Response for workflow run info.
#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub number: u32,
    pub status: String,
    pub conclusion: Option<String>,
    pub head_sha: String,
    pub head_branch: Option<String>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

impl From<&WorkflowRun> for RunResponse {
    fn from(r: &WorkflowRun) -> Self {
        Self {
            id: r.id.clone(),
            workflow_id: r.workflow_id.clone(),
            workflow_name: r.workflow_name.clone(),
            number: r.number,
            status: format!("{:?}", r.status).to_lowercase(),
            conclusion: r.conclusion.map(|c| format!("{:?}", c).to_lowercase()),
            head_sha: r.head_sha.clone(),
            head_branch: r.head_branch.clone(),
            created_at: r.created_at,
            started_at: r.started_at,
            completed_at: r.completed_at,
        }
    }
}

/// Response for job info within a run.
#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub steps: Vec<StepResponse>,
}

/// Response for step info within a job.
#[derive(Debug, Serialize)]
pub struct StepResponse {
    pub number: u32,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

/// Response for artifact info.
#[derive(Debug, Serialize)]
pub struct ArtifactResponse {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

impl From<&Artifact> for ArtifactResponse {
    fn from(a: &Artifact) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            size_bytes: a.size_bytes,
            content_type: a.content_type.clone(),
            created_at: a.created_at,
            expires_at: a.expires_at,
        }
    }
}

/// Request to create a status check.
#[derive(Debug, Deserialize)]
pub struct CreateStatusRequest {
    /// Context name (e.g., "CI / Build")
    pub context: String,
    /// State: pending, success, failure, error
    pub state: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Optional target URL
    #[serde(default)]
    pub target_url: Option<String>,
}

/// Response for status check info.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub id: String,
    pub context: String,
    pub state: String,
    pub description: Option<String>,
    pub target_url: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<&StatusCheck> for StatusResponse {
    fn from(s: &StatusCheck) -> Self {
        Self {
            id: s.id.clone(),
            context: s.context.clone(),
            state: format!("{:?}", s.state).to_lowercase(),
            description: s.description.clone(),
            target_url: s.target_url.clone(),
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

/// Response for combined status.
#[derive(Debug, Serialize)]
pub struct CombinedStatusResponse {
    pub state: String,
    pub total_count: usize,
    pub statuses: Vec<StatusResponse>,
}

// ==================== Routes ====================

/// Creates the CI/CD routes.
pub fn ci_routes() -> Router<crate::api::AppState> {
    Router::new()
        // Workflows
        .route(
            "/api/repos/{owner}/{name}/workflows",
            get(list_workflows).post(create_workflow),
        )
        .route(
            "/api/repos/{owner}/{name}/workflows/{workflow_id}",
            get(get_workflow).delete(delete_workflow),
        )
        // Runs
        .route(
            "/api/repos/{owner}/{name}/runs",
            get(list_runs).post(trigger_run),
        )
        .route("/api/repos/{owner}/{name}/runs/{run_id}", get(get_run))
        .route(
            "/api/repos/{owner}/{name}/runs/{run_id}/cancel",
            post(cancel_run),
        )
        .route(
            "/api/repos/{owner}/{name}/runs/{run_id}/jobs",
            get(list_jobs),
        )
        .route(
            "/api/repos/{owner}/{name}/runs/{run_id}/jobs/{job_id}/logs",
            get(get_job_logs),
        )
        // Artifacts
        .route(
            "/api/repos/{owner}/{name}/runs/{run_id}/artifacts",
            get(list_artifacts).post(upload_artifact),
        )
        .route(
            "/api/repos/{owner}/{name}/runs/{run_id}/artifacts/{artifact_name}",
            get(download_artifact).delete(delete_artifact),
        )
        // Status checks
        .route(
            "/api/repos/{owner}/{name}/commits/{sha}/status",
            get(get_combined_status),
        )
        .route(
            "/api/repos/{owner}/{name}/commits/{sha}/statuses",
            get(list_statuses).post(create_status),
        )
        // CI stats
        .route("/api/ci/stats", get(get_ci_stats))
}

// ==================== Workflow Handlers ====================

/// List workflows for a repository.
async fn list_workflows(
    State(state): State<crate::api::AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let workflows = state.ci.workflows.list(&repo_key);
    let response: Vec<WorkflowResponse> = workflows.iter().map(WorkflowResponse::from).collect();
    Json(response)
}

/// Create or update a workflow.
async fn create_workflow(
    State(state): State<crate::api::AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse, CiApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let workflow = Workflow::parse(&req.content, &repo_key, &req.path)
        .map_err(|e| CiApiError::InvalidWorkflow(e.to_string()))?;

    state.ci.workflows.store(workflow.clone());

    Ok((StatusCode::CREATED, Json(WorkflowResponse::from(&workflow))))
}

/// Get a specific workflow.
async fn get_workflow(
    State(state): State<crate::api::AppState>,
    Path((owner, name, workflow_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let workflow = state
        .ci
        .workflows
        .get(&repo_key, &workflow_id)
        .ok_or_else(|| CiApiError::WorkflowNotFound(workflow_id))?;

    Ok(Json(WorkflowResponse::from(&workflow)))
}

/// Delete a workflow.
async fn delete_workflow(
    State(state): State<crate::api::AppState>,
    Path((owner, name, workflow_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let repo_key = format!("{}/{}", owner, name);

    state
        .ci
        .workflows
        .delete(&repo_key, &workflow_id)
        .ok_or_else(|| CiApiError::WorkflowNotFound(workflow_id))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Run Handlers ====================

/// List workflow runs.
async fn list_runs(
    State(state): State<crate::api::AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let runs = state.ci.runs.list_by_repo(&repo_key, Some(50));
    let response: Vec<RunResponse> = runs.iter().map(RunResponse::from).collect();
    Json(response)
}

/// Trigger a manual workflow run.
async fn trigger_run(
    State(state): State<crate::api::AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<TriggerRunRequest>,
) -> Result<impl IntoResponse, CiApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let workflow = state
        .ci
        .workflows
        .get(&repo_key, &req.workflow_id)
        .ok_or_else(|| CiApiError::WorkflowNotFound(req.workflow_id.clone()))?;

    if !workflow.allows_manual_trigger() {
        return Err(CiApiError::BadRequest(
            "Workflow does not allow manual trigger".into(),
        ));
    }

    let run_number = state.ci.runs.next_run_number(&repo_key, &workflow.id);

    // Get the head SHA from the ref or use a placeholder
    let head_sha = format!("manual-{}", uuid::Uuid::new_v4());
    let ref_name = req.ref_name.unwrap_or_else(|| "main".to_string());

    let trigger = TriggerContext {
        trigger_type: TriggerType::WorkflowDispatch,
        actor: "api".to_string(),
        ref_name: Some(format!("refs/heads/{}", ref_name)),
        sha: head_sha.clone(),
        base_sha: None,
        pr_number: None,
        inputs: req.inputs,
        event: serde_json::Value::Null,
    };

    let run = WorkflowRun::new(
        uuid::Uuid::new_v4().to_string(),
        workflow.id.clone(),
        workflow.name.clone(),
        repo_key,
        run_number,
        trigger,
        head_sha,
        Some(ref_name),
    );

    state.ci.runs.store(run.clone());

    Ok((StatusCode::CREATED, Json(RunResponse::from(&run))))
}

/// Get a specific run.
async fn get_run(
    State(state): State<crate::api::AppState>,
    Path((_owner, _name, run_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let run = state
        .ci
        .runs
        .get(&run_id)
        .ok_or_else(|| CiApiError::RunNotFound(run_id))?;

    Ok(Json(RunResponse::from(&run)))
}

/// Cancel a run.
async fn cancel_run(
    State(state): State<crate::api::AppState>,
    Path((_owner, _name, run_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let mut run = state
        .ci
        .runs
        .get(&run_id)
        .ok_or_else(|| CiApiError::RunNotFound(run_id.clone()))?;

    if !run.status.is_active() {
        return Err(CiApiError::BadRequest("Run is not active".into()));
    }

    run.cancel();
    state.ci.runs.update(run.clone())?;

    Ok(Json(RunResponse::from(&run)))
}

/// List jobs in a run.
async fn list_jobs(
    State(state): State<crate::api::AppState>,
    Path((_owner, _name, run_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let run = state
        .ci
        .runs
        .get(&run_id)
        .ok_or_else(|| CiApiError::RunNotFound(run_id))?;

    let jobs: Vec<JobResponse> = run
        .jobs
        .values()
        .map(|j| JobResponse {
            id: j.id.clone(),
            name: j.name.clone(),
            status: format!("{:?}", j.status).to_lowercase(),
            conclusion: j.conclusion.map(|c| format!("{:?}", c).to_lowercase()),
            started_at: j.started_at,
            completed_at: j.completed_at,
            steps: j
                .steps
                .iter()
                .map(|s| StepResponse {
                    number: s.number,
                    name: s.name.clone(),
                    status: format!("{:?}", s.status).to_lowercase(),
                    conclusion: s.conclusion.map(|c| format!("{:?}", c).to_lowercase()),
                })
                .collect(),
        })
        .collect();

    Ok(Json(jobs))
}

/// Get job logs.
async fn get_job_logs(
    State(state): State<crate::api::AppState>,
    Path((_owner, _name, run_id, job_id)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let run = state
        .ci
        .runs
        .get(&run_id)
        .ok_or_else(|| CiApiError::RunNotFound(run_id))?;

    let job = run
        .jobs
        .values()
        .find(|j| j.id == job_id || j.job_id == job_id)
        .ok_or_else(|| CiApiError::BadRequest(format!("Job not found: {}", job_id)))?;

    // Return logs as JSON array (clone to avoid lifetime issues)
    Ok(Json(job.logs.clone()))
}

// ==================== Artifact Handlers ====================

/// List artifacts for a run.
async fn list_artifacts(
    State(state): State<crate::api::AppState>,
    Path((_owner, _name, run_id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let artifacts = state.ci.artifacts.list_by_run(&run_id);
    let response: Vec<ArtifactResponse> = artifacts.iter().map(ArtifactResponse::from).collect();
    Json(response)
}

/// Upload an artifact.
async fn upload_artifact(
    State(state): State<crate::api::AppState>,
    Path((owner, name, run_id)): Path<(String, String, String)>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, CiApiError> {
    let repo_key = format!("{}/{}", owner, name);

    // Extract artifact name from Content-Disposition header or generate one
    let artifact_name = format!("artifact-{}.bin", uuid::Uuid::new_v4());

    let artifact = state
        .ci
        .artifacts
        .upload(run_id, repo_key, artifact_name, body.to_vec(), Some(30))?;

    Ok((StatusCode::CREATED, Json(ArtifactResponse::from(&artifact))))
}

/// Download an artifact.
async fn download_artifact(
    State(state): State<crate::api::AppState>,
    Path((_owner, _name, run_id, artifact_name)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let artifact = state
        .ci
        .artifacts
        .get_by_name(&run_id, &artifact_name)
        .map_err(|_| CiApiError::ArtifactNotFound(artifact_name.clone()))?;

    let (_, content) = state
        .ci
        .artifacts
        .download(&artifact.id)
        .map_err(|_| CiApiError::ArtifactNotFound(artifact_name))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, artifact.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", artifact.name),
        )
        .body(Body::from(content.as_ref().clone()))
        .unwrap())
}

/// Delete an artifact.
async fn delete_artifact(
    State(state): State<crate::api::AppState>,
    Path((_owner, _name, run_id, artifact_name)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, CiApiError> {
    let artifact = state
        .ci
        .artifacts
        .get_by_name(&run_id, &artifact_name)
        .map_err(|_| CiApiError::ArtifactNotFound(artifact_name))?;

    state.ci.artifacts.delete(&artifact.id)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Status Check Handlers ====================

/// Get combined status for a commit.
async fn get_combined_status(
    State(state): State<crate::api::AppState>,
    Path((owner, name, sha)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let combined = state.ci.statuses.get_combined_status(&repo_key, &sha);

    Json(CombinedStatusResponse {
        state: format!("{:?}", combined.state).to_lowercase(),
        total_count: combined.total_count,
        statuses: combined.statuses.iter().map(StatusResponse::from).collect(),
    })
}

/// List all statuses for a commit.
async fn list_statuses(
    State(state): State<crate::api::AppState>,
    Path((owner, name, sha)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let statuses = state.ci.statuses.list_for_commit(&repo_key, &sha);
    let response: Vec<StatusResponse> = statuses.iter().map(StatusResponse::from).collect();
    Json(response)
}

/// Create a status check.
async fn create_status(
    State(state): State<crate::api::AppState>,
    Path((owner, name, sha)): Path<(String, String, String)>,
    Json(req): Json<CreateStatusRequest>,
) -> Result<impl IntoResponse, CiApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let check_state = match req.state.to_lowercase().as_str() {
        "pending" => CheckState::Pending,
        "success" => CheckState::Success,
        "failure" => CheckState::Failure,
        "error" => CheckState::Error,
        _ => return Err(CiApiError::BadRequest(format!("Invalid state: {}", req.state))),
    };

    let mut check = StatusCheck::new(repo_key, sha, req.context, check_state);
    if let Some(desc) = req.description {
        check = check.with_description(desc);
    }
    if let Some(url) = req.target_url {
        check = check.with_target_url(url);
    }

    let check = state.ci.statuses.create_or_update(check);

    Ok((StatusCode::CREATED, Json(StatusResponse::from(&check))))
}

/// Get CI stats.
async fn get_ci_stats(State(state): State<crate::api::AppState>) -> impl IntoResponse {
    let stats = state.ci.stats();
    Json(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_response_conversion() {
        let yaml = r#"
name: CI
on: push
jobs:
  test:
    steps:
      - run: echo test
"#;
        let workflow = Workflow::parse(yaml, "alice/repo", ".guts/workflows/ci.yml").unwrap();
        let response = WorkflowResponse::from(&workflow);

        assert_eq!(response.id, "ci");
        assert_eq!(response.name, "CI");
    }

    #[test]
    fn test_status_state_parsing() {
        assert!(matches!(
            match "pending".to_lowercase().as_str() {
                "pending" => CheckState::Pending,
                _ => CheckState::Error,
            },
            CheckState::Pending
        ));
    }
}
