//! GitLab migration implementation.

use crate::client::{CreateIssueRequest, CreatePullRequestRequest, CreateReleaseRequest, GutsClient};
use crate::error::{MigrationError, Result};
use crate::progress::{MigrationPhase, MigrationProgress};
use crate::types::{MigrationConfig, MigrationOptions, MigrationReport};

use reqwest::Client;
use serde::Deserialize;
use std::process::Command;
use tempfile::TempDir;
use tracing::{debug, info, warn};

/// GitLab API response types
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabProject {
    id: u64,
    name: String,
    path: String,
    description: Option<String>,
    visibility: String,
    http_url_to_repo: String,
    default_branch: Option<String>,
    wiki_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct GitLabIssue {
    iid: u64,
    title: String,
    description: Option<String>,
    state: String,
    labels: Vec<String>,
    author: GitLabUser,
}

#[derive(Debug, Deserialize)]
struct GitLabMergeRequest {
    iid: u64,
    title: String,
    description: Option<String>,
    state: String,
    source_branch: String,
    target_branch: String,
    author: GitLabUser,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabRelease {
    tag_name: String,
    name: Option<String>,
    description: Option<String>,
    assets: GitLabAssets,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabAssets {
    links: Vec<GitLabAssetLink>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabAssetLink {
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct GitLabUser {
    username: String,
}

#[derive(Debug, Deserialize)]
struct GitLabLabel {
    name: String,
    color: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabNote {
    body: String,
    author: GitLabUser,
}

/// Migrator for GitLab projects.
pub struct GitLabMigrator {
    gitlab_client: Client,
    gitlab_token: String,
    gitlab_url: String,
    guts_client: GutsClient,
    config: MigrationConfig,
    progress: MigrationProgress,
}

impl GitLabMigrator {
    /// Create a new GitLab migrator.
    ///
    /// # Arguments
    ///
    /// * `gitlab_token` - GitLab personal access token
    /// * `gitlab_url` - GitLab instance URL (e.g., "https://gitlab.com")
    /// * `config` - Migration configuration
    pub fn new(gitlab_token: &str, gitlab_url: &str, config: MigrationConfig) -> Result<Self> {
        let gitlab_client = Client::builder()
            .user_agent("guts-migrate")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        let guts_client = GutsClient::new(&config.guts_url, config.guts_token.clone())?;

        Ok(Self {
            gitlab_client,
            gitlab_token: gitlab_token.to_string(),
            gitlab_url: gitlab_url.trim_end_matches('/').to_string(),
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

        info!("Starting GitLab migration for {}", self.config.source_repo);
        self.progress.set_phase(MigrationPhase::Initializing, 1);

        // Parse project path (can be nested groups: group/subgroup/project)
        let project_path = &self.config.source_repo;

        // Step 1: Fetch project info from GitLab
        self.progress.message("Fetching project information...");
        let gl_project = self.fetch_project_info(project_path).await?;
        debug!("Fetched project info: {:?}", gl_project.name);

        // Extract owner from project path
        let owner = project_path
            .rsplit('/')
            .nth(1)
            .unwrap_or("unknown")
            .to_string();

        // Step 2: Create repository on Guts
        self.progress.set_phase(MigrationPhase::CreatingRepository, 1);
        let target_owner = self.config.target_owner.as_deref().unwrap_or(&owner);
        let target_name = self
            .config
            .target_name
            .as_deref()
            .unwrap_or(&gl_project.name);
        let is_private = gl_project.visibility != "public";

        match self
            .guts_client
            .create_repo(target_name, gl_project.description.as_deref(), is_private)
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
        match self.mirror_git_repo(&gl_project, target_owner, target_name).await {
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
            match self.migrate_labels(gl_project.id, target_owner, target_name).await {
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
            match self.migrate_issues(
                gl_project.id,
                target_owner,
                target_name,
                &options,
            ).await {
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

        // Step 6: Migrate merge requests
        if options.migrate_pull_requests {
            match self.migrate_merge_requests(
                gl_project.id,
                target_owner,
                target_name,
                &options,
            ).await {
                Ok(count) => {
                    report.prs_migrated = count;
                    info!("Migrated {count} merge requests");
                }
                Err(e) => {
                    report.add_error("merge_requests", &e.to_string(), false);
                    warn!("Failed to migrate merge requests: {e}");
                }
            }
        }

        // Step 7: Migrate releases
        if options.migrate_releases {
            match self.migrate_releases(gl_project.id, target_owner, target_name).await {
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
        if options.migrate_wiki && gl_project.wiki_enabled {
            self.progress.set_phase(MigrationPhase::MigratingWiki, 1);
            match self.migrate_wiki(&gl_project, target_owner, target_name).await {
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

    async fn gitlab_get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}/api/v4{path}", self.gitlab_url);
        let response = self
            .gitlab_client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.gitlab_token)
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if response.status() == 404 {
            return Err(MigrationError::RepositoryNotFound(path.to_string()));
        }

        if response.status() == 401 {
            return Err(MigrationError::AuthenticationFailed(
                "Invalid GitLab token".to_string(),
            ));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MigrationError::ApiError(format!(
                "GitLab API error ({status}): {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| MigrationError::ApiError(e.to_string()))
    }

    async fn gitlab_get_paginated<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<Vec<T>> {
        let mut all_items = Vec::new();
        let mut page = 1;

        loop {
            let paginated_path = if path.contains('?') {
                format!("{path}&page={page}&per_page=100")
            } else {
                format!("{path}?page={page}&per_page=100")
            };

            let items: Vec<T> = self.gitlab_get(&paginated_path).await?;

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

    async fn fetch_project_info(&self, project_path: &str) -> Result<GitLabProject> {
        let encoded_path = urlencoding::encode(project_path);
        self.gitlab_get(&format!("/projects/{encoded_path}")).await
    }

    async fn mirror_git_repo(
        &self,
        gl_project: &GitLabProject,
        target_owner: &str,
        target_name: &str,
    ) -> Result<(usize, usize)> {
        let temp_dir = TempDir::new()?;
        let clone_path = temp_dir.path().join("repo");

        // Clone with all branches and tags (mirror)
        // GitLab uses oauth2:TOKEN format for authentication
        let clone_url = gl_project
            .http_url_to_repo
            .replace("https://", &format!("https://oauth2:{}@", self.gitlab_token));

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

    async fn migrate_labels(
        &self,
        project_id: u64,
        target_owner: &str,
        target_name: &str,
    ) -> Result<usize> {
        let labels: Vec<GitLabLabel> = self
            .gitlab_get_paginated(&format!("/projects/{project_id}/labels"))
            .await?;

        self.progress.set_phase(MigrationPhase::MigratingLabels, labels.len() as u64);

        let mut count = 0;
        for label in &labels {
            // GitLab colors start with #, strip it for Guts
            let color = label.color.trim_start_matches('#');

            match self
                .guts_client
                .create_label(
                    target_owner,
                    target_name,
                    &label.name,
                    color,
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
        project_id: u64,
        target_owner: &str,
        target_name: &str,
        options: &MigrationOptions,
    ) -> Result<usize> {
        let state = if options.include_closed { "all" } else { "opened" };
        let issues: Vec<GitLabIssue> = self
            .gitlab_get_paginated(&format!("/projects/{project_id}/issues?state={state}"))
            .await?;

        self.progress.set_phase(MigrationPhase::MigratingIssues, issues.len() as u64);

        let mut count = 0;
        for issue in &issues {
            let body = issue.description.as_deref().unwrap_or("");
            let body_with_note = format!(
                "{body}\n\n---\n*Migrated from GitLab issue #{} by @{}*",
                issue.iid, issue.author.username
            );

            match self
                .guts_client
                .create_issue(
                    target_owner,
                    target_name,
                    &CreateIssueRequest {
                        title: issue.title.clone(),
                        body: Some(body_with_note),
                        labels: issue.labels.clone(),
                        assignees: vec![],
                    },
                )
                .await
            {
                Ok(guts_issue) => {
                    // Close if closed on GitLab
                    if issue.state == "closed" {
                        let _ = self
                            .guts_client
                            .close_issue(target_owner, target_name, guts_issue.number)
                            .await;
                    }

                    count += 1;
                    self.progress.increment(Some(&format!("Issue #{}", issue.iid)));
                }
                Err(e) => {
                    debug!("Failed to create issue #{}: {e}", issue.iid);
                }
            }
        }

        Ok(count)
    }

    async fn migrate_merge_requests(
        &self,
        project_id: u64,
        target_owner: &str,
        target_name: &str,
        options: &MigrationOptions,
    ) -> Result<usize> {
        let state = if options.include_closed { "all" } else { "opened" };
        let mrs: Vec<GitLabMergeRequest> = self
            .gitlab_get_paginated(&format!("/projects/{project_id}/merge_requests?state={state}"))
            .await?;

        self.progress.set_phase(MigrationPhase::MigratingPullRequests, mrs.len() as u64);

        let mut count = 0;
        for mr in &mrs {
            let body = mr.description.as_deref().unwrap_or("");
            let body_with_note = format!(
                "{body}\n\n---\n*Migrated from GitLab MR !{} ({}) by @{}*",
                mr.iid, mr.state, mr.author.username
            );

            match self
                .guts_client
                .create_pull_request(
                    target_owner,
                    target_name,
                    &CreatePullRequestRequest {
                        title: mr.title.clone(),
                        body: Some(body_with_note),
                        source_branch: mr.source_branch.clone(),
                        target_branch: mr.target_branch.clone(),
                    },
                )
                .await
            {
                Ok(_guts_pr) => {
                    count += 1;
                    self.progress.increment(Some(&format!("MR !{}", mr.iid)));
                }
                Err(e) => {
                    debug!("Failed to create MR !{}: {e}", mr.iid);
                }
            }
        }

        Ok(count)
    }

    async fn migrate_releases(
        &self,
        project_id: u64,
        target_owner: &str,
        target_name: &str,
    ) -> Result<(usize, usize)> {
        let releases: Vec<GitLabRelease> = self
            .gitlab_get_paginated(&format!("/projects/{project_id}/releases"))
            .await?;

        self.progress.set_phase(MigrationPhase::MigratingReleases, releases.len() as u64);

        let mut release_count = 0;
        let asset_count = 0;

        for release in &releases {
            match self
                .guts_client
                .create_release(
                    target_owner,
                    target_name,
                    &CreateReleaseRequest {
                        tag_name: release.tag_name.clone(),
                        name: release.name.clone().unwrap_or_else(|| release.tag_name.clone()),
                        body: release.description.clone(),
                        prerelease: Some(false),
                        draft: Some(false),
                    },
                )
                .await
            {
                Ok(_guts_release) => {
                    // TODO: Download and upload assets
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

    async fn migrate_wiki(
        &self,
        _gl_project: &GitLabProject,
        _target_owner: &str,
        _target_name: &str,
    ) -> Result<bool> {
        // TODO: Implement wiki migration
        Ok(false)
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.replace('/', "%2F")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("group/project"), "group%2Fproject");
        assert_eq!(
            urlencoding::encode("group/subgroup/project"),
            "group%2Fsubgroup%2Fproject"
        );
    }
}
