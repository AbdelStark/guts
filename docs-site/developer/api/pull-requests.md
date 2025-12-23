# Pull Requests API

Manage pull requests and code reviews.

## List Pull Requests

```http
GET /api/repos/{owner}/{name}/pulls
```

### Parameters

| Name | Type | Description |
|------|------|-------------|
| `state` | string | `open`, `closed`, `merged`, `all` (default: open) |
| `sort` | string | `created`, `updated` (default: created) |
| `direction` | string | `asc` or `desc` (default: desc) |
| `page` | integer | Page number |
| `per_page` | integer | Items per page |

### Example

```bash
curl "https://api.guts.network/api/repos/alice/my-project/pulls?state=open" \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "items": [
    {
      "id": "pr_abc123",
      "number": 42,
      "title": "Add new feature",
      "body": "This PR adds...",
      "state": "open",
      "author": "bob",
      "source_branch": "feature-branch",
      "target_branch": "main",
      "created_at": "2025-01-15T10:00:00Z",
      "updated_at": "2025-01-15T12:00:00Z",
      "merged_at": null,
      "merge_commit_sha": null,
      "labels": ["enhancement"],
      "reviewers": ["alice"],
      "draft": false
    }
  ],
  "total_count": 1
}
```

## Get a Pull Request

```http
GET /api/repos/{owner}/{name}/pulls/{number}
```

```bash
curl https://api.guts.network/api/repos/alice/my-project/pulls/42 \
  -H "Authorization: Bearer guts_xxx"
```

## Create a Pull Request

```http
POST /api/repos/{owner}/{name}/pulls
```

### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | PR title |
| `body` | string | No | PR description |
| `source_branch` | string | Yes | Source branch |
| `target_branch` | string | Yes | Target branch |
| `draft` | boolean | No | Create as draft |

### Example

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/pulls \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Add awesome feature",
    "body": "This PR implements...",
    "source_branch": "feature-awesome",
    "target_branch": "main"
  }'
```

## Update a Pull Request

```http
PATCH /api/repos/{owner}/{name}/pulls/{number}
```

### Request Body

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | PR title |
| `body` | string | PR description |
| `state` | string | `open` or `closed` |

```bash
curl -X PATCH https://api.guts.network/api/repos/alice/my-project/pulls/42 \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{"title": "Updated title"}'
```

## Merge a Pull Request

```http
POST /api/repos/{owner}/{name}/pulls/{number}/merge
```

### Request Body

| Field | Type | Description |
|-------|------|-------------|
| `merge_method` | string | `merge`, `squash`, `rebase` |
| `commit_title` | string | Custom merge commit title |
| `commit_message` | string | Custom merge commit message |

### Example

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/pulls/42/merge \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "merge_method": "squash",
    "commit_title": "feat: Add awesome feature (#42)"
  }'
```

### Response

```json
{
  "sha": "abc123...",
  "merged": true,
  "message": "Pull Request successfully merged"
}
```

## Reviews

### List Reviews

```http
GET /api/repos/{owner}/{name}/pulls/{number}/reviews
```

```bash
curl https://api.guts.network/api/repos/alice/my-project/pulls/42/reviews \
  -H "Authorization: Bearer guts_xxx"
```

```json
[
  {
    "id": "rev_xyz789",
    "author": "alice",
    "state": "APPROVED",
    "body": "LGTM!",
    "submitted_at": "2025-01-15T14:00:00Z",
    "commit_id": "abc123..."
  }
]
```

### Create a Review

```http
POST /api/repos/{owner}/{name}/pulls/{number}/reviews
```

### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `body` | string | No | Review comment |
| `event` | string | Yes | `APPROVE`, `REQUEST_CHANGES`, `COMMENT` |
| `comments` | array | No | Line-level comments |

### Example

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/pulls/42/reviews \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "body": "Looks good! Just one small suggestion.",
    "event": "APPROVE",
    "comments": [
      {
        "path": "src/main.rs",
        "line": 42,
        "body": "Consider using a more descriptive variable name"
      }
    ]
  }'
```

### Review States

| State | Description |
|-------|-------------|
| `PENDING` | Review in progress |
| `COMMENTED` | General feedback only |
| `APPROVED` | Changes approved |
| `CHANGES_REQUESTED` | Changes required |
| `DISMISSED` | Review dismissed |

## Comments

### List PR Comments

```http
GET /api/repos/{owner}/{name}/pulls/{number}/comments
```

### Create a Comment

```http
POST /api/repos/{owner}/{name}/pulls/{number}/comments
```

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/pulls/42/comments \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "body": "Great work on this feature!"
  }'
```

## Requested Reviewers

### Request Reviewers

```http
POST /api/repos/{owner}/{name}/pulls/{number}/requested_reviewers
```

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/pulls/42/requested_reviewers \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "reviewers": ["charlie", "dave"],
    "team_reviewers": ["core-team"]
  }'
```

### Remove Reviewer Request

```http
DELETE /api/repos/{owner}/{name}/pulls/{number}/requested_reviewers
```

## Files Changed

```http
GET /api/repos/{owner}/{name}/pulls/{number}/files
```

```json
[
  {
    "sha": "abc123...",
    "filename": "src/main.rs",
    "status": "modified",
    "additions": 10,
    "deletions": 2,
    "changes": 12,
    "patch": "@@ -1,5 +1,13 @@..."
  }
]
```

## Commits

```http
GET /api/repos/{owner}/{name}/pulls/{number}/commits
```

```json
[
  {
    "sha": "abc123...",
    "message": "Add new feature",
    "author": {
      "name": "Bob",
      "email": "bob@example.com",
      "date": "2025-01-15T10:00:00Z"
    }
  }
]
```
