//! Web route handlers for the Guts web gateway.

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use guts_storage::{GitObject, ObjectId, ObjectType};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::WebError;
use crate::markdown::render_markdown;
use crate::templates::*;

/// Shared state for web routes.
#[derive(Clone)]
pub struct WebState {
    pub repos: Arc<guts_storage::RepoStore>,
    pub collaboration: Arc<guts_collaboration::CollaborationStore>,
    pub auth: Arc<guts_auth::AuthStore>,
    pub ci: Arc<guts_ci::CiStore>,
    /// Optional consensus engine for displaying consensus status.
    pub consensus: Option<Arc<guts_consensus::ConsensusEngine>>,
    /// Optional mempool for displaying pending transactions.
    pub mempool: Option<Arc<guts_consensus::Mempool>>,
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
        // Search
        .route("/search", get(search))
        // API Documentation
        .route("/api/docs", get(api_docs))
        // Organizations
        .route("/orgs", get(org_list))
        .route("/orgs/{org}", get(org_view))
        .route("/orgs/{org}/teams", get(org_teams))
        .route("/orgs/{org}/teams/{team}", get(team_view))
        // User profiles
        .route("/users/{username}", get(user_profile))
        // Repository routes
        .route("/{owner}/{repo}", get(repo_home))
        // File browsing
        .route("/{owner}/{repo}/tree/{ref}", get(tree_root))
        .route("/{owner}/{repo}/tree/{ref}/{*path}", get(tree_path))
        .route("/{owner}/{repo}/blob/{ref}/{*path}", get(blob_view))
        // Commits
        .route("/{owner}/{repo}/commits/{ref}", get(commits_list))
        .route("/{owner}/{repo}/commit/{sha}", get(commit_view))
        // Pull requests
        .route("/{owner}/{repo}/pulls", get(pull_request_list))
        .route("/{owner}/{repo}/pull/{number}", get(pull_request_view))
        // Issues
        .route("/{owner}/{repo}/issues", get(issue_list))
        .route("/{owner}/{repo}/issues/{number}", get(issue_view))
        // Actions (CI/CD)
        .route("/{owner}/{repo}/actions", get(actions_list))
        .route("/{owner}/{repo}/actions/runs/{run_id}", get(actions_run))
        // Consensus
        .route("/consensus", get(consensus_dashboard))
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

/// Query parameters for search.
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_search_type")]
    #[serde(rename = "type")]
    pub search_type: String,
}

