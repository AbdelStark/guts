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

### Completed Milestones

| Milestone | Status | Description |
|-----------|--------|-------------|
| Milestone 1 | ✅ Complete | Foundation (Git Storage, Protocol, Node API) |
| Milestone 2 | ✅ Complete | Multi-node P2P Replication |
| Milestone 3 | ✅ Complete | Collaboration (PRs, Issues, Comments, Reviews) |
| Milestone 4 | ✅ Complete | Governance (Orgs, Teams, Permissions, Webhooks) |
| Milestone 5 | ✅ Complete | Web Gateway (Search, API Docs) |
| Milestone 6 | ✅ Complete | Real-time Updates (WebSocket, Notifications) |
| Milestone 7 | ✅ Complete | CI/CD Integration (Workflows, Runs, Artifacts) |
| Milestone 8 | ✅ Complete | Git/GitHub Compatibility (Tokens, Users, Contents) |
| Milestone 9 | ✅ Complete | Production Quality (Observability, Testing, Resilience) |
| Milestone 10 | ✅ Complete | Performance & Scalability (RocksDB, Caching, Benchmarks) |

### Production Readiness Milestones

| Milestone | Status | Description |
|-----------|--------|-------------|
| Milestone 11 | ✅ Complete | True Decentralization (BFT Consensus, P2P Bootstrap) |
| Milestone 12 | ✅ Complete | Operator Experience & Documentation |
| Milestone 13 | ✅ Complete | User Adoption & Ecosystem |
| Milestone 14 | 🚧 Next | Security Hardening & Audit Preparation |

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

## Completed: Milestone 5 (Web Gateway)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| Web Gateway | Browser access to repositories | ✅ Complete |
| Repository Browsing | File tree, commits, branches | ✅ Complete |
| Collaboration UI | PRs, Issues, Comments, Reviews | ✅ Complete |
| Organization Views | Orgs, Teams, Members | ✅ Complete |
| Search & Discovery | Repository, Code, Issue/PR search | ✅ Complete |
| API Documentation | OpenAPI 3.1 with Swagger UI | ✅ Complete |

## Completed: Milestone 6 (Real-time Updates)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| WebSocket Server | Persistent connections for real-time communication | ✅ Complete |
| Event Broadcasting | Broadcast repository events to connected clients | ✅ Complete |
| Channel Subscriptions | Subscribe to repo, user, and org channels | ✅ Complete |
| Live UI Updates | Real-time notifications in web interface | ✅ Complete |
| Connection Management | Automatic reconnection with backoff | ✅ Complete |
| Stats API | Real-time connection statistics endpoint | ✅ Complete |

## Completed: Milestone 7 (CI/CD Integration)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| Workflow Configuration | YAML-based pipeline definitions | ✅ Complete |
| Job Execution | Isolated step-by-step job processing | ✅ Complete |
| Status Checks | Integration with branch protection | ✅ Complete |
| Artifact Management | Store and retrieve build artifacts | ✅ Complete |
| Real-time Logs | Stream build logs via WebSocket | ✅ Complete |
| CLI Commands | Workflow and run management | ✅ Complete |

## Completed: Milestone 8 (Git/GitHub Compatibility)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| User Accounts | User registration and profiles | ✅ Complete |
| Personal Access Tokens | Token-based API authentication | ✅ Complete |
| SSH Key Management | SSH key storage and fingerprinting | ✅ Complete |
| Rate Limiting | GitHub-compatible rate limit headers | ✅ Complete |
| Pagination | Link header-based pagination | ✅ Complete |
| Repository Contents API | File browsing without cloning | ✅ Complete |
| Releases & Assets | Release management with assets | ✅ Complete |
| Archive Downloads | Tarball and zipball generation | ✅ Complete |

## Completed: Milestone 9 (Production Quality Improvements)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| Structured Logging | Request IDs and JSON logging | ✅ Complete |
| Prometheus Metrics | HTTP, P2P, storage, and business metrics | ✅ Complete |
| Configuration Validation | Environment variable binding and validation | ✅ Complete |
| Input Validation | API input validation with consistent errors | ✅ Complete |
| Error Handling | Proper error handling without panics | ✅ Complete |
| Resilience Patterns | Retry, circuit breaker, timeouts | ✅ Complete |
| Health Checks | Liveness, readiness, and startup probes | ✅ Complete |
| Property-Based Testing | Protocol parsing tests with proptest | ✅ Complete |
| Fuzz Testing | 7 fuzz targets for protocol/parsing | ✅ Complete |
| Chaos Testing | P2P layer chaos and failure simulation | ✅ Complete |
| Load Testing | Performance benchmarks and stress tests | ✅ Complete |
| Failure Injection | Storage/network failure recovery tests | ✅ Complete |

## Completed: Milestone 10 (Performance & Scalability)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| Benchmarking | Criterion + K6 comprehensive benchmarks | ✅ Complete |
| RocksDB Integration | Persistent storage backend | ✅ Complete |
| Consensus Optimization | Batch proposals, throughput tuning | ✅ Complete |
| Memory Optimization | Object pooling, string interning | ✅ Complete |
| Caching Strategy | Multi-level cache hierarchy | ✅ Complete |
| CDN Integration | Cache headers, archive pre-generation | ✅ Complete |

