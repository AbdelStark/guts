//! # Collaboration API
//!
//! This module provides HTTP endpoints for code collaboration features:
//!
//! - **Pull Requests**: Merge proposals with code review workflow
//! - **Issues**: Bug reports, feature requests, and task tracking
//! - **Comments**: Threaded discussions on PRs and Issues
//! - **Reviews**: Code reviews with approval/rejection states
//!
//! ## Pull Request Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/repos/{owner}/{name}/pulls` | List pull requests |
//! | POST | `/api/repos/{owner}/{name}/pulls` | Create a pull request |
//! | GET | `/api/repos/{owner}/{name}/pulls/{number}` | Get PR details |
//! | PATCH | `/api/repos/{owner}/{name}/pulls/{number}` | Update PR (title, state) |
//! | POST | `/api/repos/{owner}/{name}/pulls/{number}/merge` | Merge the PR |
//! | GET | `/api/repos/{owner}/{name}/pulls/{number}/comments` | List PR comments |
//! | POST | `/api/repos/{owner}/{name}/pulls/{number}/comments` | Add a comment |
//! | GET | `/api/repos/{owner}/{name}/pulls/{number}/reviews` | List reviews |
//! | POST | `/api/repos/{owner}/{name}/pulls/{number}/reviews` | Submit a review |
//!
//! ## Issue Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/repos/{owner}/{name}/issues` | List issues |
//! | POST | `/api/repos/{owner}/{name}/issues` | Create an issue |
//! | GET | `/api/repos/{owner}/{name}/issues/{number}` | Get issue details |
//! | PATCH | `/api/repos/{owner}/{name}/issues/{number}` | Update issue |
//! | GET | `/api/repos/{owner}/{name}/issues/{number}/comments` | List comments |
//! | POST | `/api/repos/{owner}/{name}/issues/{number}/comments` | Add a comment |
//!
//! ## State Transitions
//!
//! ### Pull Request States
//!
//! ```text
//! Open ──┬──> Closed ──> Open (reopen)
//!        └──> Merged (terminal)
//! ```
//!
//! ### Issue States
//!
//! ```text
//! Open <──> Closed
//! ```
//!
//! ### Review States
//!
//! - `Pending`: Review in progress
//! - `Commented`: Feedback without explicit approval
//! - `Approved`: Code approved
//! - `ChangesRequested`: Changes needed before merge
//! - `Dismissed`: Review dismissed by maintainer
//!
//! ## Query Parameters
//!
//! List endpoints support filtering:
//!
//! ```bash
//! # List open PRs only
//! GET /api/repos/alice/myrepo/pulls?state=open
//!
//! # List closed issues
//! GET /api/repos/alice/myrepo/issues?state=closed
//! ```
//!
//! ## Example: Creating a Pull Request
//!
//! ```bash
//! curl -X POST http://localhost:8080/api/repos/alice/myrepo/pulls \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "title": "Add new feature",
//!     "description": "Implements the requested feature",
//!     "author": "bob",
//!     "source_branch": "feature/new-feature",
//!     "target_branch": "main",
//!     "source_commit": "abc123",
//!     "target_commit": "def456"
//!   }'
//! ```
//!
//! ## Example: Submitting a Review
//!
//! ```bash
//! curl -X POST http://localhost:8080/api/repos/alice/myrepo/pulls/1/reviews \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "author": "alice",
//!     "state": "approved",
//!     "body": "LGTM!",
//!     "commit_id": "abc123"
//!   }'
//! ```

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use guts_collaboration::{
    CollaborationError, Comment, CommentTarget, Issue, IssueState, Label, PullRequest,
    PullRequestState, Review, ReviewState,
};
use guts_storage::ObjectId;
use serde::{Deserialize, Serialize};

use crate::api::AppState;

/// Creates the collaboration API routes.
pub fn collaboration_routes() -> Router<AppState> {
    Router::new()
        // Pull Request endpoints
        .route(
            "/api/repos/{owner}/{name}/pulls",
            get(list_prs).post(create_pr),
        )
        .route(
            "/api/repos/{owner}/{name}/pulls/{number}",
            get(get_pr).patch(update_pr),
        )
        .route(
            "/api/repos/{owner}/{name}/pulls/{number}/merge",
            post(merge_pr),
        )
        .route(
            "/api/repos/{owner}/{name}/pulls/{number}/comments",
            get(list_pr_comments).post(create_pr_comment),
        )
        .route(
            "/api/repos/{owner}/{name}/pulls/{number}/reviews",
            get(list_reviews).post(create_review),
        )
        // Issue endpoints
        .route(
            "/api/repos/{owner}/{name}/issues",
            get(list_issues).post(create_issue),
        )
        .route(
            "/api/repos/{owner}/{name}/issues/{number}",
            get(get_issue).patch(update_issue),
        )
        .route(
            "/api/repos/{owner}/{name}/issues/{number}/comments",
            get(list_issue_comments).post(create_issue_comment),
        )
}

