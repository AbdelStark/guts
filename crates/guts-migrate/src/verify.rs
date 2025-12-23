//! Migration verification utilities.

use crate::client::GutsClient;
use crate::error::{MigrationError, Result};
use crate::types::MigrationReport;

use std::process::Command;
use tempfile::TempDir;
use tracing::info;

/// Verification results for a migration.
#[derive(Debug, Clone, Default)]
pub struct VerificationResult {
    /// Git data verification passed.
    pub git_verified: bool,

    /// Number of commits verified.
    pub commits_verified: usize,

    /// Number of branches verified.
    pub branches_verified: usize,

    /// Number of tags verified.
    pub tags_verified: usize,

    /// Issues count matches.
    pub issues_verified: bool,

    /// Pull requests count matches.
    pub prs_verified: bool,

    /// Releases count matches.
    pub releases_verified: bool,

    /// Verification errors.
    pub errors: Vec<String>,

    /// Verification warnings.
    pub warnings: Vec<String>,
}

impl VerificationResult {
    /// Check if verification passed.
    pub fn is_success(&self) -> bool {
        self.git_verified && self.errors.is_empty()
    }

    /// Print verification summary.
    pub fn print_summary(&self) {
        println!("\n=== Verification Summary ===\n");
        println!(
            "Git data:       {}",
            if self.git_verified { "✓" } else { "✗" }
        );
        println!("  Commits:      {}", self.commits_verified);
        println!("  Branches:     {}", self.branches_verified);
        println!("  Tags:         {}", self.tags_verified);
        println!(
            "Issues:         {}",
            if self.issues_verified { "✓" } else { "✗" }
        );
        println!(
            "Pull Requests:  {}",
            if self.prs_verified { "✓" } else { "✗" }
        );
        println!(
            "Releases:       {}",
            if self.releases_verified { "✓" } else { "✗" }
        );

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {error}");
            }
        }

        if !self.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &self.warnings {
                println!("  - {warning}");
            }
        }

        println!(
            "\nVerification: {}",
            if self.is_success() {
                "PASSED"
            } else {
                "FAILED"
            }
        );
    }
}

/// Verifier for post-migration validation.
pub struct MigrationVerifier {
    #[allow(dead_code)]
    guts_client: GutsClient,
}

impl MigrationVerifier {
    /// Create a new verifier.
    pub fn new(guts_url: &str, guts_token: Option<String>) -> Result<Self> {
        let guts_client = GutsClient::new(guts_url, guts_token)?;
        Ok(Self { guts_client })
    }

