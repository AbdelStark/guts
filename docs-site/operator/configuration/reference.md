# Configuration Reference

> Complete reference for all Guts node configuration options.

## Configuration Methods

Guts supports three configuration methods, in order of precedence:

1. **Command-line flags** (highest priority)
2. **Environment variables**
3. **Configuration file** (lowest priority)

## Configuration File

Default location: `config.yaml` (current directory) or `/etc/guts/config.yaml`

### Full Configuration Example

```yaml
# Guts Node Configuration
# All values shown are defaults unless noted

#──────────────────────────────────────────────────────────────────────────────
# HTTP API Configuration
#──────────────────────────────────────────────────────────────────────────────
api:
  # Listen address for HTTP API
  # Environment: GUTS_API_ADDR
  # CLI: --api-addr
  addr: "127.0.0.1:8080"

  # Request timeout in seconds (1-3600)
  # Environment: GUTS_REQUEST_TIMEOUT_SECS
  request_timeout_secs: 30

  # Maximum request body size in bytes
  # Environment: GUTS_MAX_REQUEST_BODY_SIZE
  max_request_body_size: 104857600  # 100MB

  # Enable CORS (for browser access)
  cors:
    enabled: false
    allowed_origins:
      - "https://example.com"
    allowed_methods:
      - GET
      - POST
      - PUT
      - DELETE
    allowed_headers:
      - Content-Type
      - Authorization
    max_age_secs: 3600

#──────────────────────────────────────────────────────────────────────────────
# P2P Networking Configuration
#──────────────────────────────────────────────────────────────────────────────
p2p:
  # Listen address for P2P connections
  # Environment: GUTS_P2P_ADDR
  # CLI: --p2p-addr
  addr: "0.0.0.0:9000"

  # Bootstrap nodes for network discovery
  # Format: /ip4/<ip>/tcp/<port>/p2p/<peer-id>
  # Environment: GUTS_BOOTSTRAP_NODES (comma-separated)
  bootstrap_nodes: []
  # Example:
  # bootstrap_nodes:
  #   - "/ip4/1.2.3.4/tcp/9000/p2p/12D3KooWExample..."
  #   - "/dns4/bootstrap.guts.network/tcp/9000/p2p/12D3KooW..."

  # External address (for NAT traversal)
  # Environment: GUTS_EXTERNAL_ADDR
  external_addr: null

  # Maximum number of peer connections
  max_peers: 50

  # Connection timeout in seconds
  connection_timeout_secs: 30

  # Dial timeout in seconds
  dial_timeout_secs: 10

  # Message size limit in bytes
  max_message_size: 1048576  # 1MB

#──────────────────────────────────────────────────────────────────────────────
# Metrics Configuration
#──────────────────────────────────────────────────────────────────────────────
metrics:
  # Listen address for Prometheus metrics endpoint
  # Environment: GUTS_METRICS_ADDR
  # CLI: --metrics-addr
  addr: "0.0.0.0:9090"

  # Metrics endpoint path
  path: "/metrics"

  # Include process metrics
  include_process_metrics: true

  # Include Go runtime metrics (if applicable)
  include_runtime_metrics: true

#──────────────────────────────────────────────────────────────────────────────
# Storage Configuration
#──────────────────────────────────────────────────────────────────────────────
storage:
  # Data directory for all persistent data
  # Environment: GUTS_DATA_DIR
  # CLI: --data-dir
  data_dir: "./data"

  # Storage backend: "memory" or "rocksdb"
  # Environment: GUTS_STORAGE_BACKEND
  backend: "rocksdb"

  # RocksDB specific options
  rocksdb:
    # Write buffer size in bytes
    write_buffer_size: 67108864  # 64MB

    # Maximum number of write buffers
    max_write_buffer_number: 3

    # Block cache size in bytes
    block_cache_size: 536870912  # 512MB

    # Enable compression
    compression: true

    # Compression algorithm: "none", "snappy", "lz4", "zstd"
    compression_type: "lz4"

    # Maximum number of open files
    max_open_files: 10000

    # Enable statistics
    enable_statistics: true

  # Cache configuration
  cache:
    # Enable in-memory cache
    enabled: true

    # Maximum cache size in bytes
    max_size: 268435456  # 256MB

    # TTL for cached objects in seconds
    ttl_secs: 3600

#──────────────────────────────────────────────────────────────────────────────
# Logging Configuration
#──────────────────────────────────────────────────────────────────────────────
logging:
  # Log level: trace, debug, info, warn, error
  # Environment: GUTS_LOG_LEVEL
  # CLI: --log-level
  level: "info"

  # Log format: json, pretty
  # Environment: GUTS_LOG_FORMAT
  # CLI: --log-format
  format: "json"

  # Include source location in logs
  include_location: false

  # Include target (module path) in logs
  include_target: true

  # Log file (optional, default is stdout)
  # file: "/var/log/guts/node.log"

  # Filter specific modules
  # filters:
  #   guts_p2p: debug
  #   guts_consensus: debug
  #   tower_http: info

#──────────────────────────────────────────────────────────────────────────────
# Consensus Configuration
#──────────────────────────────────────────────────────────────────────────────
consensus:
  # Enable consensus engine
  # Environment: GUTS_CONSENSUS_ENABLED
  enabled: false

  # Use Simplex BFT (production consensus)
  # Environment: GUTS_CONSENSUS_USE_SIMPLEX_BFT
  use_simplex_bft: false

  # Block production interval in milliseconds
  # Environment: GUTS_CONSENSUS_BLOCK_TIME_MS
  block_time_ms: 2000

  # Maximum transactions per block
  # Environment: GUTS_CONSENSUS_MAX_TXS_PER_BLOCK
  max_txs_per_block: 1000

  # Mempool configuration
  mempool:
    # Maximum pending transactions
    # Environment: GUTS_CONSENSUS_MEMPOOL_MAX_TXS
    max_txs: 10000

    # Transaction TTL in seconds
    # Environment: GUTS_CONSENSUS_MEMPOOL_TTL_SECS
    ttl_secs: 600

  # Genesis file path (for validator networks)
  # Environment: GUTS_CONSENSUS_GENESIS_FILE
  genesis_file: null

  # Validator configuration
  validator:
    # Private key (hex-encoded Ed25519)
    # Environment: GUTS_PRIVATE_KEY
    # CLI: --private-key
    private_key: null

    # Or load from file
    # Environment: GUTS_PRIVATE_KEY_FILE
    private_key_file: null

#──────────────────────────────────────────────────────────────────────────────
# Security Configuration
#──────────────────────────────────────────────────────────────────────────────
security:
  # Rate limiting
  rate_limit:
    enabled: true
    # Requests per second per IP
    requests_per_second: 100
    # Burst size
    burst_size: 200

  # TLS configuration (for HTTPS)
  tls:
    enabled: false
    cert_file: null
    key_file: null
    # Minimum TLS version: "1.2" or "1.3"
    min_version: "1.2"

  # Authentication
  auth:
    # Require authentication for write operations
    require_auth_for_writes: false
    # API token validation
    validate_tokens: true

#──────────────────────────────────────────────────────────────────────────────
# Resilience Configuration
#──────────────────────────────────────────────────────────────────────────────
resilience:
  # Retry configuration
  retry:
    # Maximum retry attempts
    max_attempts: 3
    # Initial backoff in milliseconds
    initial_backoff_ms: 100
    # Maximum backoff in milliseconds
    max_backoff_ms: 10000
    # Backoff multiplier
    multiplier: 2.0
    # Add jitter to backoff
    jitter: true

  # Circuit breaker configuration
  circuit_breaker:
    # Failure threshold before opening
    failure_threshold: 5
    # Success threshold to close
    success_threshold: 3
    # Timeout in seconds before half-open
    timeout_secs: 30

  # Timeouts
  timeouts:
    # Default operation timeout in seconds
    operation_timeout_secs: 30
    # P2P operation timeout
    p2p_timeout_secs: 60
    # Consensus timeout
    consensus_timeout_secs: 120

#──────────────────────────────────────────────────────────────────────────────
# Health Check Configuration
#──────────────────────────────────────────────────────────────────────────────
health:
  # Enable health check endpoints
  enabled: true

  # Startup probe configuration
  startup:
    # Maximum time to wait for startup
    timeout_secs: 60

  # Liveness probe configuration
  liveness:
    # Check interval
    interval_secs: 10

  # Readiness probe configuration
  readiness:
    # Minimum peer count to be ready
    min_peers: 0
    # Require sync to be complete
    require_synced: false
```