fn default_search_type() -> String {
    "all".to_string()
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

// ==================== File Browsing ====================

/// Tree root handler (no path).
async fn tree_root(
    State(state): State<WebState>,
    Path((owner, repo, ref_name)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, WebError> {
    tree_handler(state, owner, repo, ref_name, String::new()).await
}

/// Tree path handler.
async fn tree_path(
    State(state): State<WebState>,
    Path((owner, repo, ref_name, path)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, WebError> {
    tree_handler(state, owner, repo, ref_name, path).await
}

/// Common tree handler.
async fn tree_handler(
    state: WebState,
    owner: String,
    repo: String,
    ref_name: String,
    path: String,
) -> Result<impl IntoResponse, WebError> {
    let repository = state
        .repos
        .get(&owner, &repo)
        .map_err(|_| WebError::NotFound(format!("Repository '{}/{}' not found", owner, repo)))?;

    // Resolve ref to commit
    let commit_id = resolve_ref(&repository, &ref_name)?;
    let commit = repository.objects.get(&commit_id)?;
    let tree_id = parse_commit_tree(&commit)?;

    // Navigate to the target tree
    let target_tree_id = if path.is_empty() {
        tree_id
    } else {
        navigate_tree(&repository, &tree_id, &path)?
    };

    let files = parse_tree_entries(&repository, &target_tree_id, &path)?;

    // Build breadcrumbs
    let breadcrumbs = build_breadcrumbs(&path);

    // Determine parent path
    let (show_parent, parent_path) = if path.is_empty() {
        (false, String::new())
    } else {
        let parts: Vec<&str> = path.split('/').collect();
        let parent = if parts.len() <= 1 {
            String::new()
        } else {
            parts[..parts.len() - 1].join("/")
        };
        (true, parent)
    };

    let template = TreeTemplate {
        owner,
        name: repo,
        ref_name,
        path,
        breadcrumbs,
        files,
        show_parent,
        parent_path,
    };

    Ok(Html(template.render()?))
}

/// Blob viewer handler.
async fn blob_view(
    State(state): State<WebState>,
    Path((owner, repo, ref_name, path)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let repository = state
        .repos
        .get(&owner, &repo)
        .map_err(|_| WebError::NotFound(format!("Repository '{}/{}' not found", owner, repo)))?;

    // Resolve ref to commit
    let commit_id = resolve_ref(&repository, &ref_name)?;
    let commit = repository.objects.get(&commit_id)?;
    let tree_id = parse_commit_tree(&commit)?;

    // Navigate to the blob
    let blob_id = navigate_to_blob(&repository, &tree_id, &path)?;
    let blob = repository.objects.get(&blob_id)?;

    // Get filename
    let filename = path.split('/').next_back().unwrap_or("file").to_string();

    // Detect language from extension
    let language = detect_language(&filename);

    // Check if binary
    let is_binary = is_binary_content(&blob.data);

    let content = if is_binary {
        String::new()
    } else {
        String::from_utf8_lossy(&blob.data).to_string()
    };

    let line_count = content.lines().count();
    let size = format_size(blob.data.len());

    // Build breadcrumbs
    let breadcrumbs = build_breadcrumbs(&path);

    let template = BlobTemplate {
        owner,
        name: repo,
        ref_name,
        path,
        filename,
        breadcrumbs,
        content,
        language,
        line_count,
        size,
        is_binary,
    };

    Ok(Html(template.render()?))
}

// ==================== Commits ====================

/// Commits list handler.
async fn commits_list(
    State(state): State<WebState>,
    Path((owner, repo, ref_name)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let repository = state
        .repos
        .get(&owner, &repo)
        .map_err(|_| WebError::NotFound(format!("Repository '{}/{}' not found", owner, repo)))?;

    // Resolve ref to commit
    let commit_id = resolve_ref(&repository, &ref_name)?;

    // Walk commit history
    let commits = walk_commits(&repository, &commit_id, 50)?;

    let template = CommitsTemplate {
        owner,
        name: repo,
        ref_name,
        commits,
    };

    Ok(Html(template.render()?))
}

/// Single commit view handler.
async fn commit_view(
    State(state): State<WebState>,
    Path((owner, repo, sha)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let repository = state
        .repos
        .get(&owner, &repo)
        .map_err(|_| WebError::NotFound(format!("Repository '{}/{}' not found", owner, repo)))?;

    let commit_id = ObjectId::from_hex(&sha)
        .map_err(|_| WebError::NotFound(format!("Invalid commit SHA: {}", sha)))?;

    let commit = repository
        .objects
        .get(&commit_id)
        .map_err(|_| WebError::NotFound(format!("Commit '{}' not found", sha)))?;

    let (message, author, parent_sha) = parse_commit_info(&commit)?;

    // For now, show empty file changes (proper diff requires tree comparison)
    let files_changed = Vec::new();

    let template = CommitTemplate {
        owner,
        name: repo,
        sha: sha.clone(),
        short_sha: sha[..7.min(sha.len())].to_string(),
        message,
        author,
        date: "recently".to_string(),
        parent_sha,
        files_changed,
        additions: 0,
        deletions: 0,
    };

    Ok(Html(template.render()?))
}

// ==================== Pull Request Detail ====================

/// Pull request detail view.
async fn pull_request_view(
    State(state): State<WebState>,
    Path((owner, repo, number)): Path<(String, String, u32)>,
) -> Result<impl IntoResponse, WebError> {
    let repo_key = format!("{}/{}", owner, repo);

    let pr = state
        .collaboration
        .get_pull_request(&repo_key, number)
        .map_err(|_| WebError::NotFound(format!("Pull request #{} not found", number)))?;

    // Get comments
    let comments: Vec<CommentView> = state
        .collaboration
        .list_pr_comments(&repo_key, number)
        .into_iter()
        .map(|c| CommentView {
            author: c.author.clone(),
            body_html: render_markdown(&c.body),
        })
        .collect();

    // Get reviews
    let reviews: Vec<ReviewView> = state
        .collaboration
        .list_reviews(&repo_key, number)
        .into_iter()
        .map(|r| ReviewView {
            author: r.author.clone(),
            state: format!("{:?}", r.state),
            body: r.body.clone(),
        })
        .collect();

    let template = PullRequestViewTemplate {
        owner,
        name: repo,
        number: pr.number,
        title: pr.title.clone(),
        description_html: render_markdown(&pr.description),
        author: pr.author.clone(),
        state: format!("{:?}", pr.state),
        source_branch: pr.source_branch.clone(),
        target_branch: pr.target_branch.clone(),
        merged_by: pr.merged_by.clone(),
        comments,
        reviews,
    };

    Ok(Html(template.render()?))
}

// ==================== Issue Detail ====================

/// Issue detail view.
async fn issue_view(
    State(state): State<WebState>,
    Path((owner, repo, number)): Path<(String, String, u32)>,
) -> Result<impl IntoResponse, WebError> {
    let repo_key = format!("{}/{}", owner, repo);

    let issue = state
        .collaboration
        .get_issue(&repo_key, number)
        .map_err(|_| WebError::NotFound(format!("Issue #{} not found", number)))?;

    // Get comments
    let comments: Vec<CommentView> = state
        .collaboration
        .list_issue_comments(&repo_key, number)
        .into_iter()
        .map(|c| CommentView {
            author: c.author.clone(),
            body_html: render_markdown(&c.body),
        })
        .collect();

    let template = IssueViewTemplate {
        owner,
        name: repo,
        number: issue.number,
        title: issue.title.clone(),
        description_html: render_markdown(&issue.description),
        author: issue.author.clone(),
        state: format!("{:?}", issue.state),
        labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
        closed_by: issue.closed_by.clone(),
        comments,
    };

    Ok(Html(template.render()?))
}

// ==================== CI/CD Actions ====================

/// Actions list page (workflows and runs).
async fn actions_list(
    State(state): State<WebState>,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let repo_key = format!("{}/{}", owner, repo);

    // Verify repository exists
    let _ = state
        .repos
        .get(&owner, &repo)
        .map_err(|_| WebError::NotFound(format!("Repository '{}' not found", repo_key)))?;

    // Get workflows
    let workflows: Vec<WorkflowSummary> = state
        .ci
        .workflows
        .list(&repo_key)
        .into_iter()
        .map(|w| WorkflowSummary {
            id: w.id.clone(),
            name: w.name.clone(),
            path: w.path.clone(),
        })
        .collect();

    // Get recent runs
    let runs: Vec<RunSummary> = state
        .ci
        .runs
        .list_by_repo(&repo_key, Some(20))
        .into_iter()
        .map(|r| RunSummary {
            id: r.id.clone(),
            workflow_name: r.workflow_name.clone(),
            number: r.number,
            status: format!("{:?}", r.status).to_lowercase(),
            conclusion: r.conclusion.map(|c| format!("{:?}", c).to_lowercase()),
            head_sha: r.head_sha.clone(),
            head_branch: r.head_branch.clone(),
            trigger_type: format!("{:?}", r.trigger.trigger_type),
        })
        .collect();

    let template = ActionsListTemplate {
        owner,
        repo,
        workflows,
        runs,
    };

    Ok(Html(template.render()?))
}

/// Action run detail page.
async fn actions_run(
    State(state): State<WebState>,
    Path((owner, repo, run_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let repo_key = format!("{}/{}", owner, repo);

    // Verify repository exists
    let _ = state
        .repos
        .get(&owner, &repo)
        .map_err(|_| WebError::NotFound(format!("Repository '{}' not found", repo_key)))?;

    // Get the run
    let run = state
        .ci
        .runs
        .get(&run_id)
        .ok_or_else(|| WebError::NotFound(format!("Run '{}' not found", run_id)))?;

    // Convert run to view model
    let run_view = RunDetailView {
        id: run.id.clone(),
        workflow_name: run.workflow_name.clone(),
        number: run.number,
        status: format!("{:?}", run.status).to_lowercase(),
        conclusion: run.conclusion.map(|c| format!("{:?}", c).to_lowercase()),
        head_sha: run.head_sha.clone(),
        head_branch: run.head_branch.clone(),
        trigger_type: format!("{:?}", run.trigger.trigger_type),
        actor: run.trigger.actor.clone(),
    };

    // Get jobs
    let jobs: Vec<JobView> = run
        .jobs
        .values()
        .map(|j| JobView {
            id: j.id.clone(),
            name: j.name.clone(),
            status: format!("{:?}", j.status).to_lowercase(),
            conclusion: j.conclusion.map(|c| format!("{:?}", c).to_lowercase()),
            steps: j
                .steps
                .iter()
                .map(|s| StepView {
                    number: s.number,
                    name: s.name.clone(),
                    status: format!("{:?}", s.status).to_lowercase(),
                    conclusion: s.conclusion.map(|c| format!("{:?}", c).to_lowercase()),
                })
                .collect(),
        })
        .collect();

    let template = ActionsRunTemplate {
        owner,
        repo,
        run: run_view,
        jobs,
    };

    Ok(Html(template.render()?))
}

// ==================== Helper Functions ====================

/// Resolve a ref name to a commit ID.
fn resolve_ref(repo: &guts_storage::Repository, ref_name: &str) -> Result<ObjectId, WebError> {
    // Try as a full ref first
    if let Ok(guts_storage::Reference::Direct(id)) =
        repo.refs.get(&format!("refs/heads/{}", ref_name))
    {
        return Ok(id);
    }

    // Try as HEAD
    if ref_name == "HEAD" {
        if let Ok(id) = repo.head() {
            return Ok(id);
        }
    }

    // Try as direct SHA
    if ref_name.len() == 40 {
        if let Ok(id) = ObjectId::from_hex(ref_name) {
            if repo.objects.contains(&id) {
                return Ok(id);
            }
        }
    }

    Err(WebError::NotFound(format!("Ref '{}' not found", ref_name)))
}

/// Parse tree ID from commit object.
fn parse_commit_tree(commit: &GitObject) -> Result<ObjectId, WebError> {
    if commit.object_type != ObjectType::Commit {
        return Err(WebError::Internal("Not a commit object".to_string()));
    }

    let content = String::from_utf8_lossy(&commit.data);
    for line in content.lines() {
        if let Some(tree_hex) = line.strip_prefix("tree ") {
            return ObjectId::from_hex(tree_hex.trim())
                .map_err(|e| WebError::Internal(e.to_string()));
        }
    }

    Err(WebError::Internal("No tree in commit".to_string()))
}

/// Parse commit info (message, author, parent).
fn parse_commit_info(commit: &GitObject) -> Result<(String, String, Option<String>), WebError> {
    if commit.object_type != ObjectType::Commit {
        return Err(WebError::Internal("Not a commit object".to_string()));
    }

    let content = String::from_utf8_lossy(&commit.data);
    let mut author = String::new();
    let mut parent = None;
    let mut in_headers = true;
    let mut message_lines = Vec::new();

    for line in content.lines() {
        if in_headers {
            if line.is_empty() {
                in_headers = false;
                continue;
            }
            if let Some(auth) = line.strip_prefix("author ") {
                // Extract just the name part
                if let Some(name_end) = auth.find('<') {
                    author = auth[..name_end].trim().to_string();
                } else {
                    author = auth.to_string();
                }
            } else if let Some(parent_hex) = line.strip_prefix("parent ") {
                parent = Some(parent_hex.trim().to_string());
            }
        } else {
            message_lines.push(line);
        }
    }

    let message = message_lines.join("\n");
    Ok((message, author, parent))
}

/// Navigate tree to find a subtree or blob.
fn navigate_tree(
    repo: &guts_storage::Repository,
    tree_id: &ObjectId,
    path: &str,
) -> Result<ObjectId, WebError> {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let mut current_id = *tree_id;

    for part in parts {
        let tree = repo.objects.get(&current_id)?;
        if tree.object_type != ObjectType::Tree {
            return Err(WebError::NotFound(format!("'{}' is not a directory", path)));
        }

        let entries = parse_tree_raw(&tree.data)?;
        let found = entries.iter().find(|(name, _, _)| name == part);

        match found {
            Some((_, _, id)) => current_id = *id,
            None => return Err(WebError::NotFound(format!("Path '{}' not found", path))),
        }
    }

    Ok(current_id)
}

/// Navigate to a blob specifically.
fn navigate_to_blob(
    repo: &guts_storage::Repository,
    tree_id: &ObjectId,
    path: &str,
) -> Result<ObjectId, WebError> {
    let id = navigate_tree(repo, tree_id, path)?;
    let obj = repo.objects.get(&id)?;

    if obj.object_type != ObjectType::Blob {
        return Err(WebError::NotFound(format!("'{}' is not a file", path)));
    }

    Ok(id)
}

/// Parse tree entries.
fn parse_tree_entries(
    repo: &guts_storage::Repository,
    tree_id: &ObjectId,
    base_path: &str,
) -> Result<Vec<FileEntry>, WebError> {
    let tree = repo.objects.get(tree_id)?;
    if tree.object_type != ObjectType::Tree {
        return Err(WebError::Internal("Not a tree object".to_string()));
    }

    let raw_entries = parse_tree_raw(&tree.data)?;
    let mut entries: Vec<FileEntry> = raw_entries
        .into_iter()
        .map(|(name, mode, _id)| {
            let is_dir = mode.starts_with("40") || mode == "40000";
            let path = if base_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", base_path, name)
            };
            FileEntry { name, path, is_dir }
        })
        .collect();

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Parse raw tree data.
fn parse_tree_raw(data: &[u8]) -> Result<Vec<(String, String, ObjectId)>, WebError> {
    let mut entries = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Find the space after mode
        let space_pos = data[i..]
            .iter()
            .position(|&b| b == b' ')
            .ok_or_else(|| WebError::Internal("Invalid tree format".to_string()))?;

        let mode = String::from_utf8_lossy(&data[i..i + space_pos]).to_string();
        i += space_pos + 1;

        // Find the null byte after name
        let null_pos = data[i..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| WebError::Internal("Invalid tree format".to_string()))?;

        let name = String::from_utf8_lossy(&data[i..i + null_pos]).to_string();
        i += null_pos + 1;

        // Read 20-byte SHA
        if i + 20 > data.len() {
            return Err(WebError::Internal("Invalid tree format".to_string()));
        }
        let mut sha_bytes = [0u8; 20];
        sha_bytes.copy_from_slice(&data[i..i + 20]);
        let id = ObjectId::from_bytes(sha_bytes);
        i += 20;

        entries.push((name, mode, id));
    }

    Ok(entries)
}

/// Walk commits from a starting point.
fn walk_commits(
    repo: &guts_storage::Repository,
    start: &ObjectId,
    limit: usize,
) -> Result<Vec<CommitSummary>, WebError> {
    let mut commits = Vec::new();
    let mut current = Some(*start);

    while let Some(commit_id) = current {
        if commits.len() >= limit {
            break;
        }

        let commit = repo.objects.get(&commit_id)?;
        let (message, author, parent) = parse_commit_info(&commit)?;

        let sha = commit_id.to_hex();
        commits.push(CommitSummary {
            sha: sha.clone(),
            short_sha: sha[..7.min(sha.len())].to_string(),
            message: message.lines().next().unwrap_or("").to_string(),
            author,
            date: "recently".to_string(),
        });

        current = parent.and_then(|p| ObjectId::from_hex(&p).ok());
    }

    Ok(commits)
}

/// Build breadcrumb navigation.
fn build_breadcrumbs(path: &str) -> Vec<Breadcrumb> {
    if path.is_empty() {
        return Vec::new();
    }

    let mut breadcrumbs = Vec::new();
    let mut current_path = String::new();

    for part in path.split('/').filter(|p| !p.is_empty()) {
        if current_path.is_empty() {
            current_path = part.to_string();
        } else {
            current_path = format!("{}/{}", current_path, part);
        }
        breadcrumbs.push(Breadcrumb {
            name: part.to_string(),
            path: current_path.clone(),
        });
    }

    breadcrumbs
}

/// Detect programming language from filename.
fn detect_language(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "typescript",
        "jsx" => "javascript",
        "go" => "go",
        "java" => "java",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "html" | "htm" => "html",
        "css" => "css",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" => "markdown",
        "sh" | "bash" => "bash",
        "sql" => "sql",
        "xml" => "xml",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" => "kotlin",
        "scala" => "scala",
        _ => "plaintext",
    }
    .to_string()
}

/// Check if content is binary.
fn is_binary_content(data: &[u8]) -> bool {
    // Check first 8000 bytes for null bytes
    let check_len = data.len().min(8000);
    data[..check_len].contains(&0)
}

/// Format file size.
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// ==================== Organizations ====================

/// Organization list page.
async fn org_list(State(state): State<WebState>) -> Result<impl IntoResponse, WebError> {
    let orgs: Vec<OrgSummary> = state
        .auth
        .list_organizations()
        .into_iter()
        .map(|org| OrgSummary {
            name: org.name.clone(),
            display_name: org.display_name.clone(),
            description: org.description.clone(),
            member_count: org.members.len(),
            team_count: org.teams.len(),
            repo_count: org.repos.len(),
        })
        .collect();

    let template = OrgListTemplate { orgs };
    Ok(Html(template.render()?))
}

/// Organization detail view.
async fn org_view(
    State(state): State<WebState>,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| WebError::NotFound(format!("Organization '{}' not found", org_name)))?;

    // Get teams for this org
    let teams: Vec<TeamSummary> = state
        .auth
        .list_teams(org.id)
        .into_iter()
        .map(|team| TeamSummary {
            name: team.name.clone(),
            description: team.description.clone(),
            member_count: team.members.len(),
            repo_count: team.repos.len(),
            permission: format!("{:?}", team.permission),
        })
        .collect();

    // Get organization's repositories
    let repos: Vec<RepoSummary> = state
        .repos
        .list()
        .into_iter()
        .filter(|r| r.owner == org_name)
        .map(|r| RepoSummary {
            owner: r.owner.clone(),
            name: r.name.clone(),
            description: String::new(),
            branch_count: 0,
        })
        .collect();

    // Get members
    let members: Vec<MemberView> = org
        .members
        .iter()
        .map(|m| MemberView {
            username: m.user.clone(),
            role: format!("{}", m.role),
        })
        .collect();

    let template = OrgViewTemplate {
        name: org.name.clone(),
        display_name: org.display_name.clone(),
        description: org.description.clone(),
        member_count: org.members.len(),
        team_count: org.teams.len(),
        repo_count: repos.len(),
        members,
        teams,
        repos,
    };

    Ok(Html(template.render()?))
}

/// Organization teams list.
async fn org_teams(
    State(state): State<WebState>,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| WebError::NotFound(format!("Organization '{}' not found", org_name)))?;

    let teams: Vec<TeamSummary> = state
        .auth
        .list_teams(org.id)
        .into_iter()
        .map(|team| TeamSummary {
            name: team.name.clone(),
            description: team.description.clone(),
            member_count: team.members.len(),
            repo_count: team.repos.len(),
            permission: format!("{:?}", team.permission),
        })
        .collect();

    let template = OrgTeamsTemplate {
        org_name: org.name.clone(),
        org_display_name: org.display_name.clone(),
        teams,
    };

    Ok(Html(template.render()?))
}

/// Team detail view.
async fn team_view(
    State(state): State<WebState>,
    Path((org_name, team_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let org = state
        .auth
        .get_organization_by_name(&org_name)
        .ok_or_else(|| WebError::NotFound(format!("Organization '{}' not found", org_name)))?;

    let team = state
        .auth
        .get_team_by_name(org.id, &team_name)
        .ok_or_else(|| WebError::NotFound(format!("Team '{}' not found", team_name)))?;

    // Get team members
    let members: Vec<String> = team.members.iter().cloned().collect();

    // Get team repositories
    let repos: Vec<String> = team.repos.iter().cloned().collect();

    let template = TeamViewTemplate {
        org_name: org.name.clone(),
        org_display_name: org.display_name.clone(),
        team_name: team.name.clone(),
        team_description: team.description.clone(),
        permission: format!("{:?}", team.permission),
        member_count: team.members.len(),
        repo_count: team.repos.len(),
        members,
        repos,
    };

    Ok(Html(template.render()?))
}

// ==================== User Profiles ====================

/// User profile page.
async fn user_profile(
    State(state): State<WebState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    // Get user's repositories
    let repos: Vec<RepoSummary> = state
        .repos
        .list()
        .into_iter()
        .filter(|r| r.owner == username)
        .map(|r| RepoSummary {
            owner: r.owner.clone(),
            name: r.name.clone(),
            description: String::new(),
            branch_count: 0,
        })
        .collect();

    // Get organizations the user belongs to
    let orgs: Vec<OrgSummary> = state
        .auth
        .list_user_organizations(&username)
        .into_iter()
        .map(|org| OrgSummary {
            name: org.name.clone(),
            display_name: org.display_name.clone(),
            description: org.description.clone(),
            member_count: org.members.len(),
            team_count: org.teams.len(),
            repo_count: org.repos.len(),
        })
        .collect();

    // Get teams the user belongs to
    let teams: Vec<UserTeamView> = state
        .auth
        .list_user_teams(&username)
        .into_iter()
        .filter_map(|team| {
            state
                .auth
                .get_organization(team.org_id)
                .map(|org| UserTeamView {
                    org_name: org.name.clone(),
                    team_name: team.name.clone(),
                    permission: format!("{:?}", team.permission),
                })
        })
        .collect();

    let template = UserProfileTemplate {
        username: username.clone(),
        repo_count: repos.len(),
        org_count: orgs.len(),
        repos,
        orgs,
        teams,
    };

    Ok(Html(template.render()?))
}

// ==================== Search ====================

/// Search handler.
async fn search(
    State(state): State<WebState>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, WebError> {
    let search_query = query.q.trim().to_lowercase();
    let search_type = query.search_type.as_str();

    // Search repositories
    let repo_results: Vec<RepoSummary> = if search_query.is_empty() {
        Vec::new()
    } else if search_type == "all" || search_type == "repositories" {
        state
            .repos
            .list()
            .into_iter()
            .filter(|r| {
                r.name.to_lowercase().contains(&search_query)
                    || r.owner.to_lowercase().contains(&search_query)
            })
            .map(|r| RepoSummary {
                owner: r.owner.clone(),
                name: r.name.clone(),
                description: String::new(),
                branch_count: 0,
            })
            .collect()
    } else {
        Vec::new()
    };

    // Search code within repositories
    let code_results: Vec<CodeSearchResult> = if search_query.is_empty() {
        Vec::new()
    } else if search_type == "all" || search_type == "code" {
        search_code(&state, &search_query, 50)?
    } else {
        Vec::new()
    };

    // Search issues
    let mut issue_results: Vec<IssueSearchResult> = Vec::new();
    let mut pr_results: Vec<IssueSearchResult> = Vec::new();

    if !search_query.is_empty() {
        for repo in state.repos.list() {
            let repo_key = format!("{}/{}", repo.owner, repo.name);

            // Search issues
            if search_type == "all" || search_type == "issues" {
                for issue in state.collaboration.list_issues(&repo_key, None) {
                    if issue.title.to_lowercase().contains(&search_query)
                        || issue.description.to_lowercase().contains(&search_query)
                    {
                        issue_results.push(IssueSearchResult {
                            repo_owner: repo.owner.clone(),
                            repo_name: repo.name.clone(),
                            number: issue.number,
                            title: issue.title.clone(),
                            author: issue.author.clone(),
                            state: format!("{:?}", issue.state),
                            labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
                            is_pull_request: false,
                        });
                    }
                }
            }

            // Search pull requests
            if search_type == "all" || search_type == "pullrequests" {
                for pr in state.collaboration.list_pull_requests(&repo_key, None) {
                    if pr.title.to_lowercase().contains(&search_query)
                        || pr.description.to_lowercase().contains(&search_query)
                    {
                        pr_results.push(IssueSearchResult {
                            repo_owner: repo.owner.clone(),
                            repo_name: repo.name.clone(),
                            number: pr.number,
                            title: pr.title.clone(),
                            author: pr.author.clone(),
                            state: format!("{:?}", pr.state),
                            labels: pr.labels.iter().map(|l| l.name.clone()).collect(),
                            is_pull_request: true,
                        });
                    }
                }
            }
        }
    }

    // Combine issue and PR results for display
    let combined_issue_results: Vec<IssueSearchResult> = match search_type {
        "issues" => issue_results.clone(),
        "pullrequests" => pr_results.clone(),
        _ => {
            let mut combined = issue_results.clone();
            combined.extend(pr_results.clone());
            combined
        }
    };

    let repo_count = repo_results.len();
    let code_count = code_results.len();
    let issue_count = issue_results.len();
    let pr_count = pr_results.len();
    let total_count = repo_count + code_count + issue_count + pr_count;

    let template = SearchTemplate {
        query: query.q.clone(),
        result_type: query.search_type.clone(),
        total_count,
        repo_results,
        code_results,
        issue_results: combined_issue_results,
        repo_count,
        code_count,
        issue_count,
        pr_count,
    };

    Ok(Html(template.render()?))
}

/// Context for code search operations.
struct CodeSearchContext<'a> {
    repo: &'a guts_storage::Repository,
    query: &'a str,
    owner: &'a str,
    repo_name: &'a str,
    results: &'a mut Vec<CodeSearchResult>,
    limit: usize,
}

/// Search code within repositories.
fn search_code(
    state: &WebState,
    query: &str,
    limit: usize,
) -> Result<Vec<CodeSearchResult>, WebError> {
    let mut results = Vec::new();

    for repo in state.repos.list() {
        if results.len() >= limit {
            break;
        }

        let repository = match state.repos.get(&repo.owner, &repo.name) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Get HEAD commit
        let commit_id = match repository.head() {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Get the commit and tree
        let commit = match repository.objects.get(&commit_id) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let tree_id = match parse_commit_tree(&commit) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Search files in the tree
        let mut ctx = CodeSearchContext {
            repo: &repository,
            query,
            owner: &repo.owner,
            repo_name: &repo.name,
            results: &mut results,
            limit,
        };
        search_tree_for_code(&mut ctx, &tree_id, "")?;
    }

    Ok(results)
}

/// Recursively search a tree for code matches.
fn search_tree_for_code(
    ctx: &mut CodeSearchContext<'_>,
    tree_id: &ObjectId,
    base_path: &str,
) -> Result<(), WebError> {
    if ctx.results.len() >= ctx.limit {
        return Ok(());
    }

    let tree = ctx.repo.objects.get(tree_id)?;
    if tree.object_type != ObjectType::Tree {
        return Ok(());
    }

    let entries = parse_tree_raw(&tree.data)?;

    for (name, mode, id) in entries {
        if ctx.results.len() >= ctx.limit {
            break;
        }

        let path = if base_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", base_path, name)
        };

        let is_dir = mode.starts_with("40") || mode == "40000";

        if is_dir {
            // Recurse into directory
            search_tree_for_code(ctx, &id, &path)?;
        } else {
            // Check if it's a searchable file
            let obj = match ctx.repo.objects.get(&id) {
                Ok(o) => o,
                Err(_) => continue,
            };

            // Skip binary files and large files
            if obj.data.len() > 1024 * 1024 || is_binary_content(&obj.data) {
                continue;
            }

            let content = String::from_utf8_lossy(&obj.data);
            let query_lower = ctx.query.to_lowercase();

            // Search each line
            for (line_idx, line) in content.lines().enumerate() {
                if ctx.results.len() >= ctx.limit {
                    break;
                }

                if line.to_lowercase().contains(&query_lower) {
                    let lines: Vec<&str> = content.lines().collect();
                    let line_num = line_idx + 1;

                    // Get context (2 lines before and after)
                    let start = line_idx.saturating_sub(2);
                    let end = (line_idx + 3).min(lines.len());

                    let context_before: Vec<String> = lines[start..line_idx]
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    let context_after: Vec<String> = lines[(line_idx + 1).min(lines.len())..end]
                        .iter()
                        .map(|s| s.to_string())
                        .collect();

                    ctx.results.push(CodeSearchResult {
                        repo_owner: ctx.owner.to_string(),
                        repo_name: ctx.repo_name.to_string(),
                        file_path: path.clone(),
                        line_number: line_num,
                        line_content: line.to_string(),
                        context_before,
                        context_after,
                        language: detect_language(&name),
                    });
                }
            }
        }
    }

    Ok(())
}

// ==================== Consensus Dashboard ====================

/// Consensus dashboard page.
async fn consensus_dashboard(State(state): State<WebState>) -> Result<impl IntoResponse, WebError> {
    // Get consensus status
    let (enabled, consensus_state, view, finalized_height, epoch, validator_count, validators) =
        if let Some(consensus) = &state.consensus {
            let engine_state = consensus.state();
            let view = consensus.view();
            let finalized_height = consensus.finalized_height();
            let validators_lock = consensus.validators();
            let validators_guard = validators_lock.read();
            let epoch = validators_guard.epoch();
            let validator_count = validators_guard.len();

            let validators: Vec<ValidatorSummary> = validators_guard
                .validators()
                .iter()
                .map(|v| {
                    let pk_hex = hex::encode(&v.pubkey.0);
                    ValidatorSummary {
                        name: v.name.clone(),
                        public_key: if pk_hex.len() > 16 {
                            format!("{}...", &pk_hex[..16])
                        } else {
                            pk_hex
                        },
                        stake: v.weight,
                        address: v.addr.to_string(),
                    }
                })
                .collect();

            (
                true,
                format!("{:?}", engine_state),
                view,
                finalized_height,
                epoch,
                validator_count,
                validators,
            )
        } else {
            (false, "Disabled".to_string(), 0, 0, 0, 0, Vec::new())
        };

    // Get mempool stats
    let (mempool_tx_count, mempool_oldest_age_secs) = if let Some(mempool) = &state.mempool {
        let stats = mempool.stats();
        (
            stats.transaction_count,
            stats.oldest_transaction_age.as_secs_f64(),
        )
    } else {
        (0, 0.0)
    };

    // Get recent blocks (placeholder - would need block store)
    let recent_blocks: Vec<BlockSummary> = Vec::new();

    let template = ConsensusDashboardTemplate {
        enabled,
        state: consensus_state,
        view,
        finalized_height,
        epoch,
        validator_count,
        mempool_tx_count,
        mempool_oldest_age_secs,
        validators,
        recent_blocks,
    };

    Ok(Html(template.render()?))
}

// ==================== API Documentation ====================

/// API documentation handler.
async fn api_docs() -> Result<impl IntoResponse, WebError> {
    let openapi_spec = generate_openapi_spec();

    let template = ApiDocsTemplate {
        openapi_spec: serde_json::to_string(&openapi_spec)
            .map_err(|e| WebError::Internal(e.to_string()))?,
    };

    Ok(Html(template.render()?))
}

/// Generate OpenAPI specification.
fn generate_openapi_spec() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Guts API",
            "description": "Decentralized code collaboration platform API. Guts provides a GitHub-like experience for decentralized, censorship-resistant code hosting.",
            "version": "1.0.0",
            "license": {
                "name": "MIT",
                "url": "https://opensource.org/licenses/MIT"
            }
        },
        "servers": [
            {
                "url": "/",
                "description": "Current server"
            }
        ],
        "tags": [
            {"name": "Health", "description": "Health check endpoints"},
            {"name": "Repositories", "description": "Repository management"},
            {"name": "Git", "description": "Git protocol endpoints"},
            {"name": "Pull Requests", "description": "Pull request management"},
            {"name": "Issues", "description": "Issue tracking"},
            {"name": "Reviews", "description": "Code review management"},
            {"name": "Organizations", "description": "Organization management"},
            {"name": "Teams", "description": "Team management"},
            {"name": "Collaborators", "description": "Repository collaborator management"},
            {"name": "Branch Protection", "description": "Branch protection rules"},
            {"name": "Webhooks", "description": "Webhook management"}
        ],
        "paths": {
            "/health": {
                "get": {
                    "tags": ["Health"],
                    "summary": "Health check",
                    "description": "Check if the API is running and get version info",
                    "operationId": "healthCheck",
                    "responses": {
                        "200": {
                            "description": "API is healthy",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": {"type": "string", "example": "ok"},
                                            "version": {"type": "string", "example": "0.1.0"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos": {
                "get": {
                    "tags": ["Repositories"],
                    "summary": "List repositories",
                    "description": "Get a list of all repositories",
                    "operationId": "listRepositories",
                    "responses": {
                        "200": {
                            "description": "List of repositories",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/RepoInfo"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Repositories"],
                    "summary": "Create repository",
                    "description": "Create a new repository",
                    "operationId": "createRepository",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateRepoRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Repository created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/RepoInfo"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}": {
                "get": {
                    "tags": ["Repositories"],
                    "summary": "Get repository",
                    "description": "Get repository details",
                    "operationId": "getRepository",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Repository details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/RepoInfo"}
                                }
                            }
                        },
                        "404": {"description": "Repository not found"}
                    }
                }
            },
            "/git/{owner}/{name}/info/refs": {
                "get": {
                    "tags": ["Git"],
                    "summary": "Reference advertisement",
                    "description": "Git smart HTTP reference advertisement",
                    "operationId": "gitInfoRefs",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {
                            "name": "service",
                            "in": "query",
                            "required": true,
                            "schema": {
                                "type": "string",
                                "enum": ["git-upload-pack", "git-receive-pack"]
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Git reference advertisement",
                            "content": {
                                "application/x-git-upload-pack-advertisement": {}
                            }
                        }
                    }
                }
            },
            "/git/{owner}/{name}/git-upload-pack": {
                "post": {
                    "tags": ["Git"],
                    "summary": "Upload pack (fetch/clone)",
                    "description": "Handle git fetch and clone operations",
                    "operationId": "gitUploadPack",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "requestBody": {
                        "content": {
                            "application/x-git-upload-pack-request": {}
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Pack data",
                            "content": {
                                "application/x-git-upload-pack-result": {}
                            }
                        }
                    }
                }
            },
            "/git/{owner}/{name}/git-receive-pack": {
                "post": {
                    "tags": ["Git"],
                    "summary": "Receive pack (push)",
                    "description": "Handle git push operations",
                    "operationId": "gitReceivePack",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "requestBody": {
                        "content": {
                            "application/x-git-receive-pack-request": {}
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Push result",
                            "content": {
                                "application/x-git-receive-pack-result": {}
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/pulls": {
                "get": {
                    "tags": ["Pull Requests"],
                    "summary": "List pull requests",
                    "description": "Get a list of pull requests for a repository",
                    "operationId": "listPullRequests",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {
                            "name": "state",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["open", "closed", "merged"]
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of pull requests",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/PullRequest"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Pull Requests"],
                    "summary": "Create pull request",
                    "description": "Create a new pull request",
                    "operationId": "createPullRequest",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreatePRRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Pull request created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/PullRequest"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/pulls/{number}": {
                "get": {
                    "tags": ["Pull Requests"],
                    "summary": "Get pull request",
                    "description": "Get pull request details",
                    "operationId": "getPullRequest",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Pull request details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/PullRequest"}
                                }
                            }
                        },
                        "404": {"description": "Pull request not found"}
                    }
                },
                "patch": {
                    "tags": ["Pull Requests"],
                    "summary": "Update pull request",
                    "description": "Update pull request title, description, or state",
                    "operationId": "updatePullRequest",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/UpdatePRRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Pull request updated",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/PullRequest"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/pulls/{number}/merge": {
                "post": {
                    "tags": ["Pull Requests"],
                    "summary": "Merge pull request",
                    "description": "Merge a pull request",
                    "operationId": "mergePullRequest",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/MergePRRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Pull request merged",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/PullRequest"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/pulls/{number}/comments": {
                "get": {
                    "tags": ["Pull Requests"],
                    "summary": "List PR comments",
                    "description": "Get comments on a pull request",
                    "operationId": "listPRComments",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "responses": {
                        "200": {
                            "description": "List of comments",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Comment"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Pull Requests"],
                    "summary": "Add PR comment",
                    "description": "Add a comment to a pull request",
                    "operationId": "addPRComment",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateCommentRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Comment added",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Comment"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/pulls/{number}/reviews": {
                "get": {
                    "tags": ["Reviews"],
                    "summary": "List reviews",
                    "description": "Get reviews on a pull request",
                    "operationId": "listReviews",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "responses": {
                        "200": {
                            "description": "List of reviews",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Review"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Reviews"],
                    "summary": "Submit review",
                    "description": "Submit a code review",
                    "operationId": "submitReview",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateReviewRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Review submitted",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Review"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/issues": {
                "get": {
                    "tags": ["Issues"],
                    "summary": "List issues",
                    "description": "Get a list of issues for a repository",
                    "operationId": "listIssues",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {
                            "name": "state",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["open", "closed"]
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of issues",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Issue"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Issues"],
                    "summary": "Create issue",
                    "description": "Create a new issue",
                    "operationId": "createIssue",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateIssueRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Issue created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Issue"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/issues/{number}": {
                "get": {
                    "tags": ["Issues"],
                    "summary": "Get issue",
                    "description": "Get issue details",
                    "operationId": "getIssue",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Issue details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Issue"}
                                }
                            }
                        },
                        "404": {"description": "Issue not found"}
                    }
                },
                "patch": {
                    "tags": ["Issues"],
                    "summary": "Update issue",
                    "description": "Update issue title, description, or state",
                    "operationId": "updateIssue",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/UpdateIssueRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Issue updated",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Issue"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/issues/{number}/comments": {
                "get": {
                    "tags": ["Issues"],
                    "summary": "List issue comments",
                    "description": "Get comments on an issue",
                    "operationId": "listIssueComments",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "responses": {
                        "200": {
                            "description": "List of comments",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Comment"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Issues"],
                    "summary": "Add issue comment",
                    "description": "Add a comment to an issue",
                    "operationId": "addIssueComment",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/number"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateCommentRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Comment added",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Comment"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/orgs": {
                "get": {
                    "tags": ["Organizations"],
                    "summary": "List organizations",
                    "description": "Get a list of all organizations",
                    "operationId": "listOrganizations",
                    "responses": {
                        "200": {
                            "description": "List of organizations",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Organization"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Organizations"],
                    "summary": "Create organization",
                    "description": "Create a new organization",
                    "operationId": "createOrganization",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateOrgRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Organization created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Organization"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/orgs/{org}": {
                "get": {
                    "tags": ["Organizations"],
                    "summary": "Get organization",
                    "description": "Get organization details",
                    "operationId": "getOrganization",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Organization details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Organization"}
                                }
                            }
                        },
                        "404": {"description": "Organization not found"}
                    }
                },
                "patch": {
                    "tags": ["Organizations"],
                    "summary": "Update organization",
                    "description": "Update organization details",
                    "operationId": "updateOrganization",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"}
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/UpdateOrgRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Organization updated",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Organization"}
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "tags": ["Organizations"],
                    "summary": "Delete organization",
                    "description": "Delete an organization",
                    "operationId": "deleteOrganization",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"}
                    ],
                    "responses": {
                        "204": {"description": "Organization deleted"}
                    }
                }
            },
            "/api/orgs/{org}/members": {
                "get": {
                    "tags": ["Organizations"],
                    "summary": "List members",
                    "description": "List organization members",
                    "operationId": "listOrgMembers",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"}
                    ],
                    "responses": {
                        "200": {
                            "description": "List of members",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/OrgMember"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Organizations"],
                    "summary": "Add member",
                    "description": "Add a member to organization",
                    "operationId": "addOrgMember",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AddOrgMemberRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Member added",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/OrgMember"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/orgs/{org}/teams": {
                "get": {
                    "tags": ["Teams"],
                    "summary": "List teams",
                    "description": "List teams in organization",
                    "operationId": "listTeams",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"}
                    ],
                    "responses": {
                        "200": {
                            "description": "List of teams",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Team"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Teams"],
                    "summary": "Create team",
                    "description": "Create a new team",
                    "operationId": "createTeam",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateTeamRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Team created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Team"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/orgs/{org}/teams/{team}": {
                "get": {
                    "tags": ["Teams"],
                    "summary": "Get team",
                    "description": "Get team details",
                    "operationId": "getTeam",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"},
                        {"$ref": "#/components/parameters/team"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Team details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Team"}
                                }
                            }
                        },
                        "404": {"description": "Team not found"}
                    }
                },
                "patch": {
                    "tags": ["Teams"],
                    "summary": "Update team",
                    "description": "Update team details",
                    "operationId": "updateTeam",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"},
                        {"$ref": "#/components/parameters/team"}
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/UpdateTeamRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Team updated",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Team"}
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "tags": ["Teams"],
                    "summary": "Delete team",
                    "description": "Delete a team",
                    "operationId": "deleteTeam",
                    "parameters": [
                        {"$ref": "#/components/parameters/org"},
                        {"$ref": "#/components/parameters/team"}
                    ],
                    "responses": {
                        "204": {"description": "Team deleted"}
                    }
                }
            },
            "/api/repos/{owner}/{name}/collaborators": {
                "get": {
                    "tags": ["Collaborators"],
                    "summary": "List collaborators",
                    "description": "List repository collaborators",
                    "operationId": "listCollaborators",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "responses": {
                        "200": {
                            "description": "List of collaborators",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Collaborator"}
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/collaborators/{user}": {
                "get": {
                    "tags": ["Collaborators"],
                    "summary": "Get collaborator",
                    "description": "Get collaborator details",
                    "operationId": "getCollaborator",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/user"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Collaborator details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Collaborator"}
                                }
                            }
                        },
                        "404": {"description": "Collaborator not found"}
                    }
                },
                "put": {
                    "tags": ["Collaborators"],
                    "summary": "Add/update collaborator",
                    "description": "Add or update a collaborator",
                    "operationId": "addCollaborator",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/user"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AddCollaboratorRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Collaborator added/updated",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Collaborator"}
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "tags": ["Collaborators"],
                    "summary": "Remove collaborator",
                    "description": "Remove a collaborator",
                    "operationId": "removeCollaborator",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/user"}
                    ],
                    "responses": {
                        "204": {"description": "Collaborator removed"}
                    }
                }
            },
            "/api/repos/{owner}/{name}/branches/{branch}/protection": {
                "get": {
                    "tags": ["Branch Protection"],
                    "summary": "Get branch protection",
                    "description": "Get branch protection rules",
                    "operationId": "getBranchProtection",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/branch"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Branch protection rules",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/BranchProtection"}
                                }
                            }
                        },
                        "404": {"description": "No protection rules"}
                    }
                },
                "put": {
                    "tags": ["Branch Protection"],
                    "summary": "Set branch protection",
                    "description": "Set or update branch protection rules",
                    "operationId": "setBranchProtection",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/branch"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/BranchProtectionRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Branch protection set",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/BranchProtection"}
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "tags": ["Branch Protection"],
                    "summary": "Remove branch protection",
                    "description": "Remove branch protection rules",
                    "operationId": "removeBranchProtection",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/branch"}
                    ],
                    "responses": {
                        "204": {"description": "Protection removed"}
                    }
                }
            },
            "/api/repos/{owner}/{name}/hooks": {
                "get": {
                    "tags": ["Webhooks"],
                    "summary": "List webhooks",
                    "description": "List repository webhooks",
                    "operationId": "listWebhooks",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "responses": {
                        "200": {
                            "description": "List of webhooks",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Webhook"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "tags": ["Webhooks"],
                    "summary": "Create webhook",
                    "description": "Create a new webhook",
                    "operationId": "createWebhook",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"}
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CreateWebhookRequest"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Webhook created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Webhook"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/hooks/{id}": {
                "get": {
                    "tags": ["Webhooks"],
                    "summary": "Get webhook",
                    "description": "Get webhook details",
                    "operationId": "getWebhook",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/id"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Webhook details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Webhook"}
                                }
                            }
                        },
                        "404": {"description": "Webhook not found"}
                    }
                },
                "patch": {
                    "tags": ["Webhooks"],
                    "summary": "Update webhook",
                    "description": "Update webhook configuration",
                    "operationId": "updateWebhook",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/id"}
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/UpdateWebhookRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Webhook updated",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Webhook"}
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "tags": ["Webhooks"],
                    "summary": "Delete webhook",
                    "description": "Delete a webhook",
                    "operationId": "deleteWebhook",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/id"}
                    ],
                    "responses": {
                        "204": {"description": "Webhook deleted"}
                    }
                }
            },
            "/api/repos/{owner}/{name}/hooks/{id}/ping": {
                "post": {
                    "tags": ["Webhooks"],
                    "summary": "Ping webhook",
                    "description": "Test a webhook by sending a ping event",
                    "operationId": "pingWebhook",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/id"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Ping result",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "id": {"type": "integer"},
                                            "url": {"type": "string"},
                                            "message": {"type": "string"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/repos/{owner}/{name}/permission/{user}": {
                "get": {
                    "tags": ["Collaborators"],
                    "summary": "Check permission",
                    "description": "Check a user's effective permission on a repository",
                    "operationId": "checkPermission",
                    "parameters": [
                        {"$ref": "#/components/parameters/owner"},
                        {"$ref": "#/components/parameters/name"},
                        {"$ref": "#/components/parameters/user"}
                    ],
                    "responses": {
                        "200": {
                            "description": "Permission level",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/PermissionResponse"}
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "parameters": {
                "owner": {
                    "name": "owner",
                    "in": "path",
                    "required": true,
                    "description": "Repository owner",
                    "schema": {"type": "string"}
                },
                "name": {
                    "name": "name",
                    "in": "path",
                    "required": true,
                    "description": "Repository name",
                    "schema": {"type": "string"}
                },
                "number": {
                    "name": "number",
                    "in": "path",
                    "required": true,
                    "description": "Issue or PR number",
                    "schema": {"type": "integer"}
                },
                "org": {
                    "name": "org",
                    "in": "path",
                    "required": true,
                    "description": "Organization name",
                    "schema": {"type": "string"}
                },
                "team": {
                    "name": "team",
                    "in": "path",
                    "required": true,
                    "description": "Team name",
                    "schema": {"type": "string"}
                },
                "user": {
                    "name": "user",
                    "in": "path",
                    "required": true,
                    "description": "Username",
                    "schema": {"type": "string"}
                },
                "branch": {
                    "name": "branch",
                    "in": "path",
                    "required": true,
                    "description": "Branch name",
                    "schema": {"type": "string"}
                },
                "id": {
                    "name": "id",
                    "in": "path",
                    "required": true,
                    "description": "Resource ID",
                    "schema": {"type": "integer"}
                }
            },
            "schemas": {
                "RepoInfo": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "owner": {"type": "string"}
                    },
                    "required": ["name", "owner"]
                },
                "CreateRepoRequest": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "owner": {"type": "string"}
                    },
                    "required": ["name", "owner"]
                },
                "PullRequest": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "number": {"type": "integer"},
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "author": {"type": "string"},
                        "state": {"type": "string", "enum": ["Open", "Closed", "Merged"]},
                        "source_branch": {"type": "string"},
                        "target_branch": {"type": "string"},
                        "source_commit": {"type": "string"},
                        "target_commit": {"type": "string"},
                        "labels": {"type": "array", "items": {"$ref": "#/components/schemas/Label"}},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"},
                        "merged_at": {"type": "integer", "nullable": true},
                        "merged_by": {"type": "string", "nullable": true}
                    }
                },
                "CreatePRRequest": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "author": {"type": "string"},
                        "source_branch": {"type": "string"},
                        "target_branch": {"type": "string"},
                        "source_commit": {"type": "string"},
                        "target_commit": {"type": "string"}
                    },
                    "required": ["title", "description", "author", "source_branch", "target_branch", "source_commit", "target_commit"]
                },
                "UpdatePRRequest": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "state": {"type": "string", "enum": ["open", "closed"]}
                    }
                },
                "MergePRRequest": {
                    "type": "object",
                    "properties": {
                        "merged_by": {"type": "string"}
                    },
                    "required": ["merged_by"]
                },
                "Issue": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "number": {"type": "integer"},
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "author": {"type": "string"},
                        "state": {"type": "string", "enum": ["Open", "Closed"]},
                        "labels": {"type": "array", "items": {"$ref": "#/components/schemas/Label"}},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"},
                        "closed_at": {"type": "integer", "nullable": true},
                        "closed_by": {"type": "string", "nullable": true}
                    }
                },
                "CreateIssueRequest": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "author": {"type": "string"},
                        "labels": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["title", "description", "author"]
                },
                "UpdateIssueRequest": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "state": {"type": "string", "enum": ["open", "closed"]},
                        "closed_by": {"type": "string"}
                    }
                },
                "Comment": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "author": {"type": "string"},
                        "body": {"type": "string"},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"}
                    }
                },
                "CreateCommentRequest": {
                    "type": "object",
                    "properties": {
                        "author": {"type": "string"},
                        "body": {"type": "string"}
                    },
                    "required": ["author", "body"]
                },
                "Review": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "pr_number": {"type": "integer"},
                        "author": {"type": "string"},
                        "state": {"type": "string", "enum": ["Pending", "Commented", "Approved", "ChangesRequested", "Dismissed"]},
                        "body": {"type": "string", "nullable": true},
                        "commit_id": {"type": "string"},
                        "created_at": {"type": "integer"}
                    }
                },
                "CreateReviewRequest": {
                    "type": "object",
                    "properties": {
                        "author": {"type": "string"},
                        "state": {"type": "string", "enum": ["approved", "changes_requested", "commented"]},
                        "body": {"type": "string"},
                        "commit_id": {"type": "string"}
                    },
                    "required": ["author", "state", "commit_id"]
                },
                "Label": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "color": {"type": "string"},
                        "description": {"type": "string", "nullable": true}
                    }
                },
                "Organization": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "name": {"type": "string"},
                        "display_name": {"type": "string"},
                        "description": {"type": "string", "nullable": true},
                        "created_by": {"type": "string"},
                        "member_count": {"type": "integer"},
                        "team_count": {"type": "integer"},
                        "repo_count": {"type": "integer"},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"}
                    }
                },
                "CreateOrgRequest": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "display_name": {"type": "string"},
                        "description": {"type": "string"},
                        "creator": {"type": "string"}
                    },
                    "required": ["name", "display_name", "creator"]
                },
                "UpdateOrgRequest": {
                    "type": "object",
                    "properties": {
                        "display_name": {"type": "string"},
                        "description": {"type": "string"}
                    }
                },
                "OrgMember": {
                    "type": "object",
                    "properties": {
                        "user": {"type": "string"},
                        "role": {"type": "string", "enum": ["Owner", "Admin", "Member"]},
                        "added_at": {"type": "integer"},
                        "added_by": {"type": "string"}
                    }
                },
                "AddOrgMemberRequest": {
                    "type": "object",
                    "properties": {
                        "user": {"type": "string"},
                        "role": {"type": "string", "enum": ["Owner", "Admin", "Member"]},
                        "added_by": {"type": "string"}
                    },
                    "required": ["user", "role", "added_by"]
                },
                "Team": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "org_id": {"type": "integer"},
                        "name": {"type": "string"},
                        "description": {"type": "string", "nullable": true},
                        "permission": {"type": "string", "enum": ["Read", "Write", "Admin"]},
                        "member_count": {"type": "integer"},
                        "repo_count": {"type": "integer"},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"}
                    }
                },
                "CreateTeamRequest": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "description": {"type": "string"},
                        "permission": {"type": "string", "enum": ["Read", "Write", "Admin"]},
                        "created_by": {"type": "string"}
                    },
                    "required": ["name", "permission", "created_by"]
                },
                "UpdateTeamRequest": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "description": {"type": "string"},
                        "permission": {"type": "string", "enum": ["Read", "Write", "Admin"]}
                    }
                },
                "Collaborator": {
                    "type": "object",
                    "properties": {
                        "user": {"type": "string"},
                        "permission": {"type": "string", "enum": ["Read", "Write", "Admin"]},
                        "added_by": {"type": "string"},
                        "created_at": {"type": "integer"}
                    }
                },
                "AddCollaboratorRequest": {
                    "type": "object",
                    "properties": {
                        "permission": {"type": "string", "enum": ["Read", "Write", "Admin"]},
                        "added_by": {"type": "string"}
                    },
                    "required": ["permission", "added_by"]
                },
                "BranchProtection": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "pattern": {"type": "string"},
                        "require_pr": {"type": "boolean"},
                        "required_reviews": {"type": "integer"},
                        "required_status_checks": {"type": "array", "items": {"type": "string"}},
                        "dismiss_stale_reviews": {"type": "boolean"},
                        "require_code_owner_review": {"type": "boolean"},
                        "restrict_pushes": {"type": "boolean"},
                        "allow_force_push": {"type": "boolean"},
                        "allow_deletion": {"type": "boolean"},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"}
                    }
                },
                "BranchProtectionRequest": {
                    "type": "object",
                    "properties": {
                        "require_pr": {"type": "boolean"},
                        "required_reviews": {"type": "integer"},
                        "required_status_checks": {"type": "array", "items": {"type": "string"}},
                        "dismiss_stale_reviews": {"type": "boolean"},
                        "require_code_owner_review": {"type": "boolean"},
                        "restrict_pushes": {"type": "boolean"},
                        "allow_force_push": {"type": "boolean"},
                        "allow_deletion": {"type": "boolean"}
                    }
                },
                "Webhook": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "url": {"type": "string"},
                        "events": {"type": "array", "items": {"type": "string"}},
                        "active": {"type": "boolean"},
                        "content_type": {"type": "string"},
                        "delivery_count": {"type": "integer"},
                        "failure_count": {"type": "integer"},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"}
                    }
                },
                "CreateWebhookRequest": {
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"},
                        "events": {"type": "array", "items": {"type": "string"}},
                        "secret": {"type": "string"}
                    },
                    "required": ["url", "events"]
                },
                "UpdateWebhookRequest": {
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"},
                        "secret": {"type": "string"},
                        "events": {"type": "array", "items": {"type": "string"}},
                        "active": {"type": "boolean"}
                    }
                },
                "PermissionResponse": {
                    "type": "object",
                    "properties": {
                        "user": {"type": "string"},
                        "permission": {"type": "string", "nullable": true, "enum": ["Read", "Write", "Admin"]},
                        "has_access": {"type": "boolean"}
                    }
                }
            }
        }
    })
}
