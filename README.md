# Guts

[![CI](https://github.com/AbdelStark/guts/actions/workflows/ci.yml/badge.svg)](https://github.com/AbdelStark/guts/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

> **Decentralized code collaboration infrastructure that can't be taken down, censored, or controlled by any single entity.**

Guts is a decentralized, censorship-resistant alternative to GitHub built on [commonware](https://github.com/commonwarexyz/monorepo) primitives. It provides Git-compatible repository hosting, pull request workflows, and issue tracking without centralized control.

## Features

- **Decentralized**: No single point of failure or control
- **Censorship-Resistant**: Content cannot be arbitrarily removed
- **Git-Compatible**: Works with standard Git workflows
- **Cryptographic Identity**: Ed25519-based identity system
- **Byzantine Fault Tolerant**: Network consensus for data integrity

## Quick Start

```bash
# Install from source
cargo install --path crates/guts-cli

# Generate identity
guts identity generate

# Initialize a repository
guts init my-project

# Run a node
cargo run --bin guts-node
```

## Documentation

- [Product Requirements Document](docs/PRD.md)
- [Contributing Guide](CONTRIBUTING.md)
- [AI Agent Guide](CLAUDE.md)

## Project Structure

```
guts/
├── crates/                 # Rust workspace crates
│   ├── guts-core/          # Core types and traits
│   ├── guts-identity/      # Cryptographic identity
│   ├── guts-storage/       # Content-addressed storage
│   ├── guts-repo/          # Git operations
│   ├── guts-protocol/      # Network protocol
│   ├── guts-consensus/     # BFT consensus
│   ├── guts-p2p/           # P2P networking
│   ├── guts-api/           # HTTP/gRPC API
│   ├── guts-node/          # Node binary
│   └── guts-cli/           # CLI binary
├── infra/                  # Infrastructure as code
├── docs/                   # Documentation
└── .github/                # CI/CD workflows
```

## Development

### Prerequisites

- Rust 1.75+
- Docker (optional, for containerized development)

### Building

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run lints
cargo clippy --workspace --all-targets

# Format code
cargo fmt --all
```

### Running Locally

```bash
# Start a development cluster
cd infra/docker && docker-compose up -d

# Or run a single node
cargo run --bin guts-node -- --api-addr 127.0.0.1:8080
```

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Runtime | Tokio |
| Git | gitoxide |
| API | axum + tonic |
| Storage | RocksDB |

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [commonware](https://github.com/commonwarexyz/monorepo) - Modular blockchain primitives
- [gitoxide](https://github.com/Byron/gitoxide) - Pure Rust Git implementation
