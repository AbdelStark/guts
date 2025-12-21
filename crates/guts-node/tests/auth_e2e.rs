//! End-to-end tests for authorization features (Organizations, Teams, Collaborators, Webhooks).

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

// ==================== Organization Tests ====================

#[tokio::test]
async fn test_create_and_list_organizations() {
    let app = create_test_app();

    // Create an organization
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "acme-corp",
                "display_name": "Acme Corporation",
                "description": "A company that makes everything",
                "creator": "owner_pubkey"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    let org_data = json_body(response).await;
    assert_eq!(org_data["name"], "acme-corp");
    assert_eq!(org_data["display_name"], "Acme Corporation");
    assert_eq!(org_data["member_count"], 1); // Creator is auto-added

    // List organizations
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/orgs")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let orgs: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["name"], "acme-corp");
}

#[tokio::test]
async fn test_get_organization() {
    let app = create_test_app();

    // Create an organization
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "my-org",
                "display_name": "My Organization",
                "creator": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(create_request).await.unwrap();

    // Get the organization
    let get_request = Request::builder()
        .method("GET")
        .uri("/api/orgs/my-org")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let org_data = json_body(response).await;
    assert_eq!(org_data["name"], "my-org");
    assert_eq!(org_data["display_name"], "My Organization");
}

#[tokio::test]
async fn test_update_organization() {
    let app = create_test_app();

    // Create an organization
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "updatable-org",
                "display_name": "Original Name",
                "creator": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(create_request).await.unwrap();

    // Update the organization
    let update_request = Request::builder()
        .method("PATCH")
        .uri("/api/orgs/updatable-org")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "display_name": "Updated Name",
                "description": "New description"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let org_data = json_body(response).await;
    assert_eq!(org_data["display_name"], "Updated Name");
    assert_eq!(org_data["description"], "New description");
}

#[tokio::test]
async fn test_duplicate_organization_fails() {
    let app = create_test_app();

    // Create an organization
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "unique-org",
                "display_name": "Unique Org",
                "creator": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(create_request).await.unwrap();

    // Try to create duplicate
    let duplicate_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "unique-org",
                "display_name": "Another Org",
                "creator": "other"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(duplicate_request).await.unwrap();
    assert_eq!(response.status(), 409); // Conflict
}

// ==================== Organization Member Tests ====================

#[tokio::test]
async fn test_add_and_list_org_members() {
    let app = create_test_app();

    // Create an organization
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "members-org",
                "display_name": "Members Org",
                "creator": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(create_request).await.unwrap();

    // Add a member
    let add_member_request = Request::builder()
        .method("POST")
        .uri("/api/orgs/members-org/members")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "user": "new_member",
                "role": "member",
                "added_by": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(add_member_request).await.unwrap();
    assert_eq!(response.status(), 201);

    // List members
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/orgs/members-org/members")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let members: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(members.len(), 2); // Owner + new member
}

// ==================== Team Tests ====================

#[tokio::test]
async fn test_create_and_list_teams() {
    let app = create_test_app();

    // Create an organization first
    let create_org_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "teams-org",
                "display_name": "Teams Org",
                "creator": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(create_org_request).await.unwrap();

    // Create a team
    let create_team_request = Request::builder()
        .method("POST")
        .uri("/api/orgs/teams-org/teams")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "backend-team",
                "description": "Backend developers",
                "permission": "write",
                "created_by": "owner"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_team_request).await.unwrap();
    assert_eq!(response.status(), 201);

    let team_data = json_body(response).await;
    assert_eq!(team_data["name"], "backend-team");
    assert_eq!(team_data["permission"], "write");

    // List teams
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/orgs/teams-org/teams")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let teams: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(teams.len(), 1);
}

#[tokio::test]
async fn test_team_members() {
    let app = create_test_app();

    // Create org and team
    let create_org_request = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "team-members-org",
                "display_name": "Team Members Org",
                "creator": "owner"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(create_org_request).await.unwrap();

    let create_team_request = Request::builder()
        .method("POST")
        .uri("/api/orgs/team-members-org/teams")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "devs",
                "permission": "write",
                "created_by": "owner"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(create_team_request).await.unwrap();

    // Add team member
    let add_member_request = Request::builder()
        .method("PUT")
        .uri("/api/orgs/team-members-org/teams/devs/members/dev1")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(add_member_request).await.unwrap();
    assert_eq!(response.status(), 204);

    // List team members
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/orgs/team-members-org/teams/devs/members")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let members: Vec<String> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(members.len(), 1);
    assert!(members.contains(&"dev1".to_string()));
}

// ==================== Collaborator Tests ====================

