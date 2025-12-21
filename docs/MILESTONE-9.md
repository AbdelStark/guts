# Milestone 9: Production Quality Improvements

> **Status:** In Progress
> **Started:** 2025-12-21

## Overview

Milestone 9 focuses on hardening the Guts platform for production deployment. This includes comprehensive observability (structured logging, metrics, tracing), robust error handling, configuration management, input validation, and resilience patterns. These improvements ensure the platform can be reliably deployed, monitored, and operated in production environments.

## Goals

1. **Observability**: Comprehensive structured logging with request tracing, Prometheus metrics, and distributed tracing support
2. **Error Handling**: Replace all panics/unwraps with proper error handling and recovery
3. **Configuration**: Validated configuration with environment variable support and sensible defaults
4. **Input Validation**: Strict validation of all API inputs with consistent error responses
5. **Resilience**: Retry logic, circuit breakers, timeouts, and graceful degradation
6. **Health Checks**: Comprehensive readiness, liveness, and startup probes
7. **Testing**: Property-based testing, fuzz testing, and chaos testing

## Architecture

### New Components

```
crates/guts-node/src/
├── observability/
│   ├── mod.rs           # Observability module
│   ├── logging.rs       # Structured logging setup
│   ├── metrics.rs       # Prometheus metrics
│   └── tracing.rs       # Distributed tracing
├── validation/
│   ├── mod.rs           # Validation module
│   └── middleware.rs    # Input validation middleware
├── resilience/
│   ├── mod.rs           # Resilience module
│   ├── retry.rs         # Retry policies
│   ├── circuit_breaker.rs # Circuit breaker pattern
│   └── timeout.rs       # Timeout management
└── health/
    ├── mod.rs           # Health check module
    └── probes.rs        # Readiness/liveness probes
```

### Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Metrics | prometheus-client | Standard Prometheus metrics |
| Tracing | tracing + opentelemetry | Distributed tracing standard |
| Logging | tracing-subscriber | Structured JSON logging |
| Validation | validator | Declarative validation |
| Config | config + envy | File + env var config |
| Circuit Breaker | Custom implementation | Tailored to our needs |

## Detailed Implementation

### 1. Structured Logging

All log entries will include:
- **Request ID**: UUID for correlating logs across a request
- **Trace ID**: For distributed tracing correlation
- **User ID**: When authenticated
- **Repository**: When in repo context
- **Duration**: For timed operations

```rust
// Example structured log
tracing::info!(
    request_id = %request_id,
    user_id = %user_id,
    repository = %repo_key,
    duration_ms = elapsed.as_millis(),
    "pull request merged successfully"
);
```

### 2. Prometheus Metrics

#### HTTP Metrics
- `guts_http_requests_total{method, path, status}` - Request counter
- `guts_http_request_duration_seconds{method, path}` - Request latency histogram
- `guts_http_request_size_bytes{method, path}` - Request body size
- `guts_http_response_size_bytes{method, path}` - Response body size

#### P2P Metrics
- `guts_p2p_peers_connected` - Number of connected peers
- `guts_p2p_messages_sent_total{type}` - Messages sent by type
- `guts_p2p_messages_received_total{type}` - Messages received by type
- `guts_p2p_message_latency_seconds{type}` - Message round-trip latency
- `guts_p2p_replication_lag_seconds` - Replication lag

#### Storage Metrics
- `guts_storage_objects_total{type}` - Git objects by type
- `guts_storage_size_bytes` - Total storage size
- `guts_storage_operation_duration_seconds{operation}` - Operation latency

#### Business Metrics
- `guts_repositories_total` - Total repositories
- `guts_pull_requests_total{state}` - PRs by state
- `guts_issues_total{state}` - Issues by state
- `guts_users_total` - Total users
- `guts_organizations_total` - Total organizations

### 3. Configuration Validation

```rust
pub struct Config {
    // Node identity
    #[validate(length(min = 64, max = 64))]
    pub private_key: String,

    // Network
    #[validate(range(min = 1, max = 65535))]
    pub api_port: u16,

    #[validate(range(min = 1, max = 65535))]
    pub p2p_port: u16,

    #[validate(range(min = 1, max = 65535))]
    pub metrics_port: u16,

    // Limits
    #[validate(range(min = 1, max = 10000))]
    pub max_connections: u32,

    #[validate(range(min = 1, max = 1000))]
    pub max_request_size_mb: u32,

    // Timeouts
    #[validate(range(min = 1, max = 3600))]
    pub request_timeout_secs: u32,

    #[validate(range(min = 1, max = 3600))]
    pub p2p_timeout_secs: u32,
}
```

