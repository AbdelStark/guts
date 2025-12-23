/**
 * Main Guts API client.
 */

import type { Repository, CreateRepositoryRequest, UpdateRepositoryRequest } from './types/repository';
import type { Issue, CreateIssueRequest, UpdateIssueRequest, ListIssuesOptions } from './types/issue';
import type { PullRequest, CreatePullRequestRequest, UpdatePullRequestRequest, ListPullRequestsOptions, MergeMethod } from './types/pull-request';
import type { Comment, CreateCommentRequest, UpdateCommentRequest } from './types/comment';
import type { Review, CreateReviewRequest } from './types/review';
import type { Release, CreateReleaseRequest, UpdateReleaseRequest } from './types/release';
import type { Label, CreateLabelRequest, UpdateLabelRequest } from './types/label';
import type { User } from './types/user';
import type { Organization, Team, TeamMember } from './types/organization';
import type { Webhook, CreateWebhookRequest } from './types/webhook';
import type { ConsensusStatus, Block, Validator } from './types/consensus';
import type { PaginatedResponse, PaginationParams } from './types/pagination';
import type { ApiError } from './types/error';

/**
 * Configuration options for the Guts client.
 */
export interface GutsClientOptions {
  /** Base URL of the Guts API (default: https://api.guts.network) */
  baseUrl?: string;
  /** Personal access token for authentication */
  token?: string;
  /** Request timeout in milliseconds (default: 30000) */
  timeout?: number;
  /** Custom fetch implementation */
  fetch?: typeof fetch;
}

/**
 * Guts API client for TypeScript/JavaScript applications.
 */
export class GutsClient {
  private baseUrl: string;
  private token?: string;
  private timeout: number;
  private fetchImpl: typeof fetch;

  constructor(options: GutsClientOptions = {}) {
    this.baseUrl = (options.baseUrl || 'https://api.guts.network').replace(/\/$/, '');
    this.token = options.token;
    this.timeout = options.timeout || 30000;
    this.fetchImpl = options.fetch || fetch;
  }

  // ===== Repository Operations =====

  /** Repository operations */
  repos = {
    /** List all repositories */
    list: (params?: PaginationParams): Promise<PaginatedResponse<Repository>> =>
      this.get('/api/repos', params),

    /** Get a repository by owner and name */
    get: (owner: string, name: string): Promise<Repository> =>
      this.get(`/api/repos/${owner}/${name}`),

    /** Create a new repository */
    create: (data: CreateRepositoryRequest): Promise<Repository> =>
      this.post('/api/repos', data),

    /** Update a repository */
    update: (owner: string, name: string, data: UpdateRepositoryRequest): Promise<Repository> =>
      this.patch(`/api/repos/${owner}/${name}`, data),

    /** Delete a repository */
    delete: (owner: string, name: string): Promise<void> =>
      this.del(`/api/repos/${owner}/${name}`),

    /** List repository branches */
    listBranches: (owner: string, name: string): Promise<string[]> =>
      this.get(`/api/repos/${owner}/${name}/branches`),

    /** List repository tags */
    listTags: (owner: string, name: string): Promise<string[]> =>
      this.get(`/api/repos/${owner}/${name}/tags`),

    /** Get repository README */
    getReadme: (owner: string, name: string): Promise<{ content: string; encoding: string }> =>
      this.get(`/api/repos/${owner}/${name}/readme`),
  };

  // ===== Issue Operations =====

