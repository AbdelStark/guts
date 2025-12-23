# Official SDKs

Guts provides official client libraries for popular programming languages.

## Available SDKs

| Language | Package | Installation | Docs |
|----------|---------|--------------|------|
| TypeScript/JavaScript | `@guts/sdk` | `npm install @guts/sdk` | [TypeScript SDK](typescript.md) |
| Python | `guts-sdk` | `pip install guts-sdk` | [Python SDK](python.md) |
| Rust | `guts-client` | *Coming soon* | - |
| Go | `guts/go` | *Coming soon* | - |

## TypeScript SDK

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

[Full TypeScript Documentation](typescript.md)

## Python SDK

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

[Full Python Documentation](python.md)

## Common Features

All official SDKs provide:

- **Type Safety**: Full type definitions
- **Authentication**: Token-based auth
- **Error Handling**: Typed exceptions
- **Pagination**: Automatic handling
- **Rate Limiting**: Retry with backoff
- **Real-time**: Event subscriptions

## Community SDKs

Community-maintained SDKs:

- None yet - [contribute one](https://github.com/AbdelStark/guts/issues)!

## Building Your Own SDK

If you're building an SDK for a language we don't support:

1. Follow the [API Reference](../api/README.md)
2. Implement authentication per [Authentication Guide](../guides/authentication.md)
3. Handle pagination and rate limiting
4. Add type definitions for all models
5. Include error handling
6. Submit a PR to list it here!
