# Guts Operator Guide

> Comprehensive documentation for deploying, operating, and maintaining Guts nodes in production environments.

## Overview

This guide is designed for operators who need to run Guts nodes reliably in production. Whether you're running a single node for development, a validator in the consensus network, or a fleet of nodes serving a large organization, this documentation covers everything you need to know.

## Quick Navigation

| Section | Description | Audience |
|---------|-------------|----------|
| [Quickstart](quickstart.md) | Deploy a node in 5 minutes | New operators |
| [Requirements](requirements.md) | Hardware, software, network requirements | All operators |
| [Architecture](architecture.md) | System architecture overview | All operators |
| [Installation](installation/) | Detailed installation guides | All operators |
| [Configuration](configuration/) | Configuration reference | All operators |
| [Operations](operations/) | Day-to-day operations | Production operators |
| [Runbooks](runbooks/) | Incident response procedures | On-call engineers |
| [Troubleshooting](troubleshooting/) | Common issues and solutions | All operators |
| [Reference](reference/) | CLI, API, and metrics reference | Advanced operators |

## Deployment Options

### By Environment

| Environment | Recommended Method | Guide |
|-------------|-------------------|-------|
| Development | Docker Compose | [Docker Guide](installation/docker.md) |
| Staging | Kubernetes | [Kubernetes Guide](installation/kubernetes.md) |
| Production | Kubernetes + Helm | [Kubernetes Guide](installation/kubernetes.md) |
| Bare Metal | Systemd | [Bare Metal Guide](installation/bare-metal.md) |

### By Cloud Provider

| Provider | Guide | Terraform |
|----------|-------|-----------|
| AWS | [AWS Quickstart](../cloud/aws/quickstart.md) | ✅ Available |
| GCP | [GCP Quickstart](../cloud/gcp/quickstart.md) | ✅ Available |
| Azure | [Azure Quickstart](../cloud/azure/quickstart.md) | ✅ Available |
| Multi-Cloud | [Federation Guide](../cloud/multi-cloud/federation.md) | ✅ Available |

## Node Types

Guts supports different node configurations depending on your use case:

### Full Node

A full node stores all repository data and participates in P2P replication but does not participate in consensus.

```bash
# Start a full node
guts-node --api-addr 0.0.0.0:8080 --p2p-addr 0.0.0.0:9000
```

**Use cases:**
- API gateway for applications
- Read replicas for load distribution
- Local development and testing

### Validator Node

A validator node participates in Simplex BFT consensus, proposing and voting on blocks.

```bash
# Start a validator node
guts-node \
  --api-addr 0.0.0.0:8080 \
  --p2p-addr 0.0.0.0:9000 \
  --private-key <hex-encoded-key> \
  --consensus-enabled \
  --consensus-use-simplex-bft \
  --genesis-file genesis.json
```

**Requirements:**
- Stable network connectivity (99.9%+ uptime)
- High-performance hardware (see [Requirements](requirements.md))
- Secure key management
- 24/7 monitoring

## Key Concepts

### Simplex BFT Consensus

Guts uses Simplex BFT (Byzantine Fault Tolerant) consensus for total ordering of all state changes. The consensus engine:

- Tolerates up to f < n/3 Byzantine (malicious) validators
- Achieves finality in 3 network hops
- Produces blocks at configurable intervals (default: 2 seconds)

### P2P Networking

All Guts nodes communicate via encrypted P2P connections using the commonware networking stack:

- **Transport:** QUIC (UDP) and TCP
- **Encryption:** Noise protocol with Ed25519 keys
- **Discovery:** Bootstrap nodes + peer exchange

### Content-Addressed Storage

Git objects are stored using content-addressed storage, enabling:

- Automatic deduplication
- Integrity verification via SHA-1/SHA-256
- Efficient P2P replication

## Operational Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Deploy    │────▶│   Operate   │────▶│   Upgrade   │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
  Installation         Monitoring          Zero-Downtime
  Configuration        Alerting            Rolling Updates
  Verification         Backup              Rollback
```

### 1. Deploy

- Choose deployment method based on environment
- Configure node identity and networking
- Verify connectivity and sync status

### 2. Operate

- Monitor metrics and logs
- Respond to alerts using runbooks
- Perform regular backups
- Apply security patches

### 3. Upgrade

- Review release notes and compatibility
- Test in staging environment
- Perform zero-downtime upgrade
- Verify functionality and rollback if needed

## Support Channels

| Channel | Purpose | Response Time |
|---------|---------|---------------|
| [GitHub Issues](https://github.com/guts-network/guts/issues) | Bug reports, feature requests | 1-3 days |
| [Discord](https://discord.gg/guts) | Community support | Best effort |
| [Documentation](https://docs.guts.network) | Self-service guides | Instant |

## Version Compatibility

| Node Version | Protocol Version | Network Compatibility |
|--------------|------------------|----------------------|
| 1.0.x | 1 | mainnet, testnet |
| 0.x.x | 0 | devnet only |

## Security Considerations

- **Key Management:** Never expose private keys in logs or environment variables visible to unauthorized users
- **Network Security:** Always use TLS for external API access
- **Access Control:** Restrict metrics and admin endpoints to internal networks
- **Updates:** Subscribe to security advisories and apply patches promptly

See [Security Hardening](configuration/security.md) for detailed recommendations.

## Contributing to Documentation

Found an error or want to improve this documentation? Contributions are welcome!

1. Fork the repository
2. Edit files in `docs/operator/`
3. Submit a pull request

## Next Steps

1. **New operators:** Start with [Quickstart](quickstart.md)
2. **Production deployment:** Read [Requirements](requirements.md) then choose an [Installation](installation/) method
3. **Existing operators:** Review [Operations](operations/) and [Runbooks](runbooks/)
