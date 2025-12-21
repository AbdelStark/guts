# Milestone 10: Security Hardening & Audit Preparation

> **Status:** ✅ Complete
> **Completed:** 2024-12-21
> **Priority:** Critical

## Overview

Milestone 10 focuses on preparing Guts for a professional security audit and establishing robust security infrastructure. For a decentralized, censorship-resistant platform that developers will trust with their code, security assurance is paramount. This milestone transforms Guts from a feature-complete prototype to a security-hardened platform ready for production use.

## Goals

1. **Audit Preparation**: Document architecture, threat model, and attack surfaces for external auditors
2. **Vulnerability Management**: Establish disclosure policy, bug bounty, and CVE tracking
3. **Cryptographic Hardening**: Review and harden all cryptographic implementations
4. **Protocol Security**: Comprehensive fuzzing and formal verification of critical protocols
5. **Supply Chain Security**: Dependency auditing, SBOM generation, and reproducible builds
6. **Access Control Hardening**: Review and strengthen authentication and authorization
7. **Secrets Management**: Implement key rotation, secure storage, and HSM support

## Architecture

### Security Documentation Structure

```
docs/security/
├── SECURITY.md               # Security policy and disclosure process
├── THREAT_MODEL.md           # Comprehensive threat model
├── ARCHITECTURE_SECURITY.md  # Security-focused architecture overview
├── CRYPTOGRAPHY.md           # Cryptographic primitives documentation
├── AUDIT_SCOPE.md           # Scope document for external auditors
└── KNOWN_ISSUES.md          # Known security issues and mitigations
```

### New Components

```
crates/guts-security/
├── src/
│   ├── lib.rs               # Security utilities
│   ├── audit.rs             # Audit logging for security events
│   ├── secrets.rs           # Secrets management
│   ├── rotation.rs          # Key rotation logic
│   └── hsm.rs               # HSM integration interface
└── tests/
    └── security_tests.rs    # Security-focused test suite
```

## Detailed Implementation

### Phase 1: Security Documentation

#### 1.1 Threat Model Document

Document all potential threats using STRIDE methodology:

| Threat Category | Examples | Mitigations |
|-----------------|----------|-------------|
| **Spoofing** | Identity impersonation, forged commits | Ed25519 signatures, key verification |
| **Tampering** | Modified repository data, consensus attacks | Content-addressed storage, BFT consensus |
| **Repudiation** | Denied actions, unsigned operations | Comprehensive audit logs, signed operations |
| **Information Disclosure** | Private repo leaks, key exposure | Access control, encryption at rest |
| **Denial of Service** | Resource exhaustion, consensus stalling | Rate limiting, timeout management |
| **Elevation of Privilege** | Admin access without authorization | RBAC, permission checks, principle of least privilege |

#### 1.2 Attack Surface Analysis

Document and minimize attack surfaces:

- **API Endpoints**: 50+ HTTP endpoints with input validation
- **Git Protocol**: Smart HTTP with pack file parsing
- **P2P Protocol**: Node-to-node communication
- **WebSocket**: Real-time event streaming
- **Consensus**: BFT message handling
- **Storage**: Content-addressed blob store

#### 1.3 Audit Scope Document

Prepare comprehensive scope for external auditors:

```markdown
## Audit Scope

### In Scope
- All Rust crates (12 crates, ~38,000 LOC)
- Cryptographic implementations
- Consensus protocol integration
- Authentication and authorization
- Input validation and sanitization
- P2P networking security

### Out of Scope
- Frontend JavaScript (minimal, template-rendered)
- Third-party dependencies (separate audit)
- Infrastructure configurations
```

### Phase 2: Vulnerability Management

#### 2.1 Security Policy

Create `SECURITY.md` with:

