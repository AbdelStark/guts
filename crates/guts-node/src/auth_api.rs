//! Authorization API endpoints for Organizations, Teams, Collaborators, Branch Protection, and Webhooks.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use guts_auth::{
    AuthError, BranchProtection, BranchProtectionRequest, Collaborator, CreateWebhookRequest,
    OrgMember, OrgRole, Organization, Permission, Team, UpdateWebhookRequest, Webhook,
    WebhookEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::api::AppState;

/// Creates the authorization API routes.
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        // Organization endpoints
        .route("/api/orgs", get(list_orgs).post(create_org))
        .route(
            "/api/orgs/{org}",
            get(get_org).patch(update_org).delete(delete_org),
        )
        .route(
            "/api/orgs/{org}/members",
            get(list_org_members).post(add_org_member),
        )
        .route(
            "/api/orgs/{org}/members/{user}",
            put(update_org_member).delete(remove_org_member),
        )
        // Team endpoints
        .route("/api/orgs/{org}/teams", get(list_teams).post(create_team))
        .route(
            "/api/orgs/{org}/teams/{team}",
            get(get_team).patch(update_team).delete(delete_team),
        )
        .route(
            "/api/orgs/{org}/teams/{team}/members",
            get(list_team_members),
        )
        .route(
            "/api/orgs/{org}/teams/{team}/members/{user}",
            put(add_team_member).delete(remove_team_member),
        )
        .route("/api/orgs/{org}/teams/{team}/repos", get(list_team_repos))
        .route(
            "/api/orgs/{org}/teams/{team}/repos/{owner}/{name}",
            put(add_team_repo).delete(remove_team_repo),
        )
        // Collaborator endpoints
        .route(
            "/api/repos/{owner}/{name}/collaborators",
            get(list_collaborators),
        )
        .route(
            "/api/repos/{owner}/{name}/collaborators/{user}",
            get(get_collaborator)
                .put(add_collaborator)
                .delete(remove_collaborator),
        )
        // Branch protection endpoints
        .route(
            "/api/repos/{owner}/{name}/branches/{branch}/protection",
            get(get_branch_protection)
                .put(set_branch_protection)
                .delete(remove_branch_protection),
        )
        // Webhook endpoints
        .route(
            "/api/repos/{owner}/{name}/hooks",
            get(list_webhooks).post(create_webhook),
        )
        .route(
            "/api/repos/{owner}/{name}/hooks/{id}",
            get(get_webhook)
                .patch(update_webhook)
                .delete(delete_webhook),
        )
        .route(
            "/api/repos/{owner}/{name}/hooks/{id}/ping",
            post(ping_webhook),
        )
        // Permission check endpoint
        .route(
            "/api/repos/{owner}/{name}/permission/{user}",
            get(check_permission),
        )
}

// ==================== Request/Response Types ====================

/// Request to create an organization.
#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub creator: String,
}

/// Request to update an organization.
#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
}

