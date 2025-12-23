---
layout: home

hero:
  name: Guts
  text: Decentralized Code Collaboration
  tagline: A censorship-resistant Git platform built on Byzantine Fault Tolerant consensus. Your code, your infrastructure, no single point of failure.
  image:
    src: /logo.svg
    alt: Guts
  actions:
    - theme: brand
      text: Get Started
      link: /developer/quickstart/
    - theme: alt
      text: View on GitHub
      link: https://github.com/AbdelStark/guts

features:
  - icon: 🔐
    title: Censorship Resistant
    details: No single entity can remove, censor, or restrict access to your repositories. Built on decentralized infrastructure that can't be shut down.

  - icon: ⚡
    title: Simplex BFT Consensus
    details: Byzantine Fault Tolerant consensus ensures all nodes agree on repository state. Tolerates up to 1/3 malicious validators with 3-hop finality.

  - icon: 🔄
    title: Git Compatible
    details: Works with standard Git clients. Push, pull, clone just like you're used to. Seamless migration from GitHub, GitLab, or Bitbucket.

  - icon: 🌐
    title: Peer-to-Peer Network
    details: Encrypted P2P communication using the Noise protocol. Content-addressed storage enables automatic deduplication and integrity verification.

  - icon: 🔑
    title: Cryptographic Identity
    details: Ed25519-based identities ensure verifiable authorship. Every commit is signed and verified across the network.

  - icon: 🛠️
    title: Full GitHub API Compatibility
    details: Organizations, teams, pull requests, issues, reviews, releases, and webhooks. Migrate your existing workflows seamlessly.
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: linear-gradient(135deg, #FF3B2E 0%, #FF6B5A 50%, #6AE4FF 100%);
}

.VPFeatures .details {
  font-size: 0.95rem;
  line-height: 1.6;
}
</style>

## Why Guts?

Modern code hosting platforms are centralized points of failure. A single company's decision can affect millions of developers. Repositories get removed, accounts get banned, and geographic restrictions limit global collaboration.

**Guts changes this.** By building on Byzantine Fault Tolerant consensus and peer-to-peer networking, Guts creates code collaboration infrastructure that:

- **Can't be taken down** - No central server means no single point of failure
- **Can't be censored** - Distributed consensus prevents arbitrary removal
- **Can't be controlled** - No single entity owns the network

## Quick Start

### For Developers

Get started with Guts in under 5 minutes:

```bash
# Install the CLI
cargo install guts-cli

# Generate your identity
guts identity generate

# Clone a repository
git clone https://guts.network/owner/repo.git
```

[Read the Developer Guide →](/developer/)

### For Operators

Deploy a Guts node:

```bash
# Docker (quickest)
docker run -d -p 8080:8080 -p 9000:9000 ghcr.io/guts-network/guts-node:latest

# Or use Kubernetes
helm install guts-node guts/guts-node --namespace guts
```

[Read the Operator Guide →](/operator/)

## Architecture

Guts is built on proven primitives from [commonware](https://github.com/commonwarexyz/monorepo):

| Layer | Technology | Purpose |
|-------|------------|---------|
| Consensus | Simplex BFT | Total ordering of state changes |
| Networking | commonware::p2p | Encrypted peer-to-peer communication |
| Storage | RocksDB + Content-Addressed | Persistent, deduplicated storage |
| Identity | Ed25519 | Cryptographic signatures |

[View Architecture Decisions →](/architecture/adr/)

## Project Status

Guts has completed 13 milestones covering core functionality, collaboration features, governance, real-time updates, CI/CD integration, performance optimization, and true decentralization with Simplex BFT consensus.

| Component | Status |
|-----------|--------|
| Git Protocol | Production Ready |
| P2P Replication | Production Ready |
| Simplex BFT Consensus | Production Ready |
| Pull Requests & Issues | Production Ready |
| Organizations & Teams | Production Ready |
| TypeScript & Python SDKs | Stable |
| Migration Tools | Stable |

**Next milestone:** Security hardening and audit preparation.

[View Full Roadmap →](/architecture/roadmap)
