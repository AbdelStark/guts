# Milestone 3: Collaboration Features

> **Status:** Completed
> **Started:** 2025-12-21
> **Completed:** 2025-12-21

## Overview

Milestone 3 implements the collaboration features that enable developers to work together on decentralized repositories. This includes Pull Requests, Issues, Comments, and Code Review infrastructure.

## Goals

1. **Pull Request System**: Create, review, and merge code changes
2. **Issue Tracking**: Track bugs, features, and tasks
3. **Comments & Discussions**: Enable threaded conversations on PRs and Issues
4. **P2P Replication**: Synchronize collaboration data across nodes

## Architecture

### New Crate: `guts-collaboration`

```
crates/guts-collaboration/
├── src/
│   ├── lib.rs           # Public API
│   ├── pull_request.rs  # PR types and logic
│   ├── issue.rs         # Issue types and logic
│   ├── comment.rs       # Comment types
│   ├── review.rs        # Code review types
│   ├── label.rs         # Label types
│   ├── store.rs         # In-memory collaboration store
│   └── error.rs         # Error types
└── Cargo.toml
```

### Data Models

#### Pull Request

```rust
pub struct PullRequest {
    pub id: u64,
    pub repo_key: String,          // "owner/repo"
    pub number: u32,               // PR #1, #2, etc.
    pub title: String,
    pub description: String,
    pub author: String,            // Public key (hex)
    pub state: PullRequestState,   // Open, Closed, Merged
    pub source_branch: String,     // "feature-branch"
    pub target_branch: String,     // "main"
    pub source_commit: ObjectId,   // Head commit of source
    pub target_commit: ObjectId,   // Head commit of target
    pub created_at: u64,           // Unix timestamp
    pub updated_at: u64,
    pub merged_at: Option<u64>,
    pub merged_by: Option<String>,
}

pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}
```

#### Issue

```rust
pub struct Issue {
    pub id: u64,
    pub repo_key: String,
    pub number: u32,               // Issue #1, #2, etc.
    pub title: String,
    pub description: String,
    pub author: String,
    pub state: IssueState,         // Open, Closed
    pub labels: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub closed_at: Option<u64>,
    pub closed_by: Option<String>,
}

pub enum IssueState {
    Open,
    Closed,
}
```

#### Comment

```rust
pub struct Comment {
    pub id: u64,
    pub target: CommentTarget,     // PR or Issue
    pub author: String,
    pub body: String,
    pub created_at: u64,
    pub updated_at: u64,
}

pub enum CommentTarget {
    PullRequest { repo_key: String, number: u32 },
    Issue { repo_key: String, number: u32 },
}
```

#### Review

```rust
pub struct Review {
    pub id: u64,
    pub pr_number: u32,
    pub repo_key: String,
    pub author: String,
    pub state: ReviewState,        // Approved, ChangesRequested, Commented
    pub body: Option<String>,
    pub created_at: u64,
}

pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
}
```

### API Endpoints

#### Pull Requests

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{name}/pulls` | List pull requests |
| POST | `/api/repos/{owner}/{name}/pulls` | Create pull request |
| GET | `/api/repos/{owner}/{name}/pulls/{number}` | Get pull request |
| PATCH | `/api/repos/{owner}/{name}/pulls/{number}` | Update pull request |
| POST | `/api/repos/{owner}/{name}/pulls/{number}/merge` | Merge pull request |
| GET | `/api/repos/{owner}/{name}/pulls/{number}/comments` | List PR comments |
| POST | `/api/repos/{owner}/{name}/pulls/{number}/comments` | Add PR comment |
| GET | `/api/repos/{owner}/{name}/pulls/{number}/reviews` | List reviews |
| POST | `/api/repos/{owner}/{name}/pulls/{number}/reviews` | Submit review |

#### Issues

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{name}/issues` | List issues |
| POST | `/api/repos/{owner}/{name}/issues` | Create issue |
| GET | `/api/repos/{owner}/{name}/issues/{number}` | Get issue |
| PATCH | `/api/repos/{owner}/{name}/issues/{number}` | Update issue |
| GET | `/api/repos/{owner}/{name}/issues/{number}/comments` | List comments |
| POST | `/api/repos/{owner}/{name}/issues/{number}/comments` | Add comment |

