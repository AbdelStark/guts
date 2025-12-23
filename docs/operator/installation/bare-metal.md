# Bare Metal Deployment Guide

> Deploy Guts nodes directly on physical or virtual machines.

## Prerequisites

- Linux server (Ubuntu 22.04+, Debian 12+, RHEL 9+)
- Root or sudo access
- 4+ CPU cores, 8+ GB RAM, 100+ GB SSD
- Public IP with ports 8080, 9000 accessible

## Quick Installation

### One-Line Install

```bash
curl -sSL https://get.guts.network | sudo sh
```

This script:
1. Detects your OS and architecture
2. Downloads the latest guts-node binary
3. Creates system user and directories
4. Installs systemd service
5. Starts the node

### Manual Installation

#### Step 1: Download Binary

```bash
# Set version
VERSION="latest"  # or specific version like "v1.0.0"

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
  x86_64) ARCH="amd64" ;;
  aarch64) ARCH="arm64" ;;
esac

# Download
curl -sSL "https://github.com/guts-network/guts/releases/${VERSION}/download/guts-node-linux-${ARCH}" \
  -o /tmp/guts-node

# Install
sudo install -m 755 /tmp/guts-node /usr/local/bin/guts-node

# Verify
guts-node --version
```

#### Step 2: Create User and Directories

```bash
# Create system user
sudo useradd -r -s /sbin/nologin -d /var/lib/guts guts

# Create directories
sudo mkdir -p /etc/guts
sudo mkdir -p /var/lib/guts
sudo mkdir -p /var/log/guts

# Set ownership
sudo chown -R guts:guts /var/lib/guts /var/log/guts
sudo chmod 700 /var/lib/guts
```

#### Step 3: Generate Node Key

```bash
# Generate Ed25519 keypair
sudo guts-node keygen | sudo tee /etc/guts/node.key > /dev/null
sudo chmod 600 /etc/guts/node.key
sudo chown guts:guts /etc/guts/node.key

# View public key (node ID)
sudo head -1 /etc/guts/node.key
```

#### Step 4: Create Configuration

```bash
sudo tee /etc/guts/config.yaml << 'EOF'
# Guts Node Configuration
# See: https://docs.guts.network/operator/configuration/reference

# HTTP API
api:
  addr: "0.0.0.0:8080"
  request_timeout_secs: 30

# P2P Networking
p2p:
  addr: "0.0.0.0:9000"
  bootstrap_nodes: []
  # bootstrap_nodes:
  #   - "/ip4/1.2.3.4/tcp/9000/p2p/12D3KooW..."

# Prometheus Metrics
metrics:
  addr: "127.0.0.1:9090"

# Storage
storage:
  data_dir: "/var/lib/guts"
  # backend: "rocksdb"  # or "memory"

# Logging
logging:
  level: "info"
  format: "json"

# Consensus (optional, for validators)
# consensus:
#   enabled: true
#   use_simplex_bft: true
#   block_time_ms: 2000
#   genesis_file: "/etc/guts/genesis.json"
EOF

sudo chown guts:guts /etc/guts/config.yaml
sudo chmod 640 /etc/guts/config.yaml
```

#### Step 5: Create Systemd Service

```bash
sudo tee /etc/systemd/system/guts-node.service << 'EOF'
[Unit]
Description=Guts Node - Decentralized Code Collaboration
Documentation=https://docs.guts.network
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=guts
Group=guts
ExecStart=/usr/local/bin/guts-node --config /etc/guts/config.yaml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
TimeoutStartSec=60
TimeoutStopSec=60

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictSUIDSGID=yes
RestrictNamespaces=yes
LockPersonality=yes

# Allow write to data directory
ReadWritePaths=/var/lib/guts /var/log/guts

# Resource limits
LimitNOFILE=65535
LimitNPROC=32768
MemoryMax=32G
TasksMax=4096

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=guts-node

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
```

#### Step 6: Start Service

