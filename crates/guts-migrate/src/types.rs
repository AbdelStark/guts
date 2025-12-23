//! Common types for migration operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a migration operation.
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Source repository identifier (e.g., "owner/repo").
    pub source_repo: String,

    /// Guts node API URL.
    pub guts_url: String,

    /// Guts API token for authentication.
    pub guts_token: Option<String>,

    /// Target repository name on Guts (defaults to source name).
    pub target_name: Option<String>,

    /// Target owner on Guts (defaults to authenticated user).
    pub target_owner: Option<String>,
}

impl MigrationConfig {
    /// Create a new migration configuration.
    pub fn new(source_repo: impl Into<String>, guts_url: impl Into<String>) -> Self {
        Self {
            source_repo: source_repo.into(),
            guts_url: guts_url.into(),
            guts_token: None,
            target_name: None,
            target_owner: None,
        }
    }

    /// Set the Guts API token.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.guts_token = Some(token.into());
        self
    }

    /// Set the target repository name.
    pub fn with_target_name(mut self, name: impl Into<String>) -> Self {
        self.target_name = Some(name.into());
        self
    }

    /// Set the target owner.
    pub fn with_target_owner(mut self, owner: impl Into<String>) -> Self {
        self.target_owner = Some(owner.into());
        self
    }
}

/// Options for controlling what gets migrated.
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Migrate issues.
    pub migrate_issues: bool,

    /// Migrate pull requests / merge requests.
    pub migrate_pull_requests: bool,

    /// Migrate releases and assets.
    pub migrate_releases: bool,

    /// Migrate wiki.
    pub migrate_wiki: bool,

    /// Migrate labels.
    pub migrate_labels: bool,

    /// Migrate milestones.
    pub migrate_milestones: bool,

    /// Include closed issues/PRs.
    pub include_closed: bool,

    /// Rewrite content links to point to Guts.
    pub rewrite_links: bool,

    /// Map of source usernames to Guts usernames.
    pub user_mapping: HashMap<String, String>,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            migrate_issues: true,
            migrate_pull_requests: true,
            migrate_releases: true,
            migrate_wiki: true,
            migrate_labels: true,
            migrate_milestones: true,
            include_closed: true,
            rewrite_links: true,
            user_mapping: HashMap::new(),
        }
    }
}

impl MigrationOptions {
    /// Enable or disable issue migration.
    pub fn with_issues(mut self, migrate: bool) -> Self {
        self.migrate_issues = migrate;
        self
    }

    /// Enable or disable pull request migration.
    pub fn with_pull_requests(mut self, migrate: bool) -> Self {
        self.migrate_pull_requests = migrate;
        self
    }

    /// Enable or disable release migration.
    pub fn with_releases(mut self, migrate: bool) -> Self {
        self.migrate_releases = migrate;
        self
    }

    /// Enable or disable wiki migration.
    pub fn with_wiki(mut self, migrate: bool) -> Self {
        self.migrate_wiki = migrate;
        self
    }

    /// Enable or disable label migration.
    pub fn with_labels(mut self, migrate: bool) -> Self {
        self.migrate_labels = migrate;
        self
    }

    /// Enable or disable milestone migration.
    pub fn with_milestones(mut self, migrate: bool) -> Self {
        self.migrate_milestones = migrate;
        self
    }

    /// Enable or disable including closed items.
    pub fn with_closed(mut self, include: bool) -> Self {
        self.include_closed = include;
        self
    }

    /// Enable or disable link rewriting.
    pub fn with_link_rewriting(mut self, rewrite: bool) -> Self {
        self.rewrite_links = rewrite;
        self
    }

    /// Add a user mapping.
    pub fn with_user_mapping(
        mut self,
        source_user: impl Into<String>,
        guts_user: impl Into<String>,
    ) -> Self {
        self.user_mapping
            .insert(source_user.into(), guts_user.into());
        self
    }
}

/// Report of a completed migration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Whether the repository was created on Guts.
    pub repo_created: bool,

    /// Whether git data was mirrored successfully.
    pub git_mirrored: bool,

    /// Number of branches migrated.
    pub branches_migrated: usize,

    /// Number of tags migrated.
    pub tags_migrated: usize,

    /// Number of issues migrated.
    pub issues_migrated: usize,

    /// Number of pull requests migrated.
    pub prs_migrated: usize,

    /// Number of releases migrated.
    pub releases_migrated: usize,

    /// Number of release assets migrated.
    pub assets_migrated: usize,

    /// Whether wiki was migrated.
    pub wiki_migrated: bool,

    /// Number of labels migrated.
    pub labels_migrated: usize,

    /// Number of milestones migrated.
    pub milestones_migrated: usize,

    /// Errors encountered during migration.
    pub errors: Vec<MigrationErrorInfo>,

    /// Warnings generated during migration.
    pub warnings: Vec<String>,

    /// Start time of migration.
    pub started_at: Option<DateTime<Utc>>,

    /// End time of migration.
    pub completed_at: Option<DateTime<Utc>>,

    /// URL of the migrated repository on Guts.
    pub guts_repo_url: Option<String>,
}

