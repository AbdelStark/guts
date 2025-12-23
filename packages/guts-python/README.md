# guts-sdk

Official Python SDK for the [Guts](https://github.com/AbdelStark/guts) decentralized code collaboration platform.

## Installation

```bash
pip install guts-sdk
```

## Quick Start

```python
from guts import GutsClient

# Create a client instance
client = GutsClient(
    base_url="https://api.guts.network",
    token="guts_your_token_here",  # optional
)

# List repositories
repos = client.repos.list()
for repo in repos:
    print(repo.name)

# Get a specific repository
repo = client.repos.get("owner", "repo-name")
print(repo)
```

## Features

- Full type hints with Pydantic models
- Sync client with httpx
- Repository management
- Issue tracking
- Pull request management
- Release management
- Organization and team management
- Webhook configuration
- Consensus status monitoring

## API Reference

### Repositories

```python
# List all repositories
repos = client.repos.list(page=1, per_page=20)

# Get a repository
repo = client.repos.get("owner", "repo-name")

# Create a repository
repo = client.repos.create(
    name="my-new-repo",
    description="A new repository",
    private=False,
)

# Delete a repository
client.repos.delete("owner", "repo-name")
```

### Issues

```python
from guts import CreateIssueRequest

# List issues
issues = client.issues.list("owner", "repo", state="open")

# Create an issue
issue = client.issues.create(
    "owner",
    "repo",
    CreateIssueRequest(
        title="Bug report",
        body="Description of the bug",
        labels=["bug"],
    ),
)

# Close an issue
client.issues.close("owner", "repo", 1)

# Add a comment
client.issues.create_comment("owner", "repo", 1, "This is a comment")
```

### Pull Requests

```python
from guts import CreatePullRequestRequest, CreateReviewRequest

# List pull requests
prs = client.pulls.list("owner", "repo", state="open")

# Create a pull request
pr = client.pulls.create(
    "owner",
    "repo",
    CreatePullRequestRequest(
        title="Feature: Add new feature",
        body="Description of changes",
        source_branch="feature-branch",
        target_branch="main",
    ),
)

# Create a review
client.pulls.create_review(
    "owner",
    "repo",
    1,
    CreateReviewRequest(state="approve", body="LGTM!"),
)

# Merge a pull request
client.pulls.merge("owner", "repo", 1, method="squash")
```

### Releases

```python
from guts import CreateReleaseRequest

# List releases
releases = client.releases.list("owner", "repo")

# Create a release
release = client.releases.create(
    "owner",
    "repo",
    CreateReleaseRequest(
        tag_name="v1.0.0",
        name="Version 1.0.0",
        body="Release notes here",
    ),
)
```

### Consensus (Network Status)

```python
# Get consensus status
status = client.consensus.status()
print(f"Block height: {status.height}")
print(f"Synced: {status.synced}")

# List recent blocks
blocks = client.consensus.list_blocks(limit=10)

# List validators
validators = client.consensus.list_validators()
```

## Error Handling

```python
from guts import (
    GutsClient,
    GutsError,
    NotFoundError,
    UnauthorizedError,
    RateLimitError,
)

try:
    repo = client.repos.get("owner", "repo")
except NotFoundError:
    print("Repository not found")
except UnauthorizedError:
    print("Invalid token")
except RateLimitError as e:
    print(f"Rate limited, retry after {e.retry_after} seconds")
except GutsError as e:
    print(f"Error {e.status_code}: {e.message}")
```

## Context Manager

```python
with GutsClient(token="guts_xxx") as client:
    repos = client.repos.list()
    # Client is automatically closed when exiting the context
```

## License

MIT OR Apache-2.0