  /** Issue operations */
  issues = {
    /** List issues for a repository */
    list: (owner: string, repo: string, options?: ListIssuesOptions): Promise<PaginatedResponse<Issue>> =>
      this.get(`/api/repos/${owner}/${repo}/issues`, options),

    /** Get a specific issue */
    get: (owner: string, repo: string, number: number): Promise<Issue> =>
      this.get(`/api/repos/${owner}/${repo}/issues/${number}`),

    /** Create a new issue */
    create: (owner: string, repo: string, data: CreateIssueRequest): Promise<Issue> =>
      this.post(`/api/repos/${owner}/${repo}/issues`, data),

    /** Update an issue */
    update: (owner: string, repo: string, number: number, data: UpdateIssueRequest): Promise<Issue> =>
      this.patch(`/api/repos/${owner}/${repo}/issues/${number}`, data),

    /** Close an issue */
    close: (owner: string, repo: string, number: number): Promise<Issue> =>
      this.patch(`/api/repos/${owner}/${repo}/issues/${number}`, { state: 'closed' }),

    /** Reopen an issue */
    reopen: (owner: string, repo: string, number: number): Promise<Issue> =>
      this.patch(`/api/repos/${owner}/${repo}/issues/${number}`, { state: 'open' }),

    /** List comments on an issue */
    listComments: (owner: string, repo: string, number: number): Promise<Comment[]> =>
      this.get(`/api/repos/${owner}/${repo}/issues/${number}/comments`),

    /** Create a comment on an issue */
    createComment: (owner: string, repo: string, number: number, data: CreateCommentRequest): Promise<Comment> =>
      this.post(`/api/repos/${owner}/${repo}/issues/${number}/comments`, data),
  };

  // ===== Pull Request Operations =====

  /** Pull request operations */
  pulls = {
    /** List pull requests for a repository */
    list: (owner: string, repo: string, options?: ListPullRequestsOptions): Promise<PaginatedResponse<PullRequest>> =>
      this.get(`/api/repos/${owner}/${repo}/pulls`, options),

    /** Get a specific pull request */
    get: (owner: string, repo: string, number: number): Promise<PullRequest> =>
      this.get(`/api/repos/${owner}/${repo}/pulls/${number}`),

    /** Create a new pull request */
    create: (owner: string, repo: string, data: CreatePullRequestRequest): Promise<PullRequest> =>
      this.post(`/api/repos/${owner}/${repo}/pulls`, data),

    /** Update a pull request */
    update: (owner: string, repo: string, number: number, data: UpdatePullRequestRequest): Promise<PullRequest> =>
      this.patch(`/api/repos/${owner}/${repo}/pulls/${number}`, data),

    /** Merge a pull request */
    merge: (owner: string, repo: string, number: number, method?: MergeMethod): Promise<void> =>
      this.post(`/api/repos/${owner}/${repo}/pulls/${number}/merge`, { merge_method: method || 'merge' }),

    /** Close a pull request */
    close: (owner: string, repo: string, number: number): Promise<PullRequest> =>
      this.patch(`/api/repos/${owner}/${repo}/pulls/${number}`, { state: 'closed' }),

    /** List reviews on a pull request */
    listReviews: (owner: string, repo: string, number: number): Promise<Review[]> =>
      this.get(`/api/repos/${owner}/${repo}/pulls/${number}/reviews`),

    /** Create a review on a pull request */
    createReview: (owner: string, repo: string, number: number, data: CreateReviewRequest): Promise<Review> =>
      this.post(`/api/repos/${owner}/${repo}/pulls/${number}/reviews`, data),

    /** List comments on a pull request */
    listComments: (owner: string, repo: string, number: number): Promise<Comment[]> =>
      this.get(`/api/repos/${owner}/${repo}/pulls/${number}/comments`),

    /** Create a comment on a pull request */
    createComment: (owner: string, repo: string, number: number, data: CreateCommentRequest): Promise<Comment> =>
      this.post(`/api/repos/${owner}/${repo}/pulls/${number}/comments`, data),
  };

  // ===== Release Operations =====