## Upcoming Milestones

The following milestones represent the path from current state to a production-grade, fully production-ready platform:

| Milestone | Status | Description | Priority |
|-----------|--------|-------------|----------|
| Milestone 14 | 🚧 Next | [Security Hardening & Audit Preparation](MILESTONE-14.md) | Critical |

## Completed: Milestone 13 (User Adoption & Ecosystem)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| Migration Tools | guts-migrate crate for GitHub/GitLab/Bitbucket migration | ✅ Complete |
| TypeScript SDK | @guts/sdk npm package with full API coverage | ✅ Complete |
| Python SDK | guts-sdk PyPI package with Pydantic models | ✅ Complete |
| Git Credential Helper | Secure token storage with system keyring | ✅ Complete |
| Developer Documentation | API reference, guides, SDK documentation | ✅ Complete |

## Completed: Milestone 11 (True Decentralization)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| Simplex BFT Consensus | Real BFT consensus via commonware-consensus | ✅ Complete |
| Transaction Ordering | Total ordering of all state changes | ✅ Complete |
| Block Production | 2-hop proposal, 3-hop finalization | ✅ Complete |
| Byzantine Tolerance | Tolerates f < n/3 Byzantine validators | ✅ Complete |
| Validator Management | Genesis-configured validator sets | ✅ Complete |
| Bootstrap Discovery | Peer exchange and bootstrap nodes | ✅ Complete |
| Consensus API | Full HTTP API for consensus status/blocks | ✅ Complete |
| 4-Node Devnet | Docker-based BFT network for testing | ✅ Complete |
| E2E Test Suite | Comprehensive BFT consensus testing | ✅ Complete |

## Completed: Milestone 12 (Operator Experience & Documentation)

The following features have been implemented:

| Feature | Description | Status |
|---------|-------------|--------|
| Operator Documentation | Comprehensive deployment guides (quickstart, architecture, installation) | ✅ Complete |
| Configuration Reference | Complete YAML config reference with all options | ✅ Complete |
| Operational Runbooks | 8 runbooks (node sync, consensus, disk, memory, shutdown, keys, corruption) | ✅ Complete |
| Prometheus Config | Scrape configs, recording rules, alert rules | ✅ Complete |
| Grafana Dashboards | Pre-built overview dashboard with 30+ panels | ✅ Complete |
| Alertmanager Setup | PagerDuty/Slack integration with routing | ✅ Complete |
| Backup/Restore | CLI commands + shell scripts with S3 support | ✅ Complete |
| CLI Operator Commands | keygen, backup, restore, diagnostics, verify-data | ✅ Complete |
| Helm Chart | Complete K8s deployment chart with StatefulSet, PDB, ServiceMonitor | ✅ Complete |
| Terraform Modules | AWS, GCP, Azure multi-cloud infrastructure | ✅ Complete |

### Milestone 13: User Adoption & Ecosystem ✅

Enable mass adoption through tooling and migration paths.

| Feature | Description | Status |
|---------|-------------|--------|
| Migration Tools | GitHub/GitLab/Bitbucket migration | ✅ Complete |
| SDKs | TypeScript, Python SDKs | ✅ Complete |
| Git Credential Helper | Secure token storage | ✅ Complete |
| Developer Docs | Comprehensive API documentation | ✅ Complete |
| IDE Integration | VS Code extension, JetBrains plugin | Planned |
| SSH Support | Git over SSH | Planned |
| Community | Forum, Discord, support infrastructure | Planned |

### Milestone 14: Security Hardening & Audit Preparation

Prepare Guts for a professional security audit and establish robust security infrastructure.

| Feature | Description | Status |
|---------|-------------|--------|
| Threat Model | Comprehensive STRIDE threat analysis | Planned |
| Security Policy | Vulnerability disclosure and bug bounty | Planned |
| Cryptographic Review | Audit all crypto implementations | Planned |
| Extended Fuzzing | 15+ fuzz targets for all protocols | Planned |
| Supply Chain | SBOM generation, reproducible builds | Planned |
| Key Rotation | Automated key rotation infrastructure | Planned |

## Future Milestones (Post-1.0)

| Feature | Description | Priority |
|---------|-------------|----------|
| Package Registry | Decentralized package hosting | P3 |
| Federation | Inter-network repository bridging | P3 |
| Mobile Apps | iOS/Android native applications | P4 |
| Enterprise Features | SSO, audit logs, compliance | P4 |

## Test Coverage

The project currently has **500+ tests** covering:
- Unit tests for all crates (including guts-consensus)
- E2E tests for HTTP API
- Integration tests for P2P replication
- Collaboration and governance scenarios
- CI/CD workflow and run tests
- Compatibility layer tests (users, tokens, releases)
- Property-based tests (proptest) for protocol parsing
- Fuzz testing (7 targets) for protocol robustness
- Chaos testing for P2P layer resilience
- Load testing for performance benchmarks
- Failure injection tests for recovery patterns
- **Simplex BFT consensus E2E tests** (block production, Byzantine tolerance, cross-validator consistency)
- **Devnet E2E test suite** (comprehensive 4-validator network testing)
