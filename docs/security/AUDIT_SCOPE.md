# Security Audit Scope

> Scope document for external security auditors evaluating the Guts codebase.

## Document Information

| Field | Value |
|-------|-------|
| Version | 1.0 |
| Date | 2024-12-21 |
| Classification | Public |
| Contact | security@guts.network |

## Executive Summary

Guts is a decentralized, censorship-resistant code collaboration platform. This document defines the scope for a comprehensive security audit of the Guts codebase, focusing on cryptographic implementations, authentication/authorization, and protocol security.

## Audit Objectives

1. **Identify vulnerabilities** in the Rust codebase
2. **Review cryptographic implementations** for correctness
3. **Assess authentication and authorization** mechanisms
4. **Evaluate protocol security** (Git, P2P, Consensus)
5. **Review input validation** and sanitization
6. **Assess denial of service** resilience

## In Scope

### Crate Overview

| Crate | LOC | Priority | Focus Areas |
|-------|-----|----------|-------------|
| `guts-types` | ~800 | P0 | Identity, signatures, core types |
| `guts-storage` | ~1,200 | P1 | Content-addressed storage |
| `guts-git` | ~2,500 | P0 | Pack file parsing, Git protocol |
| `guts-p2p` | ~1,800 | P0 | P2P networking, replication |
| `guts-collaboration` | ~3,000 | P1 | PRs, issues, comments, reviews |
| `guts-auth` | ~2,200 | P0 | Organizations, teams, permissions |
| `guts-web` | ~1,500 | P2 | Web gateway, HTML rendering |
| `guts-realtime` | ~1,000 | P2 | WebSocket, notifications |
| `guts-ci` | ~2,000 | P2 | CI/CD pipelines |
| `guts-compat` | ~2,500 | P1 | GitHub compatibility, rate limiting |
| `guts-security` | ~1,500 | P0 | Audit logging, key management |
| `guts-node` | ~3,500 | P0 | HTTP API, request handling |
| `guts-cli` | ~1,000 | P3 | Command-line interface |

**Total**: ~24,500 lines of Rust code (excluding tests)

### Priority Areas

#### P0 - Critical (Must Audit)

1. **Cryptographic Operations**
   - Ed25519 signature creation and verification
   - Token hashing with Argon2id
   - Key management and rotation
   - Domain separation in signatures

2. **Authentication**
   - API token validation
   - Session management
   - Git credential handling

3. **Authorization**
   - Permission checking logic
   - RBAC implementation
   - Branch protection enforcement

4. **Protocol Parsing**
   - Git pack file parsing
   - Git reference parsing
   - P2P message parsing
   - Consensus message handling

#### P1 - High (Should Audit)

1. **Input Validation**
   - API request validation
   - Repository name validation
   - Path traversal prevention

2. **Rate Limiting**
   - Per-IP and per-user limits
   - Resource-specific limits
   - Bypass prevention

3. **Audit Logging**
   - Event capture completeness
   - Log integrity protection

#### P2 - Medium (Consider Auditing)

1. **WebSocket Security**
   - Message handling
   - Authentication state

2. **Web Gateway**
   - HTML rendering (XSS prevention)
   - Markdown sanitization

3. **CI/CD**
   - Workflow definition parsing
   - Artifact handling

#### P3 - Low (If Time Permits)

1. **CLI**
   - Credential storage
   - Local file handling

### Specific Components to Review

#### guts-types/src/identity.rs
- Ed25519 key generation
- Signature creation/verification
- Public key serialization

#### guts-auth/src/permission.rs
- Permission level hierarchy
- `has()` permission checking
- Effective permission calculation

#### guts-auth/src/token.rs
- Token generation
- Argon2id hashing
- Token verification (constant-time)

#### guts-git/src/pack.rs
- Pack file parsing
- Delta resolution
- Object extraction

#### guts-node/src/middleware/
- Authentication middleware
- Rate limiting middleware
- Input validation

#### guts-security/src/
- Audit event logging
- Key rotation logic
- Secrets management

### Integration Points

