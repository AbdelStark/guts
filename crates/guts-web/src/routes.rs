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
