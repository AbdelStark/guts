# ADR-006: REST API Design Principles

## Status

Accepted

## Date

2025-12-20

## Context

Guts nodes expose HTTP APIs for:

1. Git operations (Smart HTTP protocol)
2. Repository management (CRUD)
3. Collaboration features (PRs, Issues)
4. Governance (Orgs, Teams, Permissions)
5. Webhooks and integrations

The API design should be:
- Intuitive for developers familiar with GitHub/GitLab
- RESTful and predictable
- Consistent across all endpoints

## Decision

We adopt a REST API design following these principles:

### URL Structure

```
/api/                                    # API root
├── repos/{owner}/{repo}/               # Repository scope
│   ├── info/refs                       # Git ref advertisement
│   ├── git-upload-pack                 # Git fetch/clone
│   ├── git-receive-pack                # Git push
│   ├── pulls                           # Pull requests
│   │   ├── {number}                    # Single PR
│   │   │   ├── comments                # PR comments
│   │   │   ├── reviews                 # Code reviews
│   │   │   └── merge                   # Merge action
│   ├── issues                          # Issues
│   │   └── {number}                    # Single issue
│   │       └── comments                # Issue comments
│   ├── collaborators                   # Access control
│   ├── hooks                           # Webhooks
│   └── branches/{branch}/protection    # Branch protection
├── orgs/                               # Organizations
│   └── {org}/
│       ├── members                     # Org members
│       └── teams/                      # Teams
│           └── {team}/
│               ├── members             # Team members
│               └── repos               # Team repo access
└── health                              # Health check
```

### HTTP Methods

| Method | Usage | Example |
|--------|-------|---------|
| GET | Read resources | `GET /api/repos/alice/project/pulls` |
| POST | Create resources | `POST /api/repos/alice/project/pulls` |
| PATCH | Partial update | `PATCH /api/repos/alice/project/pulls/1` |
| PUT | Full replace / Upsert | `PUT /api/repos/alice/project/collaborators/bob` |
| DELETE | Remove resources | `DELETE /api/repos/alice/project/hooks/123` |

### Request/Response Format

All requests and responses use JSON:

```rust
// Standard success response
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

// Error response
#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub status: u16,
}
```

### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET, PATCH |
| 201 | Created | Successful POST |
| 204 | No Content | Successful DELETE |
| 400 | Bad Request | Invalid input |
| 401 | Unauthorized | Missing authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | State conflict (e.g., already merged) |
| 422 | Unprocessable | Valid JSON, invalid semantics |
| 500 | Internal Error | Server-side failure |

### Pagination

List endpoints use offset-based pagination:

```
GET /api/repos/alice/project/issues?state=open&page=2&per_page=20

Response:
{
  "data": [...],
  "total": 156,
  "page": 2,
  "per_page": 20,
  "has_more": true
}
```

### Filtering

Query parameters for filtering:

```
GET /api/repos/alice/project/pulls?state=open&author=bob
GET /api/repos/alice/project/issues?labels=bug,urgent
```

## Implementation

Using Axum for HTTP handling:

```rust
// Router setup
pub fn router(state: AppState) -> Router {
    Router::new()
        // Git protocol
        .route("/repos/:owner/:repo/info/refs", get(info_refs))
        .route("/repos/:owner/:repo/git-upload-pack", post(git_upload_pack))
        .route("/repos/:owner/:repo/git-receive-pack", post(git_receive_pack))

        // Collaboration
        .route("/repos/:owner/:repo/pulls", get(list_prs).post(create_pr))
        .route("/repos/:owner/:repo/pulls/:number", get(get_pr).patch(update_pr))
        .route("/repos/:owner/:repo/pulls/:number/merge", post(merge_pr))

        // ... more routes
        .with_state(state)
}
```

## Consequences

### Positive

- **Familiar**: Developers can guess endpoints
- **Toolable**: Works with standard HTTP clients
- **Debuggable**: Human-readable JSON
- **Documented**: OpenAPI spec can be generated

### Negative

- **Chatty**: Multiple requests for related data
- **Overfetching**: Returns full objects even for partial needs
- **No subscriptions**: Polling required for updates

### Neutral

- Authentication header required for private resources
- Rate limiting can be applied uniformly

## Authentication

Currently using header-based identity:

```
X-Guts-Identity: alice
```

Future enhancement: Signature-based authentication:
```
X-Guts-Signature: <ed25519-signature-of-request-hash>
X-Guts-Timestamp: <unix-timestamp>
```

## Alternatives Considered

### GraphQL

Single endpoint with query language.

**Deferred because:**
- Higher complexity
- Git endpoints still need REST
- May add later for web clients

### gRPC

Binary protocol with generated clients.

**Deferred because:**
- Less accessible for debugging
- Browser compatibility requires proxy
- May add for high-performance use cases

### JSON-RPC

Remote procedure call over HTTP.

**Rejected because:**
- Less discoverable
- Non-standard patterns
- REST is more widely understood

## References

- [GitHub REST API](https://docs.github.com/en/rest)
- [RESTful Web APIs (O'Reilly)](https://www.oreilly.com/library/view/restful-web-apis/9781449359713/)
- [HTTP Status Codes (MDN)](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
