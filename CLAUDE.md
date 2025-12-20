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
cargo run --bin guts-node -- --config config/dev.toml

# Run CLI
cargo run --bin guts -- --help
```

## Project Structure

```
guts/
├── crates/                     # Rust workspace crates
│   ├── guts-core/              # Core types, traits, and errors
│   ├── guts-identity/          # Cryptographic identity (Ed25519)
│   ├── guts-storage/           # Content-addressed storage
│   ├── guts-repo/              # Git repository operations
│   ├── guts-protocol/          # P2P protocol definitions
│   ├── guts-consensus/         # BFT consensus integration
│   ├── guts-p2p/               # Peer-to-peer networking
│   ├── guts-api/               # HTTP/gRPC API server
│   ├── guts-node/              # Full node binary
│   └── guts-cli/               # CLI client binary
├── infra/                      # Infrastructure as code
│   ├── terraform/              # Cloud provisioning
│   ├── docker/                 # Container definitions
│   └── k8s/                    # Kubernetes manifests
├── docs/                       # Documentation
│   ├── PRD.md                  # Product requirements
│   └── adr/                    # Architecture decisions
├── tests/                      # Integration tests
├── benches/                    # Benchmarks
└── .claude/                    # AI agent configuration
    ├── skills/                 # Claude skills
    └── commands/               # Custom commands
```

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 1.85+ |
| Async Runtime | Tokio |
| Consensus | commonware::consensus |
| Networking | commonware::p2p |
| Storage | RocksDB |
| Git | gitoxide (gix) |
| API | axum (HTTP) + tonic (gRPC) |
| Serialization | commonware::codec |
| CLI | clap |

## Development Guidelines

### Code Style

- Follow Rust idioms and clippy recommendations
- All public items must have documentation
- Use `thiserror` for library errors
- Prefer explicit over implicit

### Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# With coverage
cargo llvm-cov --workspace --html
```

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

1. Define message in `guts-protocol/src/messages.rs`
2. Implement `Codec` trait
3. Add handler in `guts-p2p/src/handlers.rs`
4. Add tests in `guts-protocol/src/tests.rs`

### Adding an API Endpoint

1. Define request/response types in `guts-api/src/types.rs`
2. Implement handler in `guts-api/src/handlers/`
3. Add route in `guts-api/src/router.rs`
4. Add OpenAPI documentation
5. Add integration test

## Key Abstractions

### Identity

```rust
use guts_identity::Identity;

let identity = Identity::generate();
let signature = identity.sign(message);
identity.verify(message, &signature)?;
```

### Repository

```rust
use guts_repo::Repository;

let repo = Repository::create("my-repo", owner_id).await?;
let commit_id = repo.commit(tree, message).await?;
```

### P2P Network

```rust
use guts_p2p::Node;

let node = Node::new(config).await?;
node.connect(peer_addr).await?;
node.broadcast(message).await?;
```

## Error Handling

Each crate defines its own error type:

```rust
// In guts-core/src/error.rs
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
- Batch database operations where possible
- Use connection pooling for RocksDB

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
