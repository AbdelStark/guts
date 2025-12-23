# Monitoring Guide

> Set up comprehensive monitoring for Guts nodes.

## Overview

Guts nodes expose Prometheus metrics that can be scraped and visualized with Grafana. This guide covers setting up a complete monitoring stack.

## Metrics Endpoint

By default, metrics are exposed at `http://localhost:9090/metrics`:

```bash
# View metrics
curl http://localhost:9090/metrics

# Check specific metric
curl -s http://localhost:9090/metrics | grep guts_http_requests_total
```

## Available Metrics

### HTTP Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `guts_http_requests_total` | Counter | Total HTTP requests by method, path, status |
| `guts_http_request_duration_seconds` | Histogram | Request latency distribution |
| `guts_http_requests_in_flight` | Gauge | Currently processing requests |
| `guts_http_request_size_bytes` | Histogram | Request body size |
| `guts_http_response_size_bytes` | Histogram | Response body size |

### P2P Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `guts_p2p_peers_connected` | Gauge | Number of connected peers |
| `guts_p2p_messages_sent_total` | Counter | Messages sent by type |
| `guts_p2p_messages_received_total` | Counter | Messages received by type |
| `guts_p2p_message_latency_seconds` | Histogram | P2P message latency |
| `guts_p2p_bytes_sent_total` | Counter | Bytes sent |
| `guts_p2p_bytes_received_total` | Counter | Bytes received |

### Storage Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `guts_storage_objects_total` | Gauge | Objects by type (blob, tree, commit) |
| `guts_storage_size_bytes` | Gauge | Total storage size |
| `guts_storage_operation_duration_seconds` | Histogram | Storage operation latency |
| `guts_storage_cache_hits_total` | Counter | Cache hits |
| `guts_storage_cache_misses_total` | Counter | Cache misses |

### Consensus Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `guts_consensus_block_height` | Gauge | Current block height |
| `guts_consensus_validators_total` | Gauge | Number of validators |
| `guts_consensus_commits_total` | Counter | Total committed blocks |
| `guts_consensus_proposals_total` | Counter | Blocks proposed |
| `guts_consensus_round` | Gauge | Current consensus round |
| `guts_consensus_mempool_size` | Gauge | Pending transactions |

### Business Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `guts_repositories_total` | Gauge | Total repositories |
| `guts_pull_requests_total` | Gauge | PRs by state |
| `guts_issues_total` | Gauge | Issues by state |
| `guts_organizations_total` | Gauge | Organizations |
| `guts_users_total` | Gauge | Registered users |

## Prometheus Setup

### Configuration

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - alertmanager:9093

