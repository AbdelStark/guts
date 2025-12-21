# Guts Threat Model

> Comprehensive threat analysis using STRIDE methodology for the Guts decentralized code collaboration platform.

## Overview

Guts is a decentralized, censorship-resistant code collaboration platform. This threat model identifies potential threats, attack vectors, and mitigations across all system components.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         External Actors                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │
│  │ Git      │  │ Web      │  │ API      │  │ Malicious            │ │
│  │ Clients  │  │ Browsers │  │ Clients  │  │ Actors               │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────┬───────────┘ │
└───────┼─────────────┼─────────────┼────────────────────┼────────────┘
        │             │             │                    │
        ▼             ▼             ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Network Boundary                               │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Rate Limiting / WAF                          ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
        │             │             │                    │
        ▼             ▼             ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Guts Node                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │
│  │ Git      │  │ Web      │  │ REST     │  │ WebSocket            │ │
│  │ Protocol │  │ Gateway  │  │ API      │  │ Server               │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────┬───────────┘ │
│       │             │             │                    │             │
│       ▼             ▼             ▼                    ▼             │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Authentication / Authorization               ││
│  └─────────────────────────────────────────────────────────────────┘│
│       │             │             │                    │             │
│       ▼             ▼             ▼                    ▼             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │
│  │ Storage  │  │ Collab   │  │ CI/CD    │  │ Realtime             │ │
│  │ Layer    │  │ Layer    │  │ Layer    │  │ Layer                │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────┬───────────┘ │
└───────┼─────────────┼─────────────┼────────────────────┼────────────┘
        │             │             │                    │
        ▼             ▼             ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      P2P Network Layer                               │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Consensus Protocol                           ││
│  └─────────────────────────────────────────────────────────────────┘│
│       │             │             │                    │             │
│       ▼             ▼             ▼                    ▼             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │
│  │ Node 1   │◄─►│ Node 2   │◄─►│ Node 3   │◄─►│ Node N             │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## STRIDE Threat Analysis

### 1. Spoofing (Identity)

| Threat | Attack Vector | Impact | Likelihood | Mitigation |
|--------|---------------|--------|------------|------------|
| **S1**: Identity impersonation | Attacker claims to be another user | High | Medium | Ed25519 signatures on all operations |
| **S2**: Forged commits | Commit with false author | High | Medium | Commit signing enforcement |
| **S3**: Node impersonation | Malicious node pretends to be trusted | High | Low | Node identity verification via consensus |
| **S4**: Token theft | Stolen API tokens | High | Medium | Token hashing with Argon2, rotation |
| **S5**: Session hijacking | WebSocket session takeover | Medium | Low | Secure session tokens, TLS |

#### Mitigations Implemented

```rust
// All user operations are signed with Ed25519
pub fn sign_operation(private_key: &PrivateKey, operation: &Operation) -> Signature {
    // NAMESPACE prevents replay attacks across different contexts
    let message = [NAMESPACE, operation.as_bytes()].concat();
    private_key.sign(&message)
}

// Token storage uses Argon2id
pub fn hash_token(token: &str) -> String {
    let config = argon2::Config::default();
    argon2::hash_encoded(token.as_bytes(), &salt, &config).unwrap()
}
```

---

### 2. Tampering (Integrity)

| Threat | Attack Vector | Impact | Likelihood | Mitigation |
|--------|---------------|--------|------------|------------|
| **T1**: Repository data modification | Modify stored git objects | Critical | Low | Content-addressed storage (SHA-256) |
| **T2**: Consensus message tampering | Modify in-flight consensus messages | Critical | Low | BFT consensus, message signatures |
| **T3**: API response tampering | MITM modifies API responses | High | Low | TLS 1.3 everywhere |
| **T4**: Pack file injection | Malicious objects in git pack | High | Medium | Pack file validation before storage |
| **T5**: History rewriting | Force-push to rewrite history | Medium | Medium | Branch protection rules |

#### Mitigations Implemented

```rust
// Content-addressed storage ensures integrity
pub fn store_object(content: &[u8]) -> ObjectId {
    let hash = sha256(content);
    // Object ID IS the hash - any tampering changes the ID
    storage.put(hash, content);
    hash
}

// Branch protection prevents unauthorized changes
pub struct BranchProtection {
    pub require_pr: bool,
    pub required_reviews: u32,
    pub require_signed_commits: bool,
    pub restrict_force_push: bool,
}
```

---

### 3. Repudiation (Non-repudiation)

| Threat | Attack Vector | Impact | Likelihood | Mitigation |
|--------|---------------|--------|------------|------------|
| **R1**: Denied actions | User denies performing action | Medium | Medium | Comprehensive audit logging |
| **R2**: Unsigned operations | Operations without cryptographic proof | Medium | Low | All mutations require signatures |
| **R3**: Log tampering | Modify audit logs | High | Low | Append-only log with signed entries |
| **R4**: Timestamp manipulation | Forge operation timestamps | Low | Low | Consensus-based timestamps |

