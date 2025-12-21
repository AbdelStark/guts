//! End-to-end tests for collaboration features (PRs, Issues, Comments, Reviews).

use axum::{body::Body, http::Request};
use guts_collaboration::CollaborationStore;
use guts_node::{api::{create_router, AppState, RepoStore}};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

fn create_test_app() -> axum::Router {
    let state = AppState {
        repos: Arc::new(RepoStore::new()),
        p2p: None,
        collaboration: Arc::new(CollaborationStore::new()),
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
