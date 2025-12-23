# Releases API

Manage releases and release assets.

## List Releases

```http
GET /api/repos/{owner}/{name}/releases
```

### Parameters

| Name | Type | Description |
|------|------|-------------|
| `page` | integer | Page number |
| `per_page` | integer | Items per page |

### Example

```bash
curl https://api.guts.network/api/repos/alice/my-project/releases \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "items": [
    {
      "id": "rel_abc123",
      "tag_name": "v1.0.0",
      "name": "Version 1.0.0",
      "body": "## What's New\n\n- Feature 1\n- Feature 2",
      "draft": false,
      "prerelease": false,
      "author": "alice",
      "target_commitish": "main",
      "created_at": "2025-01-15T10:00:00Z",
      "published_at": "2025-01-15T10:00:00Z",
      "assets": [
        {
          "id": "asset_xyz789",
          "name": "my-project-linux-amd64.tar.gz",
          "size": 5242880,
          "download_count": 100,
          "content_type": "application/gzip"
        }
      ]
    }
  ],
  "total_count": 5
}
```

## Get a Release

```http
GET /api/repos/{owner}/{name}/releases/{id}
```

### Get Latest Release

```http
GET /api/repos/{owner}/{name}/releases/latest
```

### Get Release by Tag

```http
GET /api/repos/{owner}/{name}/releases/tags/{tag}
```

```bash
curl https://api.guts.network/api/repos/alice/my-project/releases/tags/v1.0.0 \
  -H "Authorization: Bearer guts_xxx"
```

## Create a Release

```http
POST /api/repos/{owner}/{name}/releases
```

### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tag_name` | string | Yes | Tag name (e.g., v1.0.0) |
| `name` | string | No | Release title |
| `body` | string | No | Release notes (Markdown) |
| `target_commitish` | string | No | Branch or commit SHA |
| `draft` | boolean | No | Create as draft |
| `prerelease` | boolean | No | Mark as prerelease |

### Example

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/releases \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "tag_name": "v1.1.0",
    "name": "Version 1.1.0",
    "body": "## What'\''s New\n\n- Bug fixes\n- Performance improvements",
    "target_commitish": "main",
    "draft": false,
    "prerelease": false
  }'
```

### Response

```json
{
  "id": "rel_new123",
  "tag_name": "v1.1.0",
  "name": "Version 1.1.0",
  "body": "## What's New\n\n- Bug fixes\n- Performance improvements",
  "draft": false,
  "prerelease": false,
  "author": "alice",
  "target_commitish": "main",
  "created_at": "2025-01-20T12:00:00Z",
  "published_at": "2025-01-20T12:00:00Z",
  "assets": [],
  "upload_url": "https://api.guts.network/api/repos/alice/my-project/releases/rel_new123/assets"
}
```

## Update a Release

```http
PATCH /api/repos/{owner}/{name}/releases/{id}
```

```bash
curl -X PATCH https://api.guts.network/api/repos/alice/my-project/releases/rel_new123 \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Version 1.1.0 - Stable",
    "draft": false
  }'
```

## Delete a Release

```http
DELETE /api/repos/{owner}/{name}/releases/{id}
```

::: warning
This does not delete the associated Git tag.
:::

## Release Assets

### List Assets

```http
GET /api/repos/{owner}/{name}/releases/{release_id}/assets
```

### Upload Asset

```http
POST /api/repos/{owner}/{name}/releases/{release_id}/assets
```

Upload a binary file as a release asset.

### Parameters

| Name | Type | Description |
|------|------|-------------|
| `name` | query | Filename for the asset |

### Example

```bash
curl -X POST "https://api.guts.network/api/repos/alice/my-project/releases/rel_abc123/assets?name=my-project-linux-amd64.tar.gz" \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/gzip" \
  --data-binary @my-project-linux-amd64.tar.gz
```

### Response

```json
{
  "id": "asset_new456",
  "name": "my-project-linux-amd64.tar.gz",
  "size": 5242880,
  "download_count": 0,
  "content_type": "application/gzip",
  "created_at": "2025-01-20T12:01:00Z",
  "download_url": "https://api.guts.network/api/repos/alice/my-project/releases/assets/asset_new456"
}
```

### Get Asset

```http
GET /api/repos/{owner}/{name}/releases/assets/{asset_id}
```

### Download Asset

```http
GET /api/repos/{owner}/{name}/releases/assets/{asset_id}/download
```

```bash
curl -OJ https://api.guts.network/api/repos/alice/my-project/releases/assets/asset_new456/download \
  -H "Authorization: Bearer guts_xxx"
```

### Update Asset

```http
PATCH /api/repos/{owner}/{name}/releases/assets/{asset_id}
```

```bash
curl -X PATCH https://api.guts.network/api/repos/alice/my-project/releases/assets/asset_new456 \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-project-linux-x64.tar.gz"
  }'
```

### Delete Asset

```http
DELETE /api/repos/{owner}/{name}/releases/assets/{asset_id}
```

## Generate Release Notes

```http
POST /api/repos/{owner}/{name}/releases/generate-notes
```

Automatically generate release notes based on commits.

### Request Body

| Field | Type | Description |
|-------|------|-------------|
| `tag_name` | string | Tag for the release |
| `previous_tag_name` | string | Previous tag (for comparison) |

### Example

```bash
curl -X POST https://api.guts.network/api/repos/alice/my-project/releases/generate-notes \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "tag_name": "v1.1.0",
    "previous_tag_name": "v1.0.0"
  }'
```

### Response

```json
{
  "name": "v1.1.0",
  "body": "## What's Changed\n\n* Add new feature by @bob in #42\n* Fix bug in authentication by @alice in #43\n\n**Full Changelog**: https://guts.network/alice/my-project/compare/v1.0.0...v1.1.0"
}
```
