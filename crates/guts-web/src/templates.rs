//! Askama template definitions.

use askama::Template;
use serde::Serialize;

/// Repository summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct RepoSummary {
    pub owner: String,
    pub name: String,
    pub description: String,
    pub branch_count: usize,
}

/// File entry for directory listings.
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

/// Breadcrumb navigation item.
#[derive(Debug, Clone, Serialize)]
pub struct Breadcrumb {
    pub name: String,
    pub path: String,
}

/// Pull request summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct PullRequestSummary {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub state: String,
}

/// Issue summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct IssueSummary {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub state: String,
    pub labels: Vec<String>,
}

/// Landing page template.
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub repos: Vec<RepoSummary>,
}

/// Explore repositories page.
#[derive(Template)]
#[template(path = "explore.html")]
pub struct ExploreTemplate {
    pub repos: Vec<RepoSummary>,
}

/// Repository home page.
#[derive(Template)]
#[template(path = "repo/view.html")]
pub struct RepoViewTemplate {
    pub owner: String,
    pub name: String,
    pub description: String,
    pub default_branch: String,
    pub branch_count: usize,
    pub issue_count: usize,
    pub pr_count: usize,
    pub clone_url: String,
    pub files: Vec<FileEntry>,
    pub readme_html: Option<String>,
}

/// Directory tree browser.
#[allow(dead_code)]
#[derive(Template)]
#[template(path = "repo/tree.html")]
pub struct TreeTemplate {
    pub owner: String,
    pub name: String,
    pub ref_name: String,
    pub path: String,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub files: Vec<FileEntry>,
    pub show_parent: bool,
    pub parent_path: String,
}

/// File blob viewer.
#[allow(dead_code)]
#[derive(Template)]
#[template(path = "repo/blob.html")]
pub struct BlobTemplate {
    pub owner: String,
    pub name: String,
    pub ref_name: String,
    pub path: String,
    pub filename: String,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub content: String,
    pub language: String,
    pub line_count: usize,
    pub size: String,
    pub is_binary: bool,
}

/// Pull request list page.
#[derive(Template)]
#[template(path = "pr/list.html")]
pub struct PullRequestListTemplate {
    pub owner: String,
    pub name: String,
    pub state: String,
    pub open_count: usize,
    pub closed_count: usize,
    pub pull_requests: Vec<PullRequestSummary>,
}

/// Issue list page.
#[derive(Template)]
#[template(path = "issue/list.html")]
pub struct IssueListTemplate {
    pub owner: String,
    pub name: String,
    pub state: String,
    pub open_count: usize,
    pub closed_count: usize,
    pub issues: Vec<IssueSummary>,
}
