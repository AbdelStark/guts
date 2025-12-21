# Known Security Issues and Mitigations

> Transparency document listing known security limitations and their mitigations.

## Overview

This document tracks known security issues, limitations, and their current mitigations. Guts follows a policy of transparency regarding security limitations to help users make informed decisions.

## Issue Classification

| Severity | Description | SLA |
|----------|-------------|-----|
| **Critical** | Remote code execution, data breach | Immediate fix |
| **High** | Auth bypass, privilege escalation | Fix within 7 days |
| **Medium** | Limited impact, requires conditions | Fix within 30 days |
| **Low** | Theoretical, minimal impact | Next release cycle |
| **Info** | Hardening recommendations | No SLA |

## Current Known Issues

### None Currently

*No known security vulnerabilities at this time.*

---

## Accepted Limitations

These are known limitations that are accepted as part of the current design.

### L1: No Encryption at Rest (Default)

**Status**: Accepted Limitation
**Severity**: Low
**Component**: Storage Layer

**Description**:
By default, repository data is stored unencrypted on disk. This is consistent with Git's design and allows for efficient content-addressed storage.

**Risk**:
An attacker with filesystem access can read repository contents.

**Mitigation**:
- Use full-disk encryption (LUKS, FileVault, BitLocker)
- Deploy on encrypted cloud storage
- Future: Optional at-rest encryption feature planned

**Recommendation**:
Always deploy on encrypted storage in production.

---

### L2: Git SHA-1 Collision Theoretical Risk

**Status**: Accepted Limitation
**Severity**: Low
**Component**: Git Protocol

**Description**:
Git historically uses SHA-1 for object addressing. While SHA-1 has known weaknesses (SHAttered attack), the practical risk for Git is mitigated by:
1. Git's SHA-1 implementation detects known collision patterns
2. Guts is transitioning to SHA-256 (Git's object-format=sha256)

**Risk**:
Theoretical collision attacks could allow malicious objects with same hash.

**Mitigation**:
- Git's hardened SHA-1 detects known attacks
- Commit signing provides independent verification
- SHA-256 migration in progress

**Recommendation**:
Enable commit signing for all repositories.

---

### L3: Rate Limiting Bypass via Distributed Attack

**Status**: Accepted Limitation
**Severity**: Medium
**Component**: Rate Limiting

**Description**:
Per-IP rate limiting can be bypassed by distributing requests across many IP addresses (botnet, cloud functions).

**Risk**:
Sophisticated attackers can exceed intended rate limits.

**Mitigation**:
- Per-user rate limiting for authenticated requests
- Per-repository rate limiting
- Adaptive rate limiting based on patterns
- Integration with upstream DDoS protection

**Recommendation**:
Deploy behind a CDN/WAF with DDoS protection (Cloudflare, AWS Shield).

---

### L4: Timing Side-Channels in Non-Critical Paths

**Status**: Accepted Limitation
**Severity**: Low
**Component**: Various

**Description**:
Some non-security-critical code paths may have timing variations that could theoretically leak information.

**Risk**:
Sophisticated timing attacks could leak minor metadata.

**Mitigation**:
- All authentication uses constant-time comparison
- All token verification uses constant-time operations
- Critical permission checks are constant-time

**Recommendation**:
Critical paths have been hardened. Remaining timing variations are in paths where leaked information is not sensitive.

---

### L5: WebSocket Connection Limits

**Status**: Accepted Limitation
**Severity**: Low
**Component**: Real-time

**Description**:
WebSocket connections consume server resources. A large number of idle connections could exhaust resources.

**Risk**:
Resource exhaustion from many WebSocket connections.

**Mitigation**:
- Connection limits per IP
- Idle timeout (30 minutes)
- Heartbeat requirement
- Connection pooling

**Recommendation**:
Monitor WebSocket connection counts and set appropriate limits.

---

## Resolved Issues

Issues that have been fixed in previous releases.

### None Yet

*No resolved security issues to report.*

---

## Reporting New Issues

If you discover a security issue not listed here:

1. **DO NOT** create a public GitHub issue
2. Email security@guts.network with details
3. Include reproduction steps and impact assessment
4. We will respond within 24 hours

See [SECURITY.md](./SECURITY.md) for our full security policy.

---

## Changelog

| Date | Change |
|------|--------|
| 2024-12-21 | Initial document creation |

---

*This document is updated as issues are discovered, mitigated, or resolved.*
