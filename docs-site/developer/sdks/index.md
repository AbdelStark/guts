# Official SDKs

Guts provides official client libraries for popular programming languages.

## Available SDKs

| Language | Package | Installation | Status |
|----------|---------|--------------|--------|
| [TypeScript](/developer/sdks/typescript) | `@guts/sdk` | `npm install @guts/sdk` | Stable |
| [Python](/developer/sdks/python) | `guts-sdk` | `pip install guts-sdk` | Stable |
| Rust | `guts-client` | *Coming soon* | Planned |
| Go | `guts/go` | *Coming soon* | Planned |

## Quick Start

### TypeScript

```typescript
import { GutsClient } from '@guts/sdk';

const client = new GutsClient({
  baseUrl: 'https://api.guts.network',
  token: 'guts_xxx',
});

// List repositories
const repos = await client.repos.list();

// Create an issue
const issue = await client.issues.create('owner', 'repo', {
  title: 'Bug report',
  body: 'Description',
});
```

### Python

```python
from guts import GutsClient

client = GutsClient(
    base_url="https://api.guts.network",
    token="guts_xxx",
)

# List repositories
repos = client.repos.list()

# Create an issue
issue = client.issues.create("owner", "repo", CreateIssueRequest(
    title="Bug report",
    body="Description",
))
```

## Common Features

All official SDKs provide:

| Feature | Description |
|---------|-------------|
| **Type Safety** | Full type definitions |
| **Authentication** | Token-based auth with automatic headers |
| **Error Handling** | Typed exceptions with error details |
| **Pagination** | Automatic handling of paginated results |
| **Rate Limiting** | Automatic retry with exponential backoff |
| **Real-time** | WebSocket event subscriptions |

## Choosing an SDK

### TypeScript SDK

Best for:
- Web applications
- Node.js services
- React/Vue/Svelte apps
- Deno applications

### Python SDK

Best for:
- Scripts and automation
- Data analysis
- CI/CD pipelines
- Backend services

## Building Your Own SDK

If you're building an SDK for a language we don't support:

1. Follow the [API Reference](/developer/api/)
2. Implement authentication per the [Authentication Guide](/developer/api/authentication)
3. Handle pagination and rate limiting
4. Add type definitions for all models
5. Include error handling with typed exceptions
6. Consider WebSocket support for real-time features
7. Submit a PR to list it in the community SDKs!

## Community SDKs

Community-maintained SDKs (not officially supported):

*None yet - [contribute one](https://github.com/AbdelStark/guts/issues)!*
