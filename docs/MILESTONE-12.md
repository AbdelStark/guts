# Milestone 12: Operator Experience & Documentation

> **Status:** Planned
> **Target:** Q2-Q3 2025
> **Priority:** High

## Overview

Milestone 13 focuses on making Guts easy to deploy, operate, and maintain in production environments. A decentralized network is only as strong as its operators. This milestone provides comprehensive documentation, operational runbooks, monitoring dashboards, disaster recovery procedures, and automation tools that enable anyone to run a reliable Guts node.

## Goals

1. **Operator Documentation**: Comprehensive guides for deploying and operating Guts nodes
2. **Operational Runbooks**: Step-by-step procedures for common operational scenarios
3. **Monitoring & Alerting**: Pre-built dashboards and alert rules
4. **Disaster Recovery**: Documented and tested backup/restore procedures
5. **Upgrade Procedures**: Zero-downtime upgrade paths
6. **Multi-Cloud Support**: Deployment guides for AWS, GCP, Azure, and bare metal
7. **Automation Tools**: Ansible/Terraform modules for infrastructure

## Documentation Structure

```
docs/
├── operator/
│   ├── README.md                    # Operator guide overview
│   ├── quickstart.md                # 5-minute deployment
│   ├── architecture.md              # System architecture for operators
│   ├── requirements.md              # Hardware and network requirements
│   ├── installation/
│   │   ├── docker.md                # Docker deployment
│   │   ├── kubernetes.md            # Kubernetes deployment
│   │   ├── bare-metal.md            # Bare metal deployment
│   │   └── systemd.md               # Systemd service setup
│   ├── configuration/
│   │   ├── reference.md             # Full configuration reference
│   │   ├── networking.md            # Network configuration
│   │   ├── storage.md               # Storage configuration
│   │   ├── security.md              # Security hardening
│   │   └── performance.md           # Performance tuning
│   ├── operations/
│   │   ├── monitoring.md            # Monitoring setup
│   │   ├── alerting.md              # Alert configuration
│   │   ├── logging.md               # Log management
│   │   ├── backup.md                # Backup procedures
│   │   └── upgrades.md              # Upgrade procedures
│   ├── runbooks/
│   │   ├── README.md                # Runbook index
│   │   ├── node-not-syncing.md      # Node sync issues
│   │   ├── high-memory.md           # Memory problems
│   │   ├── disk-full.md             # Storage issues
│   │   ├── consensus-stuck.md       # Consensus problems
│   │   ├── network-partition.md     # Network issues
│   │   ├── data-corruption.md       # Data recovery
│   │   ├── key-rotation.md          # Key management
│   │   └── emergency-shutdown.md    # Emergency procedures
│   ├── troubleshooting/
│   │   ├── common-issues.md         # FAQ and common problems
│   │   ├── diagnostics.md           # Diagnostic procedures
│   │   └── support.md               # Getting help
│   └── reference/
│       ├── cli.md                   # CLI reference
│       ├── api.md                   # API reference
│       └── metrics.md               # Metrics reference
├── cloud/
│   ├── aws/
│   │   ├── quickstart.md            # AWS quickstart
│   │   ├── architecture.md          # AWS architecture
│   │   └── terraform/               # Terraform modules
│   ├── gcp/
│   │   ├── quickstart.md            # GCP quickstart
│   │   └── terraform/               # Terraform modules
│   ├── azure/
│   │   ├── quickstart.md            # Azure quickstart
│   │   └── terraform/               # Terraform modules
│   └── multi-cloud/
│       └── federation.md            # Multi-cloud setup
└── monitoring/
    ├── dashboards/                  # Grafana dashboards
    ├── alerts/                      # Alert rules
    └── runbook-links/               # Dashboard to runbook links
```

## Detailed Implementation

### Phase 1: Core Documentation

#### 1.1 Quickstart Guide

