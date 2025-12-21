//! Web route handlers for the Guts web gateway.

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::WebError;
use crate::templates::*;

/// Shared state for web routes.
#[derive(Clone)]
pub struct WebState {
    pub repos: Arc<guts_storage::RepoStore>,
    pub collaboration: Arc<guts_collaboration::CollaborationStore>,
}

/// Create the web router.
pub fn web_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    WebState: axum::extract::FromRef<S>,
{
    Router::new()
        .route("/", get(index))
        .route("/explore", get(explore))
        .route("/{owner}/{repo}", get(repo_home))
        .route("/{owner}/{repo}/pulls", get(pull_request_list))
        .route("/{owner}/{repo}/issues", get(issue_list))
}

/// Query parameters for list endpoints.
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_state")]
    pub state: String,
}

fn default_state() -> String {
    "open".to_string()
}

/// Landing page handler.
async fn index(State(state): State<WebState>) -> Result<impl IntoResponse, WebError> {
    let repos: Vec<RepoSummary> = state
        .repos
        .list()
        .into_iter()
        .take(9)
        .map(|r| RepoSummary {
            owner: r.owner.clone(),
            name: r.name.clone(),
            description: String::new(),
            branch_count: 0,
        })
        .collect();

    let template = IndexTemplate { repos };
    Ok(Html(template.render()?))
}

/// Explore repositories page.
async fn explore(State(state): State<WebState>) -> Result<impl IntoResponse, WebError> {
    let repos: Vec<RepoSummary> = state
        .repos
        .list()
        .into_iter()
        .map(|r| RepoSummary {
            owner: r.owner.clone(),
            name: r.name.clone(),
            description: String::new(),
            branch_count: 0,
        })
        .collect();

    let template = ExploreTemplate { repos };
    Ok(Html(template.render()?))
}

/// Repository home page.
async fn repo_home(
    State(state): State<WebState>,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let repo_key = format!("{}/{}", owner, repo);

    let repository = state
        .repos
        .get(&owner, &repo)
        .map_err(|_| WebError::NotFound(format!("Repository '{}' not found", repo_key)))?;

    // Count issues and PRs
    let prs = state.collaboration.list_pull_requests(&repo_key, None);
    let issues = state.collaboration.list_issues(&repo_key, None);

    let pr_count = prs
        .iter()
        .filter(|p| p.state == guts_collaboration::PullRequestState::Open)
        .count();
    let issue_count = issues
        .iter()
        .filter(|i| i.state == guts_collaboration::IssueState::Open)
        .count();

    let template = RepoViewTemplate {
        owner: owner.clone(),
        name: repository.name.clone(),
        description: String::new(),
        default_branch: "main".to_string(),
        branch_count: 0,
        issue_count,
        pr_count,
        clone_url: format!("http://localhost:8080/{}.git", repo_key),
        files: vec![],
        readme_html: None,
    };

    Ok(Html(template.render()?))
}

/// Pull request list page.
async fn pull_request_list(
    State(state): State<WebState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, WebError> {
    let repo_key = format!("{}/{}", owner, repo);

    let all_prs = state.collaboration.list_pull_requests(&repo_key, None);

    let open_count = all_prs
        .iter()
        .filter(|p| p.state == guts_collaboration::PullRequestState::Open)
        .count();
    let closed_count = all_prs.len() - open_count;

    let filter_state = match query.state.as_str() {
        "closed" => Some(guts_collaboration::PullRequestState::Closed),
        _ => Some(guts_collaboration::PullRequestState::Open),
    };

    let pull_requests: Vec<PullRequestSummary> = state
        .collaboration
        .list_pull_requests(&repo_key, filter_state)
        .into_iter()
        .map(|pr| PullRequestSummary {
            number: pr.number,
            title: pr.title.clone(),
            author: pr.author.clone(),
            state: format!("{:?}", pr.state),
        })
        .collect();

    let template = PullRequestListTemplate {
        owner,
        name: repo,
        state: query.state,
        open_count,
        closed_count,
        pull_requests,
    };

    Ok(Html(template.render()?))
}

/// Issue list page.
async fn issue_list(
    State(state): State<WebState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, WebError> {
    let repo_key = format!("{}/{}", owner, repo);

    let all_issues = state.collaboration.list_issues(&repo_key, None);

    let open_count = all_issues
        .iter()
        .filter(|i| i.state == guts_collaboration::IssueState::Open)
        .count();
    let closed_count = all_issues.len() - open_count;

    let filter_state = match query.state.as_str() {
        "closed" => Some(guts_collaboration::IssueState::Closed),
        _ => Some(guts_collaboration::IssueState::Open),
    };

    let issues: Vec<IssueSummary> = state
        .collaboration
        .list_issues(&repo_key, filter_state)
        .into_iter()
        .map(|issue| IssueSummary {
            number: issue.number,
            title: issue.title.clone(),
            author: issue.author.clone(),
            state: format!("{:?}", issue.state),
            labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
        })
        .collect();

    let template = IssueListTemplate {
        owner,
        name: repo,
        state: query.state,
        open_count,
        closed_count,
        issues,
    };

    Ok(Html(template.render()?))
}