  /** Release operations */
  releases = {
    /** List releases for a repository */
    list: (owner: string, repo: string): Promise<Release[]> =>
      this.get(`/api/repos/${owner}/${repo}/releases`),

    /** Get a specific release */
    get: (owner: string, repo: string, id: string): Promise<Release> =>
      this.get(`/api/repos/${owner}/${repo}/releases/${id}`),

    /** Get the latest release */
    getLatest: (owner: string, repo: string): Promise<Release> =>
      this.get(`/api/repos/${owner}/${repo}/releases/latest`),

    /** Create a new release */
    create: (owner: string, repo: string, data: CreateReleaseRequest): Promise<Release> =>
      this.post(`/api/repos/${owner}/${repo}/releases`, data),

    /** Update a release */
    update: (owner: string, repo: string, id: string, data: UpdateReleaseRequest): Promise<Release> =>
      this.patch(`/api/repos/${owner}/${repo}/releases/${id}`, data),

    /** Delete a release */
    delete: (owner: string, repo: string, id: string): Promise<void> =>
      this.del(`/api/repos/${owner}/${repo}/releases/${id}`),

    /** Upload a release asset */
    uploadAsset: async (
      owner: string,
      repo: string,
      releaseId: string,
      name: string,
      contentType: string,
      data: ArrayBuffer | Blob
    ): Promise<void> => {
      const url = new URL(`${this.baseUrl}/api/repos/${owner}/${repo}/releases/${releaseId}/assets`);
      url.searchParams.set('name', name);

      const response = await this.fetchImpl(url.toString(), {
        method: 'POST',
        headers: {
          ...this.authHeaders(),
          'Content-Type': contentType,
        },
        body: data,
      });

      if (!response.ok) {
        throw await this.handleError(response);
      }
    },
  };

  // ===== Label Operations =====

  /** Label operations */
  labels = {
    /** List labels for a repository */
    list: (owner: string, repo: string): Promise<Label[]> =>
      this.get(`/api/repos/${owner}/${repo}/labels`),

    /** Get a specific label */
    get: (owner: string, repo: string, name: string): Promise<Label> =>
      this.get(`/api/repos/${owner}/${repo}/labels/${encodeURIComponent(name)}`),

    /** Create a new label */
    create: (owner: string, repo: string, data: CreateLabelRequest): Promise<Label> =>
      this.post(`/api/repos/${owner}/${repo}/labels`, data),

    /** Update a label */
    update: (owner: string, repo: string, name: string, data: UpdateLabelRequest): Promise<Label> =>
      this.patch(`/api/repos/${owner}/${repo}/labels/${encodeURIComponent(name)}`, data),

    /** Delete a label */
    delete: (owner: string, repo: string, name: string): Promise<void> =>
      this.del(`/api/repos/${owner}/${repo}/labels/${encodeURIComponent(name)}`),
  };

  // ===== User Operations =====

  /** User operations */
  users = {
    /** Get the authenticated user */
    me: (): Promise<User> =>
      this.get('/api/user'),

    /** Get a user by username */
    get: (username: string): Promise<User> =>
      this.get(`/api/users/${username}`),

    /** List repositories for a user */
    listRepos: (username: string): Promise<Repository[]> =>
      this.get(`/api/users/${username}/repos`),
  };

  // ===== Organization Operations =====

  /** Organization operations */
  orgs = {
    /** List organizations for the authenticated user */
    list: (): Promise<Organization[]> =>
      this.get('/api/user/orgs'),

    /** Get an organization */
    get: (org: string): Promise<Organization> =>
      this.get(`/api/orgs/${org}`),

    /** Create an organization */
    create: (data: { name: string; display_name?: string }): Promise<Organization> =>
      this.post('/api/orgs', data),

    /** List organization members */
    listMembers: (org: string): Promise<User[]> =>
      this.get(`/api/orgs/${org}/members`),

    /** List organization repositories */
    listRepos: (org: string): Promise<Repository[]> =>
      this.get(`/api/orgs/${org}/repos`),

    /** List teams in an organization */
    listTeams: (org: string): Promise<Team[]> =>
      this.get(`/api/orgs/${org}/teams`),

    /** Get a team */
    getTeam: (org: string, teamSlug: string): Promise<Team> =>
      this.get(`/api/orgs/${org}/teams/${teamSlug}`),

    /** Create a team */
    createTeam: (org: string, data: { name: string; description?: string }): Promise<Team> =>
      this.post(`/api/orgs/${org}/teams`, data),

    /** List team members */
    listTeamMembers: (org: string, teamSlug: string): Promise<TeamMember[]> =>
      this.get(`/api/orgs/${org}/teams/${teamSlug}/members`),
  };

  // ===== Webhook Operations =====