```markdown
# Guts Node Quickstart

Deploy a Guts node in 5 minutes.

## Prerequisites

- Docker 24+ or Kubernetes 1.28+
- 4 CPU cores, 8GB RAM, 100GB SSD
- Public IP with ports 8080 (HTTP), 9000 (P2P)

## Option 1: Docker (Simplest)

\`\`\`bash
# Generate node identity
docker run --rm guts/node:latest guts-node keygen > node.key

# Start node
docker run -d \
  --name guts-node \
  -p 8080:8080 \
  -p 9000:9000 \
  -v $(pwd)/data:/data \
  -v $(pwd)/node.key:/etc/guts/node.key \
  guts/node:latest

# Verify node is running
curl http://localhost:8080/health/ready
\`\`\`

## Option 2: Kubernetes (Production)

\`\`\`bash
# Add Helm repository
helm repo add guts https://charts.guts.network

# Install
helm install guts-node guts/guts-node \
  --set persistence.size=100Gi \
  --set resources.requests.memory=8Gi

# Check status
kubectl get pods -l app=guts-node
\`\`\`

## Next Steps

- [Configure networking](configuration/networking.md)
- [Set up monitoring](operations/monitoring.md)
- [Join the network](../guides/joining-network.md)
```

#### 1.2 Requirements Documentation

```markdown
# System Requirements

## Hardware Requirements

### Minimum (Development/Testing)

| Component | Minimum | Notes |
|-----------|---------|-------|
| CPU | 2 cores | x86_64 or ARM64 |
| RAM | 4 GB | |
| Storage | 50 GB SSD | NVMe preferred |
| Network | 10 Mbps | |

### Recommended (Production)

| Component | Recommended | Notes |
|-----------|-------------|-------|
| CPU | 8 cores | Dedicated, not shared |
| RAM | 32 GB | ECC preferred |
| Storage | 500 GB NVMe | RAID-1 for reliability |
| Network | 1 Gbps | Low latency preferred |

### Validator Requirements

| Component | Required | Notes |
|-----------|----------|-------|
| CPU | 16 cores | High single-thread performance |
| RAM | 64 GB | ECC required |
| Storage | 2 TB NVMe | RAID-1 required |
| Network | 1 Gbps | 99.9% uptime required |
| UPS | Yes | Graceful shutdown support |

## Software Requirements

- Linux kernel 5.10+ (Ubuntu 22.04+, Debian 12+, RHEL 9+)
- Docker 24+ or containerd 1.7+
- Kubernetes 1.28+ (for K8s deployment)

## Network Requirements

| Port | Protocol | Purpose | Required |
|------|----------|---------|----------|
| 8080 | TCP | HTTP API | Yes |
| 9000 | TCP/UDP | P2P | Yes |
| 9090 | TCP | Metrics | Optional |
| 443 | TCP | HTTPS (with proxy) | Recommended |

### Firewall Rules

\`\`\`bash
# Required inbound
ufw allow 8080/tcp  # API
ufw allow 9000/tcp  # P2P
ufw allow 9000/udp  # QUIC

# Optional (internal only)
ufw allow from 10.0.0.0/8 to any port 9090  # Metrics
\`\`\`
```

### Phase 2: Operational Runbooks

#### 2.1 Runbook Template

```markdown
# Runbook: [Issue Name]

**Severity:** P1/P2/P3/P4
**Impact:** [Description of user/system impact]
**On-Call Action:** [Immediate action required]

## Symptoms

- [ ] Symptom 1
- [ ] Symptom 2
- [ ] Symptom 3

## Detection

**Alert Name:** `guts_[metric]_critical`

**Query:**
\`\`\`promql
[Prometheus query that triggers this alert]
\`\`\`

## Diagnosis

### Step 1: Verify the Issue

\`\`\`bash
# Command to verify
guts-node status
\`\`\`

Expected output: [description]
Actual output if issue present: [description]

### Step 2: Check Related Metrics

\`\`\`bash
# Check metrics
curl -s localhost:9090/metrics | grep [relevant_metric]
\`\`\`

### Step 3: Review Logs

\`\`\`bash
# Check recent logs
journalctl -u guts-node --since "10 minutes ago" | grep -i error
\`\`\`

## Resolution

### Option A: [First Resolution Path]

\`\`\`bash
# Step-by-step commands
\`\`\`

**Expected Result:** [What should happen]

### Option B: [Alternative Resolution]

\`\`\`bash
# Alternative steps
\`\`\`

## Escalation

If the above steps don't resolve the issue:

1. Collect diagnostics: `guts-node diagnostics > diag.tar.gz`
2. Contact: [escalation contact]
3. Include: Node ID, timestamp, diagnostic bundle

## Post-Incident

- [ ] Update monitoring if detection was delayed
- [ ] Document any new resolution steps
- [ ] Create follow-up ticket if root cause needs investigation

## Related Runbooks

- [Related Runbook 1](link)
- [Related Runbook 2](link)
```

