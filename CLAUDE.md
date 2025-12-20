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
│   ├── guts-node/              # Full node binary
│   └── guts-cli/               # CLI client binary
├── infra/                      # Infrastructure as code
│   ├── terraform/              # Cloud provisioning
│   ├── docker/                 # Container definitions
│   └── k8s/                    # Kubernetes manifests
├── docs/                       # Documentation
│   ├── PRD.md                  # Product requirements
│   └── adr/                    # Architecture decisions
└── .claude/                    # AI agent configuration
    ├── skills/                 # Claude skills
    └── commands/               # Custom commands
```

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (stable) |
| Async Runtime | commonware::runtime |
| Consensus | commonware::consensus |
| Networking | commonware::p2p |
| Cryptography | commonware::cryptography |
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
