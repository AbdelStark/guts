# System Requirements

> Hardware, software, and network requirements for running Guts nodes.

## Hardware Requirements

### Minimum (Development/Testing)

Suitable for local development, CI/CD pipelines, and testing environments.

| Component | Minimum | Notes |
|-----------|---------|-------|
| **CPU** | 2 cores | x86_64 or ARM64 |
| **RAM** | 4 GB | |
| **Storage** | 50 GB SSD | NVMe preferred |
| **Network** | 10 Mbps | Stable connection |

### Recommended (Production Full Node)

Suitable for production full nodes serving API traffic and participating in P2P replication.

| Component | Recommended | Notes |
|-----------|-------------|-------|
| **CPU** | 8 cores | Dedicated, not shared/burstable |
| **RAM** | 32 GB | ECC preferred for data integrity |
| **Storage** | 500 GB NVMe | RAID-1 for reliability |
| **Network** | 1 Gbps | Low latency preferred (<50ms to peers) |

### Validator Requirements

Validators participate in consensus and must meet higher requirements for network reliability.

| Component | Required | Notes |
|-----------|----------|-------|
| **CPU** | 16+ cores | High single-thread performance (3.5GHz+) |
| **RAM** | 64 GB | ECC required |
| **Storage** | 2 TB NVMe | RAID-1 required, 3+ DWPD endurance |
| **Network** | 1 Gbps symmetric | 99.9% uptime required |
| **UPS** | Yes | Graceful shutdown capability |
| **Redundancy** | Recommended | Redundant power, network paths |

### Storage Sizing Guide

Storage requirements depend on the number and size of repositories:

| Workload | Repositories | Avg Size | Storage Needed |
|----------|--------------|----------|----------------|
| Small | < 100 | 50 MB | 50 GB |
| Medium | 100-1,000 | 100 MB | 200 GB |
| Large | 1,000-10,000 | 200 MB | 1 TB |
| Enterprise | 10,000+ | 500 MB | 5+ TB |

**Formula:** `Storage = (Repo Count × Avg Size × 1.5) + 50GB overhead`

The 1.5 multiplier accounts for:
- Git pack files and loose objects
- Collaboration data (PRs, issues, comments)
- Consensus state and logs

## Software Requirements

### Operating System

| OS | Version | Status |
|----|---------|--------|
| Ubuntu | 22.04 LTS, 24.04 LTS | ✅ Recommended |
| Debian | 12 (Bookworm) | ✅ Supported |
| RHEL/Rocky/Alma | 9.x | ✅ Supported |
| Amazon Linux | 2023 | ✅ Supported |
| macOS | 13+ (Ventura) | ⚠️ Development only |
| Windows | WSL2 | ⚠️ Development only |

**Kernel Requirements:**
- Linux kernel 5.10+ (for io_uring support)
- `CONFIG_CGROUPS` enabled (for container deployments)

### Container Runtime

| Runtime | Version | Notes |
|---------|---------|-------|
| Docker | 24.0+ | Recommended for single-node |
| containerd | 1.7+ | Kubernetes default |
| Podman | 4.0+ | Alternative to Docker |

### Kubernetes

| Component | Version | Notes |
|-----------|---------|-------|
| Kubernetes | 1.28+ | Any conformant distribution |
| Helm | 3.12+ | For Helm chart deployment |
| kubectl | 1.28+ | Match cluster version |

### Dependencies (Bare Metal)

For bare metal installations, these system libraries are required:

```bash
# Ubuntu/Debian
apt-get install -y \
  libssl3 \
  ca-certificates \
  curl \
  jq

# RHEL/Rocky/Alma
dnf install -y \
  openssl-libs \
  ca-certificates \
  curl \
  jq
```

## Network Requirements

### Port Requirements

| Port | Protocol | Direction | Purpose | Required |
|------|----------|-----------|---------|----------|
| 8080 | TCP | Inbound | HTTP API | Yes |
| 9000 | TCP | Inbound | P2P (TCP) | Yes |
| 9000 | UDP | Inbound | P2P (QUIC) | Yes |
| 9090 | TCP | Inbound | Metrics (Prometheus) | Recommended |
| 443 | TCP | Inbound | HTTPS (via proxy) | Production |

### Firewall Configuration

#### UFW (Ubuntu/Debian)

```bash
# Allow API access
sudo ufw allow 8080/tcp comment "Guts HTTP API"

# Allow P2P
sudo ufw allow 9000/tcp comment "Guts P2P TCP"
sudo ufw allow 9000/udp comment "Guts P2P QUIC"

# Metrics (internal only)
sudo ufw allow from 10.0.0.0/8 to any port 9090 comment "Guts Metrics"

# Apply
sudo ufw enable
```

#### firewalld (RHEL/Rocky)