#### 2.2 Node Not Syncing Runbook

```markdown
# Runbook: Node Not Syncing

**Severity:** P2
**Impact:** Node cannot serve current data, may serve stale content
**On-Call Action:** Investigate within 30 minutes

## Symptoms

- [ ] Node reports sync status as "syncing" for extended period
- [ ] Block height not increasing
- [ ] API returns stale data
- [ ] Alert: `guts_sync_lag_seconds > 60`

## Detection

**Alert Name:** `guts_node_sync_stalled`

**Query:**
\`\`\`promql
time() - guts_last_block_time > 60
\`\`\`

## Diagnosis

### Step 1: Check Sync Status

\`\`\`bash
guts-node status --format json | jq '.sync'
\`\`\`

Expected:
\`\`\`json
{
  "status": "synced",
  "current_height": 12345,
  "highest_known": 12345,
  "peers": 5
}
\`\`\`

### Step 2: Check Peer Connectivity

\`\`\`bash
guts-node peers list
\`\`\`

If fewer than 3 peers:
- Check firewall rules
- Verify bootstrap nodes are reachable
- Check for network issues

### Step 3: Check Disk Space

\`\`\`bash
df -h /var/lib/guts
\`\`\`

If usage > 90%, see [Disk Full Runbook](disk-full.md)

### Step 4: Check Memory

\`\`\`bash
free -h
\`\`\`

If memory exhausted, see [High Memory Runbook](high-memory.md)

### Step 5: Check for Consensus Issues

\`\`\`bash
guts-node consensus status
\`\`\`

If consensus is stuck, see [Consensus Stuck Runbook](consensus-stuck.md)

## Resolution

### Option A: Restart Node

\`\`\`bash
systemctl restart guts-node
# Wait 2 minutes
guts-node status
\`\`\`

### Option B: Force Resync from Peers

\`\`\`bash
# Stop node
systemctl stop guts-node

# Clear sync state (preserves data)
guts-node sync reset

# Restart
systemctl start guts-node
\`\`\`

### Option C: Resync from Snapshot

\`\`\`bash
# Stop node
systemctl stop guts-node

# Download latest snapshot
guts-node snapshot download --latest

# Restart
systemctl start guts-node
\`\`\`

## Escalation

If none of the above works:
1. Collect full diagnostics
2. Check if other nodes in network have same issue
3. Escalate to core team if network-wide

## Post-Incident

- [ ] Verify node caught up completely
- [ ] Check for any data inconsistencies
- [ ] Monitor for recurrence
```

### Phase 3: Monitoring & Alerting

#### 3.1 Grafana Dashboards

```json
{
  "dashboard": {
    "title": "Guts Node Overview",
    "panels": [
      {
        "title": "Node Health",
        "type": "stat",
        "gridPos": {"x": 0, "y": 0, "w": 6, "h": 4},
        "targets": [
          {
            "expr": "up{job='guts-node'}",
            "legendFormat": "Node Status"
          }
        ],
        "options": {
          "colorMode": "background",
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {"color": "red", "value": 0},
              {"color": "green", "value": 1}
            ]
          }
        }
      },
      {
        "title": "Sync Status",
        "type": "gauge",
        "gridPos": {"x": 6, "y": 0, "w": 6, "h": 4},
        "targets": [
          {
            "expr": "guts_sync_percentage",
            "legendFormat": "Sync %"
          }
        ]
      },
      {
        "title": "Connected Peers",
        "type": "stat",
        "gridPos": {"x": 12, "y": 0, "w": 6, "h": 4},
        "targets": [
          {
            "expr": "guts_p2p_peers_connected",
            "legendFormat": "Peers"
          }
        ]
      },
      {
        "title": "Request Rate",
        "type": "graph",
        "gridPos": {"x": 0, "y": 4, "w": 12, "h": 8},
        "targets": [
          {
            "expr": "rate(guts_http_requests_total[5m])",
            "legendFormat": "{{method}} {{path}}"
          }
        ]
      },
      {
        "title": "Request Latency (p99)",
        "type": "graph",
        "gridPos": {"x": 12, "y": 4, "w": 12, "h": 8},
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(guts_http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p99 Latency"
          }
        ]
      }
    ]
  }
}
```