rule_files:
  - /etc/prometheus/alerts/*.yml

scrape_configs:
  # Guts nodes
  - job_name: 'guts-node'
    static_configs:
      - targets:
          - 'guts-node-1:9090'
          - 'guts-node-2:9090'
          - 'guts-node-3:9090'
    metrics_path: /metrics
    scheme: http

  # Node Exporter (system metrics)
  - job_name: 'node-exporter'
    static_configs:
      - targets:
          - 'node-exporter:9100'

  # Prometheus self-monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
```

### Docker Compose

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:v2.47.0
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - ./alerts:/etc/prometheus/alerts:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'
      - '--web.enable-admin-api'

  alertmanager:
    image: prom/alertmanager:v0.26.0
    container_name: alertmanager
    restart: unless-stopped
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml:ro
      - alertmanager-data:/alertmanager

  grafana:
    image: grafana/grafana:10.1.0
    container_name: grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning:ro
      - ./grafana/dashboards:/var/lib/grafana/dashboards:ro
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_PASSWORD:-admin}
      GF_USERS_ALLOW_SIGN_UP: "false"
      GF_SERVER_ROOT_URL: ${GRAFANA_URL:-http://localhost:3000}

volumes:
  prometheus-data:
  alertmanager-data:
  grafana-data:
```

### Kubernetes (Prometheus Operator)

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: guts-node
  namespace: monitoring
  labels:
    app: guts-node
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: guts-node
  namespaceSelector:
    matchNames:
      - guts
  endpoints:
    - port: metrics
      interval: 15s
      path: /metrics
      scheme: http
```

## Grafana Dashboards

### Dashboard: Node Overview

Import this dashboard for a quick node health overview:

```json
{
  "dashboard": {
    "title": "Guts Node Overview",
    "uid": "guts-overview",
    "panels": [
      {
        "title": "Node Health",
        "type": "stat",
        "gridPos": {"x": 0, "y": 0, "w": 4, "h": 4},
        "targets": [{"expr": "up{job='guts-node'}"}],
        "options": {
          "colorMode": "background",
          "thresholds": {
            "steps": [
              {"color": "red", "value": 0},
              {"color": "green", "value": 1}
            ]
          }
        }
      },
      {
        "title": "Block Height",
        "type": "stat",
        "gridPos": {"x": 4, "y": 0, "w": 4, "h": 4},
        "targets": [{"expr": "guts_consensus_block_height"}]
      },
      {
        "title": "Connected Peers",
        "type": "stat",
        "gridPos": {"x": 8, "y": 0, "w": 4, "h": 4},
        "targets": [{"expr": "guts_p2p_peers_connected"}]
      },
      {
        "title": "Request Rate",
        "type": "graph",
        "gridPos": {"x": 0, "y": 4, "w": 12, "h": 8},
        "targets": [
          {"expr": "rate(guts_http_requests_total[5m])", "legendFormat": "{{method}} {{path}}"}
        ]
      },
      {
        "title": "Request Latency (p99)",
        "type": "graph",
        "gridPos": {"x": 12, "y": 4, "w": 12, "h": 8},
        "targets": [
          {"expr": "histogram_quantile(0.99, rate(guts_http_request_duration_seconds_bucket[5m]))", "legendFormat": "p99"}
        ]
      }
    ]
  }
}
```

### Dashboard: Consensus Health

For validator networks:

```json
{
  "dashboard": {
    "title": "Guts Consensus",
    "uid": "guts-consensus",
    "panels": [
      {
        "title": "Block Height by Validator",
        "type": "graph",
        "targets": [
          {"expr": "guts_consensus_block_height", "legendFormat": "{{instance}}"}
        ]
      },
      {
        "title": "Blocks Committed",
        "type": "graph",
        "targets": [
          {"expr": "rate(guts_consensus_commits_total[5m])", "legendFormat": "{{instance}}"}
        ]
      },
      {
        "title": "Mempool Size",
        "type": "graph",
        "targets": [
          {"expr": "guts_consensus_mempool_size", "legendFormat": "{{instance}}"}
        ]
      }
    ]
  }
}
```

## Key Queries

### Request Performance

```promql
# Request rate by endpoint
rate(guts_http_requests_total[5m])

# Error rate
sum(rate(guts_http_requests_total{status=~"5.."}[5m])) / sum(rate(guts_http_requests_total[5m]))

# p50, p95, p99 latency
histogram_quantile(0.50, rate(guts_http_request_duration_seconds_bucket[5m]))
histogram_quantile(0.95, rate(guts_http_request_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(guts_http_request_duration_seconds_bucket[5m]))
```

### P2P Health

```promql
# Peer count
guts_p2p_peers_connected

# Message throughput
rate(guts_p2p_messages_sent_total[5m])
rate(guts_p2p_messages_received_total[5m])

# Bandwidth
rate(guts_p2p_bytes_sent_total[5m])
rate(guts_p2p_bytes_received_total[5m])
```

### Consensus Health

```promql
# Block production rate
rate(guts_consensus_commits_total[5m])

# Block height across validators (should be same)
guts_consensus_block_height

# Mempool backlog
guts_consensus_mempool_size
```

### Storage Health

```promql
# Cache hit ratio
rate(guts_storage_cache_hits_total[5m]) / (rate(guts_storage_cache_hits_total[5m]) + rate(guts_storage_cache_misses_total[5m]))

# Storage operation latency
histogram_quantile(0.99, rate(guts_storage_operation_duration_seconds_bucket[5m]))

# Storage size growth
deriv(guts_storage_size_bytes[1h])
```

## SLO/SLI Definitions

### Service Level Indicators

| SLI | Target | Query |
|-----|--------|-------|
| Availability | 99.9% | `avg_over_time(up{job="guts-node"}[30d])` |
| Latency (p99) | < 100ms | `histogram_quantile(0.99, rate(guts_http_request_duration_seconds_bucket[5m]))` |
| Error Rate | < 0.1% | `sum(rate(guts_http_requests_total{status=~"5.."}[5m])) / sum(rate(guts_http_requests_total[5m]))` |
| Sync Lag | < 1 min | `time() - guts_last_block_time` |

### Recording Rules

```yaml
# prometheus/rules/guts-slos.yml
groups:
  - name: guts-slos
    interval: 30s
    rules:
      # Availability
      - record: guts:availability:ratio
        expr: avg_over_time(up{job="guts-node"}[5m])

      # Error rate
      - record: guts:error_rate:ratio
        expr: |
          sum(rate(guts_http_requests_total{status=~"5.."}[5m]))
          /
          sum(rate(guts_http_requests_total[5m]))

      # Latency p99
      - record: guts:latency:p99
        expr: histogram_quantile(0.99, sum(rate(guts_http_request_duration_seconds_bucket[5m])) by (le))

      # Request rate
      - record: guts:request_rate:per_second
        expr: sum(rate(guts_http_requests_total[5m]))
```

## Troubleshooting Metrics

### No Metrics Appearing

```bash
# Check metrics endpoint
curl -v http://localhost:9090/metrics

# Check Prometheus targets
curl http://prometheus:9090/api/v1/targets

# Check service discovery
curl http://prometheus:9090/api/v1/targets/metadata
```

### High Cardinality

```bash
# Check metric cardinality
curl -s http://prometheus:9090/api/v1/label/__name__/values | jq '.data | length'

# Find high-cardinality metrics
curl -s 'http://prometheus:9090/api/v1/query?query=count by (__name__)({job="guts-node"})' | jq
```

### Prometheus Storage

```bash
# Check storage usage
du -sh /prometheus

# Check TSDB stats
curl http://prometheus:9090/api/v1/status/tsdb
```

## Related Documentation

- [Alert Configuration](alerting.md)
- [Logging Guide](logging.md)
- [Metrics Reference](../reference/metrics.md)
