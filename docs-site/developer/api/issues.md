# Issues API

Manage issues and comments.

## List Issues

```http
GET /api/repos/{owner}/{name}/issues
```

### Parameters

| Name | Type | Description |
|------|------|-------------|
| `state` | string | `open`, `closed`, `all` (default: open) |
| `labels` | string | Comma-separated label names |
| `assignee` | string | Filter by assignee |
| `creator` | string | Filter by creator |
| `sort` | string | `created`, `updated`, `comments` |
| `direction` | string | `asc` or `desc` |
| `page` | integer | Page number |
| `per_page` | integer | Items per page |

### Example

```bash
curl "https://api.guts.network/api/repos/alice/my-project/issues?state=open&labels=bug" \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "items": [
    {
      "id": "issue_abc123",
      "number": 15,
      "title": "Bug: Something is broken",
      "body": "When I try to...",
      "state": "open",
      "author": "bob",
      "assignees": ["alice"],
      "labels": [
        {
          "name": "bug",
          "color": "d73a4a",
          "description": "Something isn't working"
        }
      ],
      "comments_count": 3,
      "created_at": "2025-01-10T08:00:00Z",
      "updated_at": "2025-01-12T10:00:00Z",
      "closed_at": null
    }
  ],
  "total_count": 1
}
```

## Get an Issue

```http
GET /api/repos/{owner}/{name}/issues/{number}
```

```bash
curl https://api.guts.network/api/repos/alice/my-project/issues/15 \
  -H "Authorization: Bearer guts_xxx"
```

## Create an Issue

```http
POST /api/repos/{owner}/{name}/issues
```

### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Issue title |
| `body` | string | No | Issue description (Markdown) |
| `assignees` | array | No | Usernames to assign |
| `labels` | array | No | Label names |

### Example

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/issues \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Feature request: Add dark mode",
    "body": "It would be great to have a dark mode option...",
    "labels": ["enhancement"],
    "assignees": ["alice"]
  }'
```

### Response

```json
{
  "id": "issue_xyz789",
  "number": 16,
  "title": "Feature request: Add dark mode",
  "body": "It would be great to have a dark mode option...",
  "state": "open",
  "author": "bob",
  "assignees": ["alice"],
  "labels": [
    {
      "name": "enhancement",
      "color": "a2eeef"
    }
  ],
  "created_at": "2025-01-20T10:00:00Z"
}
```

## Update an Issue

```http
PATCH /api/repos/{owner}/{name}/issues/{number}
```

### Request Body

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Issue title |
| `body` | string | Issue description |
| `state` | string | `open` or `closed` |
| `assignees` | array | Usernames to assign |
| `labels` | array | Label names |

### Example

```bash
curl -X PATCH https://api.guts.network/api/repos/alice/my-project/issues/15 \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "state": "closed"
  }'
```

## Comments

### List Comments

```http
GET /api/repos/{owner}/{name}/issues/{number}/comments
```

```bash
curl https://api.guts.network/api/repos/alice/my-project/issues/15/comments \
  -H "Authorization: Bearer guts_xxx"
```

```json
[
  {
    "id": "comment_abc123",
    "body": "I can reproduce this issue...",
    "author": "charlie",
    "created_at": "2025-01-10T09:00:00Z",
    "updated_at": "2025-01-10T09:00:00Z"
  }
]
```

### Create a Comment

```http
POST /api/repos/{owner}/{name}/issues/{number}/comments
```

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/issues/15/comments \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "body": "Thanks for reporting! I will look into this."
  }'
```

### Update a Comment

```http
PATCH /api/repos/{owner}/{name}/issues/comments/{comment_id}
```

### Delete a Comment

```http
DELETE /api/repos/{owner}/{name}/issues/comments/{comment_id}
```

## Labels

### List Labels

```http
GET /api/repos/{owner}/{name}/labels
```

```json
[
  {
    "name": "bug",
    "color": "d73a4a",
    "description": "Something isn't working"
  },
  {
    "name": "enhancement",
    "color": "a2eeef",
    "description": "New feature or request"
  },
  {
    "name": "documentation",
    "color": "0075ca",
    "description": "Improvements or additions to documentation"
  }
]
```

### Create a Label

```http
POST /api/repos/{owner}/{name}/labels
```

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/labels \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "priority-high",
    "color": "ff0000",
    "description": "High priority issue"
  }'
```

### Update a Label

```http
PATCH /api/repos/{owner}/{name}/labels/{name}
```

### Delete a Label

```http
DELETE /api/repos/{owner}/{name}/labels/{name}
```

## Assignees

### Add Assignees

```http
POST /api/repos/{owner}/{name}/issues/{number}/assignees
```

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/issues/15/assignees \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "assignees": ["alice", "bob"]
  }'
```

### Remove Assignees

```http
DELETE /api/repos/{owner}/{name}/issues/{number}/assignees
```

## Milestones

### List Milestones

```http
GET /api/repos/{owner}/{name}/milestones
```

### Create a Milestone

```http
POST /api/repos/{owner}/{name}/milestones
```

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/milestones \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "v1.0",
    "description": "First stable release",
    "due_on": "2025-03-01T00:00:00Z"
  }'
```

### Update a Milestone

```http
PATCH /api/repos/{owner}/{name}/milestones/{number}
```

### Delete a Milestone

```http
DELETE /api/repos/{owner}/{name}/milestones/{number}
```
