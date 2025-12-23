"""
Main Guts API client.
"""

from typing import Any, Optional, TypeVar

import httpx

from guts.exceptions import raise_for_status
from guts.models import (
    Block,
    Comment,
    ConsensusStatus,
    CreateIssueRequest,
    CreatePullRequestRequest,
    CreateReleaseRequest,
    CreateReviewRequest,
    CreateWebhookRequest,
    Issue,
    Label,
    Organization,
    PullRequest,
    Release,
    Repository,
    Review,
    Team,
    User,
    Validator,
    Webhook,
)

T = TypeVar("T")


class GutsClient:
    """
    Guts API client for Python applications.

    Example:
        >>> client = GutsClient(base_url="https://api.guts.network", token="guts_xxx")
        >>> repos = client.repos.list()
        >>> for repo in repos:
        ...     print(repo.name)
    """

    def __init__(
        self,
        base_url: str = "https://api.guts.network",
        token: Optional[str] = None,
        timeout: float = 30.0,
    ) -> None:
        """
        Initialize the Guts client.

        Args:
            base_url: Base URL of the Guts API.
            token: Personal access token for authentication.
            timeout: Request timeout in seconds.
        """
        self.base_url = base_url.rstrip("/")
        self.token = token
        self._client = httpx.Client(
            base_url=self.base_url,
            timeout=timeout,
            headers=self._build_headers(),
        )

        # Initialize API namespaces
        self.repos = _RepositoryAPI(self)
        self.issues = _IssueAPI(self)
        self.pulls = _PullRequestAPI(self)
        self.releases = _ReleaseAPI(self)
        self.labels = _LabelAPI(self)
        self.users = _UserAPI(self)
        self.orgs = _OrganizationAPI(self)
        self.webhooks = _WebhookAPI(self)
        self.consensus = _ConsensusAPI(self)

    def _build_headers(self) -> dict[str, str]:
        """Build request headers."""
        headers = {
            "Accept": "application/json",
            "Content-Type": "application/json",
            "User-Agent": "guts-python/0.1.0",
        }
        if self.token:
            headers["Authorization"] = f"Bearer {self.token}"
        return headers

    def _request(
        self,
        method: str,
        path: str,
        params: Optional[dict[str, Any]] = None,
        json: Optional[dict[str, Any]] = None,
    ) -> Any:
        """Make an HTTP request."""
        response = self._client.request(method, path, params=params, json=json)

        if not response.is_success:
            message = "Request failed"
            details = None
            try:
                data = response.json()
                if "message" in data:
                    message = data["message"]
                details = data
            except Exception:
                message = response.text or message

            raise_for_status(response.status_code, message, details)

        if response.status_code == 204:
            return None

        return response.json()

    def _get(self, path: str, params: Optional[dict[str, Any]] = None) -> Any:
        """Make a GET request."""
        return self._request("GET", path, params=params)

    def _post(self, path: str, json: Optional[dict[str, Any]] = None) -> Any:
        """Make a POST request."""
        return self._request("POST", path, json=json)

    def _patch(self, path: str, json: Optional[dict[str, Any]] = None) -> Any:
        """Make a PATCH request."""
        return self._request("PATCH", path, json=json)

    def _delete(self, path: str) -> None:
        """Make a DELETE request."""
        self._request("DELETE", path)

    def close(self) -> None:
        """Close the HTTP client."""
        self._client.close()

    def __enter__(self) -> "GutsClient":
        return self

    def __exit__(self, *args: Any) -> None:
        self.close()


