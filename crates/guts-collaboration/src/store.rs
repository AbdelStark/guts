//! In-memory storage for collaboration data.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    error::CollaborationError, Comment, CommentTarget, Issue, IssueState, PullRequest,
    PullRequestState, Result, Review,
};

/// In-memory store for collaboration data.
///
/// Thread-safe storage for pull requests, issues, comments, and reviews.
#[derive(Default)]
pub struct CollaborationStore {
    /// Pull requests indexed by (repo_key, number).
    pull_requests: RwLock<HashMap<(String, u32), PullRequest>>,
    /// Issues indexed by (repo_key, number).
    issues: RwLock<HashMap<(String, u32), Issue>>,
    /// Comments indexed by id.
    comments: RwLock<HashMap<u64, Comment>>,
    /// Reviews indexed by id.
    reviews: RwLock<HashMap<u64, Review>>,
    /// Counter for next PR number per repository.
    pr_counters: RwLock<HashMap<String, u32>>,
    /// Counter for next issue number per repository.
    issue_counters: RwLock<HashMap<String, u32>>,
    /// Global ID counter for entities.
    next_id: AtomicU64,
}

impl CollaborationStore {
    /// Creates a new empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a new unique ID.
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Gets the next PR number for a repository.
    fn next_pr_number(&self, repo_key: &str) -> u32 {
        let mut counters = self.pr_counters.write();
        let counter = counters.entry(repo_key.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Gets the next issue number for a repository.
    fn next_issue_number(&self, repo_key: &str) -> u32 {
        let mut counters = self.issue_counters.write();
        let counter = counters.entry(repo_key.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    // ==================== Pull Requests ====================

    /// Creates a new pull request.
    pub fn create_pull_request(&self, mut pr: PullRequest) -> Result<PullRequest> {
        let number = self.next_pr_number(&pr.repo_key);
        let id = self.next_id();

        pr.id = id;
        pr.number = number;

        let key = (pr.repo_key.clone(), pr.number);
        let mut prs = self.pull_requests.write();

        if prs.contains_key(&key) {
            return Err(CollaborationError::PullRequestExists {
                repo_key: pr.repo_key.clone(),
                number: pr.number,
            });
        }

        prs.insert(key, pr.clone());
        Ok(pr)
    }

    /// Gets a pull request by repository and number.
    pub fn get_pull_request(&self, repo_key: &str, number: u32) -> Result<PullRequest> {
        let key = (repo_key.to_string(), number);
        self.pull_requests.read().get(&key).cloned().ok_or_else(|| {
            CollaborationError::PullRequestNotFound {
                repo_key: repo_key.to_string(),
                number,
            }
        })
    }

    /// Lists pull requests for a repository.
    pub fn list_pull_requests(
        &self,
        repo_key: &str,
        state: Option<PullRequestState>,
    ) -> Vec<PullRequest> {
        self.pull_requests
            .read()
            .values()
            .filter(|pr| pr.repo_key == repo_key && state.is_none_or(|s| pr.state == s))
            .cloned()
            .collect()
    }

    /// Updates a pull request.
    pub fn update_pull_request<F>(&self, repo_key: &str, number: u32, f: F) -> Result<PullRequest>
    where
        F: FnOnce(&mut PullRequest) -> Result<()>,
    {
        let key = (repo_key.to_string(), number);
        let mut prs = self.pull_requests.write();

        let pr = prs
            .get_mut(&key)
            .ok_or_else(|| CollaborationError::PullRequestNotFound {
                repo_key: repo_key.to_string(),
                number,
            })?;

        f(pr)?;
        Ok(pr.clone())
    }

    /// Closes a pull request.
    pub fn close_pull_request(&self, repo_key: &str, number: u32) -> Result<PullRequest> {
        self.update_pull_request(repo_key, number, |pr| pr.close())
    }

    /// Reopens a pull request.
    pub fn reopen_pull_request(&self, repo_key: &str, number: u32) -> Result<PullRequest> {
        self.update_pull_request(repo_key, number, |pr| pr.reopen())
    }

    /// Merges a pull request.
    pub fn merge_pull_request(
        &self,
        repo_key: &str,
        number: u32,
        merged_by: &str,
    ) -> Result<PullRequest> {
        self.update_pull_request(repo_key, number, |pr| pr.merge(merged_by))
    }

    // ==================== Issues ====================

    /// Creates a new issue.
    pub fn create_issue(&self, mut issue: Issue) -> Result<Issue> {
        let number = self.next_issue_number(&issue.repo_key);
        let id = self.next_id();

        issue.id = id;
        issue.number = number;

        let key = (issue.repo_key.clone(), issue.number);
        let mut issues = self.issues.write();

        if issues.contains_key(&key) {
            return Err(CollaborationError::IssueExists {
                repo_key: issue.repo_key.clone(),
                number: issue.number,
            });
        }

        issues.insert(key, issue.clone());
        Ok(issue)
    }

    /// Gets an issue by repository and number.
    pub fn get_issue(&self, repo_key: &str, number: u32) -> Result<Issue> {
        let key = (repo_key.to_string(), number);
        self.issues
            .read()
            .get(&key)
            .cloned()
            .ok_or_else(|| CollaborationError::IssueNotFound {
                repo_key: repo_key.to_string(),
                number,
            })
    }

    /// Lists issues for a repository.
    pub fn list_issues(&self, repo_key: &str, state: Option<IssueState>) -> Vec<Issue> {
        self.issues
            .read()
            .values()
            .filter(|issue| issue.repo_key == repo_key && state.is_none_or(|s| issue.state == s))
            .cloned()
            .collect()
    }

    /// Updates an issue.
    pub fn update_issue<F>(&self, repo_key: &str, number: u32, f: F) -> Result<Issue>
    where
        F: FnOnce(&mut Issue) -> Result<()>,
    {
        let key = (repo_key.to_string(), number);
        let mut issues = self.issues.write();

        let issue = issues
            .get_mut(&key)
            .ok_or_else(|| CollaborationError::IssueNotFound {
                repo_key: repo_key.to_string(),
                number,
            })?;

        f(issue)?;
        Ok(issue.clone())
    }

    /// Closes an issue.
    pub fn close_issue(&self, repo_key: &str, number: u32, closed_by: &str) -> Result<Issue> {
        self.update_issue(repo_key, number, |issue| issue.close(closed_by))
    }

    /// Reopens an issue.
    pub fn reopen_issue(&self, repo_key: &str, number: u32) -> Result<Issue> {
        self.update_issue(repo_key, number, |issue| issue.reopen())
    }

    // ==================== Comments ====================

    /// Creates a new comment.
    pub fn create_comment(&self, mut comment: Comment) -> Result<Comment> {
        // Verify target exists
        match &comment.target {
            CommentTarget::PullRequest { repo_key, number } => {
                self.get_pull_request(repo_key, *number)?;
            }
            CommentTarget::Issue { repo_key, number } => {
                self.get_issue(repo_key, *number)?;
            }
        }

        let id = self.next_id();
        comment.id = id;

        self.comments.write().insert(id, comment.clone());
        Ok(comment)
    }

    /// Gets a comment by ID.
    pub fn get_comment(&self, id: u64) -> Result<Comment> {
        self.comments
            .read()
            .get(&id)
            .cloned()
            .ok_or(CollaborationError::CommentNotFound { id })
    }

    /// Lists comments for a pull request.
    pub fn list_pr_comments(&self, repo_key: &str, number: u32) -> Vec<Comment> {
        self.comments
            .read()
            .values()
            .filter(|c| match &c.target {
                CommentTarget::PullRequest {
                    repo_key: rk,
                    number: n,
                } => rk == repo_key && *n == number,
                _ => false,
            })
            .cloned()
            .collect()
    }

    /// Lists comments for an issue.
    pub fn list_issue_comments(&self, repo_key: &str, number: u32) -> Vec<Comment> {
        self.comments
            .read()
            .values()
            .filter(|c| match &c.target {
                CommentTarget::Issue {
                    repo_key: rk,
                    number: n,
                } => rk == repo_key && *n == number,
                _ => false,
            })
            .cloned()
            .collect()
    }

    /// Updates a comment.
    pub fn update_comment(&self, id: u64, body: &str) -> Result<Comment> {
        let mut comments = self.comments.write();
        let comment = comments
            .get_mut(&id)
            .ok_or(CollaborationError::CommentNotFound { id })?;

        comment.update_body(body);
        Ok(comment.clone())
    }

    /// Deletes a comment.
    pub fn delete_comment(&self, id: u64) -> Result<()> {
        self.comments
            .write()
            .remove(&id)
            .ok_or(CollaborationError::CommentNotFound { id })?;
        Ok(())
    }

    // ==================== Reviews ====================

    /// Creates a new review.
    pub fn create_review(&self, mut review: Review) -> Result<Review> {
        // Verify PR exists
        self.get_pull_request(&review.repo_key, review.pr_number)?;

        let id = self.next_id();
        review.id = id;

        self.reviews.write().insert(id, review.clone());
        Ok(review)
    }

    /// Gets a review by ID.
    pub fn get_review(&self, id: u64) -> Result<Review> {
        self.reviews
            .read()
            .get(&id)
            .cloned()
            .ok_or(CollaborationError::ReviewNotFound { id })
    }

    /// Lists reviews for a pull request.
    pub fn list_reviews(&self, repo_key: &str, pr_number: u32) -> Vec<Review> {
        self.reviews
            .read()
            .values()
            .filter(|r| r.repo_key == repo_key && r.pr_number == pr_number)
            .cloned()
            .collect()
    }

    /// Dismisses a review.
    pub fn dismiss_review(&self, id: u64) -> Result<Review> {
        let mut reviews = self.reviews.write();
        let review = reviews
            .get_mut(&id)
            .ok_or(CollaborationError::ReviewNotFound { id })?;

        review.dismiss();
        Ok(review.clone())
    }

    // ==================== Bulk Operations ====================

    /// Gets all pull requests.
    pub fn all_pull_requests(&self) -> Vec<PullRequest> {
        self.pull_requests.read().values().cloned().collect()
    }

    /// Gets all issues.
    pub fn all_issues(&self) -> Vec<Issue> {
        self.issues.read().values().cloned().collect()
    }

    /// Gets all comments.
    pub fn all_comments(&self) -> Vec<Comment> {
        self.comments.read().values().cloned().collect()
    }

    /// Gets all reviews.
    pub fn all_reviews(&self) -> Vec<Review> {
        self.reviews.read().values().cloned().collect()
    }

    /// Imports a pull request (for P2P sync).
    pub fn import_pull_request(&self, pr: PullRequest) -> Result<()> {
        let key = (pr.repo_key.clone(), pr.number);
        let mut prs = self.pull_requests.write();

        // Update counters if needed
        {
            let mut counters = self.pr_counters.write();
            let counter = counters.entry(pr.repo_key.clone()).or_insert(0);
            if pr.number > *counter {
                *counter = pr.number;
            }
        }

        // Update global ID counter if needed
        let current = self.next_id.load(Ordering::SeqCst);
        if pr.id > current {
            self.next_id.store(pr.id, Ordering::SeqCst);
        }

        prs.insert(key, pr);
        Ok(())
    }

    /// Imports an issue (for P2P sync).
    pub fn import_issue(&self, issue: Issue) -> Result<()> {
        let key = (issue.repo_key.clone(), issue.number);
        let mut issues = self.issues.write();

        // Update counters if needed
        {
            let mut counters = self.issue_counters.write();
            let counter = counters.entry(issue.repo_key.clone()).or_insert(0);
            if issue.number > *counter {
                *counter = issue.number;
            }
        }

        // Update global ID counter if needed
        let current = self.next_id.load(Ordering::SeqCst);
        if issue.id > current {
            self.next_id.store(issue.id, Ordering::SeqCst);
        }

        issues.insert(key, issue);
        Ok(())
    }

    /// Imports a comment (for P2P sync).
    pub fn import_comment(&self, comment: Comment) -> Result<()> {
        // Update global ID counter if needed
        let current = self.next_id.load(Ordering::SeqCst);
        if comment.id > current {
            self.next_id.store(comment.id, Ordering::SeqCst);
        }

        self.comments.write().insert(comment.id, comment);
        Ok(())
    }

    /// Imports a review (for P2P sync).
    pub fn import_review(&self, review: Review) -> Result<()> {
        // Update global ID counter if needed
        let current = self.next_id.load(Ordering::SeqCst);
        if review.id > current {
            self.next_id.store(review.id, Ordering::SeqCst);
        }

        self.reviews.write().insert(review.id, review);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReviewState;
    use guts_storage::ObjectId;

    fn create_store_with_pr() -> (CollaborationStore, PullRequest) {
        let store = CollaborationStore::new();

        let pr = PullRequest::new(
            0,
            "alice/repo",
            0,
            "Add feature",
            "Description",
            "alice",
            "feature",
            "main",
            ObjectId::from_bytes([1u8; 20]),
            ObjectId::from_bytes([2u8; 20]),
        );

        let pr = store.create_pull_request(pr).unwrap();
        (store, pr)
    }

    fn create_store_with_issue() -> (CollaborationStore, Issue) {
        let store = CollaborationStore::new();

        let issue = Issue::new(0, "alice/repo", 0, "Bug report", "Description", "alice");

        let issue = store.create_issue(issue).unwrap();
        (store, issue)
    }

    #[test]
    fn test_pr_lifecycle() {
        let (store, pr) = create_store_with_pr();

        assert_eq!(pr.number, 1);
        assert!(pr.is_open());

        // Close
        let pr = store.close_pull_request("alice/repo", 1).unwrap();
        assert!(pr.is_closed());

        // Reopen
        let pr = store.reopen_pull_request("alice/repo", 1).unwrap();
        assert!(pr.is_open());

        // Merge
        let pr = store.merge_pull_request("alice/repo", 1, "bob").unwrap();
        assert!(pr.is_merged());
    }

    #[test]
    fn test_issue_lifecycle() {
        let (store, issue) = create_store_with_issue();

        assert_eq!(issue.number, 1);
        assert!(issue.is_open());

        // Close
        let issue = store.close_issue("alice/repo", 1, "bob").unwrap();
        assert!(issue.is_closed());

        // Reopen
        let issue = store.reopen_issue("alice/repo", 1).unwrap();
        assert!(issue.is_open());
    }

    #[test]
    fn test_list_prs_by_state() {
        let store = CollaborationStore::new();

        // Create 3 PRs
        for i in 0..3 {
            let pr = PullRequest::new(
                0,
                "alice/repo",
                0,
                format!("PR {}", i),
                "Desc",
                "alice",
                format!("feature-{}", i),
                "main",
                ObjectId::from_bytes([i as u8; 20]),
                ObjectId::from_bytes([0u8; 20]),
            );
            store.create_pull_request(pr).unwrap();
        }

        // Close one
        store.close_pull_request("alice/repo", 1).unwrap();

        // Merge one
        store.merge_pull_request("alice/repo", 2, "bob").unwrap();

        // Check filtering
        let open = store.list_pull_requests("alice/repo", Some(PullRequestState::Open));
        assert_eq!(open.len(), 1);

        let closed = store.list_pull_requests("alice/repo", Some(PullRequestState::Closed));
        assert_eq!(closed.len(), 1);

        let merged = store.list_pull_requests("alice/repo", Some(PullRequestState::Merged));
        assert_eq!(merged.len(), 1);

        let all = store.list_pull_requests("alice/repo", None);
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_comments() {
        let (store, pr) = create_store_with_pr();

        // Create comment on PR
        let comment = Comment::new(
            0,
            CommentTarget::pull_request("alice/repo", pr.number),
            "bob",
            "Great work!",
        );
        let comment = store.create_comment(comment).unwrap();
        assert_eq!(comment.id, 2); // ID 1 was used for PR

        // List comments
        let comments = store.list_pr_comments("alice/repo", pr.number);
        assert_eq!(comments.len(), 1);

        // Update comment
        let updated = store.update_comment(comment.id, "Updated text").unwrap();
        assert_eq!(updated.body, "Updated text");

        // Delete comment
        store.delete_comment(comment.id).unwrap();
        let comments = store.list_pr_comments("alice/repo", pr.number);
        assert!(comments.is_empty());
    }

    #[test]
    fn test_reviews() {
        let (store, pr) = create_store_with_pr();

        // Create review
        let review = Review::new(
            0,
            "alice/repo",
            pr.number,
            "bob",
            ReviewState::Approved,
            "abc123",
        )
        .with_body("LGTM!");
        let review = store.create_review(review).unwrap();
        assert!(review.is_approved());

        // List reviews
        let reviews = store.list_reviews("alice/repo", pr.number);
        assert_eq!(reviews.len(), 1);

        // Dismiss review
        let dismissed = store.dismiss_review(review.id).unwrap();
        assert!(dismissed.is_dismissed());
    }

    #[test]
    fn test_pr_not_found() {
        let store = CollaborationStore::new();
        let result = store.get_pull_request("nonexistent/repo", 1);
        assert!(matches!(
            result,
            Err(CollaborationError::PullRequestNotFound { .. })
        ));
    }

    #[test]
    fn test_issue_not_found() {
        let store = CollaborationStore::new();
        let result = store.get_issue("nonexistent/repo", 1);
        assert!(matches!(
            result,
            Err(CollaborationError::IssueNotFound { .. })
        ));
    }

    #[test]
    fn test_comment_on_nonexistent_pr() {
        let store = CollaborationStore::new();
        let comment = Comment::new(
            0,
            CommentTarget::pull_request("alice/repo", 999),
            "bob",
            "Hello",
        );
        let result = store.create_comment(comment);
        assert!(matches!(
            result,
            Err(CollaborationError::PullRequestNotFound { .. })
        ));
    }

    #[test]
    fn test_import_for_p2p_sync() {
        let store = CollaborationStore::new();

        // Import PR with specific ID and number
        let mut pr = PullRequest::new(
            0,
            "alice/repo",
            0,
            "Imported PR",
            "Desc",
            "alice",
            "feature",
            "main",
            ObjectId::from_bytes([1u8; 20]),
            ObjectId::from_bytes([2u8; 20]),
        );
        pr.id = 100;
        pr.number = 50;

        store.import_pull_request(pr).unwrap();

        // Verify import
        let imported = store.get_pull_request("alice/repo", 50).unwrap();
        assert_eq!(imported.id, 100);
        assert_eq!(imported.title, "Imported PR");

        // New PR should get number 51
        let new_pr = PullRequest::new(
            0,
            "alice/repo",
            0,
            "New PR",
            "Desc",
            "bob",
            "branch",
            "main",
            ObjectId::from_bytes([3u8; 20]),
            ObjectId::from_bytes([4u8; 20]),
        );
        let new_pr = store.create_pull_request(new_pr).unwrap();
        assert_eq!(new_pr.number, 51);
    }

    // ==================== Additional Tests ====================

    #[test]
    fn test_multiple_repositories_isolation() {
        let store = CollaborationStore::new();

        // Create issues in different repos
        let issue1 = Issue::new(0, "alice/repo1", 0, "Issue 1", "Desc", "alice");
        let issue2 = Issue::new(0, "alice/repo2", 0, "Issue 2", "Desc", "alice");

        store.create_issue(issue1).unwrap();
        store.create_issue(issue2).unwrap();

        // Each repo should have its own issue #1
        let issues_repo1 = store.list_issues("alice/repo1", None);
        let issues_repo2 = store.list_issues("alice/repo2", None);

        assert_eq!(issues_repo1.len(), 1);
        assert_eq!(issues_repo2.len(), 1);
        assert_eq!(issues_repo1[0].title, "Issue 1");
        assert_eq!(issues_repo2[0].title, "Issue 2");
    }

    #[test]
    fn test_issue_comments() {
        let (store, issue) = create_store_with_issue();

        let comment = Comment::new(
            0,
            CommentTarget::issue("alice/repo", issue.number),
            "bob",
            "I can reproduce this!",
        );
        store.create_comment(comment).unwrap();

        let comments = store.list_issue_comments("alice/repo", issue.number);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].body, "I can reproduce this!");
    }

    #[test]
    fn test_comment_on_nonexistent_issue() {
        let store = CollaborationStore::new();
        let comment = Comment::new(0, CommentTarget::issue("alice/repo", 999), "bob", "Hello");
        let result = store.create_comment(comment);
        assert!(matches!(
            result,
            Err(CollaborationError::IssueNotFound { .. })
        ));
    }

    #[test]
    fn test_review_on_nonexistent_pr() {
        let store = CollaborationStore::new();
        let review = Review::new(0, "alice/repo", 999, "bob", ReviewState::Approved, "abc123");
        let result = store.create_review(review);
        assert!(matches!(
            result,
            Err(CollaborationError::PullRequestNotFound { .. })
        ));
    }

    #[test]
    fn test_get_comment_not_found() {
        let store = CollaborationStore::new();
        let result = store.get_comment(999);
        assert!(matches!(
            result,
            Err(CollaborationError::CommentNotFound { id: 999 })
        ));
    }

    #[test]
    fn test_get_review_not_found() {
        let store = CollaborationStore::new();
        let result = store.get_review(999);
        assert!(matches!(
            result,
            Err(CollaborationError::ReviewNotFound { id: 999 })
        ));
    }

    #[test]
    fn test_update_comment_not_found() {
        let store = CollaborationStore::new();
        let result = store.update_comment(999, "new body");
        assert!(matches!(
            result,
            Err(CollaborationError::CommentNotFound { id: 999 })
        ));
    }

    #[test]
    fn test_delete_comment_not_found() {
        let store = CollaborationStore::new();
        let result = store.delete_comment(999);
        assert!(matches!(
            result,
            Err(CollaborationError::CommentNotFound { id: 999 })
        ));
    }

    #[test]
    fn test_dismiss_review_not_found() {
        let store = CollaborationStore::new();
        let result = store.dismiss_review(999);
        assert!(matches!(
            result,
            Err(CollaborationError::ReviewNotFound { id: 999 })
        ));
    }

    #[test]
    fn test_bulk_operations() {
        let store = CollaborationStore::new();

        // Create multiple PRs
        for i in 0..3 {
            let pr = PullRequest::new(
                0,
                "alice/repo",
                0,
                format!("PR {}", i),
                "Desc",
                "alice",
                format!("feature-{}", i),
                "main",
                ObjectId::from_bytes([i as u8; 20]),
                ObjectId::from_bytes([0u8; 20]),
            );
            store.create_pull_request(pr).unwrap();
        }

        // Create multiple issues
        for i in 0..2 {
            let issue = Issue::new(0, "alice/repo", 0, format!("Issue {}", i), "Desc", "alice");
            store.create_issue(issue).unwrap();
        }

        // Test bulk getters
        assert_eq!(store.all_pull_requests().len(), 3);
        assert_eq!(store.all_issues().len(), 2);
        assert!(store.all_comments().is_empty());
        assert!(store.all_reviews().is_empty());
    }

    #[test]
    fn test_import_issue() {
        let store = CollaborationStore::new();

        let mut issue = Issue::new(0, "alice/repo", 0, "Imported Issue", "Desc", "alice");
        issue.id = 200;
        issue.number = 100;

        store.import_issue(issue).unwrap();

        let imported = store.get_issue("alice/repo", 100).unwrap();
        assert_eq!(imported.id, 200);
        assert_eq!(imported.title, "Imported Issue");

        // New issue should get number 101
        let new_issue = Issue::new(0, "alice/repo", 0, "New Issue", "Desc", "bob");
        let new_issue = store.create_issue(new_issue).unwrap();
        assert_eq!(new_issue.number, 101);
    }

    #[test]
    fn test_import_comment() {
        let (store, pr) = create_store_with_pr();

        let mut comment = Comment::new(
            0,
            CommentTarget::pull_request("alice/repo", pr.number),
            "bob",
            "Imported comment",
        );
        comment.id = 500;

        store.import_comment(comment).unwrap();

        let imported = store.get_comment(500).unwrap();
        assert_eq!(imported.body, "Imported comment");
    }

    #[test]
    fn test_import_review() {
        let (store, pr) = create_store_with_pr();

        let mut review = Review::new(
            0,
            "alice/repo",
            pr.number,
            "bob",
            ReviewState::Approved,
            "abc123",
        );
        review.id = 600;

        store.import_review(review).unwrap();

        let imported = store.get_review(600).unwrap();
        assert!(imported.is_approved());
    }

    #[test]
    fn test_list_issues_by_state() {
        let store = CollaborationStore::new();

        // Create issues
        for i in 0..3 {
            let issue = Issue::new(0, "alice/repo", 0, format!("Issue {}", i), "Desc", "alice");
            store.create_issue(issue).unwrap();
        }

        // Close one
        store.close_issue("alice/repo", 1, "bob").unwrap();

        // Check filtering
        let open = store.list_issues("alice/repo", Some(IssueState::Open));
        assert_eq!(open.len(), 2);

        let closed = store.list_issues("alice/repo", Some(IssueState::Closed));
        assert_eq!(closed.len(), 1);

        let all = store.list_issues("alice/repo", None);
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_concurrent_id_generation() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(CollaborationStore::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let store = Arc::clone(&store);
            handles.push(thread::spawn(move || {
                let issue = Issue::new(0, "alice/repo", 0, "Concurrent issue", "Desc", "alice");
                store.create_issue(issue).unwrap()
            }));
        }

        let mut ids: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap().id).collect();
        ids.sort();

        // All IDs should be unique
        let unique_ids: std::collections::HashSet<_> = ids.iter().cloned().collect();
        assert_eq!(unique_ids.len(), 10);
    }

    #[test]
    fn test_store_default() {
        let store: CollaborationStore = Default::default();
        assert!(store.all_pull_requests().is_empty());
        assert!(store.all_issues().is_empty());
    }

    #[test]
    fn test_pr_update_function() {
        let (store, _) = create_store_with_pr();

        let pr = store
            .update_pull_request("alice/repo", 1, |pr| {
                pr.title = "Updated title".to_string();
                Ok(())
            })
            .unwrap();

        assert_eq!(pr.title, "Updated title");
    }

    #[test]
    fn test_issue_update_function() {
        let (store, _) = create_store_with_issue();

        let issue = store
            .update_issue("alice/repo", 1, |issue| {
                issue.title = "Updated title".to_string();
                Ok(())
            })
            .unwrap();

        assert_eq!(issue.title, "Updated title");
    }
}