## Environment Variables

All configuration options can be set via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `GUTS_API_ADDR` | HTTP API listen address | `127.0.0.1:8080` |
| `GUTS_P2P_ADDR` | P2P listen address | `0.0.0.0:9000` |
| `GUTS_METRICS_ADDR` | Metrics listen address | `0.0.0.0:9090` |
| `GUTS_DATA_DIR` | Data directory | `./data` |
| `GUTS_LOG_LEVEL` | Log level | `info` |
| `GUTS_LOG_FORMAT` | Log format | `json` |
| `GUTS_PRIVATE_KEY` | Node private key (hex) | - |
| `GUTS_PRIVATE_KEY_FILE` | Path to private key file | - |
| `GUTS_BOOTSTRAP_NODES` | Bootstrap nodes (comma-separated) | - |
| `GUTS_CONSENSUS_ENABLED` | Enable consensus | `false` |
| `GUTS_CONSENSUS_USE_SIMPLEX_BFT` | Use Simplex BFT | `false` |
| `GUTS_CONSENSUS_BLOCK_TIME_MS` | Block time (ms) | `2000` |
| `GUTS_CONSENSUS_MAX_TXS_PER_BLOCK` | Max TXs per block | `1000` |
| `GUTS_CONSENSUS_MEMPOOL_MAX_TXS` | Max mempool size | `10000` |
| `GUTS_CONSENSUS_MEMPOOL_TTL_SECS` | Mempool TTL | `600` |
| `GUTS_CONSENSUS_GENESIS_FILE` | Genesis file path | - |
| `GUTS_STORAGE_BACKEND` | Storage backend | `rocksdb` |
| `GUTS_REQUEST_TIMEOUT_SECS` | Request timeout | `30` |