### P2P Message Types

New message types for collaboration data replication:

```rust
pub enum CollaborationMessage {
    // Pull Request messages
    PullRequestCreated(PullRequest),
    PullRequestUpdated { repo_key: String, number: u32, state: PullRequestState },
    PullRequestMerged { repo_key: String, number: u32, merged_by: String },

    // Issue messages
    IssueCreated(Issue),
    IssueUpdated { repo_key: String, number: u32, state: IssueState },

    // Comment messages
    CommentCreated(Comment),
    CommentUpdated { id: u64, body: String },

    // Review messages
    ReviewSubmitted(Review),

    // Sync messages
    SyncCollaborationRequest { repo_key: String },
    SyncCollaborationResponse { repo_key: String, prs: Vec<PullRequest>, issues: Vec<Issue> },
}
```

### CLI Commands

```bash
# Pull Request commands
guts pr list [--state open|closed|merged]
guts pr create --title "Title" --body "Description" --source branch --target main
guts pr show <number>
guts pr merge <number>
guts pr close <number>

# Issue commands
guts issue list [--state open|closed]
guts issue create --title "Title" --body "Description"
guts issue show <number>
guts issue close <number>
guts issue reopen <number>

# Comment commands
guts comment add --pr <number> "Comment text"
guts comment add --issue <number> "Comment text"
```

## Implementation Plan

### Phase 1: Core Types (guts-collaboration crate)

1. Create crate structure
2. Implement `PullRequest` type with state machine
3. Implement `Issue` type with state machine
4. Implement `Comment` type
5. Implement `Review` type
6. Implement `Label` type
7. Create `CollaborationStore` for in-memory storage
8. Add comprehensive unit tests

### Phase 2: API Integration (guts-node)

1. Add `guts-collaboration` dependency
2. Integrate `CollaborationStore` into `AppState`
3. Implement PR endpoints
4. Implement Issue endpoints
5. Implement Comment endpoints
6. Implement Review endpoints
7. Add API integration tests

### Phase 3: P2P Replication (guts-p2p)

1. Add collaboration message types to P2P protocol
2. Implement collaboration event broadcasting
3. Implement collaboration sync protocol
4. Handle concurrent updates and conflicts
5. Add P2P integration tests

### Phase 4: CLI Commands (guts-cli)

1. Add `pr` subcommand with list/create/show/merge/close
2. Add `issue` subcommand with list/create/show/close/reopen
3. Add `comment` subcommand with add
4. Add HTTP client for API communication

### Phase 5: E2E Testing

1. Create multi-node collaboration test
2. Test PR workflow across nodes
3. Test issue synchronization
4. Test comment replication
5. Test conflict resolution

## Success Criteria

- [ ] Create and manage pull requests via API
- [ ] Create and manage issues via API
- [ ] Add comments to PRs and issues
- [ ] Submit reviews on pull requests
- [ ] P2P replication of collaboration data across nodes
- [ ] CLI commands for common operations
- [ ] E2E test passing for multi-node collaboration

## Dependencies

- `guts-types`: Core types (Identity, ObjectId)
- `guts-storage`: Object storage (for diff generation)
- `guts-p2p`: P2P networking layer
- `serde`: Serialization
- `thiserror`: Error handling

## Future Considerations

These features are out of scope for Milestone 3 but should be considered:

1. **Diff Generation**: Generate diffs between commits for code review UI
2. **Merge Conflict Resolution**: Handle merge conflicts in PRs
3. **Notifications**: Real-time notifications for PR/Issue updates
4. **Mentions**: @-mentions in comments and descriptions
5. **Assignees**: Assign users to PRs and issues
6. **Milestones**: Group issues into milestones
7. **Projects**: Kanban-style project boards

## References

- [GitHub REST API](https://docs.github.com/en/rest)
- [GitLab API](https://docs.gitlab.com/ee/api/)
- [Guts PRD](./PRD.md)