/// Response for an organization.
#[derive(Debug, Serialize)]
pub struct OrgResponse {
    pub id: u64,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_by: String,
    pub member_count: usize,
    pub team_count: usize,
    pub repo_count: usize,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<&Organization> for OrgResponse {
    fn from(org: &Organization) -> Self {
        Self {
            id: org.id,
            name: org.name.clone(),
            display_name: org.display_name.clone(),
            description: org.description.clone(),
            created_by: org.created_by.clone(),
            member_count: org.members.len(),
            team_count: org.teams.len(),
            repo_count: org.repos.len(),
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}

/// Request to add an organization member.
#[derive(Debug, Deserialize)]
pub struct AddOrgMemberRequest {
    pub user: String,
    pub role: String,
    pub added_by: String,
}

/// Response for an organization member.
#[derive(Debug, Serialize)]
pub struct OrgMemberResponse {
    pub user: String,
    pub role: String,
    pub added_at: u64,
    pub added_by: String,
}

impl From<&OrgMember> for OrgMemberResponse {
    fn from(m: &OrgMember) -> Self {
        Self {
            user: m.user.clone(),
            role: m.role.to_string(),
            added_at: m.added_at,
            added_by: m.added_by.clone(),
        }
    }
}

/// Request to create a team.
#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
    pub permission: String,
    pub created_by: String,
}

/// Request to update a team.
#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission: Option<String>,
}

/// Response for a team.
#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: u64,
    pub org_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub permission: String,
    pub member_count: usize,
    pub repo_count: usize,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<&Team> for TeamResponse {
    fn from(t: &Team) -> Self {
        Self {
            id: t.id,
            org_id: t.org_id,
            name: t.name.clone(),
            description: t.description.clone(),
            permission: t.permission.to_string(),
            member_count: t.members.len(),
            repo_count: t.repos.len(),
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

/// Request to add a collaborator.
#[derive(Debug, Deserialize)]
pub struct AddCollaboratorRequest {
    pub permission: String,
    pub added_by: String,
}

/// Response for a collaborator.
#[derive(Debug, Serialize)]
pub struct CollaboratorResponse {
    pub user: String,
    pub permission: String,
    pub added_by: String,
    pub created_at: u64,
}

impl From<&Collaborator> for CollaboratorResponse {
    fn from(c: &Collaborator) -> Self {
        Self {
            user: c.user.clone(),
            permission: c.permission.to_string(),
            added_by: c.added_by.clone(),
            created_at: c.created_at,
        }
    }
}

/// Response for branch protection.
#[derive(Debug, Serialize)]
pub struct BranchProtectionResponse {
    pub id: u64,
    pub pattern: String,
    pub require_pr: bool,
    pub required_reviews: u32,
    pub required_status_checks: Vec<String>,
    pub dismiss_stale_reviews: bool,
    pub require_code_owner_review: bool,
    pub restrict_pushes: bool,
    pub allow_force_push: bool,
    pub allow_deletion: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<&BranchProtection> for BranchProtectionResponse {
    fn from(bp: &BranchProtection) -> Self {
        Self {
            id: bp.id,
            pattern: bp.pattern.clone(),
            require_pr: bp.require_pr,
            required_reviews: bp.required_reviews,
            required_status_checks: bp.required_status_checks.iter().cloned().collect(),
            dismiss_stale_reviews: bp.dismiss_stale_reviews,
            require_code_owner_review: bp.require_code_owner_review,
            restrict_pushes: bp.restrict_pushes,
            allow_force_push: bp.allow_force_push,
            allow_deletion: bp.allow_deletion,
            created_at: bp.created_at,
            updated_at: bp.updated_at,
        }
    }
}

/// Response for a webhook.
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub id: u64,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub content_type: String,
    pub delivery_count: u64,
    pub failure_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<&Webhook> for WebhookResponse {
    fn from(w: &Webhook) -> Self {
        Self {
            id: w.id,
            url: w.url.clone(),
            events: w.events.iter().map(|e| e.to_string()).collect(),
            active: w.active,
            content_type: w.content_type.clone(),
            delivery_count: w.delivery_count,
            failure_count: w.failure_count,
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}

/// Response for permission check.
#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub user: String,
    pub permission: Option<String>,
    pub has_access: bool,
}

/// Error response.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Wrapper for auth errors.
struct AuthApiError(AuthError);

impl From<AuthError> for AuthApiError {
    fn from(err: AuthError) -> Self {
        Self(err)
    }
}

impl IntoResponse for AuthApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self.0 {
            AuthError::NotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            AuthError::PermissionDenied(_) => (StatusCode::FORBIDDEN, self.0.to_string()),
            AuthError::AlreadyExists(_) => (StatusCode::CONFLICT, self.0.to_string()),
            AuthError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            AuthError::LastOwner => (StatusCode::BAD_REQUEST, self.0.to_string()),
            AuthError::BranchProtected(_, _) => (StatusCode::FORBIDDEN, self.0.to_string()),
            AuthError::InvalidWebhook(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            AuthError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

// ==================== Organization Handlers ====================

/// Lists all organizations.
async fn list_orgs(State(state): State<AppState>) -> impl IntoResponse {
    let orgs = state.auth.list_organizations();
    let responses: Vec<OrgResponse> = orgs.iter().map(Into::into).collect();
    Json(responses)
}

/// Creates a new organization.
async fn create_org(
    State(state): State<AppState>,
    Json(req): Json<CreateOrgRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let mut org = state
        .auth
        .create_organization(req.name, req.display_name, req.creator)?;

    if let Some(desc) = req.description {
        org = state.auth.update_organization(org.id, None, Some(desc))?;
    }

    Ok((StatusCode::CREATED, Json(OrgResponse::from(&org))))
}

/// Gets an organization by name.
async fn get_org(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    Ok(Json(OrgResponse::from(&org)))
}

/// Updates an organization.
async fn update_org(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
    Json(req): Json<UpdateOrgRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let updated = state
        .auth
        .update_organization(org.id, req.display_name, req.description)?;

    Ok(Json(OrgResponse::from(&updated)))
}

/// Deletes an organization.
async fn delete_org(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    state.auth.delete_organization(org.id)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Lists organization members.
async fn list_org_members(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let responses: Vec<OrgMemberResponse> = org.members.iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Adds a member to an organization.
async fn add_org_member(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
    Json(req): Json<AddOrgMemberRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let role = OrgRole::parse(&req.role)
        .ok_or_else(|| AuthError::InvalidInput(format!("invalid role: {}", req.role)))?;

    let member = OrgMember::new(req.user, role, req.added_by);
    state.auth.add_org_member(org.id, member.clone())?;

    Ok((StatusCode::CREATED, Json(OrgMemberResponse::from(&member))))
}

/// Updates a member's role.
async fn update_org_member(
    State(state): State<AppState>,
    Path((org_name, user)): Path<(String, String)>,
    Json(req): Json<AddOrgMemberRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let role = OrgRole::parse(&req.role)
        .ok_or_else(|| AuthError::InvalidInput(format!("invalid role: {}", req.role)))?;

    state.auth.update_org_member_role(org.id, &user, role)?;

    // Get updated org
    let org = state.auth.get_organization(org.id).unwrap();
    let member = org
        .get_member(&user)
        .ok_or_else(|| AuthError::NotFound(format!("member '{}'", user)))?;

    Ok(Json(OrgMemberResponse::from(member)))
}

/// Removes a member from an organization.
async fn remove_org_member(
    State(state): State<AppState>,
    Path((org_name, user)): Path<(String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    state.auth.remove_org_member(org.id, &user)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Team Handlers ====================

/// Lists teams in an organization.
async fn list_teams(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let teams = state.auth.list_teams(org.id);
    let responses: Vec<TeamResponse> = teams.iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Creates a new team.
async fn create_team(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let permission = Permission::parse(&req.permission).ok_or_else(|| {
        AuthError::InvalidInput(format!("invalid permission: {}", req.permission))
    })?;

    let mut team = state
        .auth
        .create_team(org.id, req.name, permission, req.created_by)?;

    if let Some(desc) = req.description {
        team = state.auth.update_team(team.id, None, Some(desc), None)?;
    }

    Ok((StatusCode::CREATED, Json(TeamResponse::from(&team))))
}

/// Gets a team by name.
async fn get_team(
    State(state): State<AppState>,
    Path((org_name, team_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    Ok(Json(TeamResponse::from(&team)))
}

/// Updates a team.
async fn update_team(
    State(state): State<AppState>,
    Path((org_name, team_name)): Path<(String, String)>,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    let permission = req
        .permission
        .as_ref()
        .map(|p| {
            Permission::parse(p)
                .ok_or_else(|| AuthError::InvalidInput(format!("invalid permission: {}", p)))
        })
        .transpose()?;

    let updated = state
        .auth
        .update_team(team.id, req.name, req.description, permission)?;

    Ok(Json(TeamResponse::from(&updated)))
}

/// Deletes a team.
async fn delete_team(
    State(state): State<AppState>,
    Path((org_name, team_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    state.auth.delete_team(team.id)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Lists team members.
async fn list_team_members(
    State(state): State<AppState>,
    Path((org_name, team_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    let members: Vec<String> = team.members.iter().cloned().collect();

    Ok(Json(members))
}

/// Adds a member to a team.
async fn add_team_member(
    State(state): State<AppState>,
    Path((org_name, team_name, user)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    state.auth.add_team_member(team.id, user)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Removes a member from a team.
async fn remove_team_member(
    State(state): State<AppState>,
    Path((org_name, team_name, user)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    state.auth.remove_team_member(team.id, &user)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Lists repositories in a team.
async fn list_team_repos(
    State(state): State<AppState>,
    Path((org_name, team_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    let repos: Vec<String> = team.repos.iter().cloned().collect();

    Ok(Json(repos))
}

/// Adds a repository to a team.
async fn add_team_repo(
    State(state): State<AppState>,
    Path((org_name, team_name, owner, name)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    let repo_key = format!("{}/{}", owner, name);
    state.auth.add_team_repo(team.id, repo_key)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Removes a repository from a team.
async fn remove_team_repo(
    State(state): State<AppState>,
    Path((org_name, team_name, owner, name)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| AuthError::NotFound(format!("organization '{}'", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| AuthError::NotFound(format!("team '{}'", team_name)))?;

    let repo_key = format!("{}/{}", owner, name);
    state.auth.remove_team_repo(team.id, &repo_key)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Collaborator Handlers ====================

/// Lists collaborators for a repository.
async fn list_collaborators(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let collaborators = state.auth.list_collaborators(&repo_key);
    let responses: Vec<CollaboratorResponse> = collaborators.iter().map(Into::into).collect();
    Json(responses)
}

/// Gets a collaborator.
async fn get_collaborator(
    State(state): State<AppState>,
    Path((owner, name, user)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let collaborator = state
        .auth
        .get_collaborator(&repo_key, &user)
        .ok_or_else(|| AuthError::NotFound(format!("collaborator '{}' on '{}'", user, repo_key)))?;

    Ok(Json(CollaboratorResponse::from(&collaborator)))
}

/// Adds or updates a collaborator.
async fn add_collaborator(
    State(state): State<AppState>,
    Path((owner, name, user)): Path<(String, String, String)>,
    Json(req): Json<AddCollaboratorRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let permission = Permission::parse(&req.permission).ok_or_else(|| {
        AuthError::InvalidInput(format!("invalid permission: {}", req.permission))
    })?;

    let collaborator = state
        .auth
        .set_collaborator(repo_key, user, permission, req.added_by);

    Ok((
        StatusCode::CREATED,
        Json(CollaboratorResponse::from(&collaborator)),
    ))
}

/// Removes a collaborator.
async fn remove_collaborator(
    State(state): State<AppState>,
    Path((owner, name, user)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);
    state.auth.remove_collaborator(&repo_key, &user)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Branch Protection Handlers ====================

/// Gets branch protection for a branch.
async fn get_branch_protection(
    State(state): State<AppState>,
    Path((owner, name, branch)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let protection = state
        .auth
        .find_branch_protection(&repo_key, &branch)
        .ok_or_else(|| {
            AuthError::NotFound(format!(
                "branch protection for '{}' on '{}'",
                branch, repo_key
            ))
        })?;

    Ok(Json(BranchProtectionResponse::from(&protection)))
}

/// Sets branch protection for a branch.
async fn set_branch_protection(
    State(state): State<AppState>,
    Path((owner, name, branch)): Path<(String, String, String)>,
    Json(req): Json<BranchProtectionRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);

    // Create or get existing protection
    let protection = state
        .auth
        .set_branch_protection(repo_key.clone(), branch.clone());

    // Update with request values
    let updated = state
        .auth
        .update_branch_protection(&repo_key, &branch, |p| {
            p.require_pr = req.require_pr;
            p.required_reviews = req.required_reviews;
            p.required_status_checks = req.required_status_checks.iter().cloned().collect();
            p.dismiss_stale_reviews = req.dismiss_stale_reviews;
            p.require_code_owner_review = req.require_code_owner_review;
            p.restrict_pushes = req.restrict_pushes;
            p.allow_force_push = req.allow_force_push;
            p.allow_deletion = req.allow_deletion;
        })?;

    // Return created status if this was a new protection
    let status = if updated.id == protection.id {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    Ok((status, Json(BranchProtectionResponse::from(&updated))))
}

/// Removes branch protection for a branch.
async fn remove_branch_protection(
    State(state): State<AppState>,
    Path((owner, name, branch)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);
    state.auth.remove_branch_protection(&repo_key, &branch)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Webhook Handlers ====================

/// Lists webhooks for a repository.
async fn list_webhooks(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let webhooks = state.auth.list_webhooks(&repo_key);
    let responses: Vec<WebhookResponse> = webhooks.iter().map(Into::into).collect();
    Json(responses)
}

/// Creates a webhook.
async fn create_webhook(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);

    // Parse events
    let events: HashSet<WebhookEvent> = req
        .events
        .iter()
        .filter_map(|e| WebhookEvent::parse(e))
        .collect();

    if events.is_empty() {
        return Err(AuthError::InvalidInput("at least one valid event is required".into()).into());
    }

    let mut webhook = state.auth.create_webhook(repo_key, req.url, events);

    // Set secret if provided
    if let Some(secret) = req.secret {
        state
            .auth
            .update_webhook(webhook.id, |w| {
                w.secret = Some(secret);
            })
            .ok();
        webhook = state.auth.get_webhook(webhook.id).unwrap();
    }

    Ok((StatusCode::CREATED, Json(WebhookResponse::from(&webhook))))
}

/// Gets a webhook by ID.
async fn get_webhook(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, u64)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let webhook = state
        .auth
        .get_webhook(id)
        .filter(|w| w.repo_key == repo_key)
        .ok_or_else(|| AuthError::NotFound(format!("webhook {}", id)))?;

    Ok(Json(WebhookResponse::from(&webhook)))
}

/// Updates a webhook.
async fn update_webhook(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, u64)>,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);

    // Verify webhook belongs to this repo
    let webhook = state
        .auth
        .get_webhook(id)
        .filter(|w| w.repo_key == repo_key)
        .ok_or_else(|| AuthError::NotFound(format!("webhook {}", id)))?;

    let updated = state.auth.update_webhook(webhook.id, |w| {
        if let Some(url) = &req.url {
            w.url = url.clone();
        }
        if let Some(secret) = &req.secret {
            w.secret = Some(secret.clone());
        }
        if let Some(events) = &req.events {
            w.events = events
                .iter()
                .filter_map(|e| WebhookEvent::parse(e))
                .collect();
        }
        if let Some(active) = req.active {
            w.active = active;
        }
    })?;

    Ok(Json(WebhookResponse::from(&updated)))
}

/// Deletes a webhook.
async fn delete_webhook(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, u64)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);

    // Verify webhook belongs to this repo
    let _webhook = state
        .auth
        .get_webhook(id)
        .filter(|w| w.repo_key == repo_key)
        .ok_or_else(|| AuthError::NotFound(format!("webhook {}", id)))?;

    state.auth.delete_webhook(id)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Pings a webhook (for testing).
async fn ping_webhook(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, u64)>,
) -> Result<impl IntoResponse, AuthApiError> {
    let repo_key = format!("{}/{}", owner, name);

    // Verify webhook belongs to this repo
    let webhook = state
        .auth
        .get_webhook(id)
        .filter(|w| w.repo_key == repo_key)
        .ok_or_else(|| AuthError::NotFound(format!("webhook {}", id)))?;

    // In a real implementation, this would send a ping request to the webhook URL
    // For now, just return success
    Ok(Json(serde_json::json!({
        "id": webhook.id,
        "url": webhook.url,
        "message": "Ping sent successfully"
    })))
}

// ==================== Permission Check Handler ====================

/// Checks a user's permission on a repository.
async fn check_permission(
    State(state): State<AppState>,
    Path((owner, name, user)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, name);
    let permission = state.auth.get_effective_permission(&user, &repo_key);

    Json(PermissionResponse {
        user,
        permission: permission.map(|p| p.to_string()),
        has_access: permission.is_some(),
    })
}