    /// Verify a migration between source and target.
    pub async fn verify(
        &self,
        source_url: &str,
        target_owner: &str,
        target_repo: &str,
        report: &MigrationReport,
    ) -> Result<VerificationResult> {
        let mut result = VerificationResult::default();

        info!("Starting verification...");

        // Verify Git data
        if report.git_mirrored {
            match self
                .verify_git(source_url, target_owner, target_repo)
                .await
            {
                Ok((commits, branches, tags)) => {
                    result.git_verified = true;
                    result.commits_verified = commits;
                    result.branches_verified = branches;
                    result.tags_verified = tags;
                    info!("Git verification passed: {commits} commits, {branches} branches, {tags} tags");
                }
                Err(e) => {
                    result.git_verified = false;
                    result.errors.push(format!("Git verification failed: {e}"));
                }
            }
        }

        // Verify issues count
        if report.issues_migrated > 0 {
            match self.verify_issues(target_owner, target_repo).await {
                Ok(count) => {
                    if count >= report.issues_migrated {
                        result.issues_verified = true;
                        info!("Issues verification passed: {count} issues found");
                    } else {
                        result.warnings.push(format!(
                            "Issue count mismatch: expected {}, found {}",
                            report.issues_migrated, count
                        ));
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Issues verification failed: {e}"));
                }
            }
        } else {
            result.issues_verified = true; // No issues to verify
        }

        // Verify PRs count
        if report.prs_migrated > 0 {
            match self.verify_prs(target_owner, target_repo).await {
                Ok(count) => {
                    if count >= report.prs_migrated {
                        result.prs_verified = true;
                        info!("PRs verification passed: {count} PRs found");
                    } else {
                        result.warnings.push(format!(
                            "PR count mismatch: expected {}, found {}",
                            report.prs_migrated, count
                        ));
                    }
                }
                Err(e) => {
                    result.errors.push(format!("PRs verification failed: {e}"));
                }
            }
        } else {
            result.prs_verified = true; // No PRs to verify
        }

        // Verify releases count
        if report.releases_migrated > 0 {
            match self.verify_releases(target_owner, target_repo).await {
                Ok(count) => {
                    if count >= report.releases_migrated {
                        result.releases_verified = true;
                        info!("Releases verification passed: {count} releases found");
                    } else {
                        result.warnings.push(format!(
                            "Release count mismatch: expected {}, found {}",
                            report.releases_migrated, count
                        ));
                    }
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Releases verification failed: {e}"));
                }
            }
        } else {
            result.releases_verified = true; // No releases to verify
        }

        Ok(result)
    }

    async fn verify_git(
        &self,
        source_url: &str,
        target_owner: &str,
        target_repo: &str,
    ) -> Result<(usize, usize, usize)> {
        let temp_dir = TempDir::new()?;
        let source_path = temp_dir.path().join("source");
        let target_path = temp_dir.path().join("target");

        // Clone source
        let output = Command::new("git")
            .args(["clone", "--mirror", source_url])
            .arg(&source_path)
            .output()?;

        if !output.status.success() {
            return Err(MigrationError::VerificationFailed(format!(
                "Failed to clone source: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Clone target from Guts
        // Note: This assumes the Guts git URL format
        let guts_url = format!("http://localhost:8080/git/{target_owner}/{target_repo}.git");
        let output = Command::new("git")
            .args(["clone", "--mirror", &guts_url])
            .arg(&target_path)
            .output()?;

        if !output.status.success() {
            return Err(MigrationError::VerificationFailed(format!(
                "Failed to clone target: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Compare commit counts
        let source_commits = count_commits(&source_path)?;
        let target_commits = count_commits(&target_path)?;

        if source_commits != target_commits {
            return Err(MigrationError::VerificationFailed(format!(
                "Commit count mismatch: source={source_commits}, target={target_commits}"
            )));
        }

        // Count branches and tags
        let branches = count_branches(&target_path)?;
        let tags = count_tags(&target_path)?;

        Ok((target_commits, branches, tags))
    }

    async fn verify_issues(&self, _owner: &str, _repo: &str) -> Result<usize> {
        // TODO: Implement API call to count issues
        Ok(0)
    }

    async fn verify_prs(&self, _owner: &str, _repo: &str) -> Result<usize> {
        // TODO: Implement API call to count PRs
        Ok(0)
    }

    async fn verify_releases(&self, _owner: &str, _repo: &str) -> Result<usize> {
        // TODO: Implement API call to count releases
        Ok(0)
    }
}

fn count_commits(repo_path: &std::path::Path) -> Result<usize> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["rev-list", "--all", "--count"])
        .output()?;

    if !output.status.success() {
        return Ok(0);
    }

    let count_str = String::from_utf8_lossy(&output.stdout);
    count_str
        .trim()
        .parse()
        .map_err(|e| MigrationError::VerificationFailed(format!("Failed to parse commit count: {e}")))
}

fn count_branches(repo_path: &std::path::Path) -> Result<usize> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["branch", "-r"])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .count())
}

fn count_tags(repo_path: &std::path::Path) -> Result<usize> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["tag"])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result() {
        let mut result = VerificationResult::default();
        result.git_verified = true;
        result.commits_verified = 100;
        result.branches_verified = 5;
        result.tags_verified = 3;
        result.issues_verified = true;
        result.prs_verified = true;
        result.releases_verified = true;

        assert!(result.is_success());
    }

    #[test]
    fn test_verification_failure() {
        let mut result = VerificationResult::default();
        result.git_verified = false;
        result.errors.push("Git mismatch".to_string());

        assert!(!result.is_success());
    }
}