#[tokio::test]
async fn test_add_and_list_collaborators() {
    let app = create_test_app();

    // Add a collaborator
    let add_request = Request::builder()
        .method("PUT")
        .uri("/api/repos/alice/myrepo/collaborators/bob")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "permission": "write",
                "added_by": "alice"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(add_request).await.unwrap();
    assert_eq!(response.status(), 201);

    let collab_data = json_body(response).await;
    assert_eq!(collab_data["user"], "bob");
    assert_eq!(collab_data["permission"], "write");

    // List collaborators
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/alice/myrepo/collaborators")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let collaborators: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(collaborators.len(), 1);
}

#[tokio::test]
async fn test_remove_collaborator() {
    let app = create_test_app();

    // Add a collaborator
    let add_request = Request::builder()
        .method("PUT")
        .uri("/api/repos/alice/repo2/collaborators/charlie")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "permission": "read",
                "added_by": "alice"
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(add_request).await.unwrap();

    // Remove the collaborator
    let remove_request = Request::builder()
        .method("DELETE")
        .uri("/api/repos/alice/repo2/collaborators/charlie")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(remove_request).await.unwrap();
    assert_eq!(response.status(), 204);

    // Verify collaborator is removed
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/alice/repo2/collaborators")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let collaborators: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(collaborators.len(), 0);
}

// ==================== Permission Check Tests ====================