Environment variables:
- `GUTS_PRIVATE_KEY` - Node private key
- `GUTS_API_ADDR` - API listen address
- `GUTS_P2P_ADDR` - P2P listen address
- `GUTS_METRICS_ADDR` - Metrics endpoint address
- `GUTS_LOG_LEVEL` - Log level (trace, debug, info, warn, error)
- `GUTS_LOG_FORMAT` - Log format (json, pretty)
- `GUTS_MAX_CONNECTIONS` - Max concurrent connections
- `GUTS_REQUEST_TIMEOUT` - Request timeout in seconds

### 4. Input Validation

All API inputs will be validated:

```rust
#[derive(Deserialize, Validate)]
pub struct CreateRepoRequest {
    #[validate(length(min = 1, max = 100))]
    #[validate(regex = "^[a-z0-9][a-z0-9-]*[a-z0-9]$")]
    pub name: String,

    #[validate(length(min = 1, max = 100))]
    #[validate(regex = "^[a-z0-9][a-z0-9-]*[a-z0-9]$")]
    pub owner: String,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    pub private: Option<bool>,
}
```

Validation errors return consistent format:
```json
{
    "error": "validation_error",
    "message": "Validation failed",
    "details": [
        {
            "field": "name",
            "code": "invalid_format",
            "message": "Name must contain only lowercase letters, numbers, and hyphens"
        }
    ]
}
```

### 5. Resilience Patterns

#### Retry Policy
```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub retryable_errors: Vec<ErrorKind>,
}

// Default policy
RetryPolicy {
    max_attempts: 3,
    initial_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(5),
    multiplier: 2.0,
    retryable_errors: vec![
        ErrorKind::NetworkError,
        ErrorKind::Timeout,
        ErrorKind::ServiceUnavailable,
    ],
}
```

#### Circuit Breaker
```rust
pub struct CircuitBreaker {
    pub failure_threshold: u32,    // Failures before opening
    pub success_threshold: u32,    // Successes to close
    pub timeout: Duration,         // How long to stay open
    pub half_open_max: u32,        // Requests in half-open state
}
```

States:
- **Closed**: Normal operation, tracking failures
- **Open**: Failing fast, not sending requests
- **Half-Open**: Testing if service recovered

#### Timeout Management
```rust
pub struct TimeoutConfig {
    pub connect: Duration,         // Connection timeout
    pub read: Duration,            // Read timeout
    pub write: Duration,           // Write timeout
    pub total: Duration,           // Total request timeout
}
```

### 6. Health Check Endpoints

#### Liveness Probe
```
GET /health/live
```
Returns 200 if the process is running. Used by Kubernetes to restart dead pods.

#### Readiness Probe
```
GET /health/ready
```
Returns 200 if the node is ready to serve traffic:
- P2P connections established
- Storage accessible
- Not in maintenance mode

Response:
```json
{
    "status": "ready",
    "checks": {
        "storage": { "status": "up", "latency_ms": 1 },
        "p2p": { "status": "up", "peers": 3 },
        "consensus": { "status": "up", "leader": true }
    }
}
```

#### Startup Probe
```
GET /health/startup
```
Returns 200 when initial startup is complete. Used by Kubernetes to know when to start liveness/readiness checks.

## Implementation Plan

### Phase 1: Observability (Core)

1. [x] Create observability module structure
2. [x] Implement structured logging with request IDs
3. [x] Add request/response logging middleware
4. [x] Implement Prometheus metrics endpoint
5. [x] Add HTTP request metrics
6. [x] Add P2P metrics
7. [x] Add storage metrics
8. [x] Add business metrics

### Phase 2: Configuration

1. [x] Define comprehensive Config struct with validation
2. [x] Add environment variable binding
3. [x] Implement config file loading (TOML/YAML)
4. [x] Add configuration validation on startup
5. [x] Add sensible defaults
6. [x] Document all configuration options

### Phase 3: Input Validation

1. [x] Add validator dependency
2. [x] Create validation middleware
3. [x] Add validation to all request types
4. [x] Implement consistent error responses
5. [x] Add validation for path parameters
6. [x] Add validation for query parameters

