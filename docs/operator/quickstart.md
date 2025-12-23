# Guts Node Quickstart

> Deploy a Guts node in 5 minutes.

## Prerequisites

Before you begin, ensure you have:

- **Docker 24+** or **Kubernetes 1.28+**
- **Hardware:** 4 CPU cores, 8GB RAM, 100GB SSD (minimum)
- **Network:** Public IP with ports 8080 (HTTP API) and 9000 (P2P)

## Option 1: Docker (Simplest)

### Step 1: Generate Node Identity

Every Guts node needs an Ed25519 keypair for P2P authentication:

```bash
# Generate a new node keypair
docker run --rm ghcr.io/guts-network/guts-node:latest \
  guts-node keygen > node.key

# View your public key (node ID)
cat node.key | head -1
```

> ⚠️ **Security:** Store `node.key` securely. This is your node's identity.

### Step 2: Start the Node

```bash
# Create data directory
mkdir -p guts-data

# Start the node
docker run -d \
  --name guts-node \
  --restart unless-stopped \
  -p 8080:8080 \
  -p 9000:9000 \
  -p 9090:9090 \
  -v $(pwd)/guts-data:/data \
  -v $(pwd)/node.key:/etc/guts/node.key:ro \
  -e GUTS_DATA_DIR=/data \
  -e GUTS_LOG_LEVEL=info \
  -e GUTS_LOG_FORMAT=json \
  ghcr.io/guts-network/guts-node:latest
```

### Step 3: Verify Node Status

```bash
# Check if node is running
docker logs guts-node

# Verify API is responding
curl http://localhost:8080/health/ready

# Expected response:
# {"status":"up","version":"0.1.0","checks":{"storage":{"status":"up"}}}
```

### Step 4: Test Basic Operations

```bash
# Create a repository
curl -X POST http://localhost:8080/api/repos \
  -H "Content-Type: application/json" \
  -d '{"name": "test-repo", "owner": "demo"}'

# List repositories
curl http://localhost:8080/api/repos

# Clone via Git
git clone http://localhost:8080/git/demo/test-repo
```

## Option 2: Docker Compose (Development)

For local development with multiple nodes:

```bash
# Clone the repository
git clone https://github.com/guts-network/guts.git
cd guts/infra/docker

# Start 4-validator devnet with Simplex BFT
docker compose up -d

# Check status
../scripts/devnet-status.sh

# View validator logs
docker logs guts-validator1 -f
```

The devnet exposes:
- Validator 1: http://localhost:8091
- Validator 2: http://localhost:8092
- Validator 3: http://localhost:8093
- Validator 4: http://localhost:8094

## Option 3: Kubernetes (Production)

### Using Helm (Recommended)

```bash
# Add the Guts Helm repository
helm repo add guts https://charts.guts.network
helm repo update

# Install with default settings
helm install guts-node guts/guts-node \
  --namespace guts \
  --create-namespace \
  --set persistence.size=100Gi \
  --set resources.requests.memory=8Gi \
  --set resources.requests.cpu=2

# Check deployment status
kubectl get pods -n guts -l app.kubernetes.io/name=guts-node

# Wait for readiness
kubectl wait --for=condition=ready pod \
  -l app.kubernetes.io/name=guts-node \
  -n guts \
  --timeout=300s
```

### Using Raw Manifests

```bash
# Apply Kubernetes manifests
kubectl apply -f infra/k8s/

# Check status
kubectl get pods -n guts
kubectl logs -n guts guts-node-0
```

## Option 4: Bare Metal

### Step 1: Download Binary

```bash
# Download latest release
curl -sSL https://github.com/guts-network/guts/releases/latest/download/guts-node-linux-amd64 \
  -o /usr/local/bin/guts-node
chmod +x /usr/local/bin/guts-node

# Verify installation
guts-node --version
```

### Step 2: Create Configuration

```bash
# Create config directory
sudo mkdir -p /etc/guts
sudo mkdir -p /var/lib/guts

# Generate node key
guts-node keygen | sudo tee /etc/guts/node.key
sudo chmod 600 /etc/guts/node.key

# Create configuration file
sudo tee /etc/guts/config.yaml << 'EOF'
api:
  addr: "0.0.0.0:8080"

p2p:
  addr: "0.0.0.0:9000"

metrics:
  addr: "0.0.0.0:9090"

storage:
  data_dir: "/var/lib/guts"

logging:
  level: "info"
  format: "json"
EOF
```

### Step 3: Create Systemd Service

```bash
sudo tee /etc/systemd/system/guts-node.service << 'EOF'
[Unit]
Description=Guts Node
After=network.target

[Service]
Type=simple
User=guts
Group=guts
ExecStart=/usr/local/bin/guts-node --config /etc/guts/config.yaml
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

# Create service user
sudo useradd -r -s /bin/false guts
sudo chown -R guts:guts /var/lib/guts /etc/guts

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable guts-node
sudo systemctl start guts-node

# Check status
sudo systemctl status guts-node
sudo journalctl -u guts-node -f
```

## Verify Connectivity

After starting your node, verify it's working correctly:

### Health Checks

```bash
# Liveness (is the process running?)
curl http://localhost:8080/health/live

# Readiness (is the node ready to serve traffic?)
curl http://localhost:8080/health/ready

# Detailed health status
curl http://localhost:8080/health | jq
```

### Metrics

```bash
# View Prometheus metrics
curl http://localhost:9090/metrics | head -50

# Key metrics to check:
# - guts_http_requests_total
# - guts_p2p_peers_connected
# - guts_storage_objects_total
```

### P2P Connectivity

```bash
# Check peer connections (via API)
curl http://localhost:8080/api/consensus/validators

# Check consensus status
curl http://localhost:8080/api/consensus/status | jq
```

## Common Issues

### Node Won't Start

```bash
# Check logs
docker logs guts-node 2>&1 | tail -50

# Common causes:
# - Port already in use: Change port mapping
# - Permission denied: Check file permissions
# - Invalid key: Regenerate node.key
```

### Can't Connect to Peers

```bash
# Check firewall rules
sudo ufw status

# Required ports:
# - 8080/tcp (HTTP API)
# - 9000/tcp (P2P TCP)
# - 9000/udp (P2P QUIC)

# Open ports if needed
sudo ufw allow 8080/tcp
sudo ufw allow 9000/tcp
sudo ufw allow 9000/udp
```

### API Returns Errors

```bash
# Check if node is synced
curl http://localhost:8080/api/consensus/status | jq '.sync_status'

# If "syncing", wait for sync to complete
# If "stalled", see runbook: docs/operator/runbooks/node-not-syncing.md
```

## Next Steps

Now that your node is running:

1. **Configure Networking:** [Networking Guide](configuration/networking.md)
2. **Set Up Monitoring:** [Monitoring Guide](operations/monitoring.md)
3. **Configure Backups:** [Backup Guide](operations/backup.md)
4. **Join the Network:** [Network Guide](../guides/joining-network.md)
5. **Security Hardening:** [Security Guide](configuration/security.md)

## Getting Help

- **Documentation:** Browse the full [Operator Guide](README.md)
- **Runbooks:** Check [Runbooks](runbooks/) for common issues
- **Community:** Join our [Discord](https://discord.gg/guts)
- **Issues:** Report bugs on [GitHub](https://github.com/guts-network/guts/issues)
