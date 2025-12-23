# Authentication

Guts supports multiple authentication methods for API access.

## Authentication Methods

### Bearer Token (Recommended)

```bash
curl -H "Authorization: Bearer guts_xxx" \
  https://api.guts.network/api/user
```

### Token Header (GitHub-style)

```bash
curl -H "Authorization: token guts_xxx" \
  https://api.guts.network/api/user
```

### Basic Auth

Use your username and token:

```bash
curl -u "alice:guts_xxx" \
  https://api.guts.network/api/user
```

## Personal Access Tokens

### Creating a Token

::: code-group

```bash [CLI]
guts auth token create --name "my-app" --scopes repo,user
```

```bash [API]
curl -X POST https://api.guts.network/api/tokens \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-app",
    "scopes": ["repo", "user"],
    "expires_at": "2025-12-31T23:59:59Z"
  }'
```

:::

### Token Scopes

| Scope | Description |
|-------|-------------|
| `repo` | Full access to repositories |
| `repo:read` | Read-only repository access |
| `user` | Read/write user profile |
| `user:read` | Read-only user profile |
| `org` | Manage organizations |
| `org:read` | Read organization info |
| `admin` | Full administrative access |

### Token Response

```json
{
  "id": "tok_abc123",
  "name": "my-app",
  "token": "guts_xxxxxxxxxxxxx",
  "scopes": ["repo", "user"],
  "created_at": "2025-01-01T00:00:00Z",
  "expires_at": "2025-12-31T23:59:59Z",
  "last_used_at": null
}
```

::: warning
The `token` value is only shown once. Store it securely.
:::

### Listing Tokens

```bash
curl https://api.guts.network/api/tokens \
  -H "Authorization: Bearer guts_xxx"
```

### Revoking a Token

```bash
curl -X DELETE https://api.guts.network/api/tokens/tok_abc123 \
  -H "Authorization: Bearer guts_xxx"
```

## Git Credential Helper

For seamless Git operations, install the credential helper:

```bash
# Install
cargo install git-credential-guts

# Configure Git to use it
git config --global credential.helper guts
```

The credential helper:
- Stores tokens securely in your system keyring
- Automatically provides credentials for Git operations
- Supports multiple accounts

## SSH Keys

### Adding an SSH Key

```bash
curl -X POST https://api.guts.network/api/user/keys \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My laptop",
    "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA..."
  }'
```

### Listing SSH Keys

```bash
curl https://api.guts.network/api/user/keys \
  -H "Authorization: Bearer guts_xxx"
```

### Deleting an SSH Key

```bash
curl -X DELETE https://api.guts.network/api/user/keys/key_123 \
  -H "Authorization: Bearer guts_xxx"
```

## Rate Limiting

Rate limits are applied per token:

| Token Type | Limit |
|------------|-------|
| Authenticated | 5000/hour |
| Unauthenticated | 60/hour |

Rate limit headers:

```
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4999
X-RateLimit-Reset: 1234567890
X-RateLimit-Used: 1
```

When rate limited, you'll receive:

```json
{
  "error": "rate_limited",
  "message": "API rate limit exceeded",
  "retry_after": 3600
}
```

## Security Best Practices

1. **Use minimal scopes** - Only request the scopes you need
2. **Set expiration** - Use short-lived tokens when possible
3. **Rotate regularly** - Rotate tokens periodically
4. **Use environment variables** - Never hardcode tokens
5. **Revoke unused tokens** - Clean up tokens you no longer need

```bash
# Good: Use environment variable
export GUTS_TOKEN="guts_xxx"
curl -H "Authorization: Bearer $GUTS_TOKEN" ...

# Bad: Hardcoded token
curl -H "Authorization: Bearer guts_xxx" ...
```