```markdown
# Security Policy

## Supported Versions
| Version | Supported |
|---------|-----------|
| 1.x.x   | ✅ Active |
| 0.x.x   | ❌ No     |

## Reporting a Vulnerability

1. **DO NOT** create public GitHub issues for security vulnerabilities
2. Email security@guts.network with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Your contact information

## Response Timeline
- **24 hours**: Initial acknowledgment
- **72 hours**: Preliminary assessment
- **7 days**: Detailed response with remediation plan
- **90 days**: Public disclosure (coordinated)

## Bug Bounty Program
- Critical: $5,000 - $25,000
- High: $2,000 - $5,000
- Medium: $500 - $2,000
- Low: $100 - $500
```

#### 2.2 CVE Tracking

Implement CVE tracking infrastructure:

- Integrate with GitHub Security Advisories
- Automated CVE assignment process
- Security release process documentation
- Downstream notification mechanism

### Phase 3: Cryptographic Hardening

#### 3.1 Cryptographic Inventory

| Primitive | Library | Usage | Status |
|-----------|---------|-------|--------|
| Ed25519 | commonware::cryptography | Signatures | Review Required |
| SHA-256 | sha2 | Content addressing | Verified |
| Argon2id | argon2 | Token hashing | Review Required |
| BLAKE3 | blake3 | Fast hashing | Verify Usage |
| TLS 1.3 | rustls | Transport security | Verify Config |
| Noise | snow | P2P encryption | Review Required |

#### 3.2 Key Management

Implement comprehensive key management:

```rust
pub struct KeyManager {
    // Primary signing key
    signing_key: Ed25519PrivateKey,

    // Key rotation state
    rotation_state: RotationState,

    // HSM connection (optional)
    hsm: Option<Box<dyn HsmProvider>>,
}

impl KeyManager {
    /// Rotate signing key with overlap period
    pub async fn rotate_key(&mut self, overlap_days: u32) -> Result<()>;

    /// Sign with automatic key selection
    pub fn sign(&self, message: &[u8]) -> Result<Signature>;

    /// Verify supporting old keys during rotation
    pub fn verify(&self, message: &[u8], sig: &Signature) -> Result<bool>;
}
```

#### 3.3 HSM Support

Add optional Hardware Security Module integration:

```rust
pub trait HsmProvider: Send + Sync {
    /// Generate key pair in HSM
    async fn generate_key(&self, key_id: &str) -> Result<PublicKey>;

    /// Sign message using HSM-stored key
    async fn sign(&self, key_id: &str, message: &[u8]) -> Result<Signature>;

    /// Verify signature
    async fn verify(&self, key_id: &str, message: &[u8], sig: &Signature) -> Result<bool>;
}

// Implementations for common HSMs
pub struct AwsCloudHsm { /* ... */ }
pub struct YubiHsm { /* ... */ }
pub struct Pkcs11Hsm { /* ... */ }
```

### Phase 4: Protocol Security

#### 4.1 Extended Fuzzing

Expand fuzzing coverage to all protocol handlers:

| Fuzz Target | Protocol | Priority |
|-------------|----------|----------|
| `fuzz_git_pack` | Git pack parsing | P0 |
| `fuzz_git_refs` | Git reference parsing | P0 |
| `fuzz_consensus_msg` | Consensus messages | P0 |
| `fuzz_p2p_msg` | P2P protocol messages | P0 |
| `fuzz_api_json` | API JSON parsing | P1 |
| `fuzz_webhook_payload` | Webhook payloads | P1 |
| `fuzz_websocket_msg` | WebSocket messages | P1 |

Add corpus management:

```bash
# Corpus directory structure
fuzz/corpus/
├── fuzz_git_pack/
├── fuzz_consensus_msg/
├── fuzz_p2p_msg/
└── artifacts/       # Crash artifacts
```

#### 4.2 Formal Verification (Critical Paths)

Target critical code paths for formal verification using Kani or Creusot:

```rust
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn verify_permission_transitivity() {
        let perm = Permission::any();
        // Admin always includes Write
        kani::assert!(
            !perm.has(Permission::Admin) || perm.has(Permission::Write)
        );
    }

    #[kani::proof]
    fn verify_consensus_safety() {
        // Verify that consensus never commits conflicting values
        // for the same slot
    }
}
```

#### 4.3 Protocol Hardening

