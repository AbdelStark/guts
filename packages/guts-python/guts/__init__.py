"""
Guts Python SDK

Official Python SDK for the Guts decentralized code collaboration platform.

Example:
    >>> from guts import GutsClient
    >>> client = GutsClient(base_url="https://api.guts.network", token="guts_xxx")
    >>> repos = client.repos.list()
    >>> for repo in repos:
    ...     print(repo.name)
"""

from guts.client import GutsClient
from guts.models import (
    Repository,
    Issue,
    IssueState,
    PullRequest,
    PullRequestState,
    Comment,
    Review,
    ReviewState,
    Release,
    ReleaseAsset,
    Label,
    User,
    Organization,
    Team,
    Webhook,
    WebhookEvent,
    ConsensusStatus,
    Block,
    Validator,
)
from guts.exceptions import (
    GutsError,
    NotFoundError,
    UnauthorizedError,
    ForbiddenError,
    RateLimitError,
    ValidationError,
    ServerError,
)

__version__ = "0.1.0"
__all__ = [
    # Client
    "GutsClient",
    # Models
    "Repository",
    "Issue",
    "IssueState",
    "PullRequest",
    "PullRequestState",
    "Comment",
    "Review",
    "ReviewState",
    "Release",
    "ReleaseAsset",
    "Label",
    "User",
    "Organization",
    "Team",
    "Webhook",
    "WebhookEvent",
    "ConsensusStatus",
    "Block",
    "Validator",
    # Exceptions
    "GutsError",
    "NotFoundError",
    "UnauthorizedError",
    "ForbiddenError",
    "RateLimitError",
    "ValidationError",
    "ServerError",
]
