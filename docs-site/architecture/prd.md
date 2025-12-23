# Guts - Product Requirements Document

> **Version:** 1.0.0
> **Last Updated:** 2025-12-20
> **Status:** Draft

## Executive Summary

Guts is a decentralized, censorship-resistant code collaboration platform built on [commonware](https://github.com/commonwarexyz/monorepo) primitives. It provides Git-compatible repository hosting, pull request workflows, and issue tracking without centralized control, enabling developers to own their code infrastructure.

## Vision

*"Code collaboration infrastructure that can't be taken down, censored, or controlled by any single entity."*

## Problem Statement

Current code hosting platforms (GitHub, GitLab, Bitbucket) are:

1. **Centralized Points of Failure**: Single company decisions affect millions of developers
2. **Subject to Censorship**: Repositories can be removed due to government pressure or ToS disputes
3. **Vendor Lock-in**: Migration is painful; developers lose issues, PRs, and community
4. **Privacy Concerns**: All code, issues, and activities are visible to the platform operator
5. **Access Restrictions**: Geographic blocks and sanctions limit global collaboration

## Solution

Guts leverages commonware's modular primitives to build a fully decentralized alternative:

- **Byzantine Fault Tolerant Consensus** for repository state agreement
- **Content-Addressed Storage** for immutable, deduplicated data
- **Peer-to-Peer Networking** for censorship-resistant communication
- **Cryptographic Identity** for verifiable authorship

## Target Users

### Primary

- **Open Source Maintainers**: Need censorship-resistant hosting
- **Privacy-Conscious Developers**: Want control over their data
- **Decentralized Projects**: Align infrastructure with project values
- **Enterprise Teams**: Require self-sovereign code infrastructure

### Secondary

- **Protocol Developers**: Building on decentralized infrastructure
- **Security Researchers**: Publishing sensitive findings
- **Distributed Teams**: Global collaboration without restrictions

## Core Features

### Phase 1: Foundation (MVP)

| Feature | Description | Priority |
|---------|-------------|----------|
| Repository Hosting | Git-compatible repos with push/pull | P0 |
| Identity System | Ed25519-based cryptographic identities | P0 |
| P2P Network | Node discovery and communication | P0 |
| Content Storage | Content-addressed blob storage | P0 |
| CLI Client | Command-line interface for all operations | P0 |

### Phase 2: Collaboration

| Feature | Description | Priority |
|---------|-------------|----------|
| Pull Requests | Decentralized code review workflow | P1 |
| Issue Tracking | Distributed issue management | P1 |
| Comments & Discussions | Threaded conversations on code | P1 |
| Notifications | Real-time activity notifications | P1 |

### Phase 3: Governance

| Feature | Description | Priority |
|---------|-------------|----------|
| Repository Permissions | Granular access control | P2 |
| Organizations | Multi-user repository ownership | P2 |
| Voting Mechanisms | Consensus-based decision making | P2 |
| Reputation System | Contribution-based trust scores | P2 |

### Phase 4: Ecosystem

| Feature | Description | Priority |
|---------|-------------|----------|
| Web Gateway | Browser access to repositories | P3 |
| CI/CD Integration | Decentralized build pipelines | P3 |
| Package Registry | Decentralized package hosting | P3 |
| Federation | Inter-network repository bridging | P3 |

## Technical Architecture

### System Overview

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

                    Node Architecture

┌─────────────────────────────────────────────────────────────────┐
│                         Guts Node                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                      API Layer                            │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐   │  │
│  │  │ Git HTTP │  │ Git SSH  │  │     REST/gRPC API    │   │  │
│  │  └──────────┘  └──────────┘  └──────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   Service Layer                           │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────────┐  │  │
│  │  │ Repository │  │   Issue    │  │   Pull Request     │  │  │
│  │  │  Service   │  │  Service   │  │     Service        │  │  │
│  │  └────────────┘  └────────────┘  └────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  Consensus Layer                          │  │
│  │              (commonware::consensus)                      │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   Storage Layer                           │  │
│  │  ┌────────────────────┐  ┌────────────────────────────┐  │  │
│  │  │ Content-Addressed  │  │    Metadata Store          │  │  │
│  │  │   Blob Storage     │  │    (commonware::storage)   │  │  │
│  │  └────────────────────┘  └────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Crate Architecture

```
guts/
├── crates/
│   ├── guts-core/          # Core types, traits, errors
│   ├── guts-identity/      # Cryptographic identity management
│   ├── guts-storage/       # Content-addressed storage
│   ├── guts-repo/          # Git repository operations
│   ├── guts-protocol/      # Network protocol definitions
│   ├── guts-consensus/     # BFT consensus integration
│   ├── guts-p2p/           # P2P networking layer
│   ├── guts-api/           # API server (HTTP/gRPC)
│   ├── guts-node/          # Full node implementation
│   └── guts-cli/           # Command-line interface
├── infra/                  # Infrastructure as code
├── docs/                   # Documentation
└── tests/                  # Integration tests
```

### Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Language | Rust | Memory safety, performance, ecosystem |
| Async Runtime | Tokio | Industry standard, commonware compatible |
| Consensus | commonware::consensus | BFT, battle-tested |
| Networking | commonware::p2p | Encrypted, authenticated P2P |
| Storage | RocksDB + Content-Addressed | Performance + deduplication |
| Serialization | commonware::codec | Efficient, stable encoding |
| Cryptography | commonware::cryptography | Ed25519, BLS signatures |
| Git Operations | gitoxide | Pure Rust Git implementation |
| API | tonic (gRPC) + axum (HTTP) | Performance + ergonomics |

### Data Model

#### Repository

```rust
pub struct Repository {
    pub id: RepositoryId,           // Content hash of initial commit
    pub name: String,               // Human-readable name
    pub description: Option<String>,
    pub owner: IdentityId,          // Creator's public key
    pub created_at: Timestamp,
    pub visibility: Visibility,
    pub default_branch: String,
}
```

#### Identity

```rust
pub struct Identity {
    pub id: IdentityId,             // Ed25519 public key
    pub username: String,           // Unique username
    pub display_name: Option<String>,
    pub created_at: Timestamp,
    pub metadata: HashMap<String, String>,
}
```

#### Commit (Extended)

```rust
pub struct GutsCommit {
    pub git_commit: GitCommit,      // Standard Git commit
    pub signature: Signature,       // Ed25519 signature
    pub previous_heads: Vec<CommitId>,
    pub consensus_proof: Option<ConsensusProof>,
}
```

## Non-Functional Requirements

### Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Git push latency | < 2s for 1MB | p95 under normal load |
| Git clone throughput | > 10 MB/s | Concurrent users |
| API response time | < 100ms | p99 for reads |
| Time to consensus | < 5s | For repository updates |

### Scalability

- Support 10,000+ repositories per node
- Handle 1,000+ concurrent connections
- Store petabytes of data across network
- Horizontal scaling through additional nodes

### Security

- All data signed with Ed25519
- TLS 1.3 for all connections
- No plaintext secrets in storage
- Audit logging for all operations
- Regular security audits

### Reliability

- 99.9% uptime for individual nodes
- Network resilience: tolerate 33% Byzantine nodes
- Automatic failover and recovery
- Data redundancy across nodes

## Success Metrics

### Phase 1 (MVP) - COMPLETE

- [x] Successfully push/pull Git repositories
- [x] Create and verify cryptographic identities
- [x] Connect 3+ nodes in test network
- [x] < 5s end-to-end git push latency
- [x] Complete PR and issue workflows
- [x] Web interface operational
- [x] Real-time updates working
- [x] CI/CD integration complete
- [x] GitHub-compatible API

### Phase 2 (Production Readiness)

- [ ] Pass third-party security audit
- [ ] 10,000 concurrent connections supported
- [ ] < 2s p95 git push latency at scale
- [ ] 20+ independent validator operators
- [ ] Public testnet stable for 30+ days

### Phase 3 (Adoption)

- [ ] 100+ repositories migrated from GitHub
- [ ] 1,000+ active users
- [ ] SDKs for 4+ programming languages
- [ ] IDE integrations published

### Long-term

- [ ] 10,000+ repositories hosted
- [ ] 100+ validator nodes globally distributed
- [ ] Self-sustaining validator economics
- [ ] Community governance established

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Low adoption | Medium | High | Focus on UX, GitHub migration tools |
| Performance issues | Medium | Medium | Extensive benchmarking, optimization |
| Security vulnerabilities | Low | Critical | Audits, bug bounty, formal verification |
| Network fragmentation | Low | High | Strong governance, incentive alignment |
| Regulatory pressure | Medium | Medium | Decentralization, no central operator |

## Dependencies

### External

- [commonware](https://github.com/commonwarexyz/monorepo) - Core primitives
- [gitoxide](https://github.com/Byron/gitoxide) - Git operations
- [tokio](https://tokio.rs) - Async runtime
- [RocksDB](https://rocksdb.org) - Storage engine

### Internal

- Infrastructure deployment automation
- CI/CD pipeline
- Documentation and developer guides

## Timeline

### Completed Milestones

#### Milestone 1-9: MVP Complete
All core MVP milestones have been completed:
- Foundation (Git storage, protocol, node API)
- P2P replication with commonware
- Collaboration (PRs, Issues, Comments, Reviews)
- Governance (Organizations, Teams, Permissions)
- Web Gateway (UI, Search, API Documentation)
- Real-time Updates (WebSocket, Notifications)
- CI/CD Integration (Workflows, Runs, Artifacts)
- Git/GitHub Compatibility (Users, Tokens, Releases)
- Production Quality (Observability, Testing, Resilience)

### Upcoming Milestones (Path to Production)

#### Milestone 10: Security Hardening
- Third-party security audit preparation
- Comprehensive threat modeling
- Extended fuzzing (15+ targets)
- Supply chain security (SBOM, reproducible builds)

#### Milestone 11: Performance & Scalability
- Load testing at 1000+ concurrent users
- RocksDB persistent storage integration
- Consensus throughput optimization
- Multi-level caching strategy

#### Milestone 12: True Decentralization
- DHT-based peer discovery (no bootstrap required)
- Stake-based validator governance
- Sybil resistance mechanisms
- Public testnet with 20+ operators

#### Milestone 13: Operator Experience
- Comprehensive operational documentation
- 15+ operational runbooks
- Monitoring dashboards and alerting
- Disaster recovery procedures
- Multi-cloud deployment guides

#### Milestone 14: User Adoption
- GitHub/GitLab migration tools
- SDKs (TypeScript, Python, Go, Rust)
- IDE integrations (VS Code, JetBrains)
- SSH protocol support
- Community platform

## Open Questions

1. **Incentive Model**: How do we incentivize node operators?
2. **Spam Prevention**: How do we prevent repository spam without central moderation?
3. **Large File Storage**: How do we handle Git LFS equivalents?
4. **Search**: How do we implement decentralized code search?

## References

- [Commonware Documentation](https://github.com/commonwarexyz/monorepo)
- [Git Protocol Specification](https://git-scm.com/docs/protocol-v2)
- [IPFS Whitepaper](https://ipfs.tech/whitepaper/)
- [Byzantine Fault Tolerance](https://pmg.csail.mit.edu/papers/osdi99.pdf)

---

*This document is a living specification and will be updated as the project evolves.*