class _RepositoryAPI:
    """Repository operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def list(self, page: int = 1, per_page: int = 30) -> list[Repository]:
        """List repositories."""
        data = self._client._get("/api/repos", {"page": page, "per_page": per_page})
        return [Repository(**item) for item in data.get("items", data)]

    def get(self, owner: str, name: str) -> Repository:
        """Get a repository."""
        data = self._client._get(f"/api/repos/{owner}/{name}")
        return Repository(**data)

    def create(
        self,
        name: str,
        description: Optional[str] = None,
        private: bool = False,
    ) -> Repository:
        """Create a repository."""
        data = self._client._post(
            "/api/repos",
            {"name": name, "description": description, "private": private},
        )
        return Repository(**data)

    def update(
        self,
        owner: str,
        name: str,
        **kwargs: Any,
    ) -> Repository:
        """Update a repository."""
        data = self._client._patch(f"/api/repos/{owner}/{name}", kwargs)
        return Repository(**data)

    def delete(self, owner: str, name: str) -> None:
        """Delete a repository."""
        self._client._delete(f"/api/repos/{owner}/{name}")


class _IssueAPI:
    """Issue operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def list(
        self,
        owner: str,
        repo: str,
        state: str = "open",
        page: int = 1,
        per_page: int = 30,
    ) -> list[Issue]:
        """List issues."""
        data = self._client._get(
            f"/api/repos/{owner}/{repo}/issues",
            {"state": state, "page": page, "per_page": per_page},
        )
        return [Issue(**item) for item in data.get("items", data)]

    def get(self, owner: str, repo: str, number: int) -> Issue:
        """Get an issue."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/issues/{number}")
        return Issue(**data)

    def create(self, owner: str, repo: str, request: CreateIssueRequest) -> Issue:
        """Create an issue."""
        data = self._client._post(
            f"/api/repos/{owner}/{repo}/issues",
            request.model_dump(exclude_none=True),
        )
        return Issue(**data)

    def update(self, owner: str, repo: str, number: int, **kwargs: Any) -> Issue:
        """Update an issue."""
        data = self._client._patch(f"/api/repos/{owner}/{repo}/issues/{number}", kwargs)
        return Issue(**data)

    def close(self, owner: str, repo: str, number: int) -> Issue:
        """Close an issue."""
        return self.update(owner, repo, number, state="closed")

    def reopen(self, owner: str, repo: str, number: int) -> Issue:
        """Reopen an issue."""
        return self.update(owner, repo, number, state="open")

    def list_comments(self, owner: str, repo: str, number: int) -> list[Comment]:
        """List comments on an issue."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/issues/{number}/comments")
        return [Comment(**item) for item in data]

    def create_comment(self, owner: str, repo: str, number: int, body: str) -> Comment:
        """Create a comment on an issue."""
        data = self._client._post(
            f"/api/repos/{owner}/{repo}/issues/{number}/comments",
            {"body": body},
        )
        return Comment(**data)


class _PullRequestAPI:
    """Pull request operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def list(
        self,
        owner: str,
        repo: str,
        state: str = "open",
        page: int = 1,
        per_page: int = 30,
    ) -> list[PullRequest]:
        """List pull requests."""
        data = self._client._get(
            f"/api/repos/{owner}/{repo}/pulls",
            {"state": state, "page": page, "per_page": per_page},
        )
        return [PullRequest(**item) for item in data.get("items", data)]

    def get(self, owner: str, repo: str, number: int) -> PullRequest:
        """Get a pull request."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/pulls/{number}")
        return PullRequest(**data)

    def create(self, owner: str, repo: str, request: CreatePullRequestRequest) -> PullRequest:
        """Create a pull request."""
        data = self._client._post(
            f"/api/repos/{owner}/{repo}/pulls",
            request.model_dump(exclude_none=True),
        )
        return PullRequest(**data)

    def update(self, owner: str, repo: str, number: int, **kwargs: Any) -> PullRequest:
        """Update a pull request."""
        data = self._client._patch(f"/api/repos/{owner}/{repo}/pulls/{number}", kwargs)
        return PullRequest(**data)

    def merge(self, owner: str, repo: str, number: int, method: str = "merge") -> None:
        """Merge a pull request."""
        self._client._post(
            f"/api/repos/{owner}/{repo}/pulls/{number}/merge",
            {"merge_method": method},
        )

    def close(self, owner: str, repo: str, number: int) -> PullRequest:
        """Close a pull request."""
        return self.update(owner, repo, number, state="closed")

    def list_reviews(self, owner: str, repo: str, number: int) -> list[Review]:
        """List reviews on a pull request."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/pulls/{number}/reviews")
        return [Review(**item) for item in data]

    def create_review(
        self, owner: str, repo: str, number: int, request: CreateReviewRequest
    ) -> Review:
        """Create a review on a pull request."""
        data = self._client._post(
            f"/api/repos/{owner}/{repo}/pulls/{number}/reviews",
            request.model_dump(exclude_none=True),
        )
        return Review(**data)


class _ReleaseAPI:
    """Release operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def list(self, owner: str, repo: str) -> list[Release]:
        """List releases."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/releases")
        return [Release(**item) for item in data]

    def get(self, owner: str, repo: str, release_id: str) -> Release:
        """Get a release."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/releases/{release_id}")
        return Release(**data)

    def get_latest(self, owner: str, repo: str) -> Release:
        """Get the latest release."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/releases/latest")
        return Release(**data)

    def create(self, owner: str, repo: str, request: CreateReleaseRequest) -> Release:
        """Create a release."""
        data = self._client._post(
            f"/api/repos/{owner}/{repo}/releases",
            request.model_dump(exclude_none=True),
        )
        return Release(**data)

    def delete(self, owner: str, repo: str, release_id: str) -> None:
        """Delete a release."""
        self._client._delete(f"/api/repos/{owner}/{repo}/releases/{release_id}")


class _LabelAPI:
    """Label operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def list(self, owner: str, repo: str) -> list[Label]:
        """List labels."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/labels")
        return [Label(**item) for item in data]

    def create(
        self,
        owner: str,
        repo: str,
        name: str,
        color: str,
        description: Optional[str] = None,
    ) -> Label:
        """Create a label."""
        data = self._client._post(
            f"/api/repos/{owner}/{repo}/labels",
            {"name": name, "color": color, "description": description},
        )
        return Label(**data)

    def delete(self, owner: str, repo: str, name: str) -> None:
        """Delete a label."""
        self._client._delete(f"/api/repos/{owner}/{repo}/labels/{name}")


class _UserAPI:
    """User operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def me(self) -> User:
        """Get the authenticated user."""
        data = self._client._get("/api/user")
        return User(**data)

    def get(self, username: str) -> User:
        """Get a user by username."""
        data = self._client._get(f"/api/users/{username}")
        return User(**data)