## Command-Line Flags

```
guts-node [OPTIONS]

Options:
  --config <FILE>           Configuration file path [default: config.yaml]
  --api-addr <ADDR>         HTTP API listen address
  --p2p-addr <ADDR>         P2P listen address
  --metrics-addr <ADDR>     Metrics listen address
  --data-dir <DIR>          Data directory
  --log-level <LEVEL>       Log level (trace, debug, info, warn, error)
  --log-format <FORMAT>     Log format (json, pretty)
  --private-key <KEY>       Node private key (hex-encoded)
  --local                   Enable local development mode (pretty logs)
  -h, --help                Print help
  -V, --version             Print version
```

## Configuration Validation

The node validates configuration on startup:

```bash
# Validate configuration without starting
guts-node --config /etc/guts/config.yaml --dry-run

# Common validation errors:
# - Invalid address format
# - Port already in use
# - Invalid private key format
# - Genesis file not found
# - Invalid log level
```

## Example Configurations

### Development Node

```yaml
api:
  addr: "127.0.0.1:8080"

p2p:
  addr: "0.0.0.0:9000"

storage:
  data_dir: "./dev-data"
  backend: "memory"

logging:
  level: "debug"
  format: "pretty"
```

### Production Full Node

```yaml
api:
  addr: "0.0.0.0:8080"
  request_timeout_secs: 30

p2p:
  addr: "0.0.0.0:9000"
  bootstrap_nodes:
    - "/dns4/bootstrap1.guts.network/tcp/9000/p2p/..."
    - "/dns4/bootstrap2.guts.network/tcp/9000/p2p/..."

metrics:
  addr: "0.0.0.0:9090"

storage:
  data_dir: "/var/lib/guts"
  backend: "rocksdb"
  rocksdb:
    block_cache_size: 1073741824  # 1GB

logging:
  level: "info"
  format: "json"

security:
  rate_limit:
    enabled: true
    requests_per_second: 1000
```

### Validator Node

```yaml
api:
  addr: "0.0.0.0:8080"

p2p:
  addr: "0.0.0.0:9000"
  bootstrap_nodes:
    - "/dns4/bootstrap1.guts.network/tcp/9000/p2p/..."

storage:
  data_dir: "/var/lib/guts"
  backend: "rocksdb"

logging:
  level: "info"
  format: "json"

consensus:
  enabled: true
  use_simplex_bft: true
  block_time_ms: 2000
  max_txs_per_block: 1000
  genesis_file: "/etc/guts/genesis.json"
  validator:
    private_key_file: "/etc/guts/node.key"
```

## Genesis File Format

For validator networks:

```json
{
  "chain_id": "guts-mainnet",
  "genesis_time": "2025-01-01T00:00:00Z",
  "validators": [
    {
      "public_key": "abc123...",
      "power": 1,
      "name": "validator-1"
    },
    {
      "public_key": "def456...",
      "power": 1,
      "name": "validator-2"
    },
    {
      "public_key": "ghi789...",
      "power": 1,
      "name": "validator-3"
    },
    {
      "public_key": "jkl012...",
      "power": 1,
      "name": "validator-4"
    }
  ],
  "consensus": {
    "block_time_ms": 2000,
    "max_txs_per_block": 1000
  }
}
```

## Related Documentation

- [Networking Configuration](networking.md)
- [Storage Configuration](storage.md)
- [Security Configuration](security.md)
- [Performance Tuning](performance.md)
