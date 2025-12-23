# API Reference

The Guts REST API provides programmatic access to all platform features.

## Base URL

```
https://api.guts.network
```

For self-hosted nodes, replace with your node's URL.

## Authentication

Include your token in the Authorization header:

```bash
curl -H "Authorization: Bearer guts_xxx" https://api.guts.network/api/user
```

## Endpoints

### Repositories

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos` | List all repositories |
| POST | `/api/repos` | Create a repository |
| GET | `/api/repos/{owner}/{name}` | Get a repository |
| PATCH | `/api/repos/{owner}/{name}` | Update a repository |
| DELETE | `/api/repos/{owner}/{name}` | Delete a repository |

### Issues

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{name}/issues` | List issues |
| POST | `/api/repos/{owner}/{name}/issues` | Create an issue |
| GET | `/api/repos/{owner}/{name}/issues/{number}` | Get an issue |
| PATCH | `/api/repos/{owner}/{name}/issues/{number}` | Update an issue |

### Pull Requests

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{name}/pulls` | List pull requests |
| POST | `/api/repos/{owner}/{name}/pulls` | Create a pull request |
| GET | `/api/repos/{owner}/{name}/pulls/{number}` | Get a pull request |
| PATCH | `/api/repos/{owner}/{name}/pulls/{number}` | Update a pull request |
| POST | `/api/repos/{owner}/{name}/pulls/{number}/merge` | Merge a pull request |

### Reviews

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{name}/pulls/{number}/reviews` | List reviews |
| POST | `/api/repos/{owner}/{name}/pulls/{number}/reviews` | Create a review |

### Releases

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/repos/{owner}/{name}/releases` | List releases |
| POST | `/api/repos/{owner}/{name}/releases` | Create a release |
| GET | `/api/repos/{owner}/{name}/releases/{id}` | Get a release |
| DELETE | `/api/repos/{owner}/{name}/releases/{id}` | Delete a release |

### Organizations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/orgs` | List organizations |
| POST | `/api/orgs` | Create an organization |
| GET | `/api/orgs/{org}` | Get an organization |
| GET | `/api/orgs/{org}/teams` | List teams |

### Consensus

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/consensus/status` | Get consensus status |
| GET | `/api/consensus/blocks` | List recent blocks |
| GET | `/api/consensus/blocks/{height}` | Get block by height |
| GET | `/api/consensus/validators` | List validators |

## Response Format

All responses are JSON:

```json
{
  "name": "my-repo",
  "owner": "alice",
  "description": "My awesome project",
  "private": false,
  "created_at": "2025-01-01T00:00:00Z"
}
```

## Pagination

List endpoints support pagination:

```
GET /api/repos?page=2&per_page=50
```

Response includes pagination info:

```json
{
  "items": [...],
  "total_count": 100,
  "page": 2,
  "per_page": 50,
  "total_pages": 2
}
```

## Errors

Errors follow this format:

```json
{
  "error": "not_found",
  "message": "Repository not found",
  "details": null
}
```

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | bad_request | Invalid request parameters |
| 401 | unauthorized | Missing or invalid token |
| 403 | forbidden | Insufficient permissions |
| 404 | not_found | Resource not found |
| 422 | validation_error | Validation failed |
| 429 | rate_limited | Rate limit exceeded |
| 500 | internal_error | Server error |

## Rate Limits

Headers included in every response:

```
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4999
X-RateLimit-Reset: 1234567890
X-RateLimit-Used: 1
```