- **Git Protocol**: Validate all pack file structures before processing
- **P2P Protocol**: Message size limits, rate limiting per peer
- **Consensus**: Validate all messages against expected state machine
- **API**: Strict Content-Type enforcement, request size limits

### Phase 5: Supply Chain Security

#### 5.1 Dependency Auditing

```yaml
# .github/workflows/security.yml enhancements
- name: Dependency Audit
  run: |
    cargo audit --deny warnings
    cargo deny check
    cargo vet audit

- name: Generate SBOM
  run: |
    cargo sbom > sbom.json
    cyclonedx-cli validate --input sbom.json
```

#### 5.2 Reproducible Builds

Ensure deterministic builds:

```dockerfile
# Dockerfile.reproducible
FROM rust:1.75-slim AS builder

# Pin all tool versions
RUN rustup default 1.75.0

# Disable incremental compilation for reproducibility
ENV CARGO_INCREMENTAL=0
ENV RUSTFLAGS="-C codegen-units=1"

# Build with locked dependencies
COPY Cargo.lock ./
RUN cargo build --release --locked
```

#### 5.3 SBOM Generation

Generate Software Bill of Materials:

```rust
// Build script addition
fn main() {
    // Generate SBOM during build
    println!("cargo:rerun-if-changed=Cargo.lock");

    // Output dependencies list
    let output = Command::new("cargo")
        .args(["tree", "--format", "{p}:{l}", "--prefix", "none"])
        .output()
        .expect("Failed to run cargo tree");

    // Write to build artifacts
    std::fs::write("target/dependencies.txt", output.stdout).ok();
}
```

### Phase 6: Access Control Hardening

#### 6.1 Permission System Review

Verify permission checks on all endpoints:

```rust
// Audit checklist for each endpoint
#[derive(Debug)]
struct EndpointAudit {
    path: &'static str,
    required_permission: Permission,
    auth_required: bool,
    rate_limited: bool,
    input_validated: bool,
    audit_logged: bool,
}

const ENDPOINT_AUDITS: &[EndpointAudit] = &[
    EndpointAudit {
        path: "POST /api/repos",
        required_permission: Permission::Write,
        auth_required: true,
        rate_limited: true,
        input_validated: true,
        audit_logged: true,
    },
    // ... all 50+ endpoints
];
```

#### 6.2 Authentication Hardening

- **Token Security**: Constant-time comparison, secure storage
- **Session Management**: Configurable timeouts, secure cookies
- **MFA Support**: TOTP infrastructure (future implementation)
- **OAuth2/OIDC**: Federated identity support (future)

#### 6.3 Rate Limiting Enhancement

```rust
pub struct EnhancedRateLimiter {
    // Per-IP limits (unauthenticated)
    ip_limits: HashMap<IpAddr, TokenBucket>,

    // Per-user limits (authenticated)
    user_limits: HashMap<UserId, TokenBucket>,

    // Per-repository limits
    repo_limits: HashMap<RepoKey, TokenBucket>,

    // Adaptive limits based on abuse detection
    adaptive: AdaptiveLimiter,
}

impl EnhancedRateLimiter {
    /// Check with abuse detection
    pub fn check(&mut self, ctx: &RequestContext) -> Result<()> {
        // Check all applicable limits
        self.check_ip(&ctx.ip)?;
        if let Some(user) = &ctx.user {
            self.check_user(user)?;
        }
        if let Some(repo) = &ctx.repo {
            self.check_repo(repo)?;
        }

        // Adaptive check for suspicious patterns
        self.adaptive.check(ctx)?;

        Ok(())
    }
}
```

### Phase 7: Secrets Management

#### 7.1 Secrets Architecture

```rust
pub trait SecretsProvider: Send + Sync {
    /// Retrieve a secret by key
    async fn get(&self, key: &str) -> Result<SecretString>;

    /// Store a secret
    async fn set(&self, key: &str, value: &SecretString) -> Result<()>;

    /// Rotate a secret
    async fn rotate(&self, key: &str) -> Result<SecretString>;
}

// Implementations
pub struct EnvSecretsProvider;      // Environment variables
pub struct FileSecretsProvider;     // Encrypted file
pub struct VaultProvider;           // HashiCorp Vault
pub struct AwsSecretsManager;       // AWS Secrets Manager
```