#### Mitigations Implemented

```rust
// All security events are logged
pub struct AuditEvent {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub actor: PublicKey,
    pub resource: String,
    pub action: String,
    pub result: Result<(), String>,
    pub metadata: serde_json::Value,
}

// Audit log entries are signed
impl AuditLog {
    pub fn record(&mut self, event: AuditEvent, signer: &PrivateKey) {
        let signature = signer.sign(&event.canonical_bytes());
        self.append(SignedAuditEvent { event, signature });
    }
}
```

---

### 4. Information Disclosure (Confidentiality)

| Threat | Attack Vector | Impact | Likelihood | Mitigation |
|--------|---------------|--------|------------|------------|
| **I1**: Private repo exposure | Unauthorized access to private repos | Critical | Medium | RBAC, permission checks on all endpoints |
| **I2**: Key exposure | Private keys leaked | Critical | Low | Secure key storage, HSM support |
| **I3**: Token leakage | API tokens in logs/errors | High | Medium | Token redaction in logs |
| **I4**: Metadata leakage | Repo existence leakage | Low | Medium | 404 for unauthorized repos |
| **I5**: Side-channel attacks | Timing attacks on auth | Medium | Low | Constant-time comparison |

#### Mitigations Implemented

```rust
// Permission checks on every endpoint
pub async fn get_repository(
    auth: AuthContext,
    path: Path<(String, String)>,
) -> Result<Json<Repository>, ApiError> {
    let (owner, name) = path.into_inner();

    // Check read permission before any operation
    require_permission(&auth, &owner, &name, Permission::Read)?;

    // Only return data if authorized
    storage.get_repository(&owner, &name)
}

// Constant-time token comparison
pub fn verify_token(provided: &str, stored_hash: &str) -> bool {
    argon2::verify_encoded(stored_hash, provided.as_bytes())
        .unwrap_or(false)  // Constant time even on error
}
```

---

### 5. Denial of Service (Availability)

| Threat | Attack Vector | Impact | Likelihood | Mitigation |
|--------|---------------|--------|------------|------------|
| **D1**: API flooding | Excessive API requests | High | High | Rate limiting (per-IP, per-user) |
| **D2**: Large push attacks | Pushing extremely large packs | High | Medium | Pack size limits, streaming validation |
| **D3**: Consensus stalling | Malicious nodes delay consensus | Medium | Low | BFT timeout mechanisms |
| **D4**: Connection exhaustion | Many idle connections | Medium | Medium | Connection limits, timeouts |
| **D5**: Storage exhaustion | Fill disk with objects | High | Medium | Quotas, deduplication |
| **D6**: WebSocket flooding | Excessive WebSocket messages | Medium | Medium | Message rate limiting |

#### Mitigations Implemented

```rust
// Multi-layer rate limiting
pub struct RateLimiter {
    // Per-IP limits for unauthenticated requests
    ip_limits: HashMap<IpAddr, TokenBucket>,

    // Per-user limits for authenticated requests
    user_limits: HashMap<UserId, TokenBucket>,

    // Per-repository limits
    repo_limits: HashMap<RepoKey, TokenBucket>,
}

// Request size limits
pub const MAX_PACK_SIZE: usize = 100 * 1024 * 1024;  // 100 MB
pub const MAX_API_BODY_SIZE: usize = 10 * 1024 * 1024;  // 10 MB
pub const MAX_WEBSOCKET_MESSAGE: usize = 64 * 1024;  // 64 KB
```

---

### 6. Elevation of Privilege

| Threat | Attack Vector | Impact | Likelihood | Mitigation |
|--------|---------------|--------|------------|------------|
| **E1**: Admin access bypass | Gain admin without authorization | Critical | Low | Strict RBAC enforcement |
| **E2**: Org owner escalation | Member becomes owner | High | Low | Role transition validation |
| **E3**: Cross-repo access | Access repos without permission | High | Medium | Repo-scoped permission checks |
| **E4**: API scope bypass | Token used beyond its scope | Medium | Medium | Scope validation on every request |
| **E5**: Branch protection bypass | Push without required reviews | Medium | Low | Server-side enforcement |

#### Mitigations Implemented

```rust
// Permission hierarchy with strict enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Read = 0,
    Write = 1,
    Admin = 2,
}

impl Permission {
    pub fn has(self, required: Permission) -> bool {
        self >= required
    }
}

// Every mutation checks appropriate permission level
pub fn require_admin(auth: &AuthContext, resource: &str) -> Result<(), AuthError> {
    let permission = get_effective_permission(auth, resource)?;
    if !permission.has(Permission::Admin) {
        return Err(AuthError::InsufficientPermission);
    }
    Ok(())
}
```

---

## Attack Surface Analysis

### External Attack Surfaces

