//! # Core HTTP API
//!
//! This module provides the main HTTP API for the Guts node, including:
//!
//! - **Git Smart HTTP Protocol**: Standard Git endpoints for clone, push, and pull
//! - **Repository Management**: CRUD operations for repositories
//! - **Health Checks**: Comprehensive liveness, readiness, and startup probes
//! - **Metrics**: Prometheus metrics endpoint
//!
//! ## Endpoint Overview
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/health` | Overall health status |
//! | GET | `/health/live` | Liveness probe |
//! | GET | `/health/ready` | Readiness probe |
//! | GET | `/health/startup` | Startup probe |
//! | GET | `/metrics` | Prometheus metrics |
//! | GET | `/api/repos` | List all repositories |
//! | POST | `/api/repos` | Create a new repository |
//! | GET | `/api/repos/{owner}/{name}` | Get repository details |
//! | GET | `/git/{owner}/{name}/info/refs` | Git reference advertisement |
//! | POST | `/git/{owner}/{name}/git-upload-pack` | Git fetch/clone |
//! | POST | `/git/{owner}/{name}/git-receive-pack` | Git push |
//!
//! ## Git Smart HTTP Protocol
//!
//! The node implements Git's Smart HTTP protocol, enabling standard Git clients
//! to interact with repositories:
//!
//! ```bash
//! # Clone a repository
//! git clone http://localhost:8080/git/alice/myrepo
//!
//! # Push changes
//! git push origin main
//!
//! # Fetch updates
//! git fetch origin
//! ```
//!
//! ## Application State
//!
//! All handlers share an [`AppState`] containing:
//!
//! - `repos`: Repository storage (Git objects and refs)
//! - `collaboration`: Pull requests, issues, comments storage
//! - `auth`: Organizations, teams, permissions storage
//! - `p2p`: Optional P2P manager for network replication
//! - `realtime`: Event hub for WebSocket real-time updates
//!
//! ## Error Handling
//!
//! Errors are returned as JSON with appropriate HTTP status codes:
//!
//! ```json
//! {
//!   "error": "repository not found: alice/myrepo"
//! }
//! ```
//!
//! | Status | Meaning |
//! |--------|---------|
//! | 404 | Repository not found |
//! | 409 | Repository already exists |
//! | 422 | Validation error |
//! | 500 | Internal server error |

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use guts_auth::AuthStore;
use guts_collaboration::CollaborationStore;
use guts_git::{advertise_refs, receive_pack, upload_pack};
use guts_realtime::{EventHub, EventKind};
use guts_storage::{Reference, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use validator::Validate;

use crate::auth_api::auth_routes;
use crate::ci_api::ci_routes;
use crate::collaboration_api::collaboration_routes;
use crate::compat_api::compat_routes;
use crate::consensus_api::consensus_routes;
use crate::health::{health_routes, HealthState};
use crate::observability::middleware::{
    metrics_handler, metrics_middleware, request_id_middleware,
};
use crate::p2p::P2PManager;
use crate::realtime_api::realtime_routes;
use crate::validation::validate_name;
use guts_ci::CiStore;
use guts_compat::CompatStore;
use guts_consensus::{ConsensusEngine, Mempool};

/// Re-export RepoStore for external use.
pub use guts_storage::RepoStore;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// Repository store.
    pub repos: Arc<RepoStore>,
    /// Optional P2P manager for replication.
    pub p2p: Option<Arc<P2PManager>>,
    /// Optional consensus engine for BFT consensus.
    pub consensus: Option<Arc<ConsensusEngine>>,
    /// Optional mempool for pending transactions.
    pub mempool: Option<Arc<Mempool>>,
    /// Collaboration store for PRs, Issues, etc.
    pub collaboration: Arc<CollaborationStore>,
    /// Authorization store for permissions, organizations, etc.
    pub auth: Arc<AuthStore>,
    /// Real-time event hub for WebSocket connections.
    pub realtime: Arc<EventHub>,
    /// CI/CD store for workflows, runs, artifacts, and status checks.
    pub ci: Arc<CiStore>,
    /// Compatibility store for users, tokens, SSH keys, releases.
    pub compat: Arc<CompatStore>,
}

