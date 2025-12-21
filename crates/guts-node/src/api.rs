//! HTTP API for the Guts node.
//!
//! Implements git smart HTTP protocol endpoints for push/pull operations.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use guts_git::{advertise_refs, receive_pack, upload_pack};
use guts_storage::{Reference, Repository};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::p2p::P2PManager;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// Repository store.
    pub repos: Arc<RepoStore>,
    /// Optional P2P manager for replication.
    pub p2p: Option<Arc<P2PManager>>,
}

/// In-memory repository store.
#[derive(Default)]
pub struct RepoStore {
    repos: RwLock<HashMap<String, Arc<Repository>>>,
}

impl RepoStore {
    /// Creates a new empty repository store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new repository.
    pub fn create(&self, owner: &str, name: &str) -> Result<Arc<Repository>, ApiError> {
        let mut repos = self.repos.write();
        let key = format!("{}/{}", owner, name);

        if repos.contains_key(&key) {
            return Err(ApiError::RepoExists(key));
        }

        let repo = Arc::new(Repository::new(name, owner));
        repos.insert(key, repo.clone());
        Ok(repo)
    }

    /// Gets a repository by owner and name.
    pub fn get(&self, owner: &str, name: &str) -> Result<Arc<Repository>, ApiError> {
        let key = format!("{}/{}", owner, name);
        self.repos
            .read()
            .get(&key)
            .cloned()
            .ok_or(ApiError::RepoNotFound(key))
    }

    /// Lists all repositories.
    pub fn list(&self) -> Vec<RepoInfo> {
        self.repos
            .read()
            .values()
            .map(|r| RepoInfo {
                name: r.name.clone(),
                owner: r.owner.clone(),
            })
            .collect()
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
    Storage(#[from] guts_storage::StorageError),
    #[error("bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::RepoNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::RepoExists(_) => (StatusCode::CONFLICT, self.to_string()),
            ApiError::Git(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

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
#[derive(Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub owner: String,
}

/// Creates the API router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Repository management
        .route("/api/repos", get(list_repos).post(create_repo))
        .route("/api/repos/{owner}/{name}", get(get_repo))
        // Git smart HTTP protocol
        .route("/{owner}/{name}.git/info/refs", get(git_info_refs))
        .route("/{owner}/{name}.git/git-upload-pack", post(git_upload_pack))
        .route(
            "/{owner}/{name}.git/git-receive-pack",
            post(git_receive_pack),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Health check endpoint.
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Lists all repositories.
async fn list_repos(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.repos.list())
}

/// Creates a new repository.
async fn create_repo(
    State(state): State<AppState>,
    Json(req): Json<CreateRepoRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = state.repos.create(&req.owner, &req.name)?;

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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header("Cache-Control", "no-cache")
        .body(Body::from(output))
        .unwrap())
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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .body(Body::from(output))
        .unwrap())
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
        Err(ApiError::RepoNotFound(_)) => {
            objects_before = std::collections::HashSet::new();
            state.repos.create(&owner, &name)?
        }
        Err(e) => return Err(e),
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

    // Notify P2P network about the update
    if let Some(p2p) = &state.p2p {
        let repo_key = format!("{}/{}", owner, name);
        p2p.notify_update(&repo_key, new_objects, refs);

        // Also register this repo with the P2P manager
        p2p.register_repo(repo_key, repo.clone());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/x-git-receive-pack-result",
        )
        .body(Body::from(output))
        .unwrap())
}