#### 3.2 Alert Rules

```yaml
# infra/monitoring/alerts/guts-alerts.yml
groups:
  - name: guts-node
    rules:
      # Node availability
      - alert: GutsNodeDown
        expr: up{job="guts-node"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Guts node is down"
          description: "Node {{ $labels.instance }} has been down for more than 1 minute"
          runbook_url: "https://docs.guts.network/runbooks/node-down"

      # Sync issues
      - alert: GutsNodeNotSyncing
        expr: time() - guts_last_block_time > 60
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Guts node not syncing"
          description: "Node {{ $labels.instance }} hasn't received a block in 60 seconds"
          runbook_url: "https://docs.guts.network/runbooks/node-not-syncing"

      # Peer connectivity
      - alert: GutsLowPeerCount
        expr: guts_p2p_peers_connected < 3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low peer count"
          description: "Node {{ $labels.instance }} has fewer than 3 peers"
          runbook_url: "https://docs.guts.network/runbooks/low-peers"

      # API latency
      - alert: GutsHighAPILatency
        expr: histogram_quantile(0.99, rate(guts_http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High API latency"
          description: "API p99 latency is above 1 second"
          runbook_url: "https://docs.guts.network/runbooks/high-latency"

      # Resource usage
      - alert: GutsHighMemoryUsage
        expr: guts_process_resident_memory_bytes / guts_config_max_memory > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is above 90%"
          runbook_url: "https://docs.guts.network/runbooks/high-memory"

      # Disk space
      - alert: GutsDiskSpaceLow
        expr: guts_storage_available_bytes / guts_storage_total_bytes < 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Disk space critically low"
          description: "Less than 10% disk space remaining"
          runbook_url: "https://docs.guts.network/runbooks/disk-full"

      # Consensus
      - alert: GutsConsensusStalled
        expr: rate(guts_consensus_commits_total[5m]) == 0
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Consensus stalled"
          description: "No consensus commits in 10 minutes"
          runbook_url: "https://docs.guts.network/runbooks/consensus-stuck"
```

### Phase 4: Disaster Recovery

#### 4.1 Backup Procedures

```markdown
# Backup Procedures

## Overview

Guts nodes should be backed up regularly to enable recovery from:
- Hardware failures
- Data corruption
- Accidental deletion
- Ransomware attacks

## What to Backup

| Component | Location | Frequency | Retention |
|-----------|----------|-----------|-----------|
| Node key | `/etc/guts/node.key` | Once (after creation) | Forever |
| Configuration | `/etc/guts/config.toml` | On change | 30 days |
| Data directory | `/var/lib/guts/data` | Daily | 7 days |
| Consensus state | `/var/lib/guts/consensus` | Hourly | 24 hours |

## Backup Methods

### Method 1: Snapshot (Recommended)

\`\`\`bash
# Create consistent snapshot
guts-node backup create --output /backup/guts-$(date +%Y%m%d).tar.gz

# Upload to S3
aws s3 cp /backup/guts-$(date +%Y%m%d).tar.gz s3://guts-backups/
\`\`\`

### Method 2: Filesystem Snapshot (LVM/ZFS)

\`\`\`bash
# Pause writes
guts-node maintenance enter

# Create LVM snapshot
lvcreate -L 10G -s -n guts-snap /dev/vg0/guts-data

# Resume writes
guts-node maintenance exit

# Backup snapshot
tar -czf /backup/guts-$(date +%Y%m%d).tar.gz /mnt/snap
\`\`\`

### Method 3: Continuous Replication

\`\`\`bash
# Set up WAL archiving to S3
guts-node config set wal.archive_command "aws s3 cp %f s3://guts-wal/"
guts-node config set wal.restore_command "aws s3 cp s3://guts-wal/%f %p"
\`\`\`

## Backup Verification

\`\`\`bash
# Verify backup integrity
guts-node backup verify /backup/guts-20250101.tar.gz

# Test restore to temporary location
guts-node backup restore /backup/guts-20250101.tar.gz --target /tmp/guts-test
\`\`\`

## Automation

\`\`\`yaml
# /etc/cron.d/guts-backup
0 */6 * * * root /usr/local/bin/guts-backup.sh >> /var/log/guts-backup.log 2>&1
\`\`\`
```

