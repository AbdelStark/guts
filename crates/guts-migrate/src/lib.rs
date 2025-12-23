//! # Guts Migration Tools
//!
//! This crate provides migration tools for importing repositories from GitHub, GitLab,
//! and Bitbucket to the Guts decentralized code collaboration platform.
//!
//! ## Features
//!
//! - **GitHub Migration**: Full repository migration including issues, PRs, releases
//! - **GitLab Migration**: Project migration with merge requests and issues
//! - **Bitbucket Migration**: Repository migration with pull requests
//! - **Verification**: Post-migration verification to ensure data integrity
//! - **Progress Tracking**: Real-time progress reporting with ETA
//!
//! ## Example
//!
//! ```rust,ignore
//! use guts_migrate::{GitHubMigrator, MigrationConfig, MigrationOptions};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = MigrationConfig {
//!         source_repo: "owner/repo".to_string(),
//!         guts_url: "https://api.guts.network".to_string(),
//!         guts_token: Some("guts_xxx".to_string()),
//!     };
//!
//!     let options = MigrationOptions::default()
//!         .with_issues(true)
//!         .with_pull_requests(true)
//!         .with_releases(true);
//!
//!     let migrator = GitHubMigrator::new("github_token", config)?;
//!     let report = migrator.migrate(options).await?;
//!
//!     report.print_summary();
//!     Ok(())
//! }
//! ```

pub mod bitbucket;
pub mod client;
pub mod error;
pub mod github;
pub mod gitlab;
pub mod progress;
pub mod types;
pub mod verify;

// Re-export main types
pub use client::GutsClient;
pub use error::{MigrationError, Result};
pub use github::GitHubMigrator;
pub use gitlab::GitLabMigrator;
pub use bitbucket::BitbucketMigrator;
pub use progress::{MigrationProgress, ProgressCallback};
pub use types::*;
pub use verify::MigrationVerifier;

/// Version of the migration tools.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_options_builder() {
        let options = MigrationOptions::default()
            .with_issues(true)
            .with_pull_requests(true)
            .with_releases(false)
            .with_wiki(false);

        assert!(options.migrate_issues);
        assert!(options.migrate_pull_requests);
        assert!(!options.migrate_releases);
        assert!(!options.migrate_wiki);
    }

    #[test]
    fn test_migration_report_summary() {
        let mut report = MigrationReport::new();
        report.repo_created = true;
        report.git_mirrored = true;
        report.issues_migrated = 10;
        report.prs_migrated = 5;

        assert!(report.is_successful());
        assert_eq!(report.total_items_migrated(), 15);
    }
}