impl axum::extract::FromRef<AppState> for guts_web::WebState {
    fn from_ref(app_state: &AppState) -> Self {
        guts_web::WebState {
            repos: app_state.repos.clone(),
            collaboration: app_state.collaboration.clone(),
            auth: app_state.auth.clone(),
            ci: app_state.ci.clone(),
            consensus: app_state.consensus.clone(),
            mempool: app_state.mempool.clone(),
        }
    }
}

/// API error type.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("repository not found: {0}")]
    RepoNotFound(String),
    #[error("repository already exists: {0}")]
    RepoExists(String),
    #[error("git error: {0}")]
    Git(#[from] guts_git::GitError),
    #[error("storage error: {0}")]
    Storage(StorageError),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("validation error: {0}")]
    Validation(String),
}

impl From<StorageError> for ApiError {
    fn from(err: StorageError) -> Self {
        match &err {
            StorageError::RepoNotFound(key) => ApiError::RepoNotFound(key.clone()),
            StorageError::RepoExists(key) => ApiError::RepoExists(key.clone()),
            _ => ApiError::Storage(err),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::RepoNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::RepoExists(_) => (StatusCode::CONFLICT, self.to_string()),
            ApiError::Git(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
        };

        tracing::warn!(
            error_type = %std::any::type_name::<Self>(),
            error = %message,
            status = %status.as_u16(),
            "API error"
        );

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Repository info for listing.
#[derive(Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub owner: String,
}

/// Request to create a repository.
#[derive(Deserialize, Validate)]
pub struct CreateRepoRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub owner: String,
}

/// Creates the API router with health state.
pub fn create_router(state: AppState, health_state: HealthState) -> Router {
    Router::new()
        // Metrics endpoint (on main router for now)
        .route("/metrics", get(metrics_handler))
        // Repository management
        .route("/api/repos", get(list_repos).post(create_repo))
        .route("/api/repos/{owner}/{name}", get(get_repo))
        // Git smart HTTP protocol (using /git/ prefix to avoid axum path parameter limitations)
        .route("/git/{owner}/{name}/info/refs", get(git_info_refs))
        .route("/git/{owner}/{name}/git-upload-pack", post(git_upload_pack))
        .route(
            "/git/{owner}/{name}/git-receive-pack",
            post(git_receive_pack),
        )
        // Collaboration API (PRs, Issues, Comments, Reviews)
        .merge(collaboration_routes())
        // Authorization API (Organizations, Teams, Permissions, Webhooks)
        .merge(auth_routes())
        // CI/CD API (Workflows, Runs, Artifacts, Status Checks)
        .merge(ci_routes())
        // Compatibility API (Users, Tokens, SSH Keys, Releases, Contents, Archives)
        .merge(compat_routes())
        // Consensus API (Blocks, Validators, Transactions)
        .merge(consensus_routes())
        // Real-time WebSocket API
        .merge(realtime_routes())
        // Health check routes
        .merge(health_routes(health_state))
        // Web UI routes
        .merge(guts_web::web_routes())
        // Observability layers
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Lists all repositories.
async fn list_repos(State(state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("Listing repositories");
    let repos: Vec<RepoInfo> = state
        .repos
        .list()
        .into_iter()
        .map(|r| RepoInfo {
            name: r.name.clone(),
            owner: r.owner.clone(),
        })
        .collect();
    Json(repos)
}

/// Creates a new repository.
async fn create_repo(
    State(state): State<AppState>,
    Json(req): Json<CreateRepoRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(ApiError::Validation(e.to_string()));
    }

    // Validate name format
    if let Err(e) = validate_name(&req.name) {
        return Err(ApiError::Validation(format!(
            "Invalid repository name: {}",
            e.message.unwrap_or_default()
        )));
    }

    if let Err(e) = validate_name(&req.owner) {
        return Err(ApiError::Validation(format!(
            "Invalid owner name: {}",
            e.message.unwrap_or_default()
        )));
    }

    tracing::info!(
        owner = %req.owner,
        name = %req.name,
        "Creating repository"
    );

    let repo = state.repos.create(&req.name, &req.owner)?;

    Ok((
        StatusCode::CREATED,
        Json(RepoInfo {
            name: repo.name.clone(),
            owner: repo.owner.clone(),
        }),
    ))
}

/// Gets repository info.
async fn get_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::debug!(owner = %owner, name = %name, "Getting repository");
    let repo = state.repos.get(&owner, &name)?;

    Ok(Json(RepoInfo {
        name: repo.name.clone(),
        owner: repo.owner.clone(),
    }))
}

/// Git info/refs endpoint - advertises references.
async fn git_info_refs(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let repo = state.repos.get(&owner, &name)?;
    let service = params.get("service").cloned().unwrap_or_default();

    let mut output = Vec::new();
    advertise_refs(&mut output, &repo, &service)?;

    let content_type = format!("application/x-{}-advertisement", service);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header("Cache-Control", "no-cache")
        .body(Body::from(output))
        .map_err(|e| ApiError::BadRequest(format!("Failed to build response: {}", e)))
}

/// Git upload-pack endpoint - handles fetch/clone.
async fn git_upload_pack(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Result<Response, ApiError> {
    let repo = state.repos.get(&owner, &name)?;

    let mut input = Cursor::new(body.to_vec());
    let mut output = Vec::new();

    upload_pack(&mut input, &mut output, &repo)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .body(Body::from(output))
        .map_err(|e| ApiError::BadRequest(format!("Failed to build response: {}", e)))
}

/// Git receive-pack endpoint - handles push.
async fn git_receive_pack(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Result<Response, ApiError> {
    // Track objects before push
    let objects_before: std::collections::HashSet<_>;

    // Auto-create repository if it doesn't exist (for initial push)
    let repo = match state.repos.get(&owner, &name) {
        Ok(repo) => {
            objects_before = repo.objects.list_objects().into_iter().collect();
            repo
        }
        Err(StorageError::RepoNotFound(_)) => {
            objects_before = std::collections::HashSet::new();
            state.repos.create(&name, &owner)?
        }
        Err(e) => return Err(e.into()),
    };

    let mut input = Cursor::new(body.to_vec());
    let mut output = Vec::new();

    receive_pack(&mut input, &mut output, &repo)?;

    // Calculate new objects
    let objects_after: std::collections::HashSet<_> =
        repo.objects.list_objects().into_iter().collect();
    let new_objects: Vec<_> = objects_after.difference(&objects_before).copied().collect();

    // Get current refs
    let refs: Vec<_> = repo
        .refs
        .list_all()
        .into_iter()
        .filter_map(|(name, reference)| match reference {
            Reference::Direct(oid) => Some((name, oid)),
            Reference::Symbolic(_) => None,
        })
        .collect();

    tracing::info!(
        owner = %owner,
        name = %name,
        objects = repo.objects.len(),
        new_objects = new_objects.len(),
        "Push completed"
    );

    let repo_key = format!("{}/{}", owner, name);

    // Notify P2P network about the update
    if let Some(p2p) = &state.p2p {
        p2p.notify_update(&repo_key, new_objects.clone(), refs.clone());

        // Also register this repo with the P2P manager
        p2p.register_repo(repo_key.clone(), repo.clone());
    }

    // Emit real-time event for WebSocket clients
    let channel = format!("repo:{}", repo_key);
    state.realtime.emit_event(
        channel,
        EventKind::Push,
        serde_json::json!({
            "repository": repo_key,
            "owner": owner,
            "name": name,
            "commit_count": new_objects.len(),
            "refs": refs.iter().map(|(name, oid)| {
                serde_json::json!({
                    "name": name,
                    "sha": format!("{:?}", oid)
                })
            }).collect::<Vec<_>>()
        }),
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/x-git-receive-pack-result",
        )
        .body(Body::from(output))
        .map_err(|e| ApiError::BadRequest(format!("Failed to build response: {}", e)))
}