#### 4.2 Restore Procedures

```markdown
# Restore Procedures

## Prerequisites

- Fresh server meeting [system requirements](../requirements.md)
- Access to backup files
- Node key backup (if restoring identity)

## Restore Scenarios

### Scenario 1: Same Server, New Disk

\`\`\`bash
# Install Guts
curl -sSL https://get.guts.network | sh

# Restore from backup
guts-node backup restore /backup/guts-latest.tar.gz

# Start node
systemctl start guts-node

# Verify
guts-node status
\`\`\`

### Scenario 2: New Server, Same Identity

\`\`\`bash
# Install Guts
curl -sSL https://get.guts.network | sh

# Restore node key
cp /backup/node.key /etc/guts/node.key
chmod 600 /etc/guts/node.key

# Restore configuration
cp /backup/config.toml /etc/guts/config.toml

# Restore data
guts-node backup restore /backup/guts-latest.tar.gz

# Update DNS/IP as needed
# ...

# Start node
systemctl start guts-node
\`\`\`

### Scenario 3: New Server, New Identity

\`\`\`bash
# Install Guts
curl -sSL https://get.guts.network | sh

# Generate new identity
guts-node keygen > /etc/guts/node.key

# Configure
guts-node config init

# Start fresh (will sync from network)
systemctl start guts-node
\`\`\`

## Recovery Time Objectives

| Scenario | RTO Target | Notes |
|----------|------------|-------|
| Restart after crash | < 5 min | Automatic |
| Restore from snapshot | < 30 min | Depends on data size |
| Full resync | < 4 hours | Depends on network |

## Validation After Restore

\`\`\`bash
# Check sync status
guts-node status

# Verify data integrity
guts-node verify --full

# Check peer connectivity
guts-node peers list

# Verify API functionality
curl http://localhost:8080/api/repos
\`\`\`
```

### Phase 5: Upgrade Procedures

#### 5.1 Zero-Downtime Upgrades

```markdown
# Upgrade Procedures

## Pre-Upgrade Checklist

- [ ] Review release notes for breaking changes
- [ ] Backup current installation
- [ ] Verify new version compatibility
- [ ] Test upgrade in staging environment
- [ ] Schedule maintenance window (for major upgrades)

## Upgrade Methods

### Method 1: Rolling Upgrade (Kubernetes)

\`\`\`bash
# Update Helm values
helm upgrade guts-node guts/guts-node --set image.tag=v1.2.0

# Monitor rollout
kubectl rollout status statefulset/guts-node
\`\`\`

### Method 2: Blue-Green Deployment

\`\`\`bash
# Deploy new version alongside old
docker run -d --name guts-node-new -p 8081:8080 guts/node:v1.2.0

# Verify new version
curl http://localhost:8081/health/ready

# Switch traffic
# (Update load balancer or DNS)

# Stop old version
docker stop guts-node-old
\`\`\`

### Method 3: In-Place Upgrade (Single Node)

\`\`\`bash
# Stop node
systemctl stop guts-node

# Backup current binary
cp /usr/local/bin/guts-node /usr/local/bin/guts-node.bak

# Download new version
curl -sSL https://get.guts.network/v1.2.0 | sh

# Start node
systemctl start guts-node

# Verify
guts-node version
\`\`\`

## Rollback Procedures

If upgrade fails:

\`\`\`bash
# Stop node
systemctl stop guts-node

# Restore previous binary
cp /usr/local/bin/guts-node.bak /usr/local/bin/guts-node

# Restore data if needed
guts-node backup restore /backup/pre-upgrade.tar.gz

# Start node
systemctl start guts-node
\`\`\`

## Version Compatibility

| Upgrade Path | Compatible | Notes |
|--------------|------------|-------|
| 1.0.x → 1.0.y | Yes | Patch upgrades always compatible |
| 1.x.0 → 1.y.0 | Yes | Minor upgrades compatible |
| 1.x.x → 2.0.0 | Check | Major upgrades may require migration |
```

