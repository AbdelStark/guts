# Guts - AI Agent Development Guide

> This file provides context for AI agents (Claude, Codex, etc.) working on the Guts codebase.
> Last updated: 2025-12-21

## Project Overview

**Guts** is a decentralized, censorship-resistant code collaboration platform - a decentralized alternative to GitHub built on [commonware](https://github.com/commonwarexyz/monorepo) primitives.

### Vision

*"Code collaboration infrastructure that can't be taken down, censored, or controlled by any single entity."*

### Key Documents

| Document | Description |
|----------|-------------|
| [PRD](docs/PRD.md) | Full product specification |
| [Roadmap](docs/ROADMAP.md) | MVP roadmap with phases |
| [ADRs](docs/adr/) | Architecture Decision Records |
| [Milestone 3](docs/MILESTONE-3.md) | Collaboration features spec |
| [Milestone 4](docs/MILESTONE-4.md) | Governance features spec |
| [Milestone 5](docs/MILESTONE-5.md) | Web gateway spec |
| [Milestone 6](docs/MILESTONE-6.md) | Real-time updates spec |
| [Milestone 7](docs/MILESTONE-7.md) | CI/CD integration spec |
| [Contributing](CONTRIBUTING.md) | Contribution guidelines |

## Quick Start

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run a local node
cargo run --bin guts-node -- --api-addr 127.0.0.1:8080

# Run CLI
cargo run --bin guts -- --help

# Run local devnet (5 nodes)
cd infra/docker && docker compose -f docker-compose.devnet.yml up
```

## Project Structure

```
guts/
├── crates/                     # Rust workspace crates (11 crates)
│   ├── guts-types/             # Core types and primitives
│   ├── guts-storage/           # Git object storage (content-addressed)
│   ├── guts-git/               # Git protocol (pack files, smart HTTP)
│   ├── guts-p2p/               # P2P networking and replication
│   ├── guts-collaboration/     # PRs, Issues, Comments, Reviews, Labels
│   ├── guts-auth/              # Organizations, Teams, Permissions, Webhooks
│   ├── guts-web/               # Web gateway (HTML views, Markdown rendering)
│   ├── guts-realtime/          # WebSocket real-time updates and notifications
│   ├── guts-ci/                # CI/CD pipelines, workflows, runs, artifacts
│   ├── guts-node/              # Full node binary & HTTP API
│   └── guts-cli/               # CLI client binary
├── infra/                      # Infrastructure as code
│   ├── terraform/              # AWS cloud provisioning
│   ├── docker/                 # Container definitions & devnet
│   └── k8s/                    # Kubernetes manifests
├── docs/                       # Documentation
│   ├── adr/                    # Architecture Decision Records
│   └── *.md                    # Milestone specs, PRD, roadmap
├── .github/workflows/          # CI/CD pipelines
└── .claude/                    # AI agent configuration
    ├── skills/                 # Claude skills
    └── commands/               # Custom commands
```

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | Rust (stable) | Memory safety, performance |
| Async Runtime | Tokio | Async I/O, task scheduling |
| Web Framework | Axum + Tower | HTTP API, middleware |
| Consensus | commonware::consensus | BFT consensus |
| Networking | commonware::p2p | Encrypted P2P |
| Cryptography | commonware::cryptography | Ed25519 signatures |
| Git Protocol | Custom implementation | Pack files, Smart HTTP |
| Serialization | serde + serde_json | JSON API |
| Web UI | Askama + pulldown-cmark | HTML templates, Markdown |
| CLI | clap | Command-line parsing |

## Current Status

| Milestone | Status | Description |
|-----------|--------|-------------|
| Milestone 1 | Complete | Foundation (Git storage, protocol, node API) |
| Milestone 2 | Complete | Multi-node P2P replication with commonware |
| Milestone 3 | Complete | Collaboration (PRs, Issues, Comments, Reviews) |
| Milestone 4 | Complete | Governance (Organizations, Teams, Permissions, Webhooks) |
| Milestone 5 | Complete | Web Gateway (Search, API Documentation, Full UI) |
| Milestone 6 | Complete | Real-time Updates (WebSocket, Notifications) |
| Milestone 7 | Complete | CI/CD Integration (Workflows, Runs, Artifacts, Status Checks) |

### Test Coverage

- **350+ tests** across all crates
- Unit tests, E2E tests, integration tests
- Multi-node P2P replication tests
- Collaboration and governance scenario tests

## Crate Dependency Graph

```
guts-types (foundation)
    ↓
guts-storage + guts-git
    ↓
guts-p2p + guts-collaboration + guts-auth + guts-realtime
    ↓
guts-node + guts-web
    ↓
guts-cli
```

## Architecture Decision Records

Key architectural decisions are documented in `docs/adr/`:

| ADR | Title |
|-----|-------|
| [ADR-001](docs/adr/001-commonware-primitives.md) | Use Commonware for P2P and Consensus |
| [ADR-002](docs/adr/002-content-addressed-storage.md) | Content-Addressed Storage for Git Objects |
| [ADR-003](docs/adr/003-git-protocol-implementation.md) | Custom Git Smart HTTP Protocol |
| [ADR-004](docs/adr/004-collaboration-data-model.md) | Collaboration Data Model |
| [ADR-005](docs/adr/005-permission-hierarchy.md) | Permission and Access Control Hierarchy |
| [ADR-006](docs/adr/006-api-design.md) | REST API Design Principles |
| [ADR-007](docs/adr/007-crate-architecture.md) | Modular Crate Architecture |

## Development Guidelines

### Code Style

- Follow Rust idioms and clippy recommendations
- All public items must have documentation
- Use `thiserror` for library errors
- Prefer explicit over implicit

### Testing

```bash
# Run all tests
cargo test --workspace

# Unit tests only
cargo test --lib --workspace

# Integration/E2E tests
cargo test --test '*' --workspace

# Test a specific crate
cargo test -p guts-collaboration

# With coverage
cargo llvm-cov --workspace --html
```

**Key test files:**
- `guts-node/tests/collaboration_e2e.rs` - Collaboration API tests
- `guts-node/tests/auth_e2e.rs` - Auth/Governance API tests
- `guts-node/tests/multi_node_replication.rs` - P2P consensus tests

### Commits

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(repo): add branch creation support
fix(p2p): resolve connection timeout issue
docs(readme): update installation instructions
test(identity): add signature verification tests
```

## API Reference

### Git Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/git/{owner}/{name}/info/refs` | Reference advertisement |
| POST | `/git/{owner}/{name}/git-upload-pack` | Clone/fetch |
| POST | `/git/{owner}/{name}/git-receive-pack` | Push |

### Repository Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos` | List all repositories |
| POST | `/api/repos` | Create repository |
| GET | `/api/repos/{owner}/{name}` | Get repository |

### Collaboration Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET/POST | `/api/repos/{owner}/{name}/pulls` | List/create PRs |
| GET/PATCH | `/api/repos/{owner}/{name}/pulls/{num}` | Get/update PR |
| POST | `/api/repos/{owner}/{name}/pulls/{num}/merge` | Merge PR |
| GET/POST | `/api/repos/{owner}/{name}/pulls/{num}/reviews` | Reviews |
| GET/POST | `/api/repos/{owner}/{name}/issues` | List/create issues |
| GET/PATCH | `/api/repos/{owner}/{name}/issues/{num}` | Get/update issue |

### Authorization Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET/POST | `/api/orgs` | List/create orgs |
| GET/PATCH/DELETE | `/api/orgs/{org}` | Manage org |
| GET/POST | `/api/orgs/{org}/teams` | List/create teams |
| PUT/DELETE | `/api/repos/{owner}/{name}/collaborators/{user}` | Manage access |
| GET/PUT/DELETE | `/api/repos/{owner}/{name}/branches/{branch}/protection` | Branch protection |
| GET/POST | `/api/repos/{owner}/{name}/hooks` | Webhooks |

## Key Abstractions

### Collaboration Types

```rust
use guts_collaboration::{PullRequest, Issue, Comment, Review};

// Pull Request states: Open -> Closed/Merged
let pr = PullRequest::new(id, repo_key, number, title, desc, author,
    source_branch, target_branch, source_commit, target_commit);

// Review states: Pending, Commented, Approved, ChangesRequested, Dismissed
let review = Review::new(id, repo_key, pr_number, author, state, commit_id);
```

### Governance Types

```rust
use guts_auth::{Permission, Organization, Team, BranchProtection};

// Permission levels: Read < Write < Admin
let perm = Permission::Admin;
assert!(perm.has(Permission::Write)); // Admin includes Write

// Organization with role-based membership (Owner, Admin, Member)
let org = Organization::new(id, "acme", "Acme Corp", creator);

// Branch protection enforces PR workflow
let protection = BranchProtection::new(id, "owner/repo", "main")
    .with_required_reviews(2)
    .with_require_pr(true);
```

## Infrastructure

### Local Development

```bash
# Single node
cargo run --bin guts-node -- --api-addr 127.0.0.1:8080

# Local devnet (5 nodes + tests)
cd infra/docker
docker compose -f docker-compose.devnet.yml up
```

### Docker

```bash
# Build image
docker build -t guts-node -f infra/docker/Dockerfile .

# Run container
docker run -p 8080:8080 -p 9000:9000 guts-node
```

### Kubernetes

```bash
kubectl apply -f infra/k8s/
```

### Terraform (AWS)

```bash
cd infra/terraform
terraform init
terraform plan
terraform apply
```

## CI/CD Workflows

| Workflow | Trigger | Description |
|----------|---------|-------------|
| `ci.yml` | PR, push | Build, test, lint, security audit |
| `release.yml` | Tags | Multi-platform builds, Docker push |
| `security.yml` | Weekly, PR | cargo-audit, cargo-deny, CodeQL, Trivy |
| `devnet-e2e-extensive.yml` | PR, manual | 5-node devnet with extensive E2E tests |

## Available Skills

The following Claude skills are configured:

| Skill | Description |
|-------|-------------|
| `rust-development` | Rust coding patterns and best practices |
| `infrastructure` | IaC and deployment patterns |
| `testing` | Testing strategies and patterns |
| `p2p-networking` | P2P protocol implementation |
| `git-protocol` | Git operations with gitoxide |

## Common Tasks

### Adding a New Crate

1. Create directory: `crates/guts-<name>/`
2. Add `Cargo.toml` with workspace dependencies
3. Add to root `Cargo.toml` workspace members
4. Create `src/lib.rs` with module structure

### Adding an API Endpoint

1. Add handler function in appropriate `*_api.rs`
2. Register route in the router
3. Add request/response types
4. Add E2E test in `tests/`
5. Update API documentation

### Running the Devnet

```bash
# Start 5-node devnet
cd infra/docker
docker compose -f docker-compose.devnet.yml up -d

# Run E2E tests against devnet
./infra/scripts/devnet-e2e-test.sh

# Stop devnet
docker compose -f docker-compose.devnet.yml down
```

## Security Requirements

- All network traffic must be encrypted (TLS 1.3 or Noise)
- All commits must be cryptographically signed
- Validate all inputs at API boundaries
- No secret logging
- Regular security audits via CI

## Performance Considerations

- Use `Arc` for shared ownership across async tasks
- Prefer `bytes::Bytes` for zero-copy networking
- Leverage commonware's optimized primitives
- Content-addressed storage enables deduplication

## Getting Help

1. Check [Architecture Decision Records](docs/adr/)
2. Read the [PRD](docs/PRD.md) for product context
3. See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines
4. Review relevant milestone docs in `docs/`