// ==================== Request/Response Types ====================

/// Query parameters for listing pull requests.
#[derive(Debug, Deserialize)]
pub struct ListPRsQuery {
    pub state: Option<String>,
}

/// Query parameters for listing issues.
#[derive(Debug, Deserialize)]
pub struct ListIssuesQuery {
    pub state: Option<String>,
}

/// Request to create a pull request.
#[derive(Debug, Deserialize)]
pub struct CreatePRRequest {
    pub title: String,
    pub description: String,
    pub author: String,
    pub source_branch: String,
    pub target_branch: String,
    pub source_commit: String,
    pub target_commit: String,
}

/// Request to update a pull request.
#[derive(Debug, Deserialize)]
pub struct UpdatePRRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
}

/// Request to merge a pull request.
#[derive(Debug, Deserialize)]
pub struct MergePRRequest {
    pub merged_by: String,
}

/// Request to create an issue.
#[derive(Debug, Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub description: String,
    pub author: String,
    pub labels: Option<Vec<String>>,
}

/// Request to update an issue.
#[derive(Debug, Deserialize)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub closed_by: Option<String>,
}

/// Request to create a comment.
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub author: String,
    pub body: String,
}

/// Request to create a review.
#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub author: String,
    pub state: String,
    pub body: Option<String>,
    pub commit_id: String,
}

/// Response for a pull request.
#[derive(Debug, Serialize)]
pub struct PullRequestResponse {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub description: String,
    pub author: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub source_commit: String,
    pub target_commit: String,
    pub labels: Vec<LabelResponse>,
    pub created_at: u64,
    pub updated_at: u64,
    pub merged_at: Option<u64>,
    pub merged_by: Option<String>,
}

impl From<PullRequest> for PullRequestResponse {
    fn from(pr: PullRequest) -> Self {
        Self {
            id: pr.id,
            number: pr.number,
            title: pr.title,
            description: pr.description,
            author: pr.author,
            state: pr.state.to_string(),
            source_branch: pr.source_branch,
            target_branch: pr.target_branch,
            source_commit: pr.source_commit.to_hex(),
            target_commit: pr.target_commit.to_hex(),
            labels: pr.labels.into_iter().map(Into::into).collect(),
            created_at: pr.created_at,
            updated_at: pr.updated_at,
            merged_at: pr.merged_at,
            merged_by: pr.merged_by,
        }
    }
}

/// Response for an issue.
#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub description: String,
    pub author: String,
    pub state: String,
    pub labels: Vec<LabelResponse>,
    pub created_at: u64,
    pub updated_at: u64,
    pub closed_at: Option<u64>,
    pub closed_by: Option<String>,
}

impl From<Issue> for IssueResponse {
    fn from(issue: Issue) -> Self {
        Self {
            id: issue.id,
            number: issue.number,
            title: issue.title,
            description: issue.description,
            author: issue.author,
            state: issue.state.to_string(),
            labels: issue.labels.into_iter().map(Into::into).collect(),
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            closed_at: issue.closed_at,
            closed_by: issue.closed_by,
        }
    }
}

/// Response for a label.
#[derive(Debug, Serialize)]
pub struct LabelResponse {
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

impl From<Label> for LabelResponse {
    fn from(label: Label) -> Self {
        Self {
            name: label.name,
            color: label.color,
            description: label.description,
        }
    }
}

/// Response for a comment.
#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<Comment> for CommentResponse {
    fn from(comment: Comment) -> Self {
        Self {
            id: comment.id,
            author: comment.author,
            body: comment.body,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        }
    }
}

/// Response for a review.
#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: u64,
    pub pr_number: u32,
    pub author: String,
    pub state: String,
    pub body: Option<String>,
    pub commit_id: String,
    pub created_at: u64,
}