### Phase 6: Cloud Deployment Guides

#### 6.1 AWS Terraform Module

```hcl
# infra/terraform/aws/modules/guts-node/main.tf

variable "instance_type" {
  default = "c6i.2xlarge"
}

variable "volume_size" {
  default = 500
}

variable "vpc_id" {
  type = string
}

variable "subnet_id" {
  type = string
}

resource "aws_security_group" "guts_node" {
  name        = "guts-node"
  description = "Security group for Guts node"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "HTTP API"
  }

  ingress {
    from_port   = 9000
    to_port     = 9000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "P2P TCP"
  }

  ingress {
    from_port   = 9000
    to_port     = 9000
    protocol    = "udp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "P2P QUIC"
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_instance" "guts_node" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = var.instance_type
  subnet_id     = var.subnet_id

  vpc_security_group_ids = [aws_security_group.guts_node.id]

  root_block_device {
    volume_size = var.volume_size
    volume_type = "gp3"
    iops        = 3000
    throughput  = 125
  }

  user_data = <<-EOF
    #!/bin/bash
    curl -sSL https://get.guts.network | sh
    systemctl enable guts-node
    systemctl start guts-node
  EOF

  tags = {
    Name = "guts-node"
  }
}

output "public_ip" {
  value = aws_instance.guts_node.public_ip
}

output "api_endpoint" {
  value = "http://${aws_instance.guts_node.public_ip}:8080"
}
```

## Implementation Plan

### Phase 1: Core Documentation (Week 1-3)
- [ ] Create documentation structure
- [ ] Write quickstart guide
- [ ] Document requirements
- [ ] Write installation guides (Docker, K8s, bare metal)
- [ ] Create configuration reference

### Phase 2: Runbooks (Week 3-5)
- [ ] Create runbook template
- [ ] Write 15+ operational runbooks
- [ ] Link runbooks to alerts
- [ ] Create troubleshooting guides
- [ ] Document diagnostic procedures

### Phase 3: Monitoring (Week 5-7)
- [ ] Create Grafana dashboards
- [ ] Define alert rules
- [ ] Set up PagerDuty/OpsGenie integration
- [ ] Create SLO/SLI documentation
- [ ] Document metrics

### Phase 4: Disaster Recovery (Week 7-8)
- [ ] Document backup procedures
- [ ] Document restore procedures
- [ ] Test and validate procedures
- [ ] Create automation scripts
- [ ] Define RTO/RPO

### Phase 5: Upgrades (Week 8-9)
- [ ] Document upgrade procedures
- [ ] Create rollback procedures
- [ ] Test zero-downtime upgrades
- [ ] Document version compatibility

### Phase 6: Cloud Guides (Week 9-11)
- [ ] Create AWS deployment guide
- [ ] Create GCP deployment guide
- [ ] Create Azure deployment guide
- [ ] Create Terraform modules
- [ ] Create Helm charts

### Phase 7: Validation (Week 11-12)
- [ ] User testing with operators
- [ ] Incorporate feedback
- [ ] Create video tutorials
- [ ] Launch documentation site

## Success Criteria

- [ ] Complete operator documentation covering all scenarios
- [ ] 15+ operational runbooks with step-by-step procedures
- [ ] Pre-built Grafana dashboards for key metrics
- [ ] Alert rules with runbook links
- [ ] Tested backup/restore procedures with documented RTO
- [ ] Zero-downtime upgrade procedure validated
- [ ] Terraform modules for AWS, GCP, Azure
- [ ] Helm charts published to public repository
- [ ] 5+ operators successfully deploy using documentation

## Dependencies

- Documentation hosting (GitHub Pages, Docusaurus)
- Grafana Cloud or self-hosted Grafana
- Alertmanager configuration
- Cloud provider accounts for testing
- Helm chart repository

## References

- [Google SRE Book](https://sre.google/sre-book/table-of-contents/)
- [Kubernetes Documentation Best Practices](https://kubernetes.io/docs/contribute/style/)
- [Terraform Module Best Practices](https://www.terraform.io/language/modules/develop)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