#[tokio::test]
async fn test_check_permission_owner() {
    let app = create_test_app();

    // Check owner permission
    let check_request = Request::builder()
        .method("GET")
        .uri("/api/repos/alice/myrepo/permission/alice")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(check_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let perm_data = json_body(response).await;
    assert_eq!(perm_data["permission"], "admin");
    assert_eq!(perm_data["has_access"], true);
}

#[tokio::test]
async fn test_check_permission_collaborator() {
    let app = create_test_app();

    // Add collaborator with write permission
    let add_request = Request::builder()
        .method("PUT")
        .uri("/api/repos/dave/project/collaborators/eve")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "permission": "write",
                "added_by": "dave"
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(add_request).await.unwrap();

    // Check eve's permission
    let check_request = Request::builder()
        .method("GET")
        .uri("/api/repos/dave/project/permission/eve")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(check_request).await.unwrap();
    let perm_data = json_body(response).await;
    assert_eq!(perm_data["permission"], "write");
    assert_eq!(perm_data["has_access"], true);
}

#[tokio::test]
async fn test_check_permission_no_access() {
    let app = create_test_app();

    // Check permission for unknown user
    let check_request = Request::builder()
        .method("GET")
        .uri("/api/repos/frank/private/permission/stranger")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(check_request).await.unwrap();
    let perm_data = json_body(response).await;
    assert_eq!(perm_data["permission"], serde_json::Value::Null);
    assert_eq!(perm_data["has_access"], false);
}

// ==================== Branch Protection Tests ====================

#[tokio::test]
async fn test_set_and_get_branch_protection() {
    let app = create_test_app();

    // Set branch protection
    let set_request = Request::builder()
        .method("PUT")
        .uri("/api/repos/grace/app/branches/main/protection")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "require_pr": true,
                "required_reviews": 2,
                "required_status_checks": ["ci/build", "ci/test"],
                "dismiss_stale_reviews": true,
                "require_code_owner_review": false,
                "restrict_pushes": true,
                "allow_force_push": false,
                "allow_deletion": false
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(set_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let protection_data = json_body(response).await;
    assert_eq!(protection_data["pattern"], "main");
    assert_eq!(protection_data["require_pr"], true);
    assert_eq!(protection_data["required_reviews"], 2);

    // Get branch protection
    let get_request = Request::builder()
        .method("GET")
        .uri("/api/repos/grace/app/branches/main/protection")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let protection_data = json_body(response).await;
    assert_eq!(protection_data["restrict_pushes"], true);
}

#[tokio::test]
async fn test_remove_branch_protection() {
    let app = create_test_app();

    // Set branch protection first
    let set_request = Request::builder()
        .method("PUT")
        .uri("/api/repos/henry/lib/branches/release/protection")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "require_pr": true,
                "required_reviews": 1,
                "required_status_checks": [],
                "dismiss_stale_reviews": false,
                "require_code_owner_review": false,
                "restrict_pushes": false,
                "allow_force_push": false,
                "allow_deletion": false
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(set_request).await.unwrap();

    // Remove branch protection
    let remove_request = Request::builder()
        .method("DELETE")
        .uri("/api/repos/henry/lib/branches/release/protection")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(remove_request).await.unwrap();
    assert_eq!(response.status(), 204);

    // Verify it's removed
    let get_request = Request::builder()
        .method("GET")
        .uri("/api/repos/henry/lib/branches/release/protection")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(response.status(), 404);
}

// ==================== Webhook Tests ====================

#[tokio::test]
async fn test_create_and_list_webhooks() {
    let app = create_test_app();

    // Create a webhook
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/ivy/service/hooks")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook",
                "secret": "my-secret",
                "events": ["push", "pull_request"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), 201);

    let webhook_data = json_body(response).await;
    assert_eq!(webhook_data["url"], "https://example.com/webhook");
    assert!(webhook_data["active"].as_bool().unwrap());

    // List webhooks
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/ivy/service/hooks")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let webhooks: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(webhooks.len(), 1);
}

#[tokio::test]
async fn test_update_webhook() {
    let app = create_test_app();

    // Create a webhook
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/jack/api/hooks")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "url": "https://old-url.com/hook",
                "events": ["push"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    let webhook_data = json_body(response).await;
    let webhook_id = webhook_data["id"].as_u64().unwrap();

    // Update the webhook
    let update_request = Request::builder()
        .method("PATCH")
        .uri(format!("/api/repos/jack/api/hooks/{}", webhook_id))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "url": "https://new-url.com/hook",
                "events": ["push", "issue"],
                "active": false
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let webhook_data = json_body(response).await;
    assert_eq!(webhook_data["url"], "https://new-url.com/hook");
    assert_eq!(webhook_data["active"], false);
}

#[tokio::test]
async fn test_delete_webhook() {
    let app = create_test_app();

    // Create a webhook
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/kate/repo/hooks")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "url": "https://delete-me.com/hook",
                "events": ["push"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    let webhook_data = json_body(response).await;
    let webhook_id = webhook_data["id"].as_u64().unwrap();

    // Delete the webhook
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/repos/kate/repo/hooks/{}", webhook_id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(response.status(), 204);

    // Verify it's deleted
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/repos/kate/repo/hooks")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_request).await.unwrap();
    let webhooks: Vec<Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(webhooks.len(), 0);
}

#[tokio::test]
async fn test_ping_webhook() {
    let app = create_test_app();

    // Create a webhook
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/repos/leo/app/hooks")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "url": "https://ping-me.com/hook",
                "events": ["push"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    let webhook_data = json_body(response).await;
    let webhook_id = webhook_data["id"].as_u64().unwrap();

    // Ping the webhook
    let ping_request = Request::builder()
        .method("POST")
        .uri(format!("/api/repos/leo/app/hooks/{}/ping", webhook_id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(ping_request).await.unwrap();
    assert_eq!(response.status(), 200);

    let ping_data = json_body(response).await;
    assert!(ping_data["message"].as_str().unwrap().contains("success"));
}

// ==================== Organization + Team Permission Integration ====================

#[tokio::test]
async fn test_org_admin_has_repo_access() {
    let app = create_test_app();

    // Create an organization
    let create_org = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "perm-test-org",
                "display_name": "Permission Test Org",
                "creator": "org_owner"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(create_org).await.unwrap();

    // Add an admin member
    let add_admin = Request::builder()
        .method("POST")
        .uri("/api/orgs/perm-test-org/members")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "user": "org_admin",
                "role": "admin",
                "added_by": "org_owner"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(add_admin).await.unwrap();

    // Check admin's permission on org-owned repo
    let check_request = Request::builder()
        .method("GET")
        .uri("/api/repos/perm-test-org/some-repo/permission/org_admin")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(check_request).await.unwrap();
    let perm_data = json_body(response).await;

    // Org admins have admin access to all org repos
    assert_eq!(perm_data["permission"], "admin");
    assert_eq!(perm_data["has_access"], true);
}

#[tokio::test]
async fn test_team_member_has_team_repo_access() {
    let app = create_test_app();

    // Create org
    let create_org = Request::builder()
        .method("POST")
        .uri("/api/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "team-perm-org",
                "display_name": "Team Permission Org",
                "creator": "owner"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(create_org).await.unwrap();

    // Create team with write permission
    let create_team = Request::builder()
        .method("POST")
        .uri("/api/orgs/team-perm-org/teams")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "writers",
                "permission": "write",
                "created_by": "owner"
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(create_team).await.unwrap();

    // Add member to team
    let add_member = Request::builder()
        .method("PUT")
        .uri("/api/orgs/team-perm-org/teams/writers/members/team_member")
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(add_member).await.unwrap();

    // Add repo to team
    let add_repo = Request::builder()
        .method("PUT")
        .uri("/api/orgs/team-perm-org/teams/writers/repos/team-perm-org/team-repo")
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(add_repo).await.unwrap();

    // Check team member's permission on team repo
    let check_request = Request::builder()
        .method("GET")
        .uri("/api/repos/team-perm-org/team-repo/permission/team_member")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(check_request).await.unwrap();
    let perm_data = json_body(response).await;

    assert_eq!(perm_data["permission"], "write");
    assert_eq!(perm_data["has_access"], true);
}