impl MigrationReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            started_at: Some(Utc::now()),
            ..Default::default()
        }
    }

    /// Mark the migration as complete.
    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
    }

    /// Check if the migration was successful (no critical errors).
    pub fn is_successful(&self) -> bool {
        self.repo_created
            && self.git_mirrored
            && self.errors.iter().all(|e| !e.is_critical)
    }

    /// Get the total number of items migrated.
    pub fn total_items_migrated(&self) -> usize {
        self.issues_migrated
            + self.prs_migrated
            + self.releases_migrated
            + self.labels_migrated
            + self.milestones_migrated
    }

    /// Add an error to the report.
    pub fn add_error(&mut self, category: &str, message: &str, is_critical: bool) {
        self.errors.push(MigrationErrorInfo {
            category: category.to_string(),
            message: message.to_string(),
            is_critical,
        });
    }

    /// Add a warning to the report.
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Get the duration of the migration.
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Print a summary of the migration.
    pub fn print_summary(&self) {
        println!("\n=== Migration Summary ===\n");
        println!(
            "Repository created: {}",
            if self.repo_created { "✓" } else { "✗" }
        );
        println!(
            "Git data mirrored:  {}",
            if self.git_mirrored { "✓" } else { "✗" }
        );

        if self.branches_migrated > 0 || self.tags_migrated > 0 {
            println!("Branches migrated:  {}", self.branches_migrated);
            println!("Tags migrated:      {}", self.tags_migrated);
        }

        println!("Issues migrated:    {}", self.issues_migrated);
        println!("PRs migrated:       {}", self.prs_migrated);
        println!("Releases migrated:  {}", self.releases_migrated);
        println!("Assets migrated:    {}", self.assets_migrated);
        println!(
            "Wiki migrated:      {}",
            if self.wiki_migrated { "✓" } else { "N/A" }
        );
        println!("Labels migrated:    {}", self.labels_migrated);
        println!("Milestones:         {}", self.milestones_migrated);

        if let Some(url) = &self.guts_repo_url {
            println!("\nRepository URL: {url}");
        }

        if let Some(duration) = self.duration() {
            println!(
                "\nCompleted in {} seconds",
                duration.num_seconds()
            );
        }

        if !self.errors.is_empty() {
            println!("\nErrors ({}):", self.errors.len());
            for error in &self.errors {
                let severity = if error.is_critical {
                    "CRITICAL"
                } else {
                    "WARNING"
                };
                println!("  [{severity}] {}: {}", error.category, error.message);
            }
        }

        if !self.warnings.is_empty() {
            println!("\nWarnings ({}):", self.warnings.len());
            for warning in &self.warnings {
                println!("  - {warning}");
            }
        }

        let status = if self.is_successful() {
            "SUCCESS"
        } else {
            "FAILED"
        };
        println!("\nOverall Status: {status}");
    }
}

/// Information about an error that occurred during migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationErrorInfo {
    /// Category of the error (e.g., "issues", "git", "api").
    pub category: String,

    /// Error message.
    pub message: String,

    /// Whether this error is critical (blocks migration success).
    pub is_critical: bool,
}

/// Source platform types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourcePlatform {
    /// GitHub.
    GitHub,
    /// GitLab.
    GitLab,
    /// Bitbucket.
    Bitbucket,
}

impl std::fmt::Display for SourcePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "GitHub"),
            Self::GitLab => write!(f, "GitLab"),
            Self::Bitbucket => write!(f, "Bitbucket"),
        }
    }
}

/// Issue state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

/// Pull request state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

/// Migrated issue representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigratedIssue {
    pub source_number: u64,
    pub guts_number: u64,
    pub title: String,
    pub state: IssueState,
}

/// Migrated pull request representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigratedPullRequest {
    pub source_number: u64,
    pub guts_number: u64,
    pub title: String,
    pub state: PullRequestState,
}

/// Migrated release representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigratedRelease {
    pub tag_name: String,
    pub name: String,
    pub assets_count: usize,
}
