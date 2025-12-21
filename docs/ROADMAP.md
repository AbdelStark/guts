# Guts MVP Roadmap

> Roadmap to a fully working MVP with E2E tests: 3 nodes + 2 collaborating git clients

## MVP Goal

Two git clients collaborating on a Guts-hosted repository:
1. **Client 1**: Creates repo, commits and pushes content
2. **Client 2**: Clones/pulls the repo, commits and pushes new content
3. **Client 1**: Pulls and sees Client 2's changes

All running on a 3-node Guts network with consensus.

## Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Node 1    │◄───►│   Node 2    │◄───►│   Node 3    │
│  (Leader)   │     │  (Replica)  │     │  (Replica)  │
└──────┬──────┘     └─────────────┘     └─────────────┘
       │
       │ HTTP API
       ▼
┌─────────────┐     ┌─────────────┐
│  Client 1   │     │  Client 2   │
│  (git CLI)  │     │  (git CLI)  │
└─────────────┘     └─────────────┘
```

## Current Status

| Milestone | Status | Description |
|-----------|--------|-------------|
| Milestone 1 | ✅ Complete | Foundation (Git Storage, Protocol, Node API) |
| Milestone 2 | ✅ Complete | Multi-node P2P Replication |
| Milestone 3 | ✅ Complete | Collaboration (PRs, Issues, Comments, Reviews) |
| Milestone 4 | ✅ Complete | Governance (Orgs, Teams, Permissions, Webhooks) |
| Milestone 5 | 🔜 Next | Ecosystem (Web Gateway, CI/CD, Package Registry) |

## Phase 1: Core Infrastructure ✅

### 1.1 Git Object Storage ✅
- [x] Implement content-addressed blob storage
- [x] Store git objects (blobs, trees, commits)
- [x] Reference management (branches, tags, HEAD)

### 1.2 Repository State ✅
- [x] Repository metadata (name, owner, refs)
- [x] Ref updates with optimistic locking
- [x] Pack file support (for efficient transfer)

## Phase 2: Networking ✅

### 2.1 P2P Node Communication ✅
- [x] Node discovery and connection
- [x] Message passing between nodes
- [x] Peer management

### 2.2 Broadcast & Replication ✅
- [x] Broadcast repository updates to all nodes
- [x] Replicate git objects across nodes
- [x] Consistency verification

## Phase 3: Git Protocol ✅

### 3.1 Git Smart HTTP Protocol ✅
- [x] `/info/refs` - Reference advertisement
- [x] `/git-upload-pack` - Fetch/clone (client pulls)
- [x] `/git-receive-pack` - Push (client pushes)

### 3.2 Pack Protocol ✅
- [x] Pack file parsing
- [x] Pack file generation
- [x] Delta compression (optional for MVP)

## Phase 4: API & CLI ✅

### 4.1 HTTP API ✅
- [x] Create repository endpoint
- [x] List repositories endpoint
- [x] Git smart HTTP endpoints

### 4.2 CLI Commands ✅
- [x] `guts repo create <name>` - Create repository
- [x] `guts repo list` - List repositories
- [x] `guts clone <repo>` - Clone via git
- [x] `guts push` / `guts pull` - Git operations

## Phase 5: Collaboration ✅

### 5.1 Pull Requests ✅
- [x] Create, update, close, merge PRs
- [x] PR comments and discussions
- [x] Code review workflow

### 5.2 Issues ✅
- [x] Create, update, close, reopen issues
- [x] Issue comments and labels

### 5.3 Reviews ✅
- [x] Submit reviews (Approve, Request Changes, Comment)
- [x] Review comments

## Phase 6: Governance ✅

### 6.1 Organizations ✅
- [x] Create and manage organizations
- [x] Member management with roles (Owner, Admin, Member)
- [x] Multi-user repository ownership

### 6.2 Teams ✅
- [x] Create teams within organizations
- [x] Team-based repository access
- [x] Default permission levels for teams

### 6.3 Permissions ✅
- [x] Granular permission levels (Read, Write, Admin)
- [x] Collaborator management
- [x] Permission resolution algorithm

### 6.4 Branch Protection ✅
- [x] Branch protection rules
- [x] Require PRs for protected branches
- [x] Required review counts

### 6.5 Webhooks ✅
- [x] Webhook subscriptions
- [x] Event notifications (push, PR, issues, etc.)
- [x] Webhook management API

## Phase 7: E2E Testing ✅

### 7.1 Test Infrastructure ✅
- [x] Multi-node test harness
- [x] Deterministic networking for tests
- [x] Test utilities for git operations

### 7.2 Collaboration Test ✅
- [x] Start 3 nodes
- [x] Client 1: init, commit, push
- [x] Client 2: clone, commit, push
- [x] Client 1: pull, verify changes
- [x] All nodes: verify consistency

## Implementation Order (Historical)

1. **Git Storage** (`guts-storage` crate) ✅
   - In-memory storage first, then persistent
   - Content-addressed object store
   - Reference store

2. **Git Protocol** (`guts-git` crate) ✅
   - Pack file parsing/generation
   - Smart HTTP protocol handlers

3. **HTTP API** (in `guts-node`) ✅
   - Repository CRUD
   - Git smart HTTP endpoints

4. **P2P Replication** (using commonware) ✅
   - Broadcast git objects
   - Replicate refs

5. **Collaboration** (`guts-collaboration` crate) ✅
   - Pull requests, issues, comments
   - Code review infrastructure

6. **Governance** (`guts-auth` crate) ✅
   - Organizations and teams
   - Permissions and branch protection
   - Webhooks

7. **E2E Tests** (`tests/` directory) ✅
   - Multi-node harness
   - Collaboration scenario

## Success Criteria ✅

- [x] 3 nodes start and form a network
- [x] Client 1 can create a repo and push commits
- [x] Client 2 can clone, modify, and push
- [x] Client 1 can pull Client 2's changes
- [x] All 3 nodes have consistent state
- [x] E2E test passes in CI
- [x] Pull requests and issues work across nodes
- [x] Organizations and teams manage access
- [x] Branch protection enforces policies

## Next Steps: Ecosystem (Milestone 5)

The following features are planned for the next milestone:

| Feature | Description | Priority |
|---------|-------------|----------|
| Web Gateway | Browser access to repositories | P3 |
| CI/CD Integration | Decentralized build pipelines | P3 |
| Package Registry | Decentralized package hosting | P3 |
| Federation | Inter-network repository bridging | P3 |

## Test Coverage

The project currently has **254+ tests** covering:
- Unit tests for all crates
- E2E tests for HTTP API
- Integration tests for P2P replication
- Collaboration and governance scenarios
