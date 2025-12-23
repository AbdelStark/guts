# Repositories API

Manage Git repositories on Guts.

## List Repositories

```http
GET /api/repos
```

### Parameters

| Name | Type | Description |
|------|------|-------------|
| `page` | integer | Page number (default: 1) |
| `per_page` | integer | Items per page (default: 30, max: 100) |
| `sort` | string | Sort by: `created`, `updated`, `name` |
| `direction` | string | `asc` or `desc` |

### Example

```bash
curl https://api.guts.network/api/repos \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "items": [
    {
      "id": "repo_abc123",
      "name": "my-project",
      "owner": "alice",
      "description": "My awesome project",
      "private": false,
      "default_branch": "main",
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-15T12:00:00Z",
      "pushed_at": "2025-01-15T12:00:00Z"
    }
  ],
  "total_count": 1,
  "page": 1,
  "per_page": 30
}
```

## Get a Repository

```http
GET /api/repos/{owner}/{name}
```

### Example

```bash
curl https://api.guts.network/api/repos/alice/my-project \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "id": "repo_abc123",
  "name": "my-project",
  "owner": "alice",
  "description": "My awesome project",
  "private": false,
  "default_branch": "main",
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-15T12:00:00Z",
  "pushed_at": "2025-01-15T12:00:00Z",
  "size": 1024,
  "open_issues_count": 5,
  "open_prs_count": 2,
  "clone_url": "https://guts.network/alice/my-project.git"
}
```

## Create a Repository

```http
POST /api/repos
```

### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Repository name |
| `owner` | string | Yes | Owner (username or org) |
| `description` | string | No | Short description |
| `private` | boolean | No | Private repository (default: false) |
| `default_branch` | string | No | Default branch (default: main) |

### Example

```bash
curl -X POST https://api.guts.network/api/repos \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-project",
    "owner": "alice",
    "description": "A new project",
    "private": false
  }'
```

### Response

```json
{
  "id": "repo_xyz789",
  "name": "new-project",
  "owner": "alice",
  "description": "A new project",
  "private": false,
  "default_branch": "main",
  "created_at": "2025-01-20T10:00:00Z",
  "clone_url": "https://guts.network/alice/new-project.git"
}
```

## Update a Repository

```http
PATCH /api/repos/{owner}/{name}
```

### Request Body

| Field | Type | Description |
|-------|------|-------------|
| `description` | string | Short description |
| `private` | boolean | Private repository |
| `default_branch` | string | Default branch |

### Example

```bash
curl -X PATCH https://api.guts.network/api/repos/alice/my-project \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated description"
  }'
```

## Delete a Repository

```http
DELETE /api/repos/{owner}/{name}
```

::: danger
This action cannot be undone. All data will be permanently deleted.
:::

### Example

```bash
curl -X DELETE https://api.guts.network/api/repos/alice/my-project \
  -H "Authorization: Bearer guts_xxx"
```

## Repository Contents

### Get File Contents

```http
GET /api/repos/{owner}/{name}/contents/{path}
```

#### Parameters

| Name | Type | Description |
|------|------|-------------|
| `ref` | string | Branch, tag, or commit SHA |

#### Example

```bash
curl "https://api.guts.network/api/repos/alice/my-project/contents/README.md?ref=main" \
  -H "Authorization: Bearer guts_xxx"
```

#### Response

```json
{
  "name": "README.md",
  "path": "README.md",
  "type": "file",
  "size": 1024,
  "sha": "abc123...",
  "content": "IyBNeSBQcm9qZWN0Cg==",
  "encoding": "base64"
}
```

### Get Directory Contents

```bash
curl "https://api.guts.network/api/repos/alice/my-project/contents/src" \
  -H "Authorization: Bearer guts_xxx"
```

```json
[
  {
    "name": "main.rs",
    "path": "src/main.rs",
    "type": "file",
    "size": 256,
    "sha": "def456..."
  },
  {
    "name": "lib",
    "path": "src/lib",
    "type": "dir"
  }
]
```

## Branches

### List Branches

```http
GET /api/repos/{owner}/{name}/branches
```

```bash
curl https://api.guts.network/api/repos/alice/my-project/branches \
  -H "Authorization: Bearer guts_xxx"
```

```json
[
  {
    "name": "main",
    "commit": {
      "sha": "abc123...",
      "message": "Latest commit"
    },
    "protected": true
  },
  {
    "name": "develop",
    "commit": {
      "sha": "def456...",
      "message": "Feature work"
    },
    "protected": false
  }
]
```

### Get Branch Protection

```http
GET /api/repos/{owner}/{name}/branches/{branch}/protection
```

```json
{
  "required_reviews": 2,
  "require_pr": true,
  "dismiss_stale_reviews": true,
  "require_code_owner_review": false
}
```

### Set Branch Protection

```http
PUT /api/repos/{owner}/{name}/branches/{branch}/protection
```

```bash
curl -X PUT "https://api.guts.network/api/repos/alice/my-project/branches/main/protection" \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "required_reviews": 2,
    "require_pr": true
  }'
```

## Collaborators

### List Collaborators

```http
GET /api/repos/{owner}/{name}/collaborators
```

### Add Collaborator

```http
PUT /api/repos/{owner}/{name}/collaborators/{username}
```

```bash
curl -X PUT "https://api.guts.network/api/repos/alice/my-project/collaborators/bob" \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{"permission": "write"}'
```

### Remove Collaborator

```http
DELETE /api/repos/{owner}/{name}/collaborators/{username}
```

## Archives

### Download Archive

```http
GET /api/repos/{owner}/{name}/archive/{format}/{ref}
```

| Format | Description |
|--------|-------------|
| `zipball` | ZIP archive |
| `tarball` | Tarball (.tar.gz) |

```bash
curl -OJ "https://api.guts.network/api/repos/alice/my-project/archive/zipball/main" \
  -H "Authorization: Bearer guts_xxx"
```