impl From<Review> for ReviewResponse {
    fn from(review: Review) -> Self {
        Self {
            id: review.id,
            pr_number: review.pr_number,
            author: review.author,
            state: review.state.to_string(),
            body: review.body,
            commit_id: review.commit_id,
            created_at: review.created_at,
        }
    }
}

/// Error response.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Convert collaboration errors to HTTP responses.
impl IntoResponse for CollaborationApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self.0 {
            CollaborationError::PullRequestNotFound { .. } => {
                (StatusCode::NOT_FOUND, self.0.to_string())
            }
            CollaborationError::IssueNotFound { .. } => (StatusCode::NOT_FOUND, self.0.to_string()),
            CollaborationError::CommentNotFound { .. } => {
                (StatusCode::NOT_FOUND, self.0.to_string())
            }
            CollaborationError::ReviewNotFound { .. } => {
                (StatusCode::NOT_FOUND, self.0.to_string())
            }
            CollaborationError::PullRequestExists { .. } => {
                (StatusCode::CONFLICT, self.0.to_string())
            }
            CollaborationError::IssueExists { .. } => (StatusCode::CONFLICT, self.0.to_string()),
            CollaborationError::InvalidStateTransition { .. } => {
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
            CollaborationError::AlreadyMerged { .. } => {
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
            CollaborationError::PullRequestClosed { .. } => {
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
            CollaborationError::IssueClosed { .. } => (StatusCode::BAD_REQUEST, self.0.to_string()),
            CollaborationError::RepoNotFound { .. } => (StatusCode::NOT_FOUND, self.0.to_string()),
            CollaborationError::Validation(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            CollaborationError::Serialization(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string())
            }
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

/// Wrapper for collaboration errors.
struct CollaborationApiError(CollaborationError);

impl From<CollaborationError> for CollaborationApiError {
    fn from(err: CollaborationError) -> Self {
        Self(err)
    }
}

// ==================== Pull Request Handlers ====================

/// Lists pull requests for a repository.
async fn list_prs(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<ListPRsQuery>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let pr_state = params.state.as_deref().and_then(parse_pr_state);

    let prs = state.collaboration.list_pull_requests(&repo_key, pr_state);
    let responses: Vec<PullRequestResponse> = prs.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Creates a new pull request.
async fn create_pr(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<CreatePRRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let source_commit = ObjectId::from_hex(&req.source_commit)
        .map_err(|e| CollaborationError::Validation(e.to_string()))?;
    let target_commit = ObjectId::from_hex(&req.target_commit)
        .map_err(|e| CollaborationError::Validation(e.to_string()))?;

    let pr = PullRequest::new(
        0,
        &repo_key,
        0,
        req.title,
        req.description,
        req.author,
        req.source_branch,
        req.target_branch,
        source_commit,
        target_commit,
    );

    let created = state.collaboration.create_pull_request(pr)?;

    Ok((
        StatusCode::CREATED,
        Json(PullRequestResponse::from(created)),
    ))
}

/// Gets a specific pull request.
async fn get_pr(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let pr = state.collaboration.get_pull_request(&repo_key, number)?;

    Ok(Json(PullRequestResponse::from(pr)))
}

/// Updates a pull request.
async fn update_pr(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
    Json(req): Json<UpdatePRRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let updated = state
        .collaboration
        .update_pull_request(&repo_key, number, |pr| {
            if let Some(title) = &req.title {
                pr.update_title(title);
            }
            if let Some(desc) = &req.description {
                pr.update_description(desc);
            }
            if let Some(state_str) = &req.state {
                match state_str.as_str() {
                    "closed" => pr.close()?,
                    "open" => pr.reopen()?,
                    _ => {
                        return Err(CollaborationError::Validation(format!(
                            "invalid state: {}",
                            state_str
                        )))
                    }
                }
            }
            Ok(())
        })?;

    Ok(Json(PullRequestResponse::from(updated)))
}

/// Merges a pull request.
async fn merge_pr(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
    Json(req): Json<MergePRRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let merged = state
        .collaboration
        .merge_pull_request(&repo_key, number, &req.merged_by)?;

    Ok(Json(PullRequestResponse::from(merged)))
}

/// Lists comments on a pull request.
async fn list_pr_comments(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let comments = state.collaboration.list_pr_comments(&repo_key, number);
    let responses: Vec<CommentResponse> = comments.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Creates a comment on a pull request.
async fn create_pr_comment(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let target = CommentTarget::pull_request(&repo_key, number);
    let comment = Comment::new(0, target, req.author, req.body);
    let created = state.collaboration.create_comment(comment)?;

    Ok((StatusCode::CREATED, Json(CommentResponse::from(created))))
}

/// Lists reviews on a pull request.
async fn list_reviews(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let reviews = state.collaboration.list_reviews(&repo_key, number);
    let responses: Vec<ReviewResponse> = reviews.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Creates a review on a pull request.
async fn create_review(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
    Json(req): Json<CreateReviewRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let review_state = parse_review_state(&req.state).ok_or_else(|| {
        CollaborationError::Validation(format!("invalid review state: {}", req.state))
    })?;

    let mut review = Review::new(
        0,
        &repo_key,
        number,
        req.author,
        review_state,
        req.commit_id,
    );
    if let Some(body) = req.body {
        review = review.with_body(body);
    }

    let created = state.collaboration.create_review(review)?;

    Ok((StatusCode::CREATED, Json(ReviewResponse::from(created))))
}

// ==================== Issue Handlers ====================

/// Lists issues for a repository.
async fn list_issues(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<ListIssuesQuery>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let issue_state = params.state.as_deref().and_then(parse_issue_state);

    let issues = state.collaboration.list_issues(&repo_key, issue_state);
    let responses: Vec<IssueResponse> = issues.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Creates a new issue.
async fn create_issue(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<CreateIssueRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let mut issue = Issue::new(0, &repo_key, 0, req.title, req.description, req.author);

    if let Some(labels) = req.labels {
        for label_name in labels {
            issue.add_label(Label::new(label_name, "888888"));
        }
    }

    let created = state.collaboration.create_issue(issue)?;

    Ok((StatusCode::CREATED, Json(IssueResponse::from(created))))
}

/// Gets a specific issue.
async fn get_issue(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let issue = state.collaboration.get_issue(&repo_key, number)?;

    Ok(Json(IssueResponse::from(issue)))
}

/// Updates an issue.
async fn update_issue(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
    Json(req): Json<UpdateIssueRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);

    let updated = state
        .collaboration
        .update_issue(&repo_key, number, |issue| {
            if let Some(title) = &req.title {
                issue.update_title(title);
            }
            if let Some(desc) = &req.description {
                issue.update_description(desc);
            }
            if let Some(state_str) = &req.state {
                match state_str.as_str() {
                    "closed" => {
                        let closed_by = req.closed_by.as_deref().unwrap_or("unknown");
                        issue.close(closed_by)?;
                    }
                    "open" => issue.reopen()?,
                    _ => {
                        return Err(CollaborationError::Validation(format!(
                            "invalid state: {}",
                            state_str
                        )))
                    }
                }
            }
            Ok(())
        })?;

    Ok(Json(IssueResponse::from(updated)))
}

/// Lists comments on an issue.
async fn list_issue_comments(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let comments = state.collaboration.list_issue_comments(&repo_key, number);
    let responses: Vec<CommentResponse> = comments.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Creates a comment on an issue.
async fn create_issue_comment(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, u32)>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, CollaborationApiError> {
    let repo_key = format!("{}/{}", owner, name);
    let target = CommentTarget::issue(&repo_key, number);
    let comment = Comment::new(0, target, req.author, req.body);
    let created = state.collaboration.create_comment(comment)?;

    Ok((StatusCode::CREATED, Json(CommentResponse::from(created))))
}

// ==================== Helper Functions ====================

fn parse_pr_state(s: &str) -> Option<PullRequestState> {
    match s.to_lowercase().as_str() {
        "open" => Some(PullRequestState::Open),
        "closed" => Some(PullRequestState::Closed),
        "merged" => Some(PullRequestState::Merged),
        _ => None,
    }
}

fn parse_issue_state(s: &str) -> Option<IssueState> {
    match s.to_lowercase().as_str() {
        "open" => Some(IssueState::Open),
        "closed" => Some(IssueState::Closed),
        _ => None,
    }
}

fn parse_review_state(s: &str) -> Option<ReviewState> {
    match s.to_lowercase().as_str() {
        "approved" => Some(ReviewState::Approved),
        "changes_requested" => Some(ReviewState::ChangesRequested),
        "commented" => Some(ReviewState::Commented),
        _ => None,
    }
}
