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

## Phase 1: Core Infrastructure

### 1.1 Git Object Storage
- [ ] Implement content-addressed blob storage
- [ ] Store git objects (blobs, trees, commits)
- [ ] Reference management (branches, tags, HEAD)

### 1.2 Repository State
- [ ] Repository metadata (name, owner, refs)
- [ ] Ref updates with optimistic locking
- [ ] Pack file support (for efficient transfer)

## Phase 2: Networking

### 2.1 P2P Node Communication
- [ ] Node discovery and connection
- [ ] Message passing between nodes
- [ ] Peer management

### 2.2 Broadcast & Replication
- [ ] Broadcast repository updates to all nodes
- [ ] Replicate git objects across nodes
- [ ] Consistency verification

## Phase 3: Git Protocol

### 3.1 Git Smart HTTP Protocol
- [ ] `/info/refs` - Reference advertisement
- [ ] `/git-upload-pack` - Fetch/clone (client pulls)
- [ ] `/git-receive-pack` - Push (client pushes)

### 3.2 Pack Protocol
- [ ] Pack file parsing
- [ ] Pack file generation
- [ ] Delta compression (optional for MVP)

## Phase 4: API & CLI

### 4.1 HTTP API
- [ ] Create repository endpoint
- [ ] List repositories endpoint
- [ ] Git smart HTTP endpoints

### 4.2 CLI Commands
- [ ] `guts repo create <name>` - Create repository
- [ ] `guts repo list` - List repositories
- [ ] `guts clone <repo>` - Clone via git
- [ ] `guts push` / `guts pull` - Git operations

## Phase 5: E2E Testing

### 5.1 Test Infrastructure
- [ ] Multi-node test harness
- [ ] Deterministic networking for tests
- [ ] Test utilities for git operations

### 5.2 Collaboration Test
- [ ] Start 3 nodes
- [ ] Client 1: init, commit, push
- [ ] Client 2: clone, commit, push
- [ ] Client 1: pull, verify changes
- [ ] All nodes: verify consistency

## Implementation Order

1. **Git Storage** (`guts-storage` crate)
   - In-memory storage first, then persistent
   - Content-addressed object store
   - Reference store

2. **Git Protocol** (`guts-git` crate)
   - Pack file parsing/generation
   - Smart HTTP protocol handlers

3. **HTTP API** (in `guts-node`)
   - Repository CRUD
   - Git smart HTTP endpoints

4. **P2P Replication** (using commonware)
   - Broadcast git objects
   - Replicate refs

5. **E2E Tests** (`tests/` directory)
   - Multi-node harness
   - Collaboration scenario

## Success Criteria

- [ ] 3 nodes start and form a network
- [ ] Client 1 can create a repo and push commits
- [ ] Client 2 can clone, modify, and push
- [ ] Client 1 can pull Client 2's changes
- [ ] All 3 nodes have consistent state
- [ ] E2E test passes in CI

## Timeline Estimate

| Phase | Components | Complexity |
|-------|-----------|------------|
| Phase 1 | Storage | Medium |
| Phase 2 | Networking | Medium |
| Phase 3 | Git Protocol | High |
| Phase 4 | API & CLI | Low |
| Phase 5 | E2E Tests | Medium |
