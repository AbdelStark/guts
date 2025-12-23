# Architecture Overview

Guts is built on proven distributed systems primitives to provide a decentralized, censorship-resistant code collaboration platform.

## Vision

> *"Code collaboration infrastructure that can't be taken down, censored, or controlled by any single entity."*

## Core Principles

| Principle | Description |
|-----------|-------------|
| **Decentralization** | No single point of failure or control |
| **Censorship Resistance** | Content cannot be arbitrarily removed |
| **Cryptographic Identity** | Verifiable authorship via Ed25519 |
| **Byzantine Fault Tolerance** | Network continues despite malicious actors |
| **Git Compatibility** | Works with standard Git clients |

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Guts Network                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │   Node A    │  │   Node B    │  │   Node C    │    ...       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│              ┌───────────┴───────────┐                          │
│              │    P2P Mesh Network   │                          │
│              │   (commonware::p2p)   │                          │
│              └───────────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

## Node Architecture

Each Guts node contains multiple layers:

```
┌─────────────────────────────────────────────────────────────────┐
│                         Guts Node                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                      API Layer                            │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐   │  │
│  │  │ Git HTTP │  │ REST API │  │     WebSocket        │   │  │
│  │  └──────────┘  └──────────┘  └──────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   Application Layer                       │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────────┐  │  │
│  │  │Collaboration│  │    Auth    │  │      CI/CD         │  │  │
│  │  │ (PRs/Issues)│  │(Orgs/Teams)│  │   (Workflows)      │  │  │
│  │  └────────────┘  └────────────┘  └────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Core Layer                             │  │
│  │  ┌────────────────────┐  ┌────────────────────────────┐  │  │
│  │  │    Git Storage     │  │   Consensus Engine         │  │  │
│  │  │  (Content-Addressed)│  │   (Simplex BFT)           │  │  │
│  │  └────────────────────┘  └────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   Network Layer                           │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │              P2P Network (commonware)               │  │  │
│  │  │  • Ed25519 authentication • Noise encryption       │  │  │
│  │  │  • QUIC + TCP transport   • Peer discovery         │  │  │
│  │  └────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Language | Rust | Memory safety, performance |
| Async Runtime | Tokio | Async I/O, task scheduling |
| Web Framework | Axum + Tower | HTTP API, middleware |
| Consensus | commonware::consensus | BFT consensus |
| Networking | commonware::p2p | Encrypted P2P |
| Cryptography | Ed25519 | Digital signatures |
| Storage | RocksDB | Persistent storage |
| Git Protocol | Custom implementation | Smart HTTP, pack files |

## Crate Architecture

The codebase is organized into focused crates:

```
guts-types (foundation)
    ↓
guts-storage + guts-git
    ↓
guts-consensus + guts-p2p + guts-collaboration + guts-auth
    ↓
guts-node + guts-web + guts-realtime + guts-ci
    ↓
guts-cli
```

| Crate | Purpose |
|-------|---------|
| `guts-types` | Core types and primitives |
| `guts-storage` | Content-addressed Git storage |
| `guts-git` | Git protocol implementation |
| `guts-p2p` | P2P networking and replication |
| `guts-consensus` | Simplex BFT consensus engine |
| `guts-collaboration` | PRs, Issues, Reviews |
| `guts-auth` | Organizations, Teams, Permissions |
| `guts-node` | Full node binary |
| `guts-cli` | CLI client |

## Key Concepts

### Simplex BFT Consensus

Guts uses Simplex BFT for total ordering of state changes:

- **Byzantine tolerance**: f < n/3 malicious validators
- **Finality**: 3 network hops to finalization
- **Block time**: Configurable (default 2 seconds)

### Content-Addressed Storage

Git objects are stored using SHA-based addressing:

- Automatic deduplication
- Integrity verification
- Efficient P2P replication

### Cryptographic Identity

Every user has an Ed25519 keypair:

- Public key = unique identifier
- All operations are signed
- Verifiable authorship

## Documentation

| Document | Description |
|----------|-------------|
| [Product Requirements](/architecture/prd) | Full product specification |
| [Roadmap](/architecture/roadmap) | Development roadmap |
| [ADRs](/architecture/adr/) | Architecture Decision Records |
