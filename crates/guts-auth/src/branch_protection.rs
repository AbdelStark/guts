//! Branch protection rules.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Branch protection rule for a repository.
///
/// Branch protection prevents direct pushes to important branches,
/// requiring pull requests with reviews instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtection {
    /// Unique rule ID.
    pub id: u64,
    /// Repository key (e.g., "owner/repo").
    pub repo_key: String,
    /// Branch pattern (e.g., "main", "release/*").
    pub pattern: String,
    /// Require changes via pull request.
    pub require_pr: bool,
    /// Minimum number of approving reviews required.
    pub required_reviews: u32,
    /// Required status checks that must pass.
    pub required_status_checks: HashSet<String>,
    /// Dismiss stale reviews when new commits are pushed.
    pub dismiss_stale_reviews: bool,
    /// Require review from code owners.
    pub require_code_owner_review: bool,
    /// Restrict who can push (only admins if true).
    pub restrict_pushes: bool,
    /// Allow force pushes.
    pub allow_force_push: bool,
    /// Allow branch deletion.
    pub allow_deletion: bool,
    /// When the rule was created (Unix timestamp).
    pub created_at: u64,
    /// When the rule was last updated (Unix timestamp).
    pub updated_at: u64,
}

impl BranchProtection {
    /// Create a new branch protection rule with defaults.
    pub fn new(id: u64, repo_key: String, pattern: String) -> Self {
        let now = Self::now();
        Self {
            id,
            repo_key,
            pattern,
            require_pr: true,
            required_reviews: 1,
            required_status_checks: HashSet::new(),
            dismiss_stale_reviews: false,
            require_code_owner_review: false,
            restrict_pushes: false,
            allow_force_push: false,
            allow_deletion: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this rule matches a branch name.
    pub fn matches(&self, branch: &str) -> bool {
        if self.pattern.contains('*') {
            // Simple glob matching
            let parts: Vec<&str> = self.pattern.split('*').collect();
            if parts.len() == 1 {
                // No wildcard
                branch == self.pattern
            } else if parts.len() == 2 {
                // Single wildcard
                let prefix = parts[0];
                let suffix = parts[1];
                branch.starts_with(prefix) && branch.ends_with(suffix)
            } else {
                // Multiple wildcards - simplify to prefix match
                branch.starts_with(parts[0])
            }
        } else {
            branch == self.pattern
        }
    }

    /// Check if a direct push is allowed (without PR).
    pub fn allows_direct_push(&self, is_admin: bool) -> bool {
        if !self.require_pr {
            return true;
        }
        if self.restrict_pushes {
            return is_admin;
        }
        false
    }

    /// Check if a force push is allowed.
    pub fn allows_force_push(&self) -> bool {
        self.allow_force_push
    }

    /// Check if branch deletion is allowed.
    pub fn allows_deletion(&self) -> bool {
        self.allow_deletion
    }

    /// Check if a PR meets the review requirements.
    pub fn check_reviews(&self, approving_reviews: u32, has_code_owner_review: bool) -> bool {
        if approving_reviews < self.required_reviews {
            return false;
        }
        if self.require_code_owner_review && !has_code_owner_review {
            return false;
        }
        true
    }

    /// Check if all required status checks have passed.
    pub fn check_status(&self, passed_checks: &HashSet<String>) -> bool {
        self.required_status_checks.is_subset(passed_checks)
    }

    /// Add a required status check.
    pub fn add_required_check(&mut self, check: String) {
        self.required_status_checks.insert(check);
        self.updated_at = Self::now();
    }

    /// Remove a required status check.
    pub fn remove_required_check(&mut self, check: &str) -> bool {
        let removed = self.required_status_checks.remove(check);
        if removed {
            self.updated_at = Self::now();
        }
        removed
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Request to create or update branch protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionRequest {
    /// Require changes via pull request.
    #[serde(default)]
    pub require_pr: bool,
    /// Minimum number of approving reviews.
    #[serde(default)]
    pub required_reviews: u32,
    /// Required status checks.
    #[serde(default)]
    pub required_status_checks: Vec<String>,
    /// Dismiss stale reviews.
    #[serde(default)]
    pub dismiss_stale_reviews: bool,
    /// Require code owner review.
    #[serde(default)]
    pub require_code_owner_review: bool,
    /// Restrict pushes to admins only.
    #[serde(default)]
    pub restrict_pushes: bool,
    /// Allow force pushes.
    #[serde(default)]
    pub allow_force_push: bool,
    /// Allow branch deletion.
    #[serde(default)]
    pub allow_deletion: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let rule = BranchProtection::new(1, "acme/api".into(), "main".into());
        assert!(rule.matches("main"));
        assert!(!rule.matches("master"));
        assert!(!rule.matches("main-backup"));

        let rule = BranchProtection::new(2, "acme/api".into(), "release/*".into());
        assert!(rule.matches("release/1.0"));
        assert!(rule.matches("release/2.0.1"));
        assert!(!rule.matches("releases/1.0"));
        assert!(!rule.matches("release"));

        let rule = BranchProtection::new(3, "acme/api".into(), "feature-*-test".into());
        assert!(rule.matches("feature-foo-test"));
        assert!(rule.matches("feature-bar-test"));
        assert!(!rule.matches("feature-foo-tests"));
    }

    #[test]
    fn test_direct_push() {
        let mut rule = BranchProtection::new(1, "acme/api".into(), "main".into());

        // Default: PR required, anyone with access can push via PR
        assert!(!rule.allows_direct_push(false));
        assert!(!rule.allows_direct_push(true));

        // Restrict to admins
        rule.restrict_pushes = true;
        assert!(!rule.allows_direct_push(false));
        assert!(rule.allows_direct_push(true));

        // No PR required
        rule.require_pr = false;
        assert!(rule.allows_direct_push(false));
        assert!(rule.allows_direct_push(true));
    }

    #[test]
    fn test_review_requirements() {
        let mut rule = BranchProtection::new(1, "acme/api".into(), "main".into());
        rule.required_reviews = 2;

        assert!(!rule.check_reviews(0, false));
        assert!(!rule.check_reviews(1, false));
        assert!(rule.check_reviews(2, false));
        assert!(rule.check_reviews(3, false));

        rule.require_code_owner_review = true;
        assert!(!rule.check_reviews(2, false));
        assert!(rule.check_reviews(2, true));
    }

    #[test]
    fn test_status_checks() {
        let mut rule = BranchProtection::new(1, "acme/api".into(), "main".into());
        rule.add_required_check("ci/build".into());
        rule.add_required_check("ci/test".into());

        let mut passed = HashSet::new();
        assert!(!rule.check_status(&passed));

        passed.insert("ci/build".into());
        assert!(!rule.check_status(&passed));

        passed.insert("ci/test".into());
        assert!(rule.check_status(&passed));

        passed.insert("ci/lint".into()); // Extra check is fine
        assert!(rule.check_status(&passed));
    }
}