| Interface | Security Relevance |
|-----------|-------------------|
| Git Smart HTTP | Pack file injection, auth bypass |
| REST API | Input validation, authorization |
| WebSocket | Session hijacking, message injection |
| P2P Protocol | Node impersonation, message tampering |
| Consensus | Byzantine fault tolerance |

## Out of Scope

### Explicitly Excluded

1. **Third-Party Dependencies**
   - commonware-* crates (separate audit recommended)
   - axum, tokio, serde (well-audited)
   - Other Cargo dependencies

2. **Infrastructure**
   - Docker configurations
   - Kubernetes manifests
   - Terraform modules
   - CI/CD workflows

3. **Client-Side Code**
   - Minimal JavaScript (template-rendered)
   - CLI local operations

4. **Documentation**
   - Markdown files
   - ADR documents

### Dependencies Worth Noting

| Dependency | Version | Concern |
|------------|---------|---------|
| commonware-cryptography | 0.0.63 | Core crypto operations |
| argon2 | 0.5 | Token hashing |
| rustls | via axum | TLS implementation |
| sha2 | 0.10 | Content hashing |

*Recommendation: Consider separate audit of commonware-* dependencies*

## Test Environment

### Local Setup

```bash
# Clone repository
git clone https://github.com/AbdelStark/guts.git
cd guts

# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Run single node
cargo run --bin guts-node -- --api-addr 127.0.0.1:8080
```

### Devnet Setup

```bash
# Start 5-node devnet
cd infra/docker
docker compose -f docker-compose.devnet.yml up -d

# Nodes available at:
# - http://localhost:8080 (node1)
# - http://localhost:8081 (node2)
# - http://localhost:8082 (node3)
# - http://localhost:8083 (node4)
# - http://localhost:8084 (node5)
```

### Test Accounts

| Purpose | Credentials |
|---------|-------------|
| Admin user | Will be provided |
| Regular user | Will be provided |
| API tokens | Will be generated |

## Deliverables Expected

### From Auditors

1. **Vulnerability Report**
   - Severity classification (Critical/High/Medium/Low/Info)
   - Detailed description of each finding
   - Proof of concept where applicable
   - Recommended remediation

2. **Code Quality Observations**
   - Potential security improvements
   - Best practice recommendations
   - Documentation gaps

3. **Executive Summary**
   - Overall security posture assessment
   - Key findings overview
   - Risk summary

### Report Format

- PDF report with findings
- Markdown version for internal tracking
- CVSS 3.1 scores for vulnerabilities
- CWE identifiers where applicable

## Communication

### Primary Contacts

| Role | Contact |
|------|---------|
| Security Lead | security@guts.network |
| Technical Lead | tech@guts.network |

### Disclosure Timeline

| Phase | Duration |
|-------|----------|
| Audit period | 4-6 weeks |
| Draft report | 1 week after audit |
| Remediation | 2-4 weeks |
| Final report | 1 week after remediation |
| Public disclosure | 90 days from initial report |

### Secure Communication

- PGP encryption for sensitive findings
- Secure file sharing for reports
- No public discussion until coordinated disclosure

## Audit Preparation Checklist

- [x] Complete codebase available on GitHub
- [x] Security documentation prepared
- [x] Threat model documented
- [x] Cryptographic inventory created
- [ ] Test credentials generated
- [ ] Isolated audit environment ready
- [ ] Point of contact available
- [ ] Remediation budget allocated

## Previous Security Work

### Internal Reviews

| Date | Scope | Findings |
|------|-------|----------|
| 2024-Q4 | Initial security review | Internal |

### Automated Tools

| Tool | Purpose | Status |
|------|---------|--------|
| cargo-audit | Dependency vulnerabilities | CI/CD |
| cargo-deny | License and security policy | CI/CD |
| clippy | Lint and security warnings | CI/CD |
| cargo-fuzz | Fuzz testing | Regular |
| proptest | Property-based testing | Regular |

## Questions for Auditors

1. What additional documentation would be helpful?
2. Are there specific attack scenarios to prioritize?
3. What is your preferred communication method?
4. Do you need additional test environment access?

---

*This scope document may be updated based on auditor feedback.*

**Contact**: security@guts.network