class _OrganizationAPI:
    """Organization operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def list(self) -> list[Organization]:
        """List organizations for the authenticated user."""
        data = self._client._get("/api/user/orgs")
        return [Organization(**item) for item in data]

    def get(self, org: str) -> Organization:
        """Get an organization."""
        data = self._client._get(f"/api/orgs/{org}")
        return Organization(**data)

    def list_teams(self, org: str) -> list[Team]:
        """List teams in an organization."""
        data = self._client._get(f"/api/orgs/{org}/teams")
        return [Team(**item) for item in data]

    def get_team(self, org: str, team_slug: str) -> Team:
        """Get a team."""
        data = self._client._get(f"/api/orgs/{org}/teams/{team_slug}")
        return Team(**data)


class _WebhookAPI:
    """Webhook operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def list(self, owner: str, repo: str) -> list[Webhook]:
        """List webhooks."""
        data = self._client._get(f"/api/repos/{owner}/{repo}/hooks")
        return [Webhook(**item) for item in data]

    def create(self, owner: str, repo: str, request: CreateWebhookRequest) -> Webhook:
        """Create a webhook."""
        data = self._client._post(
            f"/api/repos/{owner}/{repo}/hooks",
            request.model_dump(exclude_none=True),
        )
        return Webhook(**data)

    def delete(self, owner: str, repo: str, hook_id: str) -> None:
        """Delete a webhook."""
        self._client._delete(f"/api/repos/{owner}/{repo}/hooks/{hook_id}")


class _ConsensusAPI:
    """Consensus operations."""

    def __init__(self, client: GutsClient) -> None:
        self._client = client

    def status(self) -> ConsensusStatus:
        """Get consensus status."""
        data = self._client._get("/api/consensus/status")
        return ConsensusStatus(**data)

    def list_blocks(self, limit: int = 10) -> list[Block]:
        """List recent blocks."""
        data = self._client._get("/api/consensus/blocks", {"limit": limit})
        return [Block(**item) for item in data]

    def get_block(self, height: int) -> Block:
        """Get a block by height."""
        data = self._client._get(f"/api/consensus/blocks/{height}")
        return Block(**data)

    def list_validators(self) -> list[Validator]:
        """List validators."""
        data = self._client._get("/api/consensus/validators")
        return [Validator(**item) for item in data]