```bash
# Create service definition
sudo firewall-cmd --permanent --new-service=guts-node
sudo firewall-cmd --permanent --service=guts-node --add-port=8080/tcp
sudo firewall-cmd --permanent --service=guts-node --add-port=9000/tcp
sudo firewall-cmd --permanent --service=guts-node --add-port=9000/udp

# Enable service
sudo firewall-cmd --permanent --add-service=guts-node
sudo firewall-cmd --reload
```

#### iptables

```bash
# API
iptables -A INPUT -p tcp --dport 8080 -j ACCEPT

# P2P
iptables -A INPUT -p tcp --dport 9000 -j ACCEPT
iptables -A INPUT -p udp --dport 9000 -j ACCEPT

# Metrics (internal only)
iptables -A INPUT -p tcp --dport 9090 -s 10.0.0.0/8 -j ACCEPT
```

### AWS Security Group

```hcl
resource "aws_security_group" "guts_node" {
  name        = "guts-node"
  description = "Security group for Guts node"

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
```

### Network Latency Requirements

| Node Type | Max Latency to Peers | Notes |
|-----------|---------------------|-------|
| Full Node | < 500ms | Higher latency affects sync |
| Validator | < 100ms | Critical for consensus |

### Bandwidth Requirements

| Activity | Bandwidth | Notes |
|----------|-----------|-------|
| Idle | < 1 Mbps | Heartbeats, peer discovery |
| Light Usage | 10-50 Mbps | Normal operations |
| Heavy Sync | 100-500 Mbps | Initial sync, large repos |
| Peak (Validator) | 500+ Mbps | Block propagation |

## Cloud Instance Recommendations

### AWS

| Use Case | Instance Type | vCPUs | RAM | Storage |
|----------|---------------|-------|-----|---------|
| Development | t3.medium | 2 | 4 GB | 50 GB gp3 |
| Production | c6i.2xlarge | 8 | 16 GB | 500 GB gp3 |
| Validator | c6i.4xlarge | 16 | 32 GB | 2 TB io2 |

### GCP

| Use Case | Machine Type | vCPUs | RAM | Storage |
|----------|--------------|-------|-----|---------|
| Development | e2-medium | 2 | 4 GB | 50 GB pd-ssd |
| Production | c2-standard-8 | 8 | 32 GB | 500 GB pd-ssd |
| Validator | c2-standard-16 | 16 | 64 GB | 2 TB pd-extreme |

### Azure

| Use Case | VM Size | vCPUs | RAM | Storage |
|----------|---------|-------|-----|---------|
| Development | Standard_B2s | 2 | 4 GB | 50 GB Premium SSD |
| Production | Standard_F8s_v2 | 8 | 16 GB | 500 GB Premium SSD |
| Validator | Standard_F16s_v2 | 16 | 32 GB | 2 TB Ultra Disk |

## Performance Tuning

### System Limits

For production deployments, increase system limits:

```bash
# /etc/security/limits.d/guts.conf
guts soft nofile 65535
guts hard nofile 65535
guts soft nproc 32768
guts hard nproc 32768

# /etc/sysctl.d/99-guts.conf
# Network tuning
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535

# Memory
vm.swappiness = 10
vm.dirty_ratio = 60
vm.dirty_background_ratio = 2

# File system
fs.file-max = 2097152
fs.inotify.max_user_watches = 524288

# Apply without reboot
sudo sysctl -p /etc/sysctl.d/99-guts.conf
```

### Storage Optimization

For NVMe storage with high write workloads:

```bash
# Disable access time updates
# Add 'noatime' to /etc/fstab

# Use deadline scheduler for NVMe
echo "none" > /sys/block/nvme0n1/queue/scheduler

# Increase read-ahead for sequential workloads
echo 256 > /sys/block/nvme0n1/queue/read_ahead_kb
```

## Monitoring Readiness

Before going to production, ensure you can monitor:

- [ ] CPU, memory, disk, network metrics
- [ ] Guts-specific metrics (via `/metrics` endpoint)
- [ ] Log aggregation configured
- [ ] Alerting rules defined

See [Monitoring Guide](operations/monitoring.md) for setup instructions.

## Checklist

### Development Environment

- [ ] 2+ CPU cores available
- [ ] 4+ GB RAM free
- [ ] 50+ GB disk space
- [ ] Docker or Kubernetes installed
- [ ] Ports 8080, 9000 available

### Production Environment

- [ ] Meets recommended hardware specifications
- [ ] Operating system updated and hardened
- [ ] Firewall configured correctly
- [ ] Network latency to peers acceptable
- [ ] Monitoring infrastructure ready
- [ ] Backup strategy defined
- [ ] On-call procedures documented

### Validator Environment

- [ ] Meets validator hardware requirements
- [ ] 99.9%+ network uptime achievable
- [ ] UPS installed and tested
- [ ] Secure key management in place
- [ ] 24/7 monitoring configured
- [ ] Incident response team identified
