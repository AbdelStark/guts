# ADR-001: Use Commonware for P2P and Consensus

## Status

Accepted

## Date

2025-12-20

## Context

Guts requires robust infrastructure for:

1. **Peer-to-peer networking**: Nodes must discover and communicate with each other
2. **Byzantine Fault Tolerant consensus**: Repository state must be consistent across nodes
3. **Cryptographic identity**: Authors must be verifiable through digital signatures
4. **Message encoding**: Efficient serialization for network messages

Building these primitives from scratch would require significant effort and introduces risk of security vulnerabilities.

## Decision

We will use [commonware](https://github.com/commonwarexyz/monorepo) as the foundation for Guts' distributed systems layer:

- **commonware-p2p**: Encrypted peer-to-peer networking with peer discovery
- **commonware-consensus**: BFT consensus for repository state agreement
- **commonware-cryptography**: Ed25519 signatures and key management
- **commonware-codec**: Efficient binary serialization
- **commonware-runtime**: Async runtime integration

## Consequences

### Positive

- **Battle-tested**: Commonware is maintained and used in production systems
- **Security**: Cryptographic primitives are well-audited
- **Compatibility**: Designed for Rust async ecosystem (Tokio)
- **Community**: Active development and support
- **Speed**: Optimized for high-performance networking

### Negative

- **Dependency**: Tight coupling to commonware's API and release cycle
- **Version pinning**: Must track commonware releases (currently at 0.0.63)
- **Learning curve**: Contributors must understand commonware concepts
- **Less control**: Some design decisions are dictated by commonware

### Neutral

- Requires Rust nightly for some features (mitigated by workspace configuration)

## Alternatives Considered

### libp2p

A widely-used P2P library with a large ecosystem.

**Rejected because:**
- More complex API surface
- Less optimized for our specific use case
- No integrated consensus solution

### Custom Implementation

Build P2P and consensus from scratch.

**Rejected because:**
- High development cost
- Security risk without extensive auditing
- Time to market significantly increased

### Tendermint/CometBFT

Established BFT consensus implementation.

**Rejected because:**
- Go implementation (language boundary)
- Heavier runtime requirements
- Less control over consensus parameters

## References

- [Commonware GitHub](https://github.com/commonwarexyz/monorepo)
- [Byzantine Fault Tolerance](https://pmg.csail.mit.edu/papers/osdi99.pdf)
- [Guts PRD - Technology Stack](../PRD.md#technology-stack)
