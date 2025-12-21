//! End-to-end tests for collaboration features (PRs, Issues, Comments, Reviews).

use axum::{body::Body, http::Request};
use guts_auth::AuthStore;
use guts_collaboration::CollaborationStore;
use guts_node::api::{create_router, AppState, RepoStore};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

fn create_test_app() -> axum::Router {
    let state = AppState {
        repos: Arc::new(RepoStore::new()),
        p2p: None,
        collaboration: Arc::new(CollaborationStore::new()),
        auth: Arc::new(AuthStore::new()),
    };
    create_router(state)
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_create_and_list_pull_requests() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/alice/myrepo/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Add new feature",
                "description": "This PR adds a new feature",
                "author": "alice_pubkey",
                "source_branch": "feature-branch",
                "target_branch": "main",
                "source_commit": "0".repeat(40),
                "target_commit": "1".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    let pr_data = json_body(response).await;
    assert_eq!(pr_data["title"], "Add new feature");
    assert_eq!(pr_data["number"], 1);
    assert_eq!(pr_data["state"], "open");

    // List PRs
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/alice/myrepo/pulls")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let prs: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(prs.len(), 1);
    assert_eq!(prs[0]["title"], "Add new feature");
}

#[tokio::test]
async fn test_pr_lifecycle_merge() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/bob/project/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Bug fix",
                "description": "Fixes issue #1",
                "author": "bob",
                "source_branch": "fix-1",
                "target_branch": "main",
                "source_commit": "2".repeat(40),
                "target_commit": "3".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Merge the PR
    let merge_request = Request::builder()
        .method("POST")
        .uri("/api/repos/bob/project/pulls/1/merge")
        .header("content-type", "application/json")
        .body(Body::from(json!({"merged_by": "carol"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(merge_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let pr_data = json_body(response).await;
    assert_eq!(pr_data["state"], "merged");
    assert_eq!(pr_data["merged_by"], "carol");

    // Verify merged PRs can be listed
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/bob/project/pulls?state=merged")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let prs: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(prs.len(), 1);
}

#[tokio::test]
async fn test_create_and_list_issues() {
    let app = create_test_app();

    // Create an issue
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/dave/code/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Bug: Something is broken",
                "description": "Steps to reproduce...",
                "author": "dave_pubkey"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    let issue_data = json_body(response).await;
    assert_eq!(issue_data["title"], "Bug: Something is broken");
    assert_eq!(issue_data["number"], 1);
    assert_eq!(issue_data["state"], "open");

    // Create a second issue
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/dave/code/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Feature request",
                "description": "Would be nice to have...",
                "author": "eve_pubkey"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // List all issues
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/dave/code/issues")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let issues: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(issues.len(), 2);
}

#[tokio::test]
async fn test_issue_close_and_reopen() {
    let app = create_test_app();

    // Create an issue
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/frank/app/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Test issue",
                "description": "Testing close/reopen",
                "author": "frank"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Close the issue
    let close_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/frank/app/issues/1")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"state": "closed", "closed_by": "grace"}).to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(close_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let issue_data = json_body(response).await;
    assert_eq!(issue_data["state"], "closed");
    assert_eq!(issue_data["closed_by"], "grace");

    // Reopen the issue
    let reopen_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/frank/app/issues/1")
        .header("content-type", "application/json")
        .body(Body::from(json!({"state": "open"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(reopen_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let issue_data = json_body(response).await;
    assert_eq!(issue_data["state"], "open");
}

#[tokio::test]
async fn test_pr_comments() {
    let app = create_test_app();

    // Create a PR first
    let create_pr = Request::builder()
        .method("POST")
        .uri("/api/repos/henry/lib/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Add tests",
                "description": "Adding unit tests",
                "author": "henry",
                "source_branch": "tests",
                "target_branch": "main",
                "source_commit": "4".repeat(40),
                "target_commit": "5".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_pr).await.unwrap();
    assert_eq!(response.status(), 201);

    // Add a comment
    let add_comment = Request::builder()
        .method("POST")
        .uri("/api/repos/henry/lib/pulls/1/comments")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "ivy",
                "body": "Great work! LGTM"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(add_comment).await.unwrap();
    assert_eq!(response.status(), 201);

    let comment_data = json_body(response).await;
    assert_eq!(comment_data["body"], "Great work! LGTM");
    assert_eq!(comment_data["author"], "ivy");

    // Add another comment
    let add_comment2 = Request::builder()
        .method("POST")
        .uri("/api/repos/henry/lib/pulls/1/comments")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "jack",
                "body": "Could you add more tests?"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(add_comment2).await.unwrap();
    assert_eq!(response.status(), 201);

    // List comments
    let list_comments = Request::builder()
        .method("GET")
        .uri("/api/repos/henry/lib/pulls/1/comments")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_comments).await.unwrap();
    assert_eq!(response.status(), 200);

    let comments: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(comments.len(), 2);
}

#[tokio::test]
async fn test_pr_reviews() {
    let app = create_test_app();

    // Create a PR first
    let create_pr = Request::builder()
        .method("POST")
        .uri("/api/repos/kate/service/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Refactor auth",
                "description": "Refactoring authentication",
                "author": "kate",
                "source_branch": "refactor",
                "target_branch": "main",
                "source_commit": "6".repeat(40),
                "target_commit": "7".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_pr).await.unwrap();
    assert_eq!(response.status(), 201);

    // Submit an approved review
    let submit_review = Request::builder()
        .method("POST")
        .uri("/api/repos/kate/service/pulls/1/reviews")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "leo",
                "state": "approved",
                "body": "LGTM!",
                "commit_id": "abc123"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(submit_review).await.unwrap();
    assert_eq!(response.status(), 201);

    let review_data = json_body(response).await;
    assert_eq!(review_data["state"], "approved");
    assert_eq!(review_data["author"], "leo");

    // Submit a changes requested review
    let submit_review2 = Request::builder()
        .method("POST")
        .uri("/api/repos/kate/service/pulls/1/reviews")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "mary",
                "state": "changes_requested",
                "body": "Please add error handling",
                "commit_id": "abc123"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(submit_review2).await.unwrap();
    assert_eq!(response.status(), 201);

    // List reviews
    let list_reviews = Request::builder()
        .method("GET")
        .uri("/api/repos/kate/service/pulls/1/reviews")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_reviews).await.unwrap();
    assert_eq!(response.status(), 200);

    let reviews: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(reviews.len(), 2);
}

#[tokio::test]
async fn test_issue_comments() {
    let app = create_test_app();

    // Create an issue first
    let create_issue = Request::builder()
        .method("POST")
        .uri("/api/repos/nick/app/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Login fails",
                "description": "Cannot login with correct password",
                "author": "nick"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_issue).await.unwrap();
    assert_eq!(response.status(), 201);

    // Add a comment
    let add_comment = Request::builder()
        .method("POST")
        .uri("/api/repos/nick/app/issues/1/comments")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "olivia",
                "body": "I can reproduce this issue"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(add_comment).await.unwrap();
    assert_eq!(response.status(), 201);

    // List comments
    let list_comments = Request::builder()
        .method("GET")
        .uri("/api/repos/nick/app/issues/1/comments")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_comments).await.unwrap();
    assert_eq!(response.status(), 200);

    let comments: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0]["body"], "I can reproduce this issue");
}

#[tokio::test]
async fn test_get_nonexistent_pr_returns_404() {
    let app = create_test_app();

    let request = Request::builder()
        .method("GET")
        .uri("/api/repos/unknown/repo/pulls/999")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_issue_returns_404() {
    let app = create_test_app();

    let request = Request::builder()
        .method("GET")
        .uri("/api/repos/unknown/repo/issues/999")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 404);
}

// ==================== Additional E2E Tests ====================

#[tokio::test]
async fn test_pr_close_and_reopen() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/pat/project/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Feature",
                "description": "Adding feature",
                "author": "pat",
                "source_branch": "feature",
                "target_branch": "main",
                "source_commit": "a".repeat(40),
                "target_commit": "b".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Close the PR
    let close_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/pat/project/pulls/1")
        .header("content-type", "application/json")
        .body(Body::from(json!({"state": "closed"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(close_request).await.unwrap();
    assert_eq!(response.status(), 200);
    let pr_data = json_body(response).await;
    assert_eq!(pr_data["state"], "closed");

    // Reopen the PR
    let reopen_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/pat/project/pulls/1")
        .header("content-type", "application/json")
        .body(Body::from(json!({"state": "open"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(reopen_request).await.unwrap();
    assert_eq!(response.status(), 200);
    let pr_data = json_body(response).await;
    assert_eq!(pr_data["state"], "open");
}

#[tokio::test]
async fn test_cannot_merge_closed_pr() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/quinn/project/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Feature",
                "description": "Adding feature",
                "author": "quinn",
                "source_branch": "feature",
                "target_branch": "main",
                "source_commit": "c".repeat(40),
                "target_commit": "d".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Close the PR
    let close_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/quinn/project/pulls/1")
        .header("content-type", "application/json")
        .body(Body::from(json!({"state": "closed"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(close_request).await.unwrap();
    assert_eq!(response.status(), 200);

    // Try to merge the closed PR - should fail
    let merge_request = Request::builder()
        .method("POST")
        .uri("/api/repos/quinn/project/pulls/1/merge")
        .header("content-type", "application/json")
        .body(Body::from(json!({"merged_by": "rachel"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(merge_request).await.unwrap();
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_cannot_reopen_merged_pr() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/sam/project/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Feature",
                "description": "Adding feature",
                "author": "sam",
                "source_branch": "feature",
                "target_branch": "main",
                "source_commit": "e".repeat(40),
                "target_commit": "f".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Merge the PR
    let merge_request = Request::builder()
        .method("POST")
        .uri("/api/repos/sam/project/pulls/1/merge")
        .header("content-type", "application/json")
        .body(Body::from(json!({"merged_by": "tom"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(merge_request).await.unwrap();
    assert_eq!(response.status(), 200);

    // Try to reopen merged PR - should fail
    let reopen_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/sam/project/pulls/1")
        .header("content-type", "application/json")
        .body(Body::from(json!({"state": "open"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(reopen_request).await.unwrap();
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_filter_prs_by_state() {
    let app = create_test_app();

    // Create 3 PRs
    for i in 0..3 {
        let create_request = Request::builder()
            .method("POST")
            .uri("/api/repos/uma/repo/pulls")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "title": format!("PR {}", i),
                    "description": "Description",
                    "author": "uma",
                    "source_branch": format!("feature-{}", i),
                    "target_branch": "main",
                    "source_commit": format!("{}", i).repeat(40),
                    "target_commit": "0".repeat(40)
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(response.status(), 201);
    }

    // Close PR #1
    let close_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/uma/repo/pulls/1")
        .header("content-type", "application/json")
        .body(Body::from(json!({"state": "closed"}).to_string()))
        .unwrap();
    app.clone().oneshot(close_request).await.unwrap();

    // Merge PR #2
    let merge_request = Request::builder()
        .method("POST")
        .uri("/api/repos/uma/repo/pulls/2/merge")
        .header("content-type", "application/json")
        .body(Body::from(json!({"merged_by": "vic"}).to_string()))
        .unwrap();
    app.clone().oneshot(merge_request).await.unwrap();

    // Filter by open state
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/uma/repo/pulls?state=open")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let prs: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(prs.len(), 1);

    // Filter by closed state
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/uma/repo/pulls?state=closed")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let prs: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(prs.len(), 1);

    // Filter by merged state
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/uma/repo/pulls?state=merged")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let prs: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(prs.len(), 1);
}

#[tokio::test]
async fn test_filter_issues_by_state() {
    let app = create_test_app();

    // Create 2 issues
    for i in 0..2 {
        let create_request = Request::builder()
            .method("POST")
            .uri("/api/repos/walt/repo/issues")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "title": format!("Issue {}", i),
                    "description": "Description",
                    "author": "walt"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(response.status(), 201);
    }

    // Close issue #1
    let close_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/walt/repo/issues/1")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"state": "closed", "closed_by": "xavier"}).to_string(),
        ))
        .unwrap();
    app.clone().oneshot(close_request).await.unwrap();

    // Filter by open state
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/walt/repo/issues?state=open")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let issues: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(issues.len(), 1);

    // Filter by closed state
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/walt/repo/issues?state=closed")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let issues: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(issues.len(), 1);
}

#[tokio::test]
async fn test_get_single_pr() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/yara/repo/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "My PR",
                "description": "Description of my PR",
                "author": "yara",
                "source_branch": "feature",
                "target_branch": "main",
                "source_commit": "1".repeat(40),
                "target_commit": "2".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Get the PR
    let get_request = Request::builder()
        .method("GET")
        .uri("/api/repos/yara/repo/pulls/1")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let pr_data = json_body(response).await;
    assert_eq!(pr_data["title"], "My PR");
    assert_eq!(pr_data["number"], 1);
    assert_eq!(pr_data["author"], "yara");
}

#[tokio::test]
async fn test_get_single_issue() {
    let app = create_test_app();

    // Create an issue
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/zack/repo/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "My Issue",
                "description": "Description of my issue",
                "author": "zack"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Get the issue
    let get_request = Request::builder()
        .method("GET")
        .uri("/api/repos/zack/repo/issues/1")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let issue_data = json_body(response).await;
    assert_eq!(issue_data["title"], "My Issue");
    assert_eq!(issue_data["number"], 1);
    assert_eq!(issue_data["author"], "zack");
}

#[tokio::test]
async fn test_update_pr_title_and_description() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/anna/repo/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Original Title",
                "description": "Original description",
                "author": "anna",
                "source_branch": "feature",
                "target_branch": "main",
                "source_commit": "3".repeat(40),
                "target_commit": "4".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Update the PR
    let update_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/anna/repo/pulls/1")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Updated Title",
                "description": "Updated description"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let pr_data = json_body(response).await;
    assert_eq!(pr_data["title"], "Updated Title");
    assert_eq!(pr_data["description"], "Updated description");
}

#[tokio::test]
async fn test_update_issue_title_and_description() {
    let app = create_test_app();

    // Create an issue
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/ben/repo/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Original Title",
                "description": "Original description",
                "author": "ben"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Update the issue
    let update_request = Request::builder()
        .method("PATCH")
        .uri("/api/repos/ben/repo/issues/1")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Updated Title",
                "description": "Updated description"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let issue_data = json_body(response).await;
    assert_eq!(issue_data["title"], "Updated Title");
    assert_eq!(issue_data["description"], "Updated description");
}

#[tokio::test]
async fn test_review_commented_state() {
    let app = create_test_app();

    // Create a PR
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/clara/repo/pulls")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "PR for review",
                "description": "Description",
                "author": "clara",
                "source_branch": "feature",
                "target_branch": "main",
                "source_commit": "5".repeat(40),
                "target_commit": "6".repeat(40)
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(create_request).await.unwrap();

    // Submit a commented review
    let submit_review = Request::builder()
        .method("POST")
        .uri("/api/repos/clara/repo/pulls/1/reviews")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "dan",
                "state": "commented",
                "body": "Just a general comment",
                "commit_id": "abc123"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(submit_review).await.unwrap();
    assert_eq!(response.status(), 201);

    let review_data = json_body(response).await;
    assert_eq!(review_data["state"], "commented");
}

#[tokio::test]
async fn test_multiple_repos_isolation() {
    let app = create_test_app();

    // Create issue in repo1
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/owner/repo1/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Issue in repo1",
                "description": "Description",
                "author": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // Create issue in repo2
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/owner/repo2/issues")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Issue in repo2",
                "description": "Description",
                "author": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // List issues in repo1 - should only have 1
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/owner/repo1/issues")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let issues: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["title"], "Issue in repo1");

    // List issues in repo2 - should only have 1
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/owner/repo2/issues")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let issues: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["title"], "Issue in repo2");
}

#[tokio::test]
async fn test_comment_on_nonexistent_pr_returns_404() {
    let app = create_test_app();

    let add_comment = Request::builder()
        .method("POST")
        .uri("/api/repos/owner/repo/pulls/999/comments")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "someone",
                "body": "This should fail"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(add_comment).await.unwrap();
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_comment_on_nonexistent_issue_returns_404() {
    let app = create_test_app();

    let add_comment = Request::builder()
        .method("POST")
        .uri("/api/repos/owner/repo/issues/999/comments")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "author": "someone",
                "body": "This should fail"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(add_comment).await.unwrap();
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_merge_nonexistent_pr_returns_404() {
    let app = create_test_app();

    let merge_request = Request::builder()
        .method("POST")
        .uri("/api/repos/owner/repo/pulls/999/merge")
        .header("content-type", "application/json")
        .body(Body::from(json!({"merged_by": "someone"}).to_string()))
        .unwrap();

    let response = app.oneshot(merge_request).await.unwrap();
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_pr_number_increments() {
    let app = create_test_app();

    for i in 1..=3 {
        let create_request = Request::builder()
            .method("POST")
            .uri("/api/repos/ellen/repo/pulls")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "title": format!("PR {}", i),
                    "description": "Description",
                    "author": "ellen",
                    "source_branch": format!("feature-{}", i),
                    "target_branch": "main",
                    "source_commit": format!("{}", i).repeat(40),
                    "target_commit": "0".repeat(40)
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(create_request).await.unwrap();
        let pr_data = json_body(response).await;
        assert_eq!(pr_data["number"], i);
    }
}

#[tokio::test]
async fn test_issue_number_increments() {
    let app = create_test_app();

    for i in 1..=3 {
        let create_request = Request::builder()
            .method("POST")
            .uri("/api/repos/frank2/repo/issues")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "title": format!("Issue {}", i),
                    "description": "Description",
                    "author": "frank2"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(create_request).await.unwrap();
        let issue_data = json_body(response).await;
        assert_eq!(issue_data["number"], i);
    }
}