  /** Webhook operations */
  webhooks = {
    /** List webhooks for a repository */
    list: (owner: string, repo: string): Promise<Webhook[]> =>
      this.get(`/api/repos/${owner}/${repo}/hooks`),

    /** Get a webhook */
    get: (owner: string, repo: string, hookId: string): Promise<Webhook> =>
      this.get(`/api/repos/${owner}/${repo}/hooks/${hookId}`),

    /** Create a webhook */
    create: (owner: string, repo: string, data: CreateWebhookRequest): Promise<Webhook> =>
      this.post(`/api/repos/${owner}/${repo}/hooks`, data),

    /** Delete a webhook */
    delete: (owner: string, repo: string, hookId: string): Promise<void> =>
      this.del(`/api/repos/${owner}/${repo}/hooks/${hookId}`),

    /** Ping a webhook */
    ping: (owner: string, repo: string, hookId: string): Promise<void> =>
      this.post(`/api/repos/${owner}/${repo}/hooks/${hookId}/pings`, {}),
  };

  // ===== Consensus Operations =====

  /** Consensus and network operations */
  consensus = {
    /** Get consensus status */
    status: (): Promise<ConsensusStatus> =>
      this.get('/api/consensus/status'),

    /** List recent blocks */
    listBlocks: (limit?: number): Promise<Block[]> =>
      this.get('/api/consensus/blocks', { limit }),

    /** Get a block by height */
    getBlock: (height: number): Promise<Block> =>
      this.get(`/api/consensus/blocks/${height}`),

    /** List validators */
    listValidators: (): Promise<Validator[]> =>
      this.get('/api/consensus/validators'),
  };

  // ===== Health Operations =====

  /** Health check operations */
  health = {
    /** Check if the node is ready */
    ready: async (): Promise<boolean> => {
      try {
        const response = await this.fetchImpl(`${this.baseUrl}/health/ready`);
        return response.ok;
      } catch {
        return false;
      }
    },

    /** Check if the node is alive */
    live: async (): Promise<boolean> => {
      try {
        const response = await this.fetchImpl(`${this.baseUrl}/health/live`);
        return response.ok;
      } catch {
        return false;
      }
    },
  };

  // ===== Private Methods =====

  private authHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    return headers;
  }

  private async get<T>(path: string, params?: Record<string, unknown>): Promise<T> {
    const url = new URL(`${this.baseUrl}${path}`);

    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          url.searchParams.set(key, String(value));
        }
      });
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await this.fetchImpl(url.toString(), {
        method: 'GET',
        headers: this.authHeaders(),
        signal: controller.signal,
      });

      if (!response.ok) {
        throw await this.handleError(response);
      }

      return response.json();
    } finally {
      clearTimeout(timeoutId);
    }
  }

  private async post<T>(path: string, data: unknown): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
        method: 'POST',
        headers: this.authHeaders(),
        body: JSON.stringify(data),
        signal: controller.signal,
      });

      if (!response.ok) {
        throw await this.handleError(response);
      }

      const text = await response.text();
      return text ? JSON.parse(text) : undefined;
    } finally {
      clearTimeout(timeoutId);
    }
  }

  private async patch<T>(path: string, data: unknown): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
        method: 'PATCH',
        headers: this.authHeaders(),
        body: JSON.stringify(data),
        signal: controller.signal,
      });

      if (!response.ok) {
        throw await this.handleError(response);
      }

      return response.json();
    } finally {
      clearTimeout(timeoutId);
    }
  }

  private async del(path: string): Promise<void> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
        method: 'DELETE',
        headers: this.authHeaders(),
        signal: controller.signal,
      });

      if (!response.ok) {
        throw await this.handleError(response);
      }
    } finally {
      clearTimeout(timeoutId);
    }
  }

  private async handleError(response: Response): Promise<ApiError> {
    let message = `Request failed with status ${response.status}`;
    let details: unknown;

    try {
      const body = await response.json();
      if (body.message) {
        message = body.message;
      }
      details = body;
    } catch {
      // Ignore JSON parse errors
    }

    return {
      status: response.status,
      message,
      details,
    };
  }
}
