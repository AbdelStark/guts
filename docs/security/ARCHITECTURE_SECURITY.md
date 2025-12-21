# Guts Security Architecture

> Security-focused overview of the Guts architecture for security auditors and operators.

## Executive Summary

Guts is a decentralized code collaboration platform built with security as a core design principle. This document provides a security-focused view of the system architecture, highlighting security controls, cryptographic operations, and trust boundaries.

## Architecture Overview

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Guts Node                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        API Layer (Axum)                                │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │  │
│  │  │ REST    │  │ Git     │  │ Web     │  │ WS      │  │ Metrics     │  │  │
│  │  │ API     │  │ HTTP    │  │ Gateway │  │ Server  │  │ /health     │  │  │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └─────────────┘  │  │
│  └───────┼───────────┼───────────┼───────────┼───────────────────────────┘  │
│          │           │           │           │                               │
│          ▼           ▼           ▼           ▼                               │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                     Security Middleware                                │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │  │
│  │  │ Auth        │  │ Rate        │  │ Audit       │  │ Input       │   │  │
│  │  │ Middleware  │  │ Limiter     │  │ Logger      │  │ Validator   │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│          │           │           │           │                               │
│          ▼           ▼           ▼           ▼                               │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                      Business Logic Crates                             │  │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐             │  │
│  │  │guts-collab│ │guts-auth  │ │guts-ci    │ │guts-compat│             │  │
│  │  │PRs, Issues│ │Orgs, Teams│ │Workflows  │ │GH Compat  │             │  │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘             │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│          │           │           │           │                               │
│          ▼           ▼           ▼           ▼                               │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        Core Crates                                     │  │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────────┐  │  │
│  │  │guts-git   │ │guts-      │ │guts-types │ │guts-security          │  │  │
│  │  │Pack,Proto │ │storage    │ │Identities │ │Audit, Keys, Secrets   │  │  │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│          │                                                                   │
│          ▼                                                                   │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        P2P Layer                                       │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐    │  │
│  │  │ Commonware  │  │ Commonware  │  │ Commonware                  │    │  │
│  │  │ P2P         │  │ Consensus   │  │ Cryptography                │    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Security Layers

### Layer 1: Network Security

| Component | Security Control | Implementation |
|-----------|-----------------|----------------|
| HTTPS | TLS 1.3 | rustls with safe defaults |
| P2P | Noise Protocol | commonware-p2p encryption |
| WebSocket | WSS | TLS-secured WebSocket |

**Configuration**:
```rust
// TLS configuration (production)
let tls_config = ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_single_cert(cert_chain, private_key)?;
```

### Layer 2: Authentication

| Method | Use Case | Security Properties |
|--------|----------|---------------------|
| Ed25519 Signatures | Git operations, P2P | Cryptographic non-repudiation |
| API Tokens | REST API | Hashed with Argon2id |
| Session Tokens | Web Gateway | Secure random, HTTP-only cookies |

**Token Hashing**:
```rust
// Argon2id parameters (OWASP recommended)
let params = argon2::Params::new(
    65536,     // 64 MB memory
    3,         // 3 iterations
    4,         // 4 parallelism
    Some(32),  // 32 byte output
)?;
```

### Layer 3: Authorization

| Level | Scope | Enforcement |
|-------|-------|-------------|
| Repository | Read/Write/Admin | Per-endpoint middleware |
| Organization | Member/Admin/Owner | Role-based access |
| Branch | Protection rules | Server-side enforcement |

**Permission Model**:
```
Organization
    └── Team
        └── Repository
            └── Branch (Protection)
                └── User (Permission Level)
```

### Layer 4: Input Validation

| Input Type | Validation | Library |
|------------|------------|---------|
| JSON | Schema validation | serde + validator |
| Git Protocol | Format validation | Custom parser |
| Paths | Traversal prevention | Path sanitization |
| Sizes | Limit enforcement | Middleware |

### Layer 5: Audit Logging

All security-relevant events are logged:

```rust
pub enum AuditEventType {
    // Authentication
    Login,
    Logout,
    TokenCreated,
    TokenRevoked,

    // Authorization
    PermissionGranted,
    PermissionRevoked,
    PermissionDenied,

    // Repository Operations
    RepoCreated,
    RepoDeleted,
    BranchProtectionChanged,

    // Key Management
    KeyRotated,
    KeyRevoked,

    // System
    ConfigChanged,
    RateLimitExceeded,
}
```

## Cryptographic Operations

### Signature Operations

| Operation | Algorithm | Key Size | Usage |
|-----------|-----------|----------|-------|
| Commit signing | Ed25519 | 256-bit | Developer identity |
| Node identity | Ed25519 | 256-bit | P2P authentication |
| Audit log signing | Ed25519 | 256-bit | Tamper evidence |

### Hashing Operations

| Operation | Algorithm | Usage |
|-----------|-----------|-------|
| Object storage | SHA-256 | Content addressing |
| Token storage | Argon2id | Secure token hashing |
| Quick hashing | BLAKE3 | Performance-critical paths |

### Key Hierarchy