#### 7.2 Key Rotation

Implement automated key rotation:

```rust
pub struct KeyRotationPolicy {
    /// Maximum key age before rotation
    max_age: Duration,

    /// Overlap period for old key validity
    overlap_period: Duration,

    /// Notification threshold
    warn_before: Duration,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self {
            max_age: Duration::from_days(90),
            overlap_period: Duration::from_days(7),
            warn_before: Duration::from_days(14),
        }
    }
}
```

## Implementation Plan

### Phase 1: Documentation (Week 1-2)
- [ ] Create security documentation structure
- [ ] Write comprehensive threat model
- [ ] Document attack surfaces
- [ ] Prepare audit scope document
- [ ] Create cryptographic inventory

### Phase 2: Vulnerability Management (Week 2-3)
- [ ] Create SECURITY.md
- [ ] Set up security@guts.network email
- [ ] Design bug bounty program
- [ ] Integrate GitHub Security Advisories
- [ ] Document security release process

### Phase 3: Cryptographic Review (Week 3-4)
- [ ] Audit all Ed25519 usage
- [ ] Review Argon2 parameters
- [ ] Verify TLS configuration
- [ ] Implement key rotation infrastructure
- [ ] Add HSM provider interface

### Phase 4: Protocol Hardening (Week 4-6)
- [ ] Expand fuzz testing to all protocols
- [ ] Add corpus management
- [ ] Implement formal verification for critical paths
- [ ] Harden message size limits
- [ ] Add protocol-level rate limiting

### Phase 5: Supply Chain (Week 6-7)
- [ ] Enhance cargo audit/deny configuration
- [ ] Implement cargo vet
- [ ] Set up SBOM generation
- [ ] Create reproducible build process
- [ ] Document dependency update policy

### Phase 6: Access Control (Week 7-8)
- [ ] Audit all endpoint permissions
- [ ] Enhance rate limiting
- [ ] Add security audit logging
- [ ] Review authentication flows
- [ ] Document authorization model

### Phase 7: Secrets Management (Week 8-9)
- [ ] Implement SecretsProvider trait
- [ ] Add key rotation infrastructure
- [ ] Integrate with Vault (optional)
- [ ] Document secrets management

### Phase 8: Audit Preparation (Week 9-10)
- [ ] Internal security review
- [ ] Compile all documentation
- [ ] Create test credentials for auditors
- [ ] Set up isolated audit environment
- [ ] Prepare remediation tracking

## Success Criteria

- [ ] Complete threat model covering all STRIDE categories
- [ ] Security policy published with bug bounty program
- [ ] All cryptographic primitives documented and reviewed
- [ ] 15+ fuzz targets with managed corpus
- [ ] Formal verification for permission and consensus logic
- [ ] SBOM generation in CI pipeline
- [ ] Reproducible builds verified
- [ ] All endpoints have documented permission requirements
- [ ] Key rotation infrastructure tested
- [ ] Audit scope document approved by external firm

## Security Considerations

1. **Audit Firm Selection**: Choose firm with Rust and blockchain experience
2. **Scope Prioritization**: Focus on consensus and cryptography first
3. **Remediation Budget**: Reserve 30% of milestone time for fixes
4. **Disclosure Timeline**: Coordinate with auditors on findings publication
5. **Ongoing Security**: Establish process for continuous security review

## Dependencies

- External security audit firm engagement
- Bug bounty platform (HackerOne, Immunefi)
- HSM hardware or cloud service (optional)
- HashiCorp Vault or similar (optional)

## References

- [OWASP Threat Modeling](https://owasp.org/www-community/Threat_Modeling)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
- [Kani Rust Verifier](https://github.com/model-checking/kani)
- [SLSA Framework](https://slsa.dev/)