```bash
# Enable and start
sudo systemctl enable guts-node
sudo systemctl start guts-node

# Check status
sudo systemctl status guts-node

# View logs
sudo journalctl -u guts-node -f
```

## System Tuning

### Kernel Parameters

```bash
sudo tee /etc/sysctl.d/99-guts.conf << 'EOF'
# Network performance
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 5
net.ipv4.tcp_keepalive_intvl = 15

# Memory
vm.swappiness = 10
vm.dirty_ratio = 60
vm.dirty_background_ratio = 2
vm.overcommit_memory = 1

# File system
fs.file-max = 2097152
fs.inotify.max_user_watches = 524288
fs.inotify.max_user_instances = 8192
EOF

sudo sysctl -p /etc/sysctl.d/99-guts.conf
```

### File Descriptor Limits

```bash
sudo tee /etc/security/limits.d/guts.conf << 'EOF'
guts soft nofile 65535
guts hard nofile 65535
guts soft nproc 32768
guts hard nproc 32768
guts soft memlock unlimited
guts hard memlock unlimited
EOF
```

### Storage Optimization

For NVMe drives:

```bash
# Disable access time updates (add to /etc/fstab)
# /dev/nvme0n1p1 /var/lib/guts ext4 defaults,noatime,discard 0 2

# Use none scheduler for NVMe
echo "none" | sudo tee /sys/block/nvme0n1/queue/scheduler

# Increase read-ahead
echo 256 | sudo tee /sys/block/nvme0n1/queue/read_ahead_kb
```

## Firewall Configuration

### UFW (Ubuntu/Debian)

```bash
# Enable firewall
sudo ufw enable

# Allow SSH
sudo ufw allow 22/tcp

# Allow Guts API
sudo ufw allow 8080/tcp comment "Guts HTTP API"

# Allow P2P
sudo ufw allow 9000/tcp comment "Guts P2P TCP"
sudo ufw allow 9000/udp comment "Guts P2P QUIC"

# Allow metrics (internal only)
sudo ufw allow from 10.0.0.0/8 to any port 9090 comment "Guts Metrics"

# Verify
sudo ufw status verbose
```

### firewalld (RHEL/Rocky/Alma)

```bash
# Create service definition
sudo tee /etc/firewalld/services/guts.xml << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<service>
  <short>Guts Node</short>
  <description>Guts decentralized code collaboration</description>
  <port protocol="tcp" port="8080"/>
  <port protocol="tcp" port="9000"/>
  <port protocol="udp" port="9000"/>
</service>
EOF

# Reload and enable
sudo firewall-cmd --reload
sudo firewall-cmd --permanent --add-service=guts
sudo firewall-cmd --reload
```

## TLS Configuration

### Using Let's Encrypt with Caddy

```bash
# Install Caddy
sudo apt install caddy

# Configure reverse proxy
sudo tee /etc/caddy/Caddyfile << 'EOF'
guts.example.com {
    reverse_proxy localhost:8080

    # Enable compression
    encode gzip

    # Security headers
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
    }
}
EOF

sudo systemctl enable caddy
sudo systemctl start caddy
```

### Using nginx

```bash
# Install nginx
sudo apt install nginx

# Configure
sudo tee /etc/nginx/sites-available/guts << 'EOF'
server {
    listen 443 ssl http2;
    server_name guts.example.com;

    ssl_certificate /etc/letsencrypt/live/guts.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/guts.example.com/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    client_max_body_size 100M;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/guts /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## Log Management

### journald Configuration

```bash
sudo tee /etc/systemd/journald.conf.d/guts.conf << 'EOF'
[Journal]
Storage=persistent
Compress=yes
SystemMaxUse=10G
MaxRetentionSec=30day
EOF