### Phase 4: Error Handling

1. [x] Audit and fix unwrap calls in guts-node
2. [x] Audit and fix unwrap calls in guts-storage
3. [x] Audit and fix unwrap calls in guts-git
4. [x] Audit and fix unwrap calls in guts-p2p
5. [x] Audit and fix unwrap calls in guts-ci
6. [x] Audit and fix unwrap calls in guts-compat
7. [x] Add error context with thiserror
8. [x] Implement error recovery where appropriate

### Phase 5: Resilience

1. [x] Implement retry policy
2. [x] Implement circuit breaker
3. [x] Add timeout management
4. [x] Add backpressure handling
5. [x] Implement graceful degradation
6. [x] Add rate limiting improvements

### Phase 6: Health Checks

1. [x] Implement liveness probe
2. [x] Implement readiness probe
3. [x] Implement startup probe
4. [x] Add component health checks
5. [x] Update Kubernetes manifests

### Phase 7: Testing

1. [ ] Add property-based tests for protocol parsing
2. [ ] Add fuzz testing for Git protocol
3. [ ] Add chaos testing for P2P layer
4. [ ] Add load testing infrastructure
5. [ ] Add integration tests with failure injection

## API Reference

### Health Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Overall health status |
| GET | `/health/live` | Liveness probe |
| GET | `/health/ready` | Readiness probe |
| GET | `/health/startup` | Startup probe |

### Metrics Endpoint

| Method | Path | Description |
|--------|------|-------------|
| GET | `/metrics` | Prometheus metrics |

## Configuration Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `GUTS_API_ADDR` | `0.0.0.0:8080` | HTTP API listen address |
| `GUTS_P2P_ADDR` | `0.0.0.0:9000` | P2P listen address |
| `GUTS_METRICS_ADDR` | `0.0.0.0:9090` | Metrics endpoint address |
| `GUTS_LOG_LEVEL` | `info` | Log level |
| `GUTS_LOG_FORMAT` | `json` | Log format (json/pretty) |
| `GUTS_PRIVATE_KEY` | *required* | Ed25519 private key (hex) |
| `GUTS_MAX_CONNECTIONS` | `10000` | Max concurrent connections |
| `GUTS_REQUEST_TIMEOUT` | `30` | Request timeout (seconds) |
| `GUTS_P2P_TIMEOUT` | `10` | P2P operation timeout (seconds) |
| `GUTS_RETRY_MAX_ATTEMPTS` | `3` | Max retry attempts |
| `GUTS_CIRCUIT_BREAKER_THRESHOLD` | `5` | Circuit breaker failure threshold |

## Success Criteria

- [ ] All log entries include request ID and structured context
- [ ] Prometheus metrics endpoint returns valid metrics
- [ ] All configuration validated on startup
- [ ] All API inputs validated before processing
- [ ] No panic/unwrap calls in production code paths
- [ ] Health endpoints return accurate status
- [ ] Circuit breaker trips on repeated failures
- [ ] Retries work for transient failures
- [ ] Timeouts prevent request stalling
- [ ] Property-based tests cover protocol edge cases

## Security Considerations

1. **Metric Security**: Metrics endpoint should be on internal network only
2. **Health Check Security**: Don't expose sensitive info in health responses
3. **Log Security**: Never log secrets, tokens, or private keys
4. **Validation**: Prevent injection attacks through strict validation
5. **Timeouts**: Prevent resource exhaustion attacks

## Performance Considerations

1. **Metrics Overhead**: Use efficient metrics collection
2. **Logging Overhead**: Use async logging, sample high-volume logs
3. **Validation Overhead**: Cache compiled regexes
4. **Circuit Breaker**: Fail fast when services are down
5. **Retry Overhead**: Use exponential backoff to prevent thundering herd

## Dependencies

- `prometheus-client` - Prometheus metrics
- `opentelemetry` - Distributed tracing
- `tracing-opentelemetry` - Tracing integration
- `validator` - Input validation
- `config` - Configuration management
- `envy` - Environment variable binding

## References

- [Prometheus Metrics Best Practices](https://prometheus.io/docs/practices/naming/)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/)
- [Twelve-Factor App](https://12factor.net/)
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Health Check API Pattern](https://microservices.io/patterns/observability/health-check-api.html)
