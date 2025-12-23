# ADR-004: Collaboration Data Model

## Status

Accepted

## Date

2025-12-20

## Context

Guts needs to support GitHub-style collaboration features:

- Pull Requests with merge workflows
- Issues with labels and state management
- Comments on PRs, issues, and code
- Code reviews with approvals

These features must work in a decentralized environment where:
- Multiple nodes may receive updates concurrently
- State must eventually converge across nodes
- Operations must be cryptographically attributable

## Decision

We implement collaboration features in `guts-collaboration` crate with the following data model:

### Core Types

```rust
/// Pull Request - merge proposal between branches
pub struct PullRequest {
    pub id: Uuid,
    pub repo_key: String,           // "owner/repo"
    pub number: u64,                // Sequential number
    pub title: String,
    pub description: String,
    pub author: String,             // Creator's identity
    pub source_branch: String,      // Feature branch
    pub target_branch: String,      // Usually "main"
    pub source_commit: String,      // HEAD of source
    pub target_commit: String,      // HEAD of target
    pub state: PullRequestState,    // Open, Closed, Merged
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub merged_by: Option<String>,
}

/// Issue - bug report, feature request, task
pub struct Issue {
    pub id: Uuid,
    pub repo_key: String,
    pub number: u64,
    pub title: String,
    pub description: String,
    pub author: String,
    pub state: IssueState,          // Open, Closed
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Comment - discussion on PR or Issue
pub struct Comment {
    pub id: Uuid,
    pub target: CommentTarget,      // PR or Issue reference
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Review - code review on a PR
pub struct Review {
    pub id: Uuid,
    pub repo_key: String,
    pub pr_number: u64,
    pub author: String,
    pub state: ReviewState,         // Pending, Approved, etc.
    pub body: Option<String>,
    pub commit_id: String,          // Reviewed commit
    pub submitted_at: DateTime<Utc>,
}
```

### State Machines

```
PullRequest States:
  Open -> Closed (close)
  Closed -> Open (reopen)
  Open -> Merged (merge) [terminal]

Issue States:
  Open -> Closed (close)
  Closed -> Open (reopen)

Review States:
  Pending -> Commented | Approved | ChangesRequested (submit)
  * -> Dismissed (dismiss)
```

### Storage Layer

```rust
pub trait CollaborationStore: Send + Sync {
    // Pull Requests
    async fn create_pr(&self, pr: &PullRequest) -> Result<()>;
    async fn get_pr(&self, repo: &str, number: u64) -> Result<Option<PullRequest>>;
    async fn update_pr(&self, pr: &PullRequest) -> Result<()>;
    async fn list_prs(&self, repo: &str, state: Option<PullRequestState>) -> Result<Vec<PullRequest>>;

    // Issues
    async fn create_issue(&self, issue: &Issue) -> Result<()>;
    async fn get_issue(&self, repo: &str, number: u64) -> Result<Option<Issue>>;
    // ... similar pattern

    // Comments & Reviews
    async fn add_comment(&self, comment: &Comment) -> Result<()>;
    async fn submit_review(&self, review: &Review) -> Result<()>;
    // ... etc
}
```

## Consequences

### Positive

- **Familiar model**: Matches GitHub/GitLab mental model
- **Rich workflows**: Supports review cycles and merge requirements
- **Auditable**: All actions have author attribution
- **Extensible**: Labels, assignments can be added later

### Negative

- **Eventual consistency**: Concurrent updates may need conflict resolution
- **State management**: Complex state machines to maintain
- **Storage growth**: Comments and reviews can be voluminous

### Neutral

- Sequential numbering requires coordination
- Cross-repository references need consideration

## Design Decisions

### UUIDs + Sequential Numbers

We use both:
- **UUID**: Globally unique identifier for referencing
- **Number**: Human-friendly sequential ID (like GitHub's #123)

The sequential number is assigned during creation and provides a user-friendly reference within a repository.

### Soft State vs Hard State

- **Soft state** (can be recomputed): PR mergability, review summary
- **Hard state** (must be stored): Comments, reviews, core PR data

### Replication Strategy

Collaboration data replicates via the P2P layer using:
- Create/update messages broadcast to all nodes
- Eventual consistency with last-write-wins for conflicts
- Logical clocks for ordering (future enhancement)

## Alternatives Considered

### CRDT-based Model

Use Conflict-free Replicated Data Types for all collaboration.

**Deferred because:**
- Increased complexity
- Standard model sufficient for MVP
- Can migrate later if needed

### Append-only Log

Store only operations, compute state on read.

**Rejected because:**
- Read performance concerns
- Snapshot management complexity
- Harder to query

## References

- [GitHub Pull Request API](https://docs.github.com/en/rest/pulls)
- [GitHub Issues API](https://docs.github.com/en/rest/issues)
- [Guts Milestone 3 Spec](../MILESTONE-3.md)
