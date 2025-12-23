# Developer Documentation

Welcome to the Guts developer documentation! This guide will help you integrate with and build on the Guts decentralized code collaboration platform.

## Overview

Guts is a decentralized, censorship-resistant code collaboration platform that provides:

- **Git Compatibility** - Works with standard Git clients
- **GitHub-Compatible API** - Easy migration from GitHub
- **Decentralized Storage** - Content-addressed, replicated across nodes
- **BFT Consensus** - Simplex BFT for total ordering of state changes
- **Real-time Updates** - WebSocket subscriptions for live data

## Quick Links

| Resource | Description |
|----------|-------------|
| [Quickstart](/developer/quickstart/) | Get started in 5 minutes |
| [API Reference](/developer/api/) | Complete REST API documentation |
| [SDKs](/developer/sdks/) | Official client libraries |
| [Guides](/developer/guides/) | Step-by-step tutorials |

## Getting Started

### For Developers

1. **[Create an Identity](/developer/quickstart/)** - Generate your Ed25519 keypair
2. **[Get a Token](/developer/api/authentication)** - Create a personal access token
3. **[Install an SDK](/developer/sdks/)** - Choose TypeScript or Python

### For Tool Authors

1. **[API Overview](/developer/api/)** - Understand the API structure
2. **[Authentication](/developer/api/authentication)** - Implement auth flows
3. **[Webhooks](/developer/guides/webhooks)** - Subscribe to events

## SDKs

| Language | Package | Status |
|----------|---------|--------|
| TypeScript | `@guts/sdk` | Stable |
| Python | `guts-sdk` | Stable |
| Rust | `guts-client` | Planned |
| Go | `github.com/AbdelStark/guts/go` | Planned |

## API Endpoints

The Guts API is organized into these main areas:

- **Repositories** - `/api/repos/*`
- **Issues** - `/api/repos/{owner}/{repo}/issues/*`
- **Pull Requests** - `/api/repos/{owner}/{repo}/pulls/*`
- **Releases** - `/api/repos/{owner}/{repo}/releases/*`
- **Organizations** - `/api/orgs/*`
- **Users** - `/api/users/*`
- **Consensus** - `/api/consensus/*`

## Authentication

Guts supports multiple authentication methods:

```bash
# Bearer token (recommended)
curl -H "Authorization: Bearer guts_xxx" https://api.guts.network/api/user

# Token header (GitHub-style)
curl -H "Authorization: token guts_xxx" https://api.guts.network/api/user

# Basic auth (username:token)
curl -u "alice:guts_xxx" https://api.guts.network/api/user
```

## Rate Limiting

API responses include rate limit headers:

```
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4999
X-RateLimit-Reset: 1234567890
```

- **Authenticated requests**: 5000/hour
- **Unauthenticated requests**: 60/hour

## Getting Help

- [GitHub Issues](https://github.com/AbdelStark/guts/issues) - Report bugs
- [API Reference](/developer/api/) - Detailed API docs
- [Operator Documentation](/operator/) - Deployment guides
