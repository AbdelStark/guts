//! GitHub migration implementation.

use crate::client::{
    CreateIssueRequest, CreatePullRequestRequest, CreateReleaseRequest, GutsClient,
};
use crate::error::{MigrationError, Result};
use crate::progress::{MigrationPhase, MigrationProgress};
use crate::types::{MigrationConfig, MigrationOptions, MigrationReport};

use reqwest::Client;
use serde::Deserialize;
use std::process::Command;
use tempfile::TempDir;
use tracing::{debug, info, warn};

/// GitHub API response types
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRepo {
    name: String,
    description: Option<String>,
    private: bool,
    clone_url: String,
    has_wiki: bool,
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    labels: Vec<GitHubLabel>,
    user: GitHubUser,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequest {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    merged: bool,
    head: GitHubRef,
    base: GitHubRef,
    user: GitHubUser,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRef {
    #[serde(rename = "ref")]
    ref_name: String,
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GitHubLabel {
    name: String,
    color: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    prerelease: bool,
    draft: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubAsset {
    name: String,
    content_type: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubComment {
    body: String,
    user: GitHubUser,
}

/// Migrator for GitHub repositories.
pub struct GitHubMigrator {
    github_client: Client,
    github_token: String,
    guts_client: GutsClient,
    config: MigrationConfig,
    progress: MigrationProgress,
}

impl GitHubMigrator {
    /// Create a new GitHub migrator.
    pub fn new(github_token: &str, config: MigrationConfig) -> Result<Self> {
        let github_client = Client::builder()
            .user_agent("guts-migrate")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        let guts_client = GutsClient::new(&config.guts_url, config.guts_token.clone())?;

        Ok(Self {
            github_client,
            github_token: github_token.to_string(),
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

        info!("Starting GitHub migration for {}", self.config.source_repo);
        self.progress.set_phase(MigrationPhase::Initializing, 1);

        // Parse owner/repo
        let (owner, repo_name) = self.parse_repo()?;

        // Step 1: Fetch repository info from GitHub
        self.progress.message("Fetching repository information...");
        let gh_repo = self.fetch_repo_info(&owner, &repo_name).await?;
        debug!("Fetched repo info: {:?}", gh_repo.name);

        // Step 2: Create repository on Guts
        self.progress
            .set_phase(MigrationPhase::CreatingRepository, 1);
        let target_owner = self.config.target_owner.as_deref().unwrap_or(&owner);
        let target_name = self.config.target_name.as_deref().unwrap_or(&gh_repo.name);

        match self
            .guts_client
            .create_repo(target_name, gh_repo.description.as_deref(), gh_repo.private)
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
        self.progress
            .set_phase(MigrationPhase::CloningRepository, 1);
        match self
            .mirror_git_repo(&gh_repo, target_owner, target_name)
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

        // Step 4: Migrate labels
        if options.migrate_labels {
            self.progress.set_phase(MigrationPhase::MigratingLabels, 1);
            match self
                .migrate_labels(&owner, &repo_name, target_owner, target_name)
                .await
            {
                Ok(count) => {
                    report.labels_migrated = count;
                    info!("Migrated {count} labels");
                }
                Err(e) => {
                    report.add_error("labels", &e.to_string(), false);
                    warn!("Failed to migrate labels: {e}");
                }
            }
        }

        // Step 5: Migrate issues
        if options.migrate_issues {
            match self
                .migrate_issues(&owner, &repo_name, target_owner, target_name, &options)
                .await
            {
                Ok(count) => {
                    report.issues_migrated = count;
                    info!("Migrated {count} issues");
                }
                Err(e) => {
                    report.add_error("issues", &e.to_string(), false);
                    warn!("Failed to migrate issues: {e}");
                }
            }
        }

        // Step 6: Migrate pull requests
        if options.migrate_pull_requests {
            match self
                .migrate_pull_requests(&owner, &repo_name, target_owner, target_name, &options)
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

        // Step 7: Migrate releases
        if options.migrate_releases {
            match self
                .migrate_releases(&owner, &repo_name, target_owner, target_name)
                .await
            {
                Ok((releases, assets)) => {
                    report.releases_migrated = releases;
                    report.assets_migrated = assets;
                    info!("Migrated {releases} releases with {assets} assets");
                }
                Err(e) => {
                    report.add_error("releases", &e.to_string(), false);
                    warn!("Failed to migrate releases: {e}");
                }
            }
        }

        // Step 8: Migrate wiki (if available)
        if options.migrate_wiki && gh_repo.has_wiki {
            self.progress.set_phase(MigrationPhase::MigratingWiki, 1);
            match self
                .migrate_wiki(&owner, &repo_name, target_owner, target_name)
                .await
            {
                Ok(migrated) => {
                    report.wiki_migrated = migrated;
                    if migrated {
                        info!("Wiki migrated successfully");
                    }
                }
                Err(e) => {
                    report.add_warning(format!("Wiki migration skipped: {e}"));
                    warn!("Failed to migrate wiki: {e}");
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
                "Invalid repository format: {}. Expected 'owner/repo'",
                self.config.source_repo
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    async fn github_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self
            .github_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if response.status() == 404 {
            return Err(MigrationError::RepositoryNotFound(url.to_string()));
        }

        if response.status() == 403 {
            // Check for rate limiting
            if let Some(reset) = response.headers().get("x-ratelimit-reset") {
                if let Ok(reset_time) = reset.to_str().unwrap_or("0").parse::<u64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if reset_time > now {
                        return Err(MigrationError::RateLimitExceeded(reset_time - now));
                    }
                }
            }
            return Err(MigrationError::AuthenticationFailed(
                "Access denied. Check your token permissions.".to_string(),
            ));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MigrationError::ApiError(format!(
                "GitHub API error ({status}): {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| MigrationError::ApiError(e.to_string()))
    }

    async fn github_get_paginated<T: serde::de::DeserializeOwned>(
        &self,
        base_url: &str,
    ) -> Result<Vec<T>> {
        let mut all_items = Vec::new();
        let mut page = 1;

        loop {
            let url = format!("{base_url}?page={page}&per_page=100");
            let items: Vec<T> = self.github_get(&url).await?;

            if items.is_empty() {
                break;
            }

            let count = items.len();
            all_items.extend(items);

            if count < 100 {
                break;
            }
            page += 1;
        }

        Ok(all_items)
    }

    async fn fetch_repo_info(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}");
        self.github_get(&url).await
    }

    async fn mirror_git_repo(
        &self,
        _gh_repo: &GitHubRepo,
        target_owner: &str,
        target_name: &str,
    ) -> Result<(usize, usize)> {
        let temp_dir = TempDir::new()?;
        let clone_path = temp_dir.path().join("repo");

        // Clone with all branches and tags (mirror)
        let clone_url = format!(
            "https://{}@github.com/{}.git",
            self.github_token, self.config.source_repo
        );

        let output = Command::new("git")
            .args(["clone", "--mirror", &clone_url])
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
        self.progress
            .set_phase(MigrationPhase::PushingRepository, 1);

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

    async fn migrate_labels(
        &self,
        owner: &str,
        repo: &str,
        target_owner: &str,
        target_name: &str,
    ) -> Result<usize> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/labels");
        let labels: Vec<GitHubLabel> = self.github_get_paginated(&url).await?;

        self.progress
            .set_phase(MigrationPhase::MigratingLabels, labels.len() as u64);

        let mut count = 0;
        for label in &labels {
            match self
                .guts_client
                .create_label(
                    target_owner,
                    target_name,
                    &label.name,
                    &label.color,
                    label.description.as_deref(),
                )
                .await
            {
                Ok(()) => {
                    count += 1;
                    self.progress.increment(Some(&label.name));
                }
                Err(e) => {
                    debug!("Failed to create label {}: {e}", label.name);
                }
            }
        }

        Ok(count)
    }

    async fn migrate_issues(
        &self,
        owner: &str,
        repo: &str,
        target_owner: &str,
        target_name: &str,
        options: &MigrationOptions,
    ) -> Result<usize> {
        let state = if options.include_closed {
            "all"
        } else {
            "open"
        };
        let url = format!("https://api.github.com/repos/{owner}/{repo}/issues?state={state}");
        let issues: Vec<GitHubIssue> = self.github_get_paginated(&url).await?;

        // Filter out pull requests (GitHub API returns PRs in issues endpoint)
        let issues: Vec<_> = issues
            .into_iter()
            .filter(|i| {
                !i.body
                    .as_deref()
                    .map(|b| b.contains("<!-- PR -->"))
                    .unwrap_or(false)
            })
            .collect();

        self.progress
            .set_phase(MigrationPhase::MigratingIssues, issues.len() as u64);

        let mut count = 0;
        for issue in &issues {
            let body =
                self.rewrite_content(issue.body.as_deref().unwrap_or(""), owner, repo, options);

            // Add migration note
            let body_with_note = format!(
                "{body}\n\n---\n*Migrated from GitHub issue #{} by @{}*",
                issue.number, issue.user.login
            );

            let labels: Vec<String> = issue.labels.iter().map(|l| l.name.clone()).collect();

            match self
                .guts_client
                .create_issue(
                    target_owner,
                    target_name,
                    &CreateIssueRequest {
                        title: issue.title.clone(),
                        body: Some(body_with_note),
                        labels,
                        assignees: vec![],
                    },
                )
                .await
            {
                Ok(guts_issue) => {
                    // Migrate comments
                    if let Err(e) = self
                        .migrate_issue_comments(
                            owner,
                            repo,
                            issue.number,
                            target_owner,
                            target_name,
                            guts_issue.number,
                            options,
                        )
                        .await
                    {
                        debug!(
                            "Failed to migrate comments for issue #{}: {e}",
                            issue.number
                        );
                    }

                    // Close if closed on GitHub
                    if issue.state == "closed" {
                        let _ = self
                            .guts_client
                            .close_issue(target_owner, target_name, guts_issue.number)
                            .await;
                    }

                    count += 1;
                    self.progress
                        .increment(Some(&format!("Issue #{}", issue.number)));
                }
                Err(e) => {
                    debug!("Failed to create issue #{}: {e}", issue.number);
                }
            }
        }

        Ok(count)
    }

    #[allow(clippy::too_many_arguments)]
    async fn migrate_issue_comments(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        target_owner: &str,
        target_name: &str,
        guts_issue_number: u64,
        options: &MigrationOptions,
    ) -> Result<()> {
        let url =
            format!("https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}/comments");
        let comments: Vec<GitHubComment> = self.github_get_paginated(&url).await?;

        for comment in comments {
            let body = self.rewrite_content(&comment.body, owner, repo, options);
            let body_with_note = format!(
                "{body}\n\n---\n*Comment by @{} migrated from GitHub*",
                comment.user.login
            );

            let _ = self
                .guts_client
                .create_issue_comment(
                    target_owner,
                    target_name,
                    guts_issue_number,
                    &body_with_note,
                )
                .await;
        }

        Ok(())
    }

    async fn migrate_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        target_owner: &str,
        target_name: &str,
        options: &MigrationOptions,
    ) -> Result<usize> {
        let state = if options.include_closed {
            "all"
        } else {
            "open"
        };
        let url = format!("https://api.github.com/repos/{owner}/{repo}/pulls?state={state}");
        let prs: Vec<GitHubPullRequest> = self.github_get_paginated(&url).await?;

        self.progress
            .set_phase(MigrationPhase::MigratingPullRequests, prs.len() as u64);

        let mut count = 0;
        for pr in &prs {
            let body = self.rewrite_content(pr.body.as_deref().unwrap_or(""), owner, repo, options);

            // Add migration note with status
            let status = if pr.merged {
                "merged"
            } else if pr.state == "closed" {
                "closed"
            } else {
                "open"
            };

            let body_with_note = format!(
                "{body}\n\n---\n*Migrated from GitHub PR #{} ({}) by @{}*",
                pr.number, status, pr.user.login
            );

            match self
                .guts_client
                .create_pull_request(
                    target_owner,
                    target_name,
                    &CreatePullRequestRequest {
                        title: pr.title.clone(),
                        body: Some(body_with_note),
                        source_branch: pr.head.ref_name.clone(),
                        target_branch: pr.base.ref_name.clone(),
                    },
                )
                .await
            {
                Ok(_guts_pr) => {
                    count += 1;
                    self.progress.increment(Some(&format!("PR #{}", pr.number)));
                }
                Err(e) => {
                    debug!("Failed to create PR #{}: {e}", pr.number);
                }
            }
        }

        Ok(count)
    }

    async fn migrate_releases(
        &self,
        owner: &str,
        repo: &str,
        target_owner: &str,
        target_name: &str,
    ) -> Result<(usize, usize)> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");
        let releases: Vec<GitHubRelease> = self.github_get_paginated(&url).await?;

        self.progress
            .set_phase(MigrationPhase::MigratingReleases, releases.len() as u64);

        let mut release_count = 0;
        let mut asset_count = 0;

        for release in &releases {
            match self
                .guts_client
                .create_release(
                    target_owner,
                    target_name,
                    &CreateReleaseRequest {
                        tag_name: release.tag_name.clone(),
                        name: release
                            .name
                            .clone()
                            .unwrap_or_else(|| release.tag_name.clone()),
                        body: release.body.clone(),
                        prerelease: Some(release.prerelease),
                        draft: Some(release.draft),
                    },
                )
                .await
            {
                Ok(guts_release) => {
                    // Upload assets
                    for asset in &release.assets {
                        if let Ok(data) = self.download_asset(&asset.browser_download_url).await {
                            match self
                                .guts_client
                                .upload_release_asset(
                                    target_owner,
                                    target_name,
                                    &guts_release.id,
                                    &asset.name,
                                    &asset.content_type,
                                    data,
                                )
                                .await
                            {
                                Ok(()) => asset_count += 1,
                                Err(e) => debug!("Failed to upload asset {}: {e}", asset.name),
                            }
                        }
                    }

                    release_count += 1;
                    self.progress.increment(Some(&release.tag_name));
                }
                Err(e) => {
                    debug!("Failed to create release {}: {e}", release.tag_name);
                }
            }
        }

        Ok((release_count, asset_count))
    }

    async fn download_asset(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .github_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("Accept", "application/octet-stream")
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MigrationError::ApiError(format!(
                "Failed to download asset: {}",
                response.status()
            )));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| MigrationError::NetworkError(e.to_string()))
    }

    async fn migrate_wiki(
        &self,
        owner: &str,
        repo: &str,
        _target_owner: &str,
        _target_name: &str,
    ) -> Result<bool> {
        // Wiki migration is optional - it's a separate git repository
        let wiki_url = format!("https://github.com/{owner}/{repo}.wiki.git");

        // Just check if wiki exists
        let output = Command::new("git")
            .args(["ls-remote", &wiki_url])
            .output()?;

        if !output.status.success() {
            return Ok(false);
        }

        // TODO: Implement full wiki migration when Guts supports wiki
        Ok(false)
    }

    fn rewrite_content(
        &self,
        content: &str,
        owner: &str,
        repo: &str,
        options: &MigrationOptions,
    ) -> String {
        if !options.rewrite_links {
            return content.to_string();
        }

        let mut result = content.to_string();

        // Rewrite GitHub URLs to Guts URLs
        let github_url = format!("https://github.com/{owner}/{repo}");
        let guts_url = format!("{}/{owner}/{repo}", self.config.guts_url);
        result = result.replace(&github_url, &guts_url);

        // Rewrite user mentions if mapping exists
        for (github_user, guts_user) in &options.user_mapping {
            result = result.replace(&format!("@{github_user}"), &format!("@{guts_user}"));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo() {
        let config = MigrationConfig::new("owner/repo", "http://localhost:8080");
        let migrator = GitHubMigrator::new("token", config).unwrap();

        let (owner, repo) = migrator.parse_repo().unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_rewrite_content() {
        let config = MigrationConfig::new("old-owner/old-repo", "https://guts.network");
        let migrator = GitHubMigrator::new("token", config).unwrap();

        let options = MigrationOptions::default().with_user_mapping("github-user", "guts-user");

        let content = "Check https://github.com/old-owner/old-repo/issues/1 by @github-user";
        let rewritten = migrator.rewrite_content(content, "old-owner", "old-repo", &options);

        assert!(rewritten.contains("https://guts.network/old-owner/old-repo"));
        assert!(rewritten.contains("@guts-user"));
    }
}
