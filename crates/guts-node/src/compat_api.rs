//! # Compatibility API
//!
//! This module provides GitHub-compatible API endpoints for:
//!
//! - **User Accounts**: User registration and profile management
//! - **Personal Access Tokens**: Token-based authentication
//! - **SSH Keys**: SSH key management
//! - **Releases**: Release and asset management
//! - **Contents**: Repository file browsing
//! - **Archives**: Tarball and zipball downloads
//! - **Rate Limits**: Rate limit status endpoint
//!
//! ## User Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/users` | Create user account |
//! | GET | `/api/users` | List users |
//! | GET | `/api/users/{username}` | Get user profile |
//! | PATCH | `/api/users/{username}` | Update user profile |
//! | GET | `/api/user` | Get authenticated user |
//! | PATCH | `/api/user` | Update authenticated user |
//!
//! ## Token Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/user/tokens` | Create personal access token |
//! | GET | `/api/user/tokens` | List tokens |
//! | DELETE | `/api/user/tokens/{id}` | Revoke token |
//!
//! ## SSH Key Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/user/keys` | Add SSH key |
//! | GET | `/api/user/keys` | List SSH keys |
//! | GET | `/api/user/keys/{id}` | Get SSH key |
//! | DELETE | `/api/user/keys/{id}` | Remove SSH key |
//!
//! ## Release Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/repos/{owner}/{name}/releases` | Create release |
//! | GET | `/api/repos/{owner}/{name}/releases` | List releases |
//! | GET | `/api/repos/{owner}/{name}/releases/latest` | Get latest release |
//! | GET | `/api/repos/{owner}/{name}/releases/tags/{tag}` | Get by tag |
//! | GET | `/api/repos/{owner}/{name}/releases/{id}` | Get release |
//! | PATCH | `/api/repos/{owner}/{name}/releases/{id}` | Update release |
//! | DELETE | `/api/repos/{owner}/{name}/releases/{id}` | Delete release |
//!
//! ## Contents Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/repos/{owner}/{name}/contents` | Get root contents |
//! | GET | `/api/repos/{owner}/{name}/contents/*path` | Get file/directory |
//! | GET | `/api/repos/{owner}/{name}/readme` | Get README |
//!
//! ## Archive Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/repos/{owner}/{name}/tarball/{ref}` | Download tarball |
//! | GET | `/api/repos/{owner}/{name}/zipball/{ref}` | Download zipball |
//!
//! ## Rate Limit Endpoint
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/rate_limit` | Get rate limit status |

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use guts_compat::{
    base64_encode, create_archive, is_readme_file, paginate, AddSshKeyRequest, ArchiveEntry,
    ArchiveFormat, CompatError, CompatStore, ContentEntry, ContentType, CreateReleaseRequest,
    CreateTokenRequest, CreateUserRequest, PaginationParams, UpdateReleaseRequest,
    UpdateUserRequest, User,
};
use guts_storage::{GitObject, ObjectId, ObjectType, Reference, Repository};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

/// Creates the compatibility API routes.
pub fn compat_routes() -> Router<AppState> {
    Router::new()
        // User endpoints
        .route("/api/users", get(list_users).post(create_user))
        .route(
            "/api/users/{username}",
            get(get_user).patch(update_user_by_name),
        )
        .route(
            "/api/user",
            get(get_current_user).patch(update_current_user),
        )
        // Token endpoints
        .route("/api/user/tokens", get(list_tokens).post(create_token))
        .route("/api/user/tokens/{id}", get(get_token).delete(revoke_token))
        // SSH key endpoints
        .route("/api/user/keys", get(list_ssh_keys).post(add_ssh_key))
        .route(
            "/api/user/keys/{id}",
            get(get_ssh_key).delete(remove_ssh_key),
        )
        // Release endpoints
        .route(
            "/api/repos/{owner}/{name}/releases",
            get(list_releases).post(create_release),
        )
        .route(
            "/api/repos/{owner}/{name}/releases/latest",
            get(get_latest_release),
        )
        .route(
            "/api/repos/{owner}/{name}/releases/tags/{tag}",
            get(get_release_by_tag),
        )
        .route(
            "/api/repos/{owner}/{name}/releases/{id}",
            get(get_release)
                .patch(update_release)
                .delete(delete_release),
        )
        // Contents endpoints
        .route("/api/repos/{owner}/{name}/contents", get(get_contents_root))
        .route(
            "/api/repos/{owner}/{name}/contents/{*path}",
            get(get_contents),
        )
        .route("/api/repos/{owner}/{name}/readme", get(get_readme))
        // Archive endpoints
        .route("/api/repos/{owner}/{name}/tarball/{ref}", get(get_tarball))
        .route("/api/repos/{owner}/{name}/zipball/{ref}", get(get_zipball))
        // Rate limit endpoint
        .route("/api/rate_limit", get(get_rate_limit))
}

// ==================== Error Handling ====================

/// Wrapper for compat errors.
struct CompatApiError(CompatError);

impl From<CompatError> for CompatApiError {
    fn from(err: CompatError) -> Self {
        Self(err)
    }
}

impl IntoResponse for CompatApiError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let message = self.0.github_message();

        (
            status,
            Json(ErrorResponse {
                message: message.to_string(),
                documentation_url: None,
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    documentation_url: Option<String>,
}

// ==================== Helper Functions ====================

/// Extract user from X-Guts-Identity header (temporary until token auth is fully integrated).
fn get_identity_from_header(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("X-Guts-Identity")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Get user ID from identity header.
fn get_user_from_identity(compat: &CompatStore, identity: &str) -> Option<User> {
    compat
        .users
        .get_by_username(identity)
        .or_else(|| compat.users.get_by_public_key(identity))
}

// ==================== User Handlers ====================

/// Lists all users.
async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let users = state.compat.users.list();
    let profiles: Vec<_> = users.iter().map(|u| u.to_profile(0, 0, 0)).collect();
    let response = paginate(&profiles, &params);
    Json(response.items)
}

/// Creates a new user.
async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, CompatApiError> {
    let mut user = state.compat.users.create(req.username, req.public_key)?;

    if let Some(email) = req.email {
        user.email = Some(email);
    }
    if let Some(name) = req.name {
        user.display_name = Some(name);
    }
    if user.email.is_some() || user.display_name.is_some() {
        user = state.compat.users.update(user)?;
    }

    Ok((StatusCode::CREATED, Json(user.to_profile(0, 0, 0))))
}

/// Gets a user by username.
async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, CompatApiError> {
    let user = state
        .compat
        .users
        .get_by_username(&username)
        .ok_or(CompatError::UserNotFound(username))?;

    // Count repos owned by user
    let repos = state.repos.list();
    let repo_count = repos.iter().filter(|r| r.owner == user.username).count() as u64;

    Ok(Json(user.to_profile(repo_count, 0, 0)))
}

/// Updates a user by username.
async fn update_user_by_name(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, CompatApiError> {
    let mut user = state
        .compat
        .users
        .get_by_username(&username)
        .ok_or(CompatError::UserNotFound(username))?;

    if let Some(name) = req.name {
        user.display_name = Some(name);
    }
    if let Some(email) = req.email {
        user.email = Some(email);
    }
    if let Some(bio) = req.bio {
        user.bio = Some(bio);
    }
    if let Some(location) = req.location {
        user.location = Some(location);
    }
    if let Some(blog) = req.blog {
        user.website = Some(blog);
    }
    if let Some(email_public) = req.email_public {
        user.email_public = email_public;
    }

    user.touch();
    let updated = state.compat.users.update(user)?;

    Ok(Json(updated.to_profile(0, 0, 0)))
}

/// Gets the current authenticated user.
async fn get_current_user(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    let repos = state.repos.list();
    let repo_count = repos.iter().filter(|r| r.owner == user.username).count() as u64;

    Ok(Json(user.to_profile(repo_count, 0, 0)))
}

/// Updates the current authenticated user.
async fn update_current_user(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let mut user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    if let Some(name) = req.name {
        user.display_name = Some(name);
    }
    if let Some(email) = req.email {
        user.email = Some(email);
    }
    if let Some(bio) = req.bio {
        user.bio = Some(bio);
    }
    if let Some(location) = req.location {
        user.location = Some(location);
    }
    if let Some(blog) = req.blog {
        user.website = Some(blog);
    }
    if let Some(email_public) = req.email_public {
        user.email_public = email_public;
    }

    user.touch();
    let updated = state.compat.users.update(user)?;

    Ok(Json(updated.to_profile(0, 0, 0)))
}

// ==================== Token Handlers ====================

/// Lists tokens for the authenticated user.
async fn list_tokens(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    let tokens = state.compat.tokens.list_for_user(user.id);
    let responses: Vec<_> = tokens.iter().map(|t| t.to_response(None)).collect();

    Ok(Json(responses))
}

/// Creates a new personal access token.
async fn create_token(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    let expires_at = req.expires_in_days.map(|days| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + (days as u64 * 86400)
    });

    let (token, plaintext) = state
        .compat
        .tokens
        .create(user.id, req.name, req.scopes, expires_at)?;

    Ok((
        StatusCode::CREATED,
        Json(token.to_response(Some(&plaintext))),
    ))
}

/// Gets a token by ID.
async fn get_token(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    let token = state
        .compat
        .tokens
        .get(id)
        .filter(|t| t.user_id == user.id)
        .ok_or(CompatError::TokenNotFound)?;

    Ok(Json(token.to_response(None)))
}

/// Revokes a token.
async fn revoke_token(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    // Verify token belongs to user
    let token = state
        .compat
        .tokens
        .get(id)
        .filter(|t| t.user_id == user.id)
        .ok_or(CompatError::TokenNotFound)?;

    state.compat.tokens.revoke(token.id)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== SSH Key Handlers ====================

/// Lists SSH keys for the authenticated user.
async fn list_ssh_keys(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    let keys = state.compat.ssh_keys.list_for_user(user.id);
    let responses: Vec<_> = keys.iter().map(|k| k.to_response()).collect();

    Ok(Json(responses))
}

/// Adds an SSH key.
async fn add_ssh_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<AddSshKeyRequest>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    let key = state.compat.ssh_keys.add(user.id, req.title, req.key)?;

    Ok((StatusCode::CREATED, Json(key.to_response())))
}

/// Gets an SSH key by ID.
async fn get_ssh_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    let key = state
        .compat
        .ssh_keys
        .get(id)
        .filter(|k| k.user_id == user.id)
        .ok_or(CompatError::SshKeyNotFound)?;

    Ok(Json(key.to_response()))
}

/// Removes an SSH key.
async fn remove_ssh_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).ok_or(CompatError::TokenNotFound)?;

    let user = get_user_from_identity(&state.compat, &identity)
        .ok_or(CompatError::UserNotFound(identity))?;

    // Verify key belongs to user
    let key = state
        .compat
        .ssh_keys
        .get(id)
        .filter(|k| k.user_id == user.id)
        .ok_or(CompatError::SshKeyNotFound)?;

    state.compat.ssh_keys.remove(key.id)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Release Handlers ====================

/// Lists releases for a repository.
async fn list_releases(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let releases = state.compat.releases.list(&repo_key);
    let responses: Vec<_> = releases.iter().map(|r| r.to_response()).collect();
    let paginated = paginate(&responses, &params);
    Json(paginated.items)
}

/// Creates a new release.
async fn create_release(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateReleaseRequest>,
) -> Result<impl IntoResponse, CompatApiError> {
    let identity = get_identity_from_header(&headers).unwrap_or_else(|| "anonymous".to_string());

    let repo_key = format!("{}/{}", owner, name);
    let target = req.target_commitish.unwrap_or_else(|| "main".to_string());

    let mut release = state
        .compat
        .releases
        .create(repo_key, req.tag_name, target, identity)?;

    release.name = req.name;
    release.body = req.body;
    release.draft = req.draft;
    release.prerelease = req.prerelease;

    if release.draft {
        release.published_at = None;
    }

    let updated = state.compat.releases.update(release)?;

    Ok((StatusCode::CREATED, Json(updated.to_response())))
}

/// Gets the latest release.
async fn get_latest_release(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Result<impl IntoResponse, CompatApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let release = state
        .compat
        .releases
        .get_latest(&repo_key)
        .ok_or_else(|| CompatError::ReleaseNotFound("latest".to_string()))?;

    Ok(Json(release.to_response()))
}

/// Gets a release by tag.
async fn get_release_by_tag(
    State(state): State<AppState>,
    Path((owner, name, tag)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, CompatApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let release = state
        .compat
        .releases
        .get_by_tag(&repo_key, &tag)
        .ok_or(CompatError::ReleaseNotFound(tag))?;

    Ok(Json(release.to_response()))
}

/// Gets a release by ID.
async fn get_release(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, u64)>,
) -> Result<impl IntoResponse, CompatApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let release = state
        .compat
        .releases
        .get(id)
        .filter(|r| r.repo_key == repo_key)
        .ok_or_else(|| CompatError::ReleaseNotFound(id.to_string()))?;

    Ok(Json(release.to_response()))
}

/// Updates a release.
async fn update_release(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, u64)>,
    Json(req): Json<UpdateReleaseRequest>,
) -> Result<impl IntoResponse, CompatApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let mut release = state
        .compat
        .releases
        .get(id)
        .filter(|r| r.repo_key == repo_key)
        .ok_or_else(|| CompatError::ReleaseNotFound(id.to_string()))?;

    if let Some(tag) = req.tag_name {
        release.tag_name = tag;
    }
    if let Some(target) = req.target_commitish {
        release.target_commitish = target;
    }
    if let Some(name) = req.name {
        release.name = Some(name);
    }
    if let Some(body) = req.body {
        release.body = Some(body);
    }
    if let Some(draft) = req.draft {
        release.draft = draft;
        if !draft && release.published_at.is_none() {
            release.publish();
        }
    }
    if let Some(prerelease) = req.prerelease {
        release.prerelease = prerelease;
    }

    let updated = state.compat.releases.update(release)?;

    Ok(Json(updated.to_response()))
}

/// Deletes a release.
async fn delete_release(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, u64)>,
) -> Result<impl IntoResponse, CompatApiError> {
    let repo_key = format!("{}/{}", owner, name);

    // Verify release belongs to this repo
    let _ = state
        .compat
        .releases
        .get(id)
        .filter(|r| r.repo_key == repo_key)
        .ok_or_else(|| CompatError::ReleaseNotFound(id.to_string()))?;

    state.compat.releases.delete(id)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Contents Handlers ====================

#[derive(Deserialize)]
struct ContentsQuery {
    #[serde(rename = "ref")]
    git_ref: Option<String>,
}

/// Gets root contents of a repository.
async fn get_contents_root(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<impl IntoResponse, CompatApiError> {
    get_contents_internal(&state, &owner, &name, "", query.git_ref.as_deref()).await
}

/// Gets contents at a specific path.
async fn get_contents(
    State(state): State<AppState>,
    Path((owner, name, path)): Path<(String, String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<impl IntoResponse, CompatApiError> {
    get_contents_internal(&state, &owner, &name, &path, query.git_ref.as_deref()).await
}

/// Internal function to get contents.
async fn get_contents_internal(
    state: &AppState,
    owner: &str,
    name: &str,
    path: &str,
    git_ref: Option<&str>,
) -> Result<impl IntoResponse, CompatApiError> {
    let repo = state
        .repos
        .get(owner, name)
        .map_err(|_| CompatError::PathNotFound(format!("{}/{}", owner, name)))?;

    // Resolve ref to commit
    let ref_name = git_ref.unwrap_or("HEAD");
    let commit_sha = resolve_ref(&repo, ref_name)
        .ok_or_else(|| CompatError::InvalidRef(ref_name.to_string()))?;

    // Get commit and tree
    let commit = repo
        .objects
        .get(&commit_sha)
        .map_err(|_| CompatError::InvalidRef(ref_name.to_string()))?;

    let tree_sha =
        parse_commit_tree(&commit).ok_or_else(|| CompatError::InvalidRef(ref_name.to_string()))?;

    // Navigate to the requested path
    let entries = if path.is_empty() {
        // Root directory
        let tree = get_tree(&repo, &tree_sha)
            .ok_or_else(|| CompatError::PathNotFound("root".to_string()))?;

        tree.iter()
            .map(|e| {
                let content_type = match e.mode {
                    0o040000 => ContentType::Dir,
                    0o120000 => ContentType::Symlink,
                    0o160000 => ContentType::Submodule,
                    _ => ContentType::File,
                };

                let mut entry = match content_type {
                    ContentType::Dir => {
                        ContentEntry::dir(e.name.clone(), e.name.clone(), e.oid.to_hex())
                    }
                    ContentType::File => {
                        let size = get_blob(&repo, &e.oid).map(|b| b.len()).unwrap_or(0) as u64;
                        ContentEntry::file(e.name.clone(), e.name.clone(), e.oid.to_hex(), size)
                    }
                    _ => ContentEntry::file(e.name.clone(), e.name.clone(), e.oid.to_hex(), 0),
                };
                entry.content_type = content_type;
                entry
            })
            .collect()
    } else {
        // Navigate path
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let mut current_tree_sha = tree_sha;

        for (i, part) in parts.iter().enumerate() {
            let tree = get_tree(&repo, &current_tree_sha)
                .ok_or_else(|| CompatError::PathNotFound(path.to_string()))?;

            let entry = tree
                .iter()
                .find(|e| e.name == *part)
                .ok_or_else(|| CompatError::PathNotFound(path.to_string()))?;

            if i == parts.len() - 1 {
                // Last part - could be file or directory
                if entry.mode == 0o040000 {
                    // Directory
                    let tree = get_tree(&repo, &entry.oid)
                        .ok_or_else(|| CompatError::PathNotFound(path.to_string()))?;

                    let entries: Vec<ContentEntry> = tree
                        .iter()
                        .map(|e| {
                            let full_path = format!("{}/{}", path, e.name);
                            let content_type = match e.mode {
                                0o040000 => ContentType::Dir,
                                0o120000 => ContentType::Symlink,
                                0o160000 => ContentType::Submodule,
                                _ => ContentType::File,
                            };

                            let mut entry = match content_type {
                                ContentType::Dir => {
                                    ContentEntry::dir(e.name.clone(), full_path, e.oid.to_hex())
                                }
                                ContentType::File => {
                                    let size = get_blob(&repo, &e.oid).map(|b| b.len()).unwrap_or(0)
                                        as u64;
                                    ContentEntry::file(
                                        e.name.clone(),
                                        full_path,
                                        e.oid.to_hex(),
                                        size,
                                    )
                                }
                                _ => {
                                    ContentEntry::file(e.name.clone(), full_path, e.oid.to_hex(), 0)
                                }
                            };
                            entry.content_type = content_type;
                            entry
                        })
                        .collect();

                    return Ok(Json(serde_json::to_value(entries).unwrap()));
                } else {
                    // File
                    let blob = get_blob(&repo, &entry.oid)
                        .ok_or_else(|| CompatError::PathNotFound(path.to_string()))?;

                    let content = base64_encode(&blob);
                    let file_entry = ContentEntry::file(
                        entry.name.clone(),
                        path.to_string(),
                        entry.oid.to_hex(),
                        blob.len() as u64,
                    )
                    .with_content(content);

                    return Ok(Json(serde_json::to_value(file_entry).unwrap()));
                }
            } else {
                // Not last part - must be directory
                if entry.mode != 0o040000 {
                    return Err(CompatError::PathNotFound(path.to_string()).into());
                }
                current_tree_sha = entry.oid;
            }
        }

        Vec::new()
    };

    Ok(Json(serde_json::to_value(entries).unwrap()))
}

/// Gets the README file.
async fn get_readme(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<impl IntoResponse, CompatApiError> {
    let repo = state
        .repos
        .get(&owner, &name)
        .map_err(|_| CompatError::PathNotFound(format!("{}/{}", owner, name)))?;

    // Resolve ref
    let ref_name = query.git_ref.as_deref().unwrap_or("HEAD");
    let commit_sha = resolve_ref(&repo, ref_name)
        .ok_or_else(|| CompatError::InvalidRef(ref_name.to_string()))?;

    let commit = repo
        .objects
        .get(&commit_sha)
        .map_err(|_| CompatError::InvalidRef(ref_name.to_string()))?;

    let tree_sha =
        parse_commit_tree(&commit).ok_or_else(|| CompatError::InvalidRef(ref_name.to_string()))?;

    let tree =
        get_tree(&repo, &tree_sha).ok_or_else(|| CompatError::PathNotFound("root".to_string()))?;

    // Find README file
    let readme_entry = tree
        .iter()
        .find(|e| is_readme_file(&e.name))
        .ok_or_else(|| CompatError::PathNotFound("README".to_string()))?;

    let blob = get_blob(&repo, &readme_entry.oid)
        .ok_or_else(|| CompatError::PathNotFound("README".to_string()))?;

    let content = base64_encode(&blob);
    let entry = ContentEntry::file(
        readme_entry.name.clone(),
        readme_entry.name.clone(),
        readme_entry.oid.to_hex(),
        blob.len() as u64,
    )
    .with_content(content);

    Ok(Json(entry))
}

// ==================== Archive Handlers ====================

/// Downloads a tarball.
async fn get_tarball(
    State(state): State<AppState>,
    Path((owner, name, git_ref)): Path<(String, String, String)>,
) -> Result<Response, CompatApiError> {
    get_archive(&state, &owner, &name, &git_ref, ArchiveFormat::TarGz).await
}

/// Downloads a zipball.
async fn get_zipball(
    State(state): State<AppState>,
    Path((owner, name, git_ref)): Path<(String, String, String)>,
) -> Result<Response, CompatApiError> {
    get_archive(&state, &owner, &name, &git_ref, ArchiveFormat::Zip).await
}

/// Internal function to generate an archive.
async fn get_archive(
    state: &AppState,
    owner: &str,
    name: &str,
    git_ref: &str,
    format: ArchiveFormat,
) -> Result<Response, CompatApiError> {
    let repo = state
        .repos
        .get(owner, name)
        .map_err(|_| CompatError::PathNotFound(format!("{}/{}", owner, name)))?;

    // Resolve ref
    let commit_sha =
        resolve_ref(&repo, git_ref).ok_or_else(|| CompatError::InvalidRef(git_ref.to_string()))?;

    let commit = repo
        .objects
        .get(&commit_sha)
        .map_err(|_| CompatError::InvalidRef(git_ref.to_string()))?;

    let tree_sha =
        parse_commit_tree(&commit).ok_or_else(|| CompatError::InvalidRef(git_ref.to_string()))?;

    // Collect all files
    let mut entries = Vec::new();
    collect_tree_entries(&repo, tree_sha, "", &mut entries);

    // Create archive
    let prefix = format!("{}-{}", name, git_ref.replace('/', "-"));
    let archive = create_archive(format, prefix.clone(), entries)
        .map_err(|e| CompatError::ArchiveFailed(e.to_string()))?;

    let filename = format.filename(name, git_ref);
    let content_type = format.content_type();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(archive))
        .unwrap())
}

/// Recursively collect tree entries for archive.
fn collect_tree_entries(
    repo: &Repository,
    tree_sha: ObjectId,
    prefix: &str,
    entries: &mut Vec<ArchiveEntry>,
) {
    if let Some(tree) = get_tree(repo, &tree_sha) {
        for entry in &tree {
            let path = if prefix.is_empty() {
                entry.name.clone()
            } else {
                format!("{}/{}", prefix, entry.name)
            };

            if entry.mode == 0o040000 {
                // Directory - recurse
                collect_tree_entries(repo, entry.oid, &path, entries);
            } else if let Some(blob) = get_blob(repo, &entry.oid) {
                // File
                let archive_entry = if entry.mode == 0o100755 {
                    ArchiveEntry::executable(path, blob)
                } else {
                    ArchiveEntry::file(path, blob)
                };
                entries.push(archive_entry);
            }
        }
    }
}

// ==================== Rate Limit Handler ====================

/// Gets the rate limit status.
async fn get_rate_limit(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let identity = get_identity_from_header(&headers).unwrap_or_else(|| "anonymous".to_string());
    let authenticated = state.compat.users.get_by_username(&identity).is_some();

    let response = state
        .compat
        .rate_limiter
        .get_response(&identity, authenticated);

    Json(response)
}

// ==================== Git Object Parsing Helpers ====================

/// Parsed tree entry.
struct TreeEntry {
    name: String,
    mode: u32,
    oid: ObjectId,
}

/// Parse tree ID from commit object.
fn parse_commit_tree(commit: &GitObject) -> Option<ObjectId> {
    if commit.object_type != ObjectType::Commit {
        return None;
    }

    let content = String::from_utf8_lossy(&commit.data);
    for line in content.lines() {
        if let Some(tree_hex) = line.strip_prefix("tree ") {
            return ObjectId::from_hex(tree_hex.trim()).ok();
        }
    }

    None
}

/// Parse raw tree data into entries.
fn parse_tree_entries(data: &[u8]) -> Vec<TreeEntry> {
    let mut entries = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Find the space after mode
        let space_pos = match data[i..].iter().position(|&b| b == b' ') {
            Some(pos) => pos,
            None => break,
        };

        let mode_str = String::from_utf8_lossy(&data[i..i + space_pos]);
        let mode = u32::from_str_radix(&mode_str, 8).unwrap_or(0);
        i += space_pos + 1;

        // Find the null byte after name
        let null_pos = match data[i..].iter().position(|&b| b == 0) {
            Some(pos) => pos,
            None => break,
        };

        let name = String::from_utf8_lossy(&data[i..i + null_pos]).to_string();
        i += null_pos + 1;

        // Read 20-byte SHA
        if i + 20 > data.len() {
            break;
        }
        let mut sha_bytes = [0u8; 20];
        sha_bytes.copy_from_slice(&data[i..i + 20]);
        let oid = ObjectId::from_bytes(sha_bytes);
        i += 20;

        entries.push(TreeEntry { name, mode, oid });
    }

    entries
}

/// Get a tree object from the store.
fn get_tree(repo: &Repository, id: &ObjectId) -> Option<Vec<TreeEntry>> {
    let obj = repo.objects.get(id).ok()?;
    if obj.object_type != ObjectType::Tree {
        return None;
    }
    Some(parse_tree_entries(&obj.data))
}

/// Get blob content from the store.
fn get_blob(repo: &Repository, id: &ObjectId) -> Option<Vec<u8>> {
    let obj = repo.objects.get(id).ok()?;
    if obj.object_type != ObjectType::Blob {
        return None;
    }
    Some(obj.data.to_vec())
}

// ==================== Helper Functions ====================

/// Resolve a ref to a commit SHA.
fn resolve_ref(repo: &Repository, ref_name: &str) -> Option<ObjectId> {
    // Try as direct ref
    if let Ok(reference) = repo.refs.get(ref_name) {
        return Some(resolve_reference(repo, reference));
    }

    // Try as branch
    let branch_ref = format!("refs/heads/{}", ref_name);
    if let Ok(reference) = repo.refs.get(&branch_ref) {
        return Some(resolve_reference(repo, reference));
    }

    // Try as tag
    let tag_ref = format!("refs/tags/{}", ref_name);
    if let Ok(reference) = repo.refs.get(&tag_ref) {
        return Some(resolve_reference(repo, reference));
    }

    // Try as SHA
    if ref_name.len() >= 7 {
        // Find commit by prefix
        for sha in repo.objects.list_objects() {
            let sha_str = sha.to_hex();
            if sha_str.starts_with(ref_name) {
                return Some(sha);
            }
        }
    }

    None
}

/// Resolve a reference to a direct object ID.
fn resolve_reference(repo: &Repository, reference: Reference) -> ObjectId {
    match reference {
        Reference::Direct(oid) => oid,
        Reference::Symbolic(name) => {
            if let Ok(r) = repo.refs.get(&name) {
                resolve_reference(repo, r)
            } else {
                // Shouldn't happen, but return zero as fallback
                ObjectId::from_bytes([0u8; 20])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response() {
        let err = CompatApiError(CompatError::UserNotFound("test".into()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
