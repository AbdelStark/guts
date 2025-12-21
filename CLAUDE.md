# Guts - AI Agent Development Guide

> This file provides context for AI agents (Claude, Codex, etc.) working on the Guts codebase.

## Project Overview

**Guts** is a decentralized, censorship-resistant code collaboration platform - a decentralized alternative to GitHub built on [commonware](https://github.com/commonwarexyz/monorepo) primitives.

### Key Documents

- [Product Requirements Document](docs/PRD.md) - Full product specification
- [Architecture Decision Records](docs/adr/) - Design decisions
- [Contributing Guide](CONTRIBUTING.md) - How to contribute

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
```

## Project Structure

```
guts/
├── crates/                     # Rust workspace crates
│   ├── guts-types/             # Core types and primitives
│   ├── guts-storage/           # Git object storage (content-addressed)
│   ├── guts-git/               # Git protocol (pack files, smart HTTP)
│   ├── guts-p2p/               # P2P networking and replication
│   ├── guts-collaboration/     # PRs, Issues, Comments, Reviews
│   ├── guts-auth/              # Organizations, Teams, Permissions, Webhooks
│   ├── guts-node/              # Full node binary & HTTP API
│   └── guts-cli/               # CLI client binary
├── infra/                      # Infrastructure as code
│   ├── terraform/              # Cloud provisioning
│   ├── docker/                 # Container definitions
│   └── k8s/                    # Kubernetes manifests
├── docs/                       # Documentation
│   ├── PRD.md                  # Product requirements
│   ├── MILESTONE-3.md          # Latest collaboration features
│   ├── ROADMAP.md              # MVP roadmap
│   └── adr/                    # Architecture decisions
└── .claude/                    # AI agent configuration
    ├── skills/                 # Claude skills
    └── commands/               # Custom commands
```

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (stable) |
| Async Runtime | Tokio |
| Web Framework | Axum + Tower |
| Consensus | commonware::consensus |
| Networking | commonware::p2p |
| Cryptography | commonware::cryptography (Ed25519) |
| Git Protocol | Custom pack files + Smart HTTP |
| Serialization | serde + serde_json |
| CLI | clap |

## Current Status

Completed milestones:
- **Milestone 1**: Foundation (git storage, protocol, node API)
- **Milestone 2**: Multi-node P2P replication with commonware
- **Milestone 3**: Collaboration features (PRs, Issues, Comments, Reviews)
- **Milestone 4**: Governance (Organizations, Teams, Permissions, Webhooks)

## Development Guidelines

### Code Style

- Follow Rust idioms and clippy recommendations
- All public items must have documentation
- Use `thiserror` for library errors
- Prefer explicit over implicit

### Testing

The project has comprehensive test coverage (250+ tests):

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

**Test Categories:**
- **Unit tests**: Core logic for types, storage, git protocol, collaboration
- **E2E tests**: HTTP API tests for PRs, Issues, Comments, Reviews
- **Integration tests**: Multi-node P2P replication, git push/pull simulation

**Key test files:**
- `guts-node/tests/collaboration_e2e.rs` - Collaboration API tests
- `guts-node/tests/multi_node_replication.rs` - P2P consensus tests
- `guts-collaboration/src/store.rs` - Store operations tests

### Commits

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(repo): add branch creation support
fix(p2p): resolve connection timeout issue
docs(readme): update installation instructions
test(identity): add signature verification tests
```

## Common Tasks

### Adding a New Crate

1. Create directory: `crates/guts-<name>/`
2. Add `Cargo.toml` with workspace dependencies
3. Add to root `Cargo.toml` workspace members
4. Create `src/lib.rs` with module structure

### Implementing a New Protocol Message

1. Define message in `guts-types/src/`
2. Implement `Codec` trait from commonware
3. Add tests for the new message type

## Key Abstractions

### Identity

```rust
use guts_types::{Identity, PublicKey, Signature};
use commonware_cryptography::ed25519;

// Generate a new keypair
let signer = ed25519::Keypair::random(&mut rand::thread_rng());
let public_key = signer.public_key();
```

### Repository

```rust
use guts_types::{Repository, RepositoryId};

let repo = Repository::new("my-repo", owner_id, "A description");
let repo_id = repo.id();
```

### Collaboration Types

```rust
use guts_collaboration::{PullRequest, Issue, Comment, Review};

// Pull Requests have states: Open, Closed, Merged
let pr = PullRequest::new(id, repo_key, number, title, desc, author,
    source_branch, target_branch, source_commit, target_commit);
pr.close()?;
pr.reopen()?;
pr.merge(merged_by)?;

// Issues have states: Open, Closed
let issue = Issue::new(id, repo_key, number, title, description, author);

// Comments can target PRs or Issues
let comment = Comment::new(id, target, author, body);

// Reviews: Pending, Commented, Approved, ChangesRequested, Dismissed
let review = Review::new(id, repo_key, pr_number, author, state, commit_id);
```

## API Endpoints

The node exposes a REST API at `/api`:

**Pull Requests:**
- `GET/POST /api/repos/{owner}/{repo}/pulls` - List/create PRs
- `GET/PATCH /api/repos/{owner}/{repo}/pulls/{number}` - Get/update PR
- `POST /api/repos/{owner}/{repo}/pulls/{number}/merge` - Merge PR
- `GET/POST /api/repos/{owner}/{repo}/pulls/{number}/comments` - PR comments
- `GET/POST /api/repos/{owner}/{repo}/pulls/{number}/reviews` - Code reviews

**Issues:**
- `GET/POST /api/repos/{owner}/{repo}/issues` - List/create issues
- `GET/PATCH /api/repos/{owner}/{repo}/issues/{number}` - Get/update issue
- `GET/POST /api/repos/{owner}/{repo}/issues/{number}/comments` - Issue comments

**Git:**
- `GET /api/repos/{owner}/{repo}/info/refs` - Reference advertisement
- `POST /api/repos/{owner}/{repo}/git-upload-pack` - Clone/fetch
- `POST /api/repos/{owner}/{repo}/git-receive-pack` - Push

## Error Handling

Each crate defines its own error type:

```rust
// In guts-types/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied")]
    PermissionDenied,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

## Performance Considerations

- Use `Arc` for shared ownership across async tasks
- Prefer `bytes::Bytes` for zero-copy networking
- Leverage commonware's optimized primitives

## Security Requirements

- All network traffic must be encrypted (TLS 1.3 or Noise)
- All commits must be cryptographically signed
- Validate all inputs at API boundaries
- No secret logging

## Available Skills

The following Claude skills are configured for this project:

- `rust-development` - Rust coding patterns and best practices
- `infrastructure` - IaC and deployment patterns
- `testing` - Testing strategies and patterns
- `p2p-networking` - P2P protocol implementation
- `git-protocol` - Git operations with gitoxide

## CI/CD

GitHub Actions workflows:

- `ci.yml` - Build, test, lint on every PR
- `release.yml` - Build releases on tags
- `security.yml` - Security scanning

## Getting Help

- Check existing [Architecture Decision Records](docs/adr/)
- Read the [PRD](docs/PRD.md) for product context
- See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines
