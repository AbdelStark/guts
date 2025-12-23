# System Architecture for Operators

> Understanding Guts architecture to effectively operate and troubleshoot nodes.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              GUTS NODE                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │   HTTP API   │  │   Git HTTP   │  │  WebSocket   │  │   Metrics   │ │
│  │   (Axum)     │  │   Protocol   │  │  (Realtime)  │  │ (Prometheus)│ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘ │
│         │                 │                 │                  │        │
│         └────────────────┬┴─────────────────┴──────────────────┘        │
│                          │                                               │
│  ┌───────────────────────┴────────────────────────────────────────────┐ │
│  │                        APPLICATION LAYER                            │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌─────────────┐   │ │
│  │  │Collaboration│  │    Auth    │  │   CI/CD    │  │  Compat     │   │ │
│  │  │ (PRs/Issues)│  │(Orgs/Teams)│  │(Workflows) │  │(GitHub API) │   │ │
│  │  └────────────┘  └────────────┘  └────────────┘  └─────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│  ┌─────────────────────────────────┴──────────────────────────────────┐ │
│  │                         CORE LAYER                                  │ │
│  │  ┌────────────────────┐  ┌────────────────────┐                    │ │
│  │  │    Git Storage     │  │   Consensus Engine │                    │ │
│  │  │  (RocksDB/Memory)  │  │   (Simplex BFT)    │                    │ │
│  │  └────────────────────┘  └────────────────────┘                    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│  ┌─────────────────────────────────┴──────────────────────────────────┐ │
│  │                       NETWORK LAYER                                 │ │
│  │  ┌────────────────────────────────────────────────────────────────┐│ │
│  │  │                    P2P Network (commonware)                    ││ │
│  │  │  • Authenticated connections (Ed25519)                         ││ │
│  │  │  • Multi-channel messaging (consensus, data, sync)            ││ │
│  │  │  • QUIC + TCP transport                                        ││ │
│  │  └────────────────────────────────────────────────────────────────┘│ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Component Overview

### HTTP API Layer

The HTTP API layer handles all external requests:

| Component | Port | Protocol | Purpose |
|-----------|------|----------|---------|
| REST API | 8080 | HTTP/1.1, HTTP/2 | Repository, collaboration, auth APIs |
| Git HTTP | 8080 | HTTP/1.1 | Git smart protocol (clone, push, fetch) |
| WebSocket | 8080 | WS/WSS | Real-time updates, notifications |
| Metrics | 9090 | HTTP/1.1 | Prometheus metrics scraping |

**Key endpoints:**
- `/api/*` - REST API
- `/git/{owner}/{repo}/*` - Git protocol
- `/ws` - WebSocket connections
- `/health/*` - Health checks
- `/metrics` - Prometheus metrics

### Application Layer

Business logic is organized into feature modules:

| Module | Purpose | Key Data |
|--------|---------|----------|
| **Collaboration** | Pull requests, issues, reviews | PRs, issues, comments, reviews |
| **Auth** | Organizations, teams, permissions | Orgs, teams, members, ACLs |
| **CI/CD** | Workflows, runs, artifacts | Pipelines, jobs, artifacts |
| **Compat** | GitHub API compatibility | Users, tokens, releases |

### Core Layer

#### Git Storage

Content-addressed storage for Git objects:

```
/var/lib/guts/
├── objects/           # Git objects (blobs, trees, commits)
│   ├── pack/          # Pack files
│   └── loose/         # Loose objects
├── refs/              # Branch and tag references
├── consensus/         # Consensus state
└── metadata/          # Repository metadata
```

**Storage backends:**
- **Memory** (development): Fast, ephemeral
- **RocksDB** (production): Persistent, optimized for SSDs

#### Consensus Engine

Simplex BFT consensus provides:

- **Total ordering:** All state changes ordered globally
- **Finality:** Blocks are final after 3 network hops
- **Byzantine tolerance:** Tolerates f < n/3 malicious validators

**Consensus flow:**

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Propose  │───▶│   Vote   │───▶│ Finalize │───▶│  Apply   │
│  (Hop 1) │    │  (Hop 2) │    │  (Hop 3) │    │ (Local)  │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

### Network Layer

P2P networking using commonware primitives:

| Channel | ID | Purpose |
|---------|-----|---------|
| Pending | 0 | Consensus votes in progress |
| Recovered | 1 | Recovered/replayed messages |
| Resolver | 2 | Certificate resolution |
| Broadcast | 3 | Block broadcast to peers |
| Sync | 4 | Block sync and state transfer |

**Connection properties:**
- Authenticated via Ed25519 keys
- Encrypted via Noise protocol
- Multiplexed over QUIC/TCP

## Data Flow

### Write Path (Push)

```
Client                    Node                      Network
  │                        │                          │
  │  git push              │                          │
  │───────────────────────▶│                          │
  │                        │  1. Parse pack file      │
  │                        │  2. Store objects        │
  │                        │  3. Submit to consensus  │
  │                        │─────────────────────────▶│
  │                        │                          │  Broadcast
  │                        │                          │  to validators
  │                        │◀─────────────────────────│
  │                        │  4. Wait for finality    │
  │                        │  5. Update refs          │
  │◀───────────────────────│                          │
  │  OK (refs updated)     │                          │
```

### Read Path (Clone/Fetch)

```
Client                    Node                      Storage
  │                        │                          │
  │  git clone             │                          │
  │───────────────────────▶│                          │
  │                        │  1. Check refs           │
  │                        │─────────────────────────▶│
  │                        │◀─────────────────────────│
  │                        │  2. Negotiate objects    │
  │                        │  3. Generate pack file   │
  │                        │─────────────────────────▶│
  │◀───────────────────────│                          │
  │  Pack file stream      │                          │
```

## High Availability

### Single Node

For non-critical deployments:

```
┌────────────────┐
│   Load Balancer │
└────────┬───────┘
         │
    ┌────┴────┐
    │  Node   │
    └─────────┘
```

**Pros:** Simple, low cost
**Cons:** Single point of failure

### Multi-Node (Recommended)

For production deployments:

```
┌────────────────────────────────────────────────┐
│                Load Balancer                    │
└────────┬───────────────┬───────────────┬───────┘
         │               │               │
    ┌────┴────┐     ┌────┴────┐     ┌────┴────┐
    │ Node 1  │◀───▶│ Node 2  │◀───▶│ Node 3  │
    │(Validat)│     │(Validat)│     │(Validat)│
    └─────────┘     └─────────┘     └─────────┘
         │               │               │
         └───────────────┴───────────────┘
                    P2P Mesh
```

**Pros:** High availability, fault tolerance
**Cons:** Higher complexity and cost

### Geographic Distribution

For global deployments:

```
    US-East              EU-West              AP-Tokyo
  ┌─────────┐          ┌─────────┐          ┌─────────┐
  │ Node 1  │◀────────▶│ Node 3  │◀────────▶│ Node 5  │
  │ Node 2  │          │ Node 4  │          │ Node 6  │
  └─────────┘          └─────────┘          └─────────┘
       │                    │                    │
       └────────────────────┴────────────────────┘
              Global P2P Network
```

## Failure Modes

### Node Failure

| Scenario | Impact | Recovery |
|----------|--------|----------|
| Single node crash | API unavailable | Restart node, automatic rejoin |
| Storage corruption | Data loss possible | Restore from backup or resync |
| Network partition | Split-brain possible | Consensus handles (2f+1 required) |

### Consensus Failure

| Scenario | Impact | Recovery |
|----------|--------|----------|
| < f nodes down | None | Continue normally |
| f nodes down | Degraded performance | Restore nodes |
| > f nodes down | Consensus halts | Restore to quorum |

**Note:** f = floor((n-1)/3) for n validators

### Network Failure

| Scenario | Impact | Recovery |
|----------|--------|----------|
| Peer disconnection | Reduced replication | Automatic reconnect |
| Bootstrap failure | Can't join network | Check bootstrap nodes |
| Firewall blocking | P2P not working | Check firewall rules |

## Operational Metrics

### Key Health Indicators

| Metric | Healthy Range | Alert Threshold |
|--------|---------------|-----------------|
| `guts_p2p_peers_connected` | > 3 | < 3 |
| `guts_consensus_block_height` | Increasing | Stalled > 1min |
| `guts_http_request_duration_seconds` | p99 < 100ms | p99 > 1s |
| `guts_storage_available_bytes` | > 10% capacity | < 10% |

### Performance Baselines

| Operation | Expected Latency | Notes |
|-----------|------------------|-------|
| API read | < 10ms | Local storage read |
| API write | < 100ms | Includes consensus |
| Git clone (1MB) | < 1s | Depends on network |
| Git push (1MB) | < 2s | Includes consensus finality |

## Security Architecture

### Network Security

```
┌─────────────────────────────────────────────────────┐
│                    Internet                          │
└────────────────────────┬────────────────────────────┘
                         │
                    ┌────┴────┐
                    │   TLS   │  (API, Git HTTPS)
                    │ Termination │
                    └────┬────┘
                         │
┌────────────────────────┼────────────────────────────┐
│   Private Network      │                            │
│                   ┌────┴────┐                       │
│                   │  Node   │                       │
│                   └────┬────┘                       │
│                        │                            │
│              ┌─────────┴─────────┐                  │
│              │   Noise Protocol  │  (P2P)           │
│              └───────────────────┘                  │
└─────────────────────────────────────────────────────┘
```

### Key Management

| Key Type | Purpose | Storage |
|----------|---------|---------|
| Node key | P2P authentication | File, HSM, or KMS |
| TLS cert | HTTPS termination | File or cert manager |
| API tokens | User authentication | Database (hashed) |

## Scaling Considerations

### Vertical Scaling

Increase resources on existing nodes:

| Bottleneck | Solution |
|------------|----------|
| CPU | More cores, faster clock |
| Memory | More RAM |
| Storage I/O | NVMe, RAID |
| Network | Higher bandwidth |

### Horizontal Scaling

Add more nodes for read scaling:

- **Read replicas:** Full nodes for read-heavy workloads
- **Load balancing:** Distribute API traffic
- **CDN:** Cache static assets, archives

### Limitations

- **Write scaling:** Limited by consensus throughput
- **Storage:** All nodes store all data (no sharding yet)
- **Consensus:** Max recommended validators: 100

## Integration Points

### Monitoring Stack

```
┌────────────┐     ┌────────────┐     ┌────────────┐
│ Guts Node  │────▶│ Prometheus │────▶│  Grafana   │
│ (/metrics) │     │            │     │            │
└────────────┘     └────────────┘     └────────────┘
       │
       │           ┌────────────┐     ┌────────────┐
       └──────────▶│    Loki    │────▶│  Grafana   │
         (logs)    │            │     │            │
                   └────────────┘     └────────────┘
```

### CI/CD Integration

```
┌────────────┐     ┌────────────┐     ┌────────────┐
│    Git     │────▶│ Guts Node  │────▶│  Webhook   │
│   Push     │     │            │     │            │
└────────────┘     └────────────┘     └────────────┘
                          │
                          ▼
                   ┌────────────┐
                   │  CI Runner │
                   │ (Jenkins,  │
                   │  GitHub,   │
                   │  etc.)     │
                   └────────────┘
```

## Related Documentation

- [Requirements](requirements.md) - Hardware and software requirements
- [Configuration Reference](configuration/reference.md) - All configuration options
- [Monitoring Guide](operations/monitoring.md) - Setting up monitoring
- [Troubleshooting](troubleshooting/common-issues.md) - Common issues