| Surface | Entry Point | Protocol | Risk Level |
|---------|-------------|----------|------------|
| REST API | `POST/GET/PUT/DELETE /api/*` | HTTPS | High |
| Git Protocol | `GET/POST /git/*` | HTTPS | High |
| Web Gateway | `GET /{owner}/{repo}/*` | HTTPS | Medium |
| WebSocket | `WS /realtime/*` | WSS | Medium |

### Internal Attack Surfaces

| Surface | Entry Point | Protocol | Risk Level |
|---------|-------------|----------|------------|
| P2P Network | Peer connections | Noise/TLS | High |
| Consensus | BFT messages | Internal | Critical |
| Storage | Object operations | Internal | Medium |

### Input Validation Requirements

| Input Type | Validation | Max Size |
|------------|------------|----------|
| JSON bodies | Schema validation | 10 MB |
| Git pack files | Format validation | 100 MB |
| Repository names | `[a-zA-Z0-9._-]+` | 100 chars |
| Branch names | Git ref format | 256 chars |
| Commit messages | UTF-8 validation | 64 KB |
| File paths | Path traversal check | 4 KB |

---

## Trust Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                    Untrusted Zone                                │
│  External Users, Git Clients, Web Browsers, API Clients         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
           ┌─────────────────────────────────────────┐
           │         Trust Boundary 1                 │
           │    (Authentication Required)             │
           └─────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Authenticated Zone                              │
│  Validated users with verified identity                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
           ┌─────────────────────────────────────────┐
           │         Trust Boundary 2                 │
           │    (Authorization Required)              │
           └─────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Authorized Zone                                 │
│  Users with verified permissions for specific resources          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
           ┌─────────────────────────────────────────┐
           │         Trust Boundary 3                 │
           │      (Node-to-Node Trust)                │
           └─────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Consensus Zone                                  │
│  Nodes participating in BFT consensus                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Threat Scenarios

### Scenario 1: Malicious Repository Owner

**Attacker Profile**: Authenticated user with write access to a repository

**Attack Chain**:
1. Push malicious code disguised as legitimate update
2. Attempt to bypass branch protection
3. Modify webhooks to exfiltrate data

**Mitigations**:
- Require code reviews before merge
- Audit all webhook modifications
- Limit webhook destinations

### Scenario 2: Sybil Attack on Network

**Attacker Profile**: Operator of multiple malicious nodes

**Attack Chain**:
1. Spin up many nodes to gain network influence
2. Attempt to partition the network
3. Delay or prevent consensus

**Mitigations**:
- BFT consensus tolerates up to f Byzantine nodes
- Node identity tied to stake/reputation
- Network monitoring for anomalies

### Scenario 3: Supply Chain Attack

**Attacker Profile**: External actor targeting dependencies

**Attack Chain**:
1. Compromise upstream dependency
2. Inject malicious code via dependency update
3. Gain code execution in Guts nodes

**Mitigations**:
- Dependency auditing with cargo-audit
- Lock file enforcement
- SBOM generation and monitoring
- Reproducible builds

---

## Security Controls Summary

| Control Category | Controls Implemented |
|-----------------|---------------------|
| **Authentication** | Ed25519 signatures, API tokens (Argon2), Session management |
| **Authorization** | RBAC, Permission levels, Branch protection |
| **Input Validation** | Schema validation, Size limits, Format checks |
| **Cryptography** | Ed25519 (signatures), SHA-256 (content), TLS 1.3 (transport) |
| **Logging** | Audit logs, Security events, Error tracking |
| **Rate Limiting** | Per-IP, Per-user, Per-repository limits |
| **Network Security** | TLS everywhere, Noise protocol for P2P |
| **Consensus Security** | BFT protocol, Message signing, Timeout handling |

---

## Recommendations

### High Priority

1. **HSM Integration**: Store signing keys in hardware security modules
2. **Key Rotation**: Implement automated key rotation with overlap periods
3. **Secrets Management**: Integrate with HashiCorp Vault for secret storage
4. **Extended Fuzzing**: Fuzz all protocol parsers continuously
5. **Security Audit**: Engage external firm for comprehensive audit

### Medium Priority

1. **MFA Support**: Add TOTP/WebAuthn for critical operations
2. **IP Allowlisting**: Allow users to restrict API access by IP
3. **Anomaly Detection**: ML-based detection of suspicious patterns
4. **Incident Response**: Documented runbooks for security incidents

### Low Priority

1. **Bug Bounty Platform**: Integrate with HackerOne/Immunefi
2. **Security Training**: Documentation for operators on security best practices
3. **Compliance**: SOC 2 Type II preparation

---

## Review Schedule

This threat model is reviewed:
- **Quarterly**: Regular review for new threats
- **On Major Release**: Before each major version
- **After Incidents**: Post-incident analysis updates
- **After Audits**: Incorporate audit findings

---

*Last Updated: 2024-12-21*
*Next Review: 2025-03-21*
