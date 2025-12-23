//! Bitbucket migration implementation.

use crate::client::{CreateIssueRequest, CreatePullRequestRequest, GutsClient};
use crate::error::{MigrationError, Result};
use crate::progress::{MigrationPhase, MigrationProgress};
use crate::types::{MigrationConfig, MigrationOptions, MigrationReport};

use reqwest::Client;
use serde::Deserialize;
use std::process::Command;
use tempfile::TempDir;
use tracing::{debug, info, warn};

/// Bitbucket API response types
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BitbucketRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    is_private: bool,
    mainbranch: Option<BitbucketBranch>,
    links: BitbucketLinks,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BitbucketBranch {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketLinks {
    clone: Vec<BitbucketCloneLink>,
}

#[derive(Debug, Deserialize)]
struct BitbucketCloneLink {
    href: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketPaginated<T> {
    values: Vec<T>,
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitbucketIssue {
    id: u64,
    title: String,
    content: Option<BitbucketContent>,
    state: String,
    reporter: Option<BitbucketUser>,
}

#[derive(Debug, Deserialize)]
struct BitbucketContent {
    raw: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitbucketPullRequest {
    id: u64,
    title: String,
    description: Option<String>,
    state: String,
    source: BitbucketPRBranch,
    destination: BitbucketPRBranch,
    author: BitbucketUser,
}

#[derive(Debug, Deserialize)]
struct BitbucketPRBranch {
    branch: BitbucketBranchInfo,
}

#[derive(Debug, Deserialize)]
struct BitbucketBranchInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketUser {
    display_name: Option<String>,
    nickname: Option<String>,
}

impl BitbucketUser {
    fn name(&self) -> &str {
        self.display_name
            .as_deref()
            .or(self.nickname.as_deref())
            .unwrap_or("unknown")
    }
}

/// Migrator for Bitbucket repositories.
pub struct BitbucketMigrator {
    bb_client: Client,
    bb_username: String,
    bb_app_password: String,
    guts_client: GutsClient,
    config: MigrationConfig,
    progress: MigrationProgress,
}

impl BitbucketMigrator {
    /// Create a new Bitbucket migrator.
    ///
    /// # Arguments
    ///
    /// * `username` - Bitbucket username
    /// * `app_password` - Bitbucket app password
    /// * `config` - Migration configuration
    pub fn new(username: &str, app_password: &str, config: MigrationConfig) -> Result<Self> {
        let bb_client = Client::builder()
            .user_agent("guts-migrate")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        let guts_client = GutsClient::new(&config.guts_url, config.guts_token.clone())?;

        Ok(Self {
            bb_client,
            bb_username: username.to_string(),
            bb_app_password: app_password.to_string(),
            guts_client,
            config,
            progress: MigrationProgress::new(),
        })
    }

    /// Set a progress callback.
    pub fn with_progress(mut self, progress: MigrationProgress) -> Self {
        self.progress = progress;
        self
    }

    /// Run the migration.
    pub async fn migrate(&self, options: MigrationOptions) -> Result<MigrationReport> {
        let mut report = MigrationReport::new();

        info!(
            "Starting Bitbucket migration for {}",
            self.config.source_repo
        );
        self.progress.set_phase(MigrationPhase::Initializing, 1);

        // Parse workspace/repo
        let (workspace, repo_slug) = self.parse_repo()?;

        // Step 1: Fetch repository info from Bitbucket
        self.progress.message("Fetching repository information...");
        let bb_repo = self.fetch_repo_info(&workspace, &repo_slug).await?;
        debug!("Fetched repo info: {:?}", bb_repo.name);

        // Step 2: Create repository on Guts
        self.progress.set_phase(MigrationPhase::CreatingRepository, 1);
        let target_owner = self.config.target_owner.as_deref().unwrap_or(&workspace);
        let target_name = self.config.target_name.as_deref().unwrap_or(&bb_repo.name);

        match self
            .guts_client
            .create_repo(target_name, bb_repo.description.as_deref(), bb_repo.is_private)
            .await
        {
            Ok(guts_repo) => {
                report.repo_created = true;
                report.guts_repo_url = Some(guts_repo.clone_url.clone());
                info!("Created repository on Guts: {}", guts_repo.clone_url);
            }
            Err(e) => {
                report.add_error("repository", &e.to_string(), true);
                return Ok(report);
            }
        }

        // Step 3: Mirror Git repository
        self.progress.set_phase(MigrationPhase::CloningRepository, 1);
        match self
            .mirror_git_repo(&bb_repo, target_owner, target_name)
            .await
        {
            Ok((branches, tags)) => {
                report.git_mirrored = true;
                report.branches_migrated = branches;
                report.tags_migrated = tags;
                info!("Git repository mirrored successfully");
            }
            Err(e) => {
                report.add_error("git", &e.to_string(), true);
                return Ok(report);
            }
        }

        // Step 4: Migrate issues (if issue tracker is enabled)
        if options.migrate_issues {
            match self
                .migrate_issues(&workspace, &repo_slug, target_owner, target_name, &options)
                .await
            {
                Ok(count) => {
                    report.issues_migrated = count;
                    info!("Migrated {count} issues");
                }
                Err(e) => {
                    // Bitbucket Cloud may not have issues enabled
                    report.add_warning(format!("Issues migration skipped: {e}"));
                    warn!("Failed to migrate issues: {e}");
                }
            }
        }

        // Step 5: Migrate pull requests
        if options.migrate_pull_requests {
            match self
                .migrate_pull_requests(
                    &workspace,
                    &repo_slug,
                    target_owner,
                    target_name,
                    &options,
                )
                .await
            {
                Ok(count) => {
                    report.prs_migrated = count;
                    info!("Migrated {count} pull requests");
                }
                Err(e) => {
                    report.add_error("pull_requests", &e.to_string(), false);
                    warn!("Failed to migrate pull requests: {e}");
                }
            }
        }

        self.progress.set_phase(MigrationPhase::Complete, 1);
        report.complete();

        Ok(report)
    }

    fn parse_repo(&self) -> Result<(String, String)> {
        let parts: Vec<&str> = self.config.source_repo.split('/').collect();
        if parts.len() != 2 {
            return Err(MigrationError::InvalidConfig(format!(
                "Invalid repository format: {}. Expected 'workspace/repo'",
                self.config.source_repo
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    async fn bitbucket_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self
            .bb_client
            .get(url)
            .basic_auth(&self.bb_username, Some(&self.bb_app_password))
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if response.status() == 404 {
            return Err(MigrationError::RepositoryNotFound(url.to_string()));
        }

        if response.status() == 401 {
            return Err(MigrationError::AuthenticationFailed(
                "Invalid Bitbucket credentials".to_string(),
            ));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MigrationError::ApiError(format!(
                "Bitbucket API error ({status}): {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| MigrationError::ApiError(e.to_string()))
    }

    async fn bitbucket_get_paginated<T: serde::de::DeserializeOwned>(
        &self,
        initial_url: &str,
    ) -> Result<Vec<T>> {
        let mut all_items = Vec::new();
        let mut url = Some(initial_url.to_string());

        while let Some(current_url) = url {
            let page: BitbucketPaginated<T> = self.bitbucket_get(&current_url).await?;
            all_items.extend(page.values);
            url = page.next;
        }

        Ok(all_items)
    }

    async fn fetch_repo_info(&self, workspace: &str, repo_slug: &str) -> Result<BitbucketRepo> {
        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{workspace}/{repo_slug}"
        );
        self.bitbucket_get(&url).await
    }

    async fn mirror_git_repo(
        &self,
        bb_repo: &BitbucketRepo,
        target_owner: &str,
        target_name: &str,
    ) -> Result<(usize, usize)> {
        let temp_dir = TempDir::new()?;
        let clone_path = temp_dir.path().join("repo");

        // Find HTTPS clone URL
        let clone_url = bb_repo
            .links
            .clone
            .iter()
            .find(|l| l.name == "https")
            .map(|l| &l.href)
            .ok_or_else(|| MigrationError::InvalidConfig("No HTTPS clone URL found".to_string()))?;

        // Embed credentials in URL
        let authenticated_url = clone_url.replace(
            "https://",
            &format!("https://{}:{}@", self.bb_username, self.bb_app_password),
        );

        let output = Command::new("git")
            .args(["clone", "--mirror", &authenticated_url])
            .arg(&clone_path)
            .output()?;

        if !output.status.success() {
            return Err(MigrationError::GitCloneFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Count branches and tags
        let branches_output = Command::new("git")
            .current_dir(&clone_path)
            .args(["branch", "-r"])
            .output()?;
        let branches = String::from_utf8_lossy(&branches_output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count();

        let tags_output = Command::new("git")
            .current_dir(&clone_path)
            .args(["tag"])
            .output()?;
        let tags = String::from_utf8_lossy(&tags_output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count();

        // Push to Guts
        self.progress.set_phase(MigrationPhase::PushingRepository, 1);

        let guts_url = format!(
            "{}/git/{}/{}.git",
            self.config.guts_url, target_owner, target_name
        );

        let output = Command::new("git")
            .current_dir(&clone_path)
            .args(["push", "--mirror", &guts_url])
            .output()?;

        if !output.status.success() {
            return Err(MigrationError::GitPushFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok((branches, tags))
    }

    async fn migrate_issues(
        &self,
        workspace: &str,
        repo_slug: &str,
        target_owner: &str,
        target_name: &str,
        _options: &MigrationOptions,
    ) -> Result<usize> {
        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{workspace}/{repo_slug}/issues"
        );
        let issues: Vec<BitbucketIssue> = self.bitbucket_get_paginated(&url).await?;

        self.progress
            .set_phase(MigrationPhase::MigratingIssues, issues.len() as u64);

        let mut count = 0;
        for issue in &issues {
            let body = issue
                .content
                .as_ref()
                .and_then(|c| c.raw.as_deref())
                .unwrap_or("");
            let reporter = issue
                .reporter
                .as_ref()
                .map(|r| r.name())
                .unwrap_or("unknown");

            let body_with_note = format!(
                "{body}\n\n---\n*Migrated from Bitbucket issue #{} by {}*",
                issue.id, reporter
            );

            match self
                .guts_client
                .create_issue(
                    target_owner,
                    target_name,
                    &CreateIssueRequest {
                        title: issue.title.clone(),
                        body: Some(body_with_note),
                        labels: vec![],
                        assignees: vec![],
                    },
                )
                .await
            {
                Ok(guts_issue) => {
                    // Close if not open on Bitbucket
                    if issue.state != "open" && issue.state != "new" {
                        let _ = self
                            .guts_client
                            .close_issue(target_owner, target_name, guts_issue.number)
                            .await;
                    }

                    count += 1;
                    self.progress
                        .increment(Some(&format!("Issue #{}", issue.id)));
                }
                Err(e) => {
                    debug!("Failed to create issue #{}: {e}", issue.id);
                }
            }
        }

        Ok(count)
    }

    async fn migrate_pull_requests(
        &self,
        workspace: &str,
        repo_slug: &str,
        target_owner: &str,
        target_name: &str,
        options: &MigrationOptions,
    ) -> Result<usize> {
        let state = if options.include_closed {
            "" // All states
        } else {
            "?state=OPEN"
        };
        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{workspace}/{repo_slug}/pullrequests{state}"
        );
        let prs: Vec<BitbucketPullRequest> = self.bitbucket_get_paginated(&url).await?;

        self.progress
            .set_phase(MigrationPhase::MigratingPullRequests, prs.len() as u64);

        let mut count = 0;
        for pr in &prs {
            let body = pr.description.as_deref().unwrap_or("");
            let body_with_note = format!(
                "{body}\n\n---\n*Migrated from Bitbucket PR #{} ({}) by {}*",
                pr.id, pr.state, pr.author.name()
            );

            match self
                .guts_client
                .create_pull_request(
                    target_owner,
                    target_name,
                    &CreatePullRequestRequest {
                        title: pr.title.clone(),
                        body: Some(body_with_note),
                        source_branch: pr.source.branch.name.clone(),
                        target_branch: pr.destination.branch.name.clone(),
                    },
                )
                .await
            {
                Ok(_guts_pr) => {
                    count += 1;
                    self.progress.increment(Some(&format!("PR #{}", pr.id)));
                }
                Err(e) => {
                    debug!("Failed to create PR #{}: {e}", pr.id);
                }
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo() {
        let config = MigrationConfig::new("workspace/repo", "http://localhost:8080");
        let migrator = BitbucketMigrator::new("user", "pass", config).unwrap();

        let (workspace, repo) = migrator.parse_repo().unwrap();
        assert_eq!(workspace, "workspace");
        assert_eq!(repo, "repo");
    }
}
