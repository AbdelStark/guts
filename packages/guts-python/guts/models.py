"""
Data models for the Guts Python SDK.
"""

from datetime import datetime
from enum import Enum
from typing import Any, Optional

from pydantic import BaseModel, Field


class IssueState(str, Enum):
    """Issue state."""

    OPEN = "open"
    CLOSED = "closed"


class PullRequestState(str, Enum):
    """Pull request state."""

    OPEN = "open"
    CLOSED = "closed"
    MERGED = "merged"


class ReviewState(str, Enum):
    """Review state."""

    PENDING = "pending"
    COMMENTED = "commented"
    APPROVED = "approved"
    CHANGES_REQUESTED = "changes_requested"
    DISMISSED = "dismissed"


class MergeMethod(str, Enum):
    """Merge method."""

    MERGE = "merge"
    SQUASH = "squash"
    REBASE = "rebase"


class WebhookEvent(str, Enum):
    """Webhook event types."""

    PUSH = "push"
    PULL_REQUEST = "pull_request"
    PULL_REQUEST_REVIEW = "pull_request_review"
    ISSUES = "issues"
    ISSUE_COMMENT = "issue_comment"
    CREATE = "create"
    DELETE = "delete"
    RELEASE = "release"
    FORK = "fork"
    STAR = "star"


class Repository(BaseModel):
    """Repository model."""

    key: str
    name: str
    owner: str
    description: Optional[str] = None
    private: bool = False
    default_branch: str = "main"
    clone_url: str
    html_url: str
    open_issues_count: int = 0
    open_pull_requests_count: int = 0
    created_at: datetime
    updated_at: datetime
    pushed_at: Optional[datetime] = None


class Issue(BaseModel):
    """Issue model."""

    id: str
    number: int
    title: str
    body: Optional[str] = None
    state: IssueState
    author: str
    assignees: list[str] = Field(default_factory=list)
    labels: list[str] = Field(default_factory=list)
    milestone: Optional[str] = None
    comments_count: int = 0
    created_at: datetime
    updated_at: datetime
    closed_at: Optional[datetime] = None
    closed_by: Optional[str] = None


class PullRequest(BaseModel):
    """Pull request model."""

    id: str
    number: int
    title: str
    body: Optional[str] = None
    state: PullRequestState
    author: str
    source_branch: str
    target_branch: str
    source_commit: str
    target_commit: str
    mergeable: Optional[bool] = None
    draft: bool = False
    reviewers: list[str] = Field(default_factory=list)
    labels: list[str] = Field(default_factory=list)
    comments_count: int = 0
    commits_count: int = 0
    changed_files: int = 0
    additions: int = 0
    deletions: int = 0
    created_at: datetime
    updated_at: datetime
    merged_at: Optional[datetime] = None
    merged_by: Optional[str] = None
    closed_at: Optional[datetime] = None


class Comment(BaseModel):
    """Comment model."""

    id: str
    body: str
    author: str
    created_at: datetime
    updated_at: datetime


class Review(BaseModel):
    """Review model."""

    id: str
    pr_number: int
    author: str
    state: ReviewState
    body: Optional[str] = None
    commit_id: str
    created_at: datetime
    submitted_at: Optional[datetime] = None


class ReleaseAsset(BaseModel):
    """Release asset model."""

    id: str
    name: str
    content_type: str
    size: int
    download_count: int = 0
    browser_download_url: str
    created_at: datetime
    uploader: str


class Release(BaseModel):
    """Release model."""

    id: str
    tag_name: str
    name: str
    body: Optional[str] = None
    draft: bool = False
    prerelease: bool = False
    author: str
    target_commitish: str
    assets: list[ReleaseAsset] = Field(default_factory=list)
    created_at: datetime
    published_at: Optional[datetime] = None


class Label(BaseModel):
    """Label model."""

    name: str
    color: str
    description: Optional[str] = None


class User(BaseModel):
    """User model."""

    id: str
    username: str
    display_name: Optional[str] = None
    avatar_url: Optional[str] = None
    bio: Optional[str] = None
    public_key: str
    created_at: datetime


class Organization(BaseModel):
    """Organization model."""

    id: str
    slug: str
    display_name: Optional[str] = None
    description: Optional[str] = None
    avatar_url: Optional[str] = None
    members_count: int = 0
    repos_count: int = 0
    created_at: datetime


class Team(BaseModel):
    """Team model."""

    id: str
    slug: str
    name: str
    description: Optional[str] = None
    permission: str = "read"
    members_count: int = 0
    repos_count: int = 0
    created_at: datetime


class Webhook(BaseModel):
    """Webhook model."""

    id: str
    url: str
    events: list[WebhookEvent]
    active: bool = True
    secret: Optional[str] = None
    content_type: str = "json"
    created_at: datetime
    updated_at: datetime
    last_delivery_at: Optional[datetime] = None


class ConsensusStatus(BaseModel):
    """Consensus status model."""

    height: int
    round: int
    phase: str
    synced: bool
    peers_count: int
    validators_count: int
    mempool_size: int
    last_block_time: Optional[datetime] = None


class Block(BaseModel):
    """Block model."""

    height: int
    hash: str
    parent_hash: str
    timestamp: datetime
    proposer: str
    transactions_count: int
    size: int


class Validator(BaseModel):
    """Validator model."""

    public_key: str
    address: str
    active: bool
    voting_power: int
    last_seen: Optional[datetime] = None


# Request models


class CreateRepositoryRequest(BaseModel):
    """Create repository request."""

    name: str
    description: Optional[str] = None
    private: bool = False
    auto_init: bool = False


class CreateIssueRequest(BaseModel):
    """Create issue request."""

    title: str
    body: Optional[str] = None
    labels: list[str] = Field(default_factory=list)
    assignees: list[str] = Field(default_factory=list)


class UpdateIssueRequest(BaseModel):
    """Update issue request."""

    title: Optional[str] = None
    body: Optional[str] = None
    state: Optional[IssueState] = None
    labels: Optional[list[str]] = None
    assignees: Optional[list[str]] = None


class CreatePullRequestRequest(BaseModel):
    """Create pull request request."""

    title: str
    body: Optional[str] = None
    source_branch: str
    target_branch: str
    draft: bool = False


class CreateReviewRequest(BaseModel):
    """Create review request."""

    body: Optional[str] = None
    state: str  # 'approve', 'request_changes', 'comment'
    commit_id: Optional[str] = None


class CreateReleaseRequest(BaseModel):
    """Create release request."""

    tag_name: str
    name: Optional[str] = None
    body: Optional[str] = None
    draft: bool = False
    prerelease: bool = False
    target_commitish: Optional[str] = None


class CreateWebhookRequest(BaseModel):
    """Create webhook request."""

    url: str
    events: list[WebhookEvent]
    active: bool = True
    secret: Optional[str] = None
