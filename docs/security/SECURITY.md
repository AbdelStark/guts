# Security Policy

> Guts takes security seriously. This document outlines our security practices and vulnerability disclosure process.

## Supported Versions

| Version | Supported          | Notes |
|---------|-------------------|-------|
| 1.x.x   | :white_check_mark: Active | Full security support |
| 0.x.x   | :x: No             | Pre-release, use at own risk |

## Security Principles

Guts is built on the following security principles:

1. **Defense in Depth**: Multiple layers of security controls
2. **Least Privilege**: Minimal permissions by default
3. **Secure by Default**: Safe configurations out of the box
4. **Transparency**: Open source for community review
5. **Cryptographic Integrity**: All operations are cryptographically verifiable

## Reporting a Vulnerability

### DO NOT

- **DO NOT** create public GitHub issues for security vulnerabilities
- **DO NOT** discuss vulnerabilities in public channels (Discord, Twitter, etc.)
- **DO NOT** share vulnerability details before coordinated disclosure

### How to Report

1. **Email**: Send details to `security@guts.network`
2. **PGP Encryption**: Use our PGP key (fingerprint below) for sensitive reports
3. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Suggested remediation (if any)
   - Your contact information for follow-up

### PGP Key

```
Fingerprint: [TO BE GENERATED]
Public Key: https://guts.network/.well-known/security.txt
```

## Response Timeline

| Stage | Timeframe | Description |
|-------|-----------|-------------|
| Acknowledgment | 24 hours | We confirm receipt of your report |
| Assessment | 72 hours | Initial triage and severity assessment |
| Response | 7 days | Detailed response with remediation plan |
| Fix | 30-90 days | Patch development and testing (severity dependent) |
| Disclosure | 90 days | Coordinated public disclosure |

## Severity Classification

We use the following severity levels based on CVSS 3.1:

| Severity | CVSS Score | Description |
|----------|------------|-------------|
| **Critical** | 9.0 - 10.0 | Remote code execution, complete system compromise |
| **High** | 7.0 - 8.9 | Significant data breach, privilege escalation |
| **Medium** | 4.0 - 6.9 | Limited impact, requires user interaction |
| **Low** | 0.1 - 3.9 | Minimal impact, theoretical attacks |

## Bug Bounty Program

Guts operates a bug bounty program to reward security researchers:

| Severity | Reward Range |
|----------|--------------|
| Critical | $5,000 - $25,000 |
| High | $2,000 - $5,000 |
| Medium | $500 - $2,000 |
| Low | $100 - $500 |

### Eligible Vulnerabilities

- Remote code execution
- Authentication bypass
- Authorization bypass
- Cryptographic vulnerabilities
- Consensus protocol attacks
- P2P network attacks
- Data integrity violations
- Denial of service (with significant impact)

### Out of Scope

- Social engineering attacks
- Physical attacks
- Vulnerabilities in dependencies (report to upstream)
- Self-XSS
- Rate limiting bypass (unless leading to significant DoS)
- Missing security headers (unless exploitable)
- Theoretical attacks without proof of concept

### Rules of Engagement

1. **No Destructive Testing**: Do not delete data or disrupt services
2. **No Privacy Violations**: Do not access others' private data
3. **Test Accounts Only**: Use test repositories and accounts
4. **Good Faith**: Act in good faith throughout the process
5. **Coordination**: Work with us on disclosure timing

## Security Updates

Security updates are distributed through:

1. **GitHub Security Advisories**: [github.com/AbdelStark/guts/security/advisories](https://github.com/AbdelStark/guts/security/advisories)
2. **Release Notes**: Security fixes noted in release changelog
3. **Security Mailing List**: Subscribe at security-announce@guts.network

## Security Audit History

| Date | Auditor | Scope | Report |
|------|---------|-------|--------|
| TBD | TBD | Full codebase | [Link] |

## Contact

- **Security Email**: security@guts.network
- **Response Time**: 24 hours for acknowledgment
- **PGP Key**: Available at https://guts.network/.well-known/security.txt

## Acknowledgments

We thank the following researchers for responsible disclosure:

*No vulnerabilities reported yet.*

---

*This security policy is reviewed and updated quarterly.*
