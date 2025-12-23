# Docker Deployment Guide

> Deploy Guts nodes using Docker and Docker Compose.

## Prerequisites

- Docker 24.0+ installed
- Docker Compose v2.20+ (for multi-node setups)
- 4GB RAM minimum (8GB recommended)
- 50GB disk space minimum

## Single Node Deployment

### Pull the Image

```bash
# Pull latest stable release
docker pull ghcr.io/guts-network/guts-node:latest

# Or pull a specific version
docker pull ghcr.io/guts-network/guts-node:v1.0.0
```

### Generate Node Identity

```bash
# Generate Ed25519 keypair
docker run --rm ghcr.io/guts-network/guts-node:latest \
  guts-node keygen > node.key

# View public key (your node ID)
head -1 node.key
```

### Start the Node

```bash
# Create data directory
mkdir -p guts-data

# Run the node
docker run -d \
  --name guts-node \
  --restart unless-stopped \
  -p 8080:8080 \
  -p 9000:9000 \
  -p 9090:9090 \
  -v $(pwd)/guts-data:/data \
  -v $(pwd)/node.key:/etc/guts/node.key:ro \
  -e GUTS_DATA_DIR=/data \
  -e GUTS_API_ADDR=0.0.0.0:8080 \
  -e GUTS_P2P_ADDR=0.0.0.0:9000 \
  -e GUTS_METRICS_ADDR=0.0.0.0:9090 \
  -e GUTS_LOG_LEVEL=info \
  -e GUTS_LOG_FORMAT=json \
  ghcr.io/guts-network/guts-node:latest
```

### Verify Deployment

```bash
# Check container status
docker ps -f name=guts-node

# View logs
docker logs guts-node -f

# Test health endpoint
curl http://localhost:8080/health/ready
```

## Docker Compose Deployment

### Single Node with Monitoring

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  guts-node:
    image: ghcr.io/guts-network/guts-node:latest
    container_name: guts-node
    restart: unless-stopped
    ports:
      - "8080:8080"   # HTTP API
      - "9000:9000"   # P2P
      - "9090:9090"   # Metrics
    volumes:
      - guts-data:/data
      - ./node.key:/etc/guts/node.key:ro
    environment:
      GUTS_DATA_DIR: /data
      GUTS_API_ADDR: 0.0.0.0:8080
      GUTS_P2P_ADDR: 0.0.0.0:9000
      GUTS_METRICS_ADDR: 0.0.0.0:9090
      GUTS_LOG_LEVEL: info
      GUTS_LOG_FORMAT: json
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health/ready"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    logging:
      driver: json-file
      options:
        max-size: "100m"
        max-file: "5"

  prometheus:
    image: prom/prometheus:v2.47.0
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'

  grafana:
    image: grafana/grafana:10.1.0
    container_name: grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./dashboards:/etc/grafana/provisioning/dashboards:ro
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
      GF_USERS_ALLOW_SIGN_UP: "false"

volumes:
  guts-data:
  prometheus-data:
  grafana-data:
```

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'guts-node'
    static_configs:
      - targets: ['guts-node:9090']
    metrics_path: /metrics
```

Start the stack:

```bash
docker compose up -d
```

### Multi-Node Devnet (4 Validators)

For a local development network with Simplex BFT consensus:

```bash
# Clone repository
git clone https://github.com/guts-network/guts.git
cd guts/infra/docker

# Start 4-validator network
docker compose up -d

# Check status
../scripts/devnet-status.sh
```

Access points:
- Validator 1: http://localhost:8091
- Validator 2: http://localhost:8092
- Validator 3: http://localhost:8093
- Validator 4: http://localhost:8094

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `GUTS_API_ADDR` | `127.0.0.1:8080` | HTTP API listen address |
| `GUTS_P2P_ADDR` | `0.0.0.0:9000` | P2P listen address |
| `GUTS_METRICS_ADDR` | `0.0.0.0:9090` | Prometheus metrics address |
| `GUTS_DATA_DIR` | `./data` | Data directory |
| `GUTS_LOG_LEVEL` | `info` | Log level (trace/debug/info/warn/error) |
| `GUTS_LOG_FORMAT` | `json` | Log format (json/pretty) |
| `GUTS_PRIVATE_KEY` | - | Node private key (hex) |
| `GUTS_CONSENSUS_ENABLED` | `false` | Enable consensus engine |
| `GUTS_CONSENSUS_USE_SIMPLEX_BFT` | `false` | Use Simplex BFT |
| `GUTS_CONSENSUS_BLOCK_TIME_MS` | `2000` | Block time in milliseconds |

## Resource Limits

For production, set resource limits:

```yaml
services:
  guts-node:
    # ...
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G
        reservations:
          cpus: '2'
          memory: 4G
```

## Persistent Storage

### Using Named Volumes (Recommended)

```yaml
volumes:
  guts-data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/guts-data
```

### Using Host Mounts

```yaml
services:
  guts-node:
    volumes:
      - /var/lib/guts:/data
```

## Networking

### Bridge Network (Default)

```yaml
networks:
  default:
    driver: bridge
    ipam:
      config:
        - subnet: 172.30.0.0/16
```

### Host Network (Performance)

```yaml
services:
  guts-node:
    network_mode: host
    environment:
      GUTS_API_ADDR: 0.0.0.0:8080
      GUTS_P2P_ADDR: 0.0.0.0:9000
```

## Security Hardening

### Run as Non-Root

The image runs as the `guts` user (UID 1000) by default:

```yaml
services:
  guts-node:
    user: "1000:1000"
```

### Read-Only Root Filesystem

```yaml
services:
  guts-node:
    read_only: true
    tmpfs:
      - /tmp
    volumes:
      - guts-data:/data
```

### Security Options

```yaml
services:
  guts-node:
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
```

## Backup and Restore

### Backup

```bash
# Stop the node
docker compose stop guts-node

# Create backup
docker run --rm \
  -v guts-data:/data:ro \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/guts-$(date +%Y%m%d).tar.gz -C /data .

# Restart
docker compose start guts-node
```

### Restore

```bash
# Stop the node
docker compose stop guts-node

# Restore from backup
docker run --rm \
  -v guts-data:/data \
  -v $(pwd)/backups:/backup \
  alpine sh -c "rm -rf /data/* && tar xzf /backup/guts-20240101.tar.gz -C /data"

# Restart
docker compose start guts-node
```

## Upgrades

### Rolling Upgrade

```bash
# Pull new image
docker compose pull guts-node

# Recreate container
docker compose up -d guts-node

# Verify
docker logs guts-node | head -20
curl http://localhost:8080/health/ready
```

### Rollback

```bash
# Stop current version
docker compose stop guts-node

# Run previous version
docker run -d \
  --name guts-node-old \
  -p 8080:8080 \
  ghcr.io/guts-network/guts-node:v0.9.0 \
  # ... same config
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs guts-node 2>&1 | tail -100

# Common issues:
# - Port already in use
# - Permission denied on volumes
# - Invalid environment variables
```

### High Memory Usage

```bash
# Check memory
docker stats guts-node

# Set memory limit
docker update --memory=8g guts-node
```

### Network Issues

```bash
# Check network
docker network inspect bridge

# Test connectivity
docker exec guts-node curl -v http://localhost:8080/health
```

## Next Steps

- [Configure networking](../configuration/networking.md)
- [Set up monitoring](../operations/monitoring.md)
- [Configure backups](../operations/backup.md)
