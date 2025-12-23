# Organizations API

Manage organizations and teams.

## Organizations

### List Organizations

```http
GET /api/orgs
```

```bash
curl https://api.guts.network/api/orgs \
  -H "Authorization: Bearer guts_xxx"
```

### Get an Organization

```http
GET /api/orgs/{org}
```

```json
{
  "id": "org_abc123",
  "name": "acme",
  "display_name": "Acme Corp",
  "description": "Building the future",
  "email": "contact@acme.com",
  "created_at": "2025-01-01T00:00:00Z",
  "members_count": 25,
  "repos_count": 42
}
```

### Create an Organization

```http
POST /api/orgs
```

```bash
curl -X POST https://api.guts.network/api/orgs \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "acme",
    "display_name": "Acme Corp",
    "description": "Building the future",
    "email": "contact@acme.com"
  }'
```

### Update an Organization

```http
PATCH /api/orgs/{org}
```

### Delete an Organization

```http
DELETE /api/orgs/{org}
```

## Members

### List Organization Members

```http
GET /api/orgs/{org}/members
```

```json
[
  {
    "user": "alice",
    "role": "owner",
    "joined_at": "2025-01-01T00:00:00Z"
  },
  {
    "user": "bob",
    "role": "admin",
    "joined_at": "2025-01-05T00:00:00Z"
  },
  {
    "user": "charlie",
    "role": "member",
    "joined_at": "2025-01-10T00:00:00Z"
  }
]
```

### Member Roles

| Role | Description |
|------|-------------|
| `owner` | Full access, can delete org |
| `admin` | Manage members and settings |
| `member` | Access to org resources |

### Add/Update Member

```http
PUT /api/orgs/{org}/members/{username}
```

```bash
curl -X PUT https://api.guts.network/api/orgs/acme/members/dave \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "member"
  }'
```

### Remove Member

```http
DELETE /api/orgs/{org}/members/{username}
```

## Teams

### List Teams

```http
GET /api/orgs/{org}/teams
```

```json
[
  {
    "id": "team_xyz789",
    "name": "core-team",
    "description": "Core maintainers",
    "permission": "admin",
    "members_count": 5,
    "repos_count": 10
  },
  {
    "id": "team_abc123",
    "name": "contributors",
    "description": "External contributors",
    "permission": "write",
    "members_count": 20,
    "repos_count": 5
  }
]
```

### Get a Team

```http
GET /api/orgs/{org}/teams/{team}
```

### Create a Team

```http
POST /api/orgs/{org}/teams
```

```bash
curl -X POST https://api.guts.network/api/orgs/acme/teams \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "backend-team",
    "description": "Backend developers",
    "permission": "write"
  }'
```

### Update a Team

```http
PATCH /api/orgs/{org}/teams/{team}
```

### Delete a Team

```http
DELETE /api/orgs/{org}/teams/{team}
```

## Team Members

### List Team Members

```http
GET /api/orgs/{org}/teams/{team}/members
```

### Add Team Member

```http
PUT /api/orgs/{org}/teams/{team}/members/{username}
```

```bash
curl -X PUT https://api.guts.network/api/orgs/acme/teams/backend-team/members/eve \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "member"
  }'
```

### Remove Team Member

```http
DELETE /api/orgs/{org}/teams/{team}/members/{username}
```

## Team Repositories

### List Team Repositories

```http
GET /api/orgs/{org}/teams/{team}/repos
```

### Add Repository to Team

```http
PUT /api/orgs/{org}/teams/{team}/repos/{owner}/{repo}
```

```bash
curl -X PUT https://api.guts.network/api/orgs/acme/teams/backend-team/repos/acme/api-server \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "permission": "write"
  }'
```

### Remove Repository from Team

```http
DELETE /api/orgs/{org}/teams/{team}/repos/{owner}/{repo}
```

## Invitations

### List Pending Invitations

```http
GET /api/orgs/{org}/invitations
```

### Create Invitation

```http
POST /api/orgs/{org}/invitations
```

```bash
curl -X POST https://api.guts.network/api/orgs/acme/invitations \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com",
    "role": "member",
    "team_ids": ["team_xyz789"]
  }'
```

### Cancel Invitation

```http
DELETE /api/orgs/{org}/invitations/{invitation_id}
```

## Organization Repositories

### List Organization Repositories

```http
GET /api/orgs/{org}/repos
```

### Create Repository in Organization

```http
POST /api/orgs/{org}/repos
```

```bash
curl -X POST https://api.guts.network/api/orgs/acme/repos \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-project",
    "description": "A new project",
    "private": true
  }'
```
