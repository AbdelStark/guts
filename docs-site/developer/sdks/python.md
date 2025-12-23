# Python SDK

The official Python SDK for Guts.

## Installation

```bash
pip install guts-sdk
```

Or with Poetry:

```bash
poetry add guts-sdk
```

## Quick Start

```python
from guts import GutsClient

# Create a client
client = GutsClient(
    base_url="https://api.guts.network",
    token="guts_xxx",
)

# List your repositories
repos = client.repos.list()
for repo in repos.items:
    print(repo.name)
```

## Configuration

```python
from guts import GutsClient, GutsClientConfig

config = GutsClientConfig(
    # Required: API base URL
    base_url="https://api.guts.network",

    # Required: Authentication token
    token="guts_xxx",

    # Optional: Request timeout (default: 30 seconds)
    timeout=30.0,

    # Optional: Retry configuration
    max_retries=3,
    retry_delay=1.0,
)

client = GutsClient(config=config)
```

## Repositories

```python
from guts import CreateRepositoryRequest, UpdateRepositoryRequest

# List all repositories
repos = client.repos.list()

# Get a specific repository
repo = client.repos.get("owner", "repo-name")

# Create a repository
new_repo = client.repos.create(CreateRepositoryRequest(
    name="my-new-repo",
    owner="myusername",
    description="A great project",
    private=False,
))

# Update a repository
client.repos.update("owner", "repo-name", UpdateRepositoryRequest(
    description="Updated description",
))

# Delete a repository
client.repos.delete("owner", "repo-name")
```

## Issues

```python
from guts import CreateIssueRequest, UpdateIssueRequest, IssueState

# List issues
issues = client.issues.list(
    "owner", "repo",
    state=IssueState.OPEN,
    labels=["bug"],
)

# Get an issue
issue = client.issues.get("owner", "repo", 42)

# Create an issue
new_issue = client.issues.create("owner", "repo", CreateIssueRequest(
    title="Bug: Something is broken",
    body="When I click the button...",
    labels=["bug"],
    assignees=["developer"],
))

# Update an issue
client.issues.update("owner", "repo", 42, UpdateIssueRequest(
    state=IssueState.CLOSED,
))

# Add a comment
client.issues.create_comment("owner", "repo", 42, body="Fixed in PR #43")
```

## Pull Requests

```python
from guts import CreatePullRequestRequest, MergeMethod, ReviewEvent

# List pull requests
prs = client.pulls.list("owner", "repo", state="open")

# Get a pull request
pr = client.pulls.get("owner", "repo", 10)

# Create a pull request
new_pr = client.pulls.create("owner", "repo", CreatePullRequestRequest(
    title="Add new feature",
    body="This PR implements...",
    source_branch="feature-branch",
    target_branch="main",
))

# Merge a pull request
client.pulls.merge("owner", "repo", 10,
    merge_method=MergeMethod.SQUASH,
    commit_title="feat: Add new feature (#10)",
)

# Create a review
client.pulls.create_review("owner", "repo", 10,
    body="LGTM!",
    event=ReviewEvent.APPROVE,
)
```

## Organizations

```python
from guts import CreateOrganizationRequest, CreateTeamRequest, TeamPermission

# List organizations
orgs = client.orgs.list()

# Get an organization
org = client.orgs.get("acme")

# Create an organization
new_org = client.orgs.create(CreateOrganizationRequest(
    name="acme",
    display_name="Acme Corp",
    description="Building the future",
))

# List teams
teams = client.orgs.list_teams("acme")

# Create a team
client.orgs.create_team("acme", CreateTeamRequest(
    name="engineering",
    description="Engineering team",
    permission=TeamPermission.WRITE,
))
```

## Releases

```python
from guts import CreateReleaseRequest
from pathlib import Path

# List releases
releases = client.releases.list("owner", "repo")

# Get latest release
latest = client.releases.get_latest("owner", "repo")

# Create a release
release = client.releases.create("owner", "repo", CreateReleaseRequest(
    tag_name="v1.0.0",
    name="Version 1.0.0",
    body="## What's New\n\n- Feature 1\n- Feature 2",
))

# Upload asset
with open("app-linux-amd64.tar.gz", "rb") as f:
    client.releases.upload_asset(
        "owner", "repo", release.id,
        name="app-linux-amd64.tar.gz",
        data=f.read(),
        content_type="application/gzip",
    )
```

## Consensus

```python
# Get consensus status
status = client.consensus.get_status()
print(f"Block height: {status.block_height}")

# List validators
validators = client.consensus.get_validators()
for v in validators.validators:
    print(f"{v.name}: {v.status}")

# Get recent blocks
blocks = client.consensus.list_blocks(per_page=10)
```

## Error Handling

```python
from guts import GutsError, NotFoundError, RateLimitError, ValidationError

try:
    repo = client.repos.get("owner", "nonexistent")
except NotFoundError:
    print("Repository not found")
except RateLimitError as e:
    print(f"Rate limited. Retry after: {e.retry_after}s")
except ValidationError as e:
    print(f"Validation error: {e.details}")
except GutsError as e:
    print(f"API error: {e.message}")
    print(f"Error code: {e.code}")
    print(f"Status: {e.status}")
```

## Pagination

```python
# Iterate through all pages automatically
for repo in client.repos.list_all():
    print(repo.name)

# Manual pagination
page = 1
while True:
    response = client.repos.list(page=page, per_page=100)
    for repo in response.items:
        print(repo.name)
    if page >= response.total_pages:
        break
    page += 1
```

## Async Support

The SDK supports async/await with the async client:

```python
from guts import AsyncGutsClient
import asyncio

async def main():
    client = AsyncGutsClient(
        base_url="https://api.guts.network",
        token="guts_xxx",
    )

    # All methods are async
    repos = await client.repos.list()
    for repo in repos.items:
        print(repo.name)

    # Concurrent requests
    issues, prs = await asyncio.gather(
        client.issues.list("owner", "repo"),
        client.pulls.list("owner", "repo"),
    )

asyncio.run(main())
```

## Context Manager

```python
# Automatically close connections
with GutsClient(base_url="...", token="...") as client:
    repos = client.repos.list()

# Async context manager
async with AsyncGutsClient(base_url="...", token="...") as client:
    repos = await client.repos.list()
```

## Type Hints

The SDK is fully typed with Pydantic models:

```python
from guts import Repository, Issue, PullRequest
from guts import CreateIssueRequest, CreatePullRequestRequest

# All responses are typed Pydantic models
repo: Repository = client.repos.get("owner", "repo")
print(repo.name)
print(repo.created_at)  # datetime object

# Request models with validation
request = CreateIssueRequest(
    title="Bug report",
    body="Description",
    labels=["bug"],
)
```

## Logging

```python
import logging

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)
logging.getLogger("guts").setLevel(logging.DEBUG)

# Now all API calls will be logged
client = GutsClient(...)
```

## Environment Variables

```python
import os
from guts import GutsClient

# Client can read from environment variables
os.environ["GUTS_TOKEN"] = "guts_xxx"
os.environ["GUTS_BASE_URL"] = "https://api.guts.network"

client = GutsClient.from_env()
```
