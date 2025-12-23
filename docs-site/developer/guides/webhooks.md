# Webhooks

Subscribe to repository events with webhooks.

## Overview

Webhooks allow you to receive HTTP POST notifications when events occur in your repositories. Use webhooks to:

- Trigger CI/CD pipelines
- Sync with external systems
- Send notifications
- Update dashboards

## Creating a Webhook

### Using the API

```bash
curl -X POST https://api.guts.network/api/repos/owner/repo/hooks \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-server.com/webhook",
    "secret": "your-webhook-secret",
    "events": ["push", "pull_request", "issues"],
    "active": true
  }'
```

### Using the CLI

```bash
guts webhook create owner/repo \
  --url https://your-server.com/webhook \
  --secret your-webhook-secret \
  --events push,pull_request,issues
```

## Available Events

| Event | Description |
|-------|-------------|
| `push` | Any push to the repository |
| `pull_request` | PR opened, closed, merged, updated |
| `pull_request_review` | Review submitted |
| `issues` | Issue opened, closed, updated |
| `issue_comment` | Comment on issue or PR |
| `release` | Release created, published, deleted |
| `create` | Branch or tag created |
| `delete` | Branch or tag deleted |
| `fork` | Repository forked |
| `star` | Repository starred |

## Webhook Payload

All webhook payloads include these headers:

```
X-Guts-Event: push
X-Guts-Delivery: abc123-uuid
X-Guts-Signature-256: sha256=...
Content-Type: application/json
```

### Push Event

```json
{
  "action": "push",
  "ref": "refs/heads/main",
  "before": "abc123...",
  "after": "def456...",
  "repository": {
    "name": "my-repo",
    "owner": "alice"
  },
  "pusher": {
    "name": "alice",
    "email": "alice@example.com"
  },
  "commits": [
    {
      "id": "def456...",
      "message": "Add new feature",
      "author": {
        "name": "Alice",
        "email": "alice@example.com"
      },
      "timestamp": "2025-01-20T12:00:00Z"
    }
  ]
}
```

### Pull Request Event

```json
{
  "action": "opened",
  "number": 42,
  "pull_request": {
    "title": "Add new feature",
    "body": "This PR adds...",
    "state": "open",
    "source_branch": "feature-branch",
    "target_branch": "main",
    "author": "bob",
    "created_at": "2025-01-20T10:00:00Z"
  },
  "repository": {
    "name": "my-repo",
    "owner": "alice"
  }
}
```

### Issue Event

```json
{
  "action": "opened",
  "issue": {
    "number": 15,
    "title": "Bug report",
    "body": "Something is broken...",
    "state": "open",
    "author": "charlie",
    "labels": ["bug"],
    "created_at": "2025-01-20T08:00:00Z"
  },
  "repository": {
    "name": "my-repo",
    "owner": "alice"
  }
}
```

## Verifying Webhooks

Always verify webhook signatures to ensure authenticity:

### Node.js

```javascript
const crypto = require('crypto');

function verifySignature(payload, signature, secret) {
  const expected = 'sha256=' + crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');

  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expected)
  );
}

app.post('/webhook', (req, res) => {
  const signature = req.headers['x-guts-signature-256'];
  const payload = JSON.stringify(req.body);

  if (!verifySignature(payload, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(401).send('Invalid signature');
  }

  // Process the webhook
  const event = req.headers['x-guts-event'];
  console.log(`Received ${event} event`);

  res.status(200).send('OK');
});
```

### Python

```python
import hmac
import hashlib
from flask import Flask, request

app = Flask(__name__)

def verify_signature(payload, signature, secret):
    expected = 'sha256=' + hmac.new(
        secret.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected)

@app.route('/webhook', methods=['POST'])
def webhook():
    signature = request.headers.get('X-Guts-Signature-256')
    payload = request.get_data()

    if not verify_signature(payload, signature, WEBHOOK_SECRET):
        return 'Invalid signature', 401

    event = request.headers.get('X-Guts-Event')
    data = request.get_json()

    print(f'Received {event} event')

    return 'OK', 200
```

## Managing Webhooks

### List Webhooks

```bash
curl https://api.guts.network/api/repos/owner/repo/hooks \
  -H "Authorization: Bearer guts_xxx"
```

### Update Webhook

```bash
curl -X PATCH https://api.guts.network/api/repos/owner/repo/hooks/hook_id \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "events": ["push", "pull_request"],
    "active": true
  }'
```

### Delete Webhook

```bash
curl -X DELETE https://api.guts.network/api/repos/owner/repo/hooks/hook_id \
  -H "Authorization: Bearer guts_xxx"
```

### Test Webhook

```bash
curl -X POST https://api.guts.network/api/repos/owner/repo/hooks/hook_id/test \
  -H "Authorization: Bearer guts_xxx"
```

## Delivery History

View recent webhook deliveries:

```bash
curl https://api.guts.network/api/repos/owner/repo/hooks/hook_id/deliveries \
  -H "Authorization: Bearer guts_xxx"
```

```json
{
  "items": [
    {
      "id": "del_abc123",
      "event": "push",
      "status": "success",
      "status_code": 200,
      "duration_ms": 150,
      "delivered_at": "2025-01-20T12:00:00Z"
    },
    {
      "id": "del_xyz789",
      "event": "pull_request",
      "status": "failed",
      "status_code": 500,
      "duration_ms": 5000,
      "delivered_at": "2025-01-20T11:00:00Z",
      "error": "Connection timeout"
    }
  ]
}
```

### Redeliver

```bash
curl -X POST https://api.guts.network/api/repos/owner/repo/hooks/hook_id/deliveries/del_xyz789/redeliver \
  -H "Authorization: Bearer guts_xxx"
```

## Best Practices

1. **Always verify signatures** - Don't process unverified webhooks
2. **Respond quickly** - Return 200 within 10 seconds
3. **Process asynchronously** - Queue webhook processing
4. **Handle retries** - Webhooks may be delivered multiple times
5. **Use HTTPS** - Always use secure endpoints
6. **Rotate secrets** - Change webhook secrets periodically
