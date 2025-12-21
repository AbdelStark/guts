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

/// Commit summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct CommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

/// File change in a commit.
#[derive(Debug, Clone, Serialize)]
pub struct FileChange {
    pub path: String,
    pub status: String,
    pub additions: usize,
    pub deletions: usize,
    pub diff: Option<String>,
}

/// Comment with rendered body.
#[derive(Debug, Clone, Serialize)]
pub struct CommentView {
    pub author: String,
    pub body_html: String,
}

/// Review with state.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewView {
    pub author: String,
    pub state: String,
    pub body: Option<String>,
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

/// Commit history page.
#[derive(Template)]
#[template(path = "repo/commits.html")]
pub struct CommitsTemplate {
    pub owner: String,
    pub name: String,
    pub ref_name: String,
    pub commits: Vec<CommitSummary>,
}

/// Single commit view page.
#[derive(Template)]
#[template(path = "repo/commit.html")]
pub struct CommitTemplate {
    pub owner: String,
    pub name: String,
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub parent_sha: Option<String>,
    pub files_changed: Vec<FileChange>,
    pub additions: usize,
    pub deletions: usize,
}

/// Pull request detail page.
#[derive(Template)]
#[template(path = "pr/view.html")]
pub struct PullRequestViewTemplate {
    pub owner: String,
    pub name: String,
    pub number: u32,
    pub title: String,
    pub description_html: String,
    pub author: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub merged_by: Option<String>,
    pub comments: Vec<CommentView>,
    pub reviews: Vec<ReviewView>,
}

/// Issue detail page.
#[derive(Template)]
#[template(path = "issue/view.html")]
pub struct IssueViewTemplate {
    pub owner: String,
    pub name: String,
    pub number: u32,
    pub title: String,
    pub description_html: String,
    pub author: String,
    pub state: String,
    pub labels: Vec<String>,
    pub closed_by: Option<String>,
    pub comments: Vec<CommentView>,
}

// ==================== Organization Templates ====================

/// Organization summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct OrgSummary {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub team_count: usize,
    pub repo_count: usize,
}

/// Team summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct TeamSummary {
    pub name: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub repo_count: usize,
    pub permission: String,
}

/// Member view for org members.
#[derive(Debug, Clone, Serialize)]
pub struct MemberView {
    pub username: String,
    pub role: String,
}

/// Team view for user profiles (includes org context).
#[derive(Debug, Clone, Serialize)]
pub struct UserTeamView {
    pub org_name: String,
    pub team_name: String,
    pub permission: String,
}

/// Organization list page.
#[derive(Template)]
#[template(path = "org/list.html")]
pub struct OrgListTemplate {
    pub orgs: Vec<OrgSummary>,
}

/// Organization detail page.
#[derive(Template)]
#[template(path = "org/view.html")]
pub struct OrgViewTemplate {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub team_count: usize,
    pub repo_count: usize,
    pub members: Vec<MemberView>,
    pub teams: Vec<TeamSummary>,
    pub repos: Vec<RepoSummary>,
}

/// Organization teams page.
#[derive(Template)]
#[template(path = "org/teams.html")]
pub struct OrgTeamsTemplate {
    pub org_name: String,
    pub org_display_name: String,
    pub teams: Vec<TeamSummary>,
}

/// Team detail page.
#[derive(Template)]
#[template(path = "org/team.html")]
pub struct TeamViewTemplate {
    pub org_name: String,
    pub org_display_name: String,
    pub team_name: String,
    pub team_description: Option<String>,
    pub permission: String,
    pub member_count: usize,
    pub repo_count: usize,
    pub members: Vec<String>,
    pub repos: Vec<String>,
}

// ==================== User Profile Templates ====================

/// User profile page.
#[derive(Template)]
#[template(path = "user/profile.html")]
pub struct UserProfileTemplate {
    pub username: String,
    pub repo_count: usize,
    pub org_count: usize,
    pub repos: Vec<RepoSummary>,
    pub orgs: Vec<OrgSummary>,
    pub teams: Vec<UserTeamView>,
}

// ==================== Search Templates ====================

/// Search result types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SearchResultType {
    Repository,
    Code,
    Issue,
    PullRequest,
}

impl std::fmt::Display for SearchResultType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchResultType::Repository => write!(f, "repository"),
            SearchResultType::Code => write!(f, "code"),
            SearchResultType::Issue => write!(f, "issue"),
            SearchResultType::PullRequest => write!(f, "pull_request"),
        }
    }
}

/// Code search result.
#[derive(Debug, Clone, Serialize)]
pub struct CodeSearchResult {
    pub repo_owner: String,
    pub repo_name: String,
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    pub language: String,
}

/// Issue/PR search result.
#[derive(Debug, Clone, Serialize)]
pub struct IssueSearchResult {
    pub repo_owner: String,
    pub repo_name: String,
    pub number: u32,
    pub title: String,
    pub author: String,
    pub state: String,
    pub labels: Vec<String>,
    pub is_pull_request: bool,
}

/// Search page template.
#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
    pub query: String,
    pub result_type: String,
    pub total_count: usize,
    pub repo_results: Vec<RepoSummary>,
    pub code_results: Vec<CodeSearchResult>,
    pub issue_results: Vec<IssueSearchResult>,
    pub repo_count: usize,
    pub code_count: usize,
    pub issue_count: usize,
    pub pr_count: usize,
}

// ==================== API Documentation Templates ====================

/// API Documentation page.
#[derive(Template)]
#[template(path = "api/docs.html")]
pub struct ApiDocsTemplate {
    pub openapi_spec: String,
}

// ==================== CI/CD Actions Templates ====================

/// Workflow summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub name: String,
    pub path: String,
}

/// Run summary for lists.
#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub id: String,
    pub workflow_name: String,
    pub number: u32,
    pub status: String,
    pub conclusion: Option<String>,
    pub head_sha: String,
    pub head_branch: Option<String>,
    pub trigger_type: String,
}

/// Run detail view.
#[derive(Debug, Clone, Serialize)]
pub struct RunDetailView {
    pub id: String,
    pub workflow_name: String,
    pub number: u32,
    pub status: String,
    pub conclusion: Option<String>,
    pub head_sha: String,
    pub head_branch: Option<String>,
    pub trigger_type: String,
    pub actor: String,
}

/// Job view for run details.
#[derive(Debug, Clone, Serialize)]
pub struct JobView {
    pub id: String,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub steps: Vec<StepView>,
}

/// Step view for job details.
#[derive(Debug, Clone, Serialize)]
pub struct StepView {
    pub number: u32,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

/// Actions list page (workflows and runs).
#[derive(Template)]
#[template(path = "actions/list.html")]
pub struct ActionsListTemplate {
    pub owner: String,
    pub repo: String,
    pub workflows: Vec<WorkflowSummary>,
    pub runs: Vec<RunSummary>,
}

/// Actions run detail page.
#[derive(Template)]
#[template(path = "actions/run.html")]
pub struct ActionsRunTemplate {
    pub owner: String,
    pub repo: String,
    pub run: RunDetailView,
    pub jobs: Vec<JobView>,
}
