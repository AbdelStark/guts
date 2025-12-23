# Guts Developer Documentation

Welcome to the Guts developer documentation! This guide will help you integrate with and build on the Guts decentralized code collaboration platform.

## Quick Links

| Resource | Description |
|----------|-------------|
| [Quickstart](quickstart/README.md) | Get started in 5 minutes |
| [API Reference](api/README.md) | Complete REST API documentation |
| [SDKs](sdks/README.md) | Official client libraries |
| [Guides](guides/README.md) | Step-by-step tutorials |
| [Integrations](integrations/README.md) | IDE and tool integrations |

## Overview

Guts is a decentralized, censorship-resistant code collaboration platform. It provides:

- **Git Compatibility**: Works with standard Git clients
- **GitHub-Compatible API**: Easy migration from GitHub
- **Decentralized Storage**: Content-addressed, replicated across nodes
- **BFT Consensus**: Simplex BFT for total ordering of state changes
- **Real-time Updates**: WebSocket subscriptions for live data

## Getting Started

### For Developers

1. **[Create an Account](quickstart/account.md)** - Set up your identity
2. **[Get a Token](quickstart/tokens.md)** - Generate a personal access token
3. **[Install an SDK](sdks/README.md)** - Choose your language

### For Tool Authors

1. **[API Overview](api/README.md)** - Understand the API structure
2. **[Authentication](guides/authentication.md)** - Implement auth flows
3. **[Webhooks](guides/webhooks.md)** - Subscribe to events

### For Operators

See the [Operator Documentation](../operator/README.md) for deployment guides.

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
- [API Reference](api/README.md) - Detailed API docs
- [FAQ](guides/faq.md) - Common questions

## Contributing

We welcome contributions! See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