sudo systemctl restart systemd-journald
```

### Log Rotation (if using file logging)

```bash
sudo tee /etc/logrotate.d/guts << 'EOF'
/var/log/guts/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0640 guts guts
    postrotate
        systemctl reload guts-node > /dev/null 2>&1 || true
    endscript
}
EOF
```

## Monitoring Setup

### Install Prometheus Node Exporter

```bash
# Download
curl -sSL https://github.com/prometheus/node_exporter/releases/download/v1.6.1/node_exporter-1.6.1.linux-amd64.tar.gz | tar xz
sudo mv node_exporter-1.6.1.linux-amd64/node_exporter /usr/local/bin/

# Create service
sudo tee /etc/systemd/system/node_exporter.service << 'EOF'
[Unit]
Description=Prometheus Node Exporter
After=network.target

[Service]
Type=simple
User=nobody
ExecStart=/usr/local/bin/node_exporter
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable node_exporter
sudo systemctl start node_exporter
```

## Backup Script

```bash
sudo tee /usr/local/bin/guts-backup.sh << 'EOF'
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/var/backups/guts"
RETENTION_DAYS=7
DATE=$(date +%Y%m%d-%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Create backup
echo "Creating backup..."
tar -czf "$BACKUP_DIR/guts-$DATE.tar.gz" \
    -C /var/lib/guts . \
    --exclude='*.lock'

# Copy config
cp /etc/guts/config.yaml "$BACKUP_DIR/config-$DATE.yaml"

# Remove old backups
find "$BACKUP_DIR" -name "guts-*.tar.gz" -mtime +$RETENTION_DAYS -delete
find "$BACKUP_DIR" -name "config-*.yaml" -mtime +$RETENTION_DAYS -delete

echo "Backup complete: $BACKUP_DIR/guts-$DATE.tar.gz"
EOF

sudo chmod +x /usr/local/bin/guts-backup.sh

# Schedule daily backups
echo "0 2 * * * root /usr/local/bin/guts-backup.sh >> /var/log/guts-backup.log 2>&1" | sudo tee /etc/cron.d/guts-backup
```

## Upgrade Procedure

```bash
# Download new version
VERSION="v1.1.0"
curl -sSL "https://github.com/guts-network/guts/releases/download/${VERSION}/guts-node-linux-amd64" \
  -o /tmp/guts-node-new

# Verify binary
/tmp/guts-node-new --version

# Create backup
sudo /usr/local/bin/guts-backup.sh

# Stop service
sudo systemctl stop guts-node

# Replace binary
sudo mv /usr/local/bin/guts-node /usr/local/bin/guts-node.bak
sudo install -m 755 /tmp/guts-node-new /usr/local/bin/guts-node

# Start service
sudo systemctl start guts-node

# Verify
sudo systemctl status guts-node
curl http://localhost:8080/health/ready

# If failed, rollback
# sudo systemctl stop guts-node
# sudo mv /usr/local/bin/guts-node.bak /usr/local/bin/guts-node
# sudo systemctl start guts-node
```

## Health Checks

```bash
# Check service status
sudo systemctl status guts-node

# Check API health
curl -s http://localhost:8080/health | jq

# Check metrics
curl -s http://localhost:9090/metrics | head -20

# Check logs for errors
sudo journalctl -u guts-node --since "1 hour ago" | grep -i error
```

## Troubleshooting

### Service Won't Start

```bash
# Check configuration
guts-node --config /etc/guts/config.yaml --dry-run

# Check permissions
ls -la /var/lib/guts /etc/guts

# Check logs
sudo journalctl -u guts-node -b --no-pager
```

### High Resource Usage

```bash
# Check CPU/Memory
top -p $(pgrep guts-node)

# Check file descriptors
ls /proc/$(pgrep guts-node)/fd | wc -l

# Check network connections
ss -tlnp | grep guts
```

### Network Issues

```bash
# Test API
curl -v http://localhost:8080/health

# Test P2P port
nc -zv localhost 9000

# Check firewall
sudo ufw status
```

## Next Steps

- [Configure networking](../configuration/networking.md)
- [Set up monitoring](../operations/monitoring.md)
- [Configure backups](../operations/backup.md)