```
Root Key (HSM-stored, optional)
    │
    ├── Node Identity Key
    │       └── Used for P2P and consensus
    │
    ├── Signing Key
    │       └── Used for audit logs
    │
    └── Encryption Key (future)
            └── At-rest encryption
```

## Data Flow Security

### Git Push Flow

```
Client                    Node                     Consensus
   │                        │                          │
   │─── TLS Handshake ─────►│                          │
   │                        │                          │
   │─── Auth Token ────────►│                          │
   │                        │── Verify Token ────────► │
   │                        │                          │
   │─── Pack File ─────────►│                          │
   │                        │── Validate Pack ───────► │
   │                        │                          │
   │                        │── Check Permissions ───► │
   │                        │                          │
   │                        │── Store Objects ───────► │
   │                        │                          │
   │                        │── Replicate ────────────►│
   │                        │                          │
   │◄── Success/Error ─────│◄── Consensus Commit ────│
   │                        │                          │
```

### API Request Flow

```
Client                    Middleware                Handler
   │                          │                        │
   │─── HTTPS Request ───────►│                        │
   │                          │                        │
   │                          │── Rate Limit Check ───►│
   │                          │                        │
   │                          │── Auth Extraction ────►│
   │                          │                        │
   │                          │── Input Validation ───►│
   │                          │                        │
   │                          │── Audit Log ──────────►│
   │                          │                        │
   │                          │────────────────────────►│
   │                          │                        │── Business Logic
   │                          │                        │
   │◄── HTTPS Response ──────│◄───────────────────────│
   │                          │                        │
```

## Security Configuration

### Recommended Production Settings

```toml
[security]
# TLS
tls_min_version = "1.3"
tls_cipher_suites = ["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"]

# Rate Limiting
rate_limit_unauthenticated = 60
rate_limit_authenticated = 5000
rate_limit_window_seconds = 3600

# Tokens
token_hash_algorithm = "argon2id"
token_expiry_days = 90

# Keys
key_rotation_days = 90
key_overlap_days = 7

# Audit
audit_log_retention_days = 365
audit_log_signed = true

# Size Limits
max_pack_size_bytes = 104857600  # 100 MB
max_api_body_bytes = 10485760    # 10 MB
max_websocket_message_bytes = 65536  # 64 KB
```

### Security Headers

All HTTP responses include:

```
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Content-Security-Policy: default-src 'self'
X-XSS-Protection: 0
Referrer-Policy: strict-origin-when-cross-origin
```

## Secrets Management

### Supported Backends

| Backend | Use Case | Security Level |
|---------|----------|---------------|
| Environment Variables | Development | Low |
| Encrypted Files | Small deployments | Medium |
| HashiCorp Vault | Production | High |
| AWS Secrets Manager | Cloud deployments | High |
| HSM (PKCS#11) | High security | Very High |

### Key Rotation

```rust
pub struct KeyRotationPolicy {
    // Rotate keys every 90 days
    pub max_age: Duration,

    // Old key valid for 7 days after rotation
    pub overlap_period: Duration,

    // Warn 14 days before expiry
    pub warn_before: Duration,
}
```

## Monitoring and Alerting

### Security Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `auth_failures_total` | Failed auth attempts | > 100/hour |
| `rate_limit_exceeded_total` | Rate limit hits | > 1000/hour |
| `permission_denied_total` | Authorization failures | > 50/hour |
| `suspicious_activity_total` | Anomaly detections | > 10/hour |

### Audit Log Queries

```rust
// Query suspicious patterns
let suspicious = audit_log.query(AuditQuery {
    event_types: vec![
        AuditEventType::PermissionDenied,
        AuditEventType::RateLimitExceeded,
    ],
    time_range: last_hour(),
    group_by: "actor",
    having: "count > 10",
});
```

## Incident Response

### Security Event Categories

| Category | Examples | Response |
|----------|----------|----------|
| **P1 Critical** | Key compromise, data breach | Immediate rotation, notify users |
| **P2 High** | Auth bypass, privilege escalation | Hotfix within 24h |
| **P3 Medium** | Rate limit bypass, info disclosure | Fix within 7 days |
| **P4 Low** | Minor hardening issues | Next release cycle |

### Emergency Procedures

1. **Key Compromise**: Immediate rotation via HSM
2. **Node Compromise**: Remove from consensus, revoke identity
3. **Data Breach**: Audit log analysis, user notification
4. **DDoS**: Activate upstream protection, rate limit escalation

## Compliance Considerations

### Data Handling

| Data Type | Classification | Retention | Encryption |
|-----------|---------------|-----------|------------|
| Repository content | User data | User-controlled | At rest (optional) |
| Audit logs | Security data | 1 year minimum | At rest |
| API tokens | Credentials | User-controlled | Always hashed |
| Node keys | Infrastructure | Rotation-controlled | HSM (recommended) |

### Standards Alignment

| Standard | Relevant Controls |
|----------|------------------|
| OWASP Top 10 | Input validation, auth, crypto |
| NIST 800-53 | AC, AU, IA, SC controls |
| CIS Benchmarks | OS and container hardening |
| SOC 2 | Security, availability, confidentiality |

---

*This document is intended for security auditors and operators. For user-facing security information, see [SECURITY.md](./SECURITY.md).*
