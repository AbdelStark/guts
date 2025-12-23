/**
 * Pull request types.
 */

export type PullRequestState = 'open' | 'closed' | 'merged';
export type MergeMethod = 'merge' | 'squash' | 'rebase';

export interface PullRequest {
  /** Pull request ID */
  id: string;
  /** Pull request number */
  number: number;
  /** Pull request title */
  title: string;
  /** Pull request body/description */
  body?: string;
  /** Pull request state */
  state: PullRequestState;
  /** Pull request author */
  author: string;
  /** Source branch */
  source_branch: string;
  /** Target branch */
  target_branch: string;
  /** Source commit SHA */
  source_commit: string;
  /** Target commit SHA */
  target_commit: string;
  /** Whether the PR is mergeable */
  mergeable?: boolean;
  /** Whether the PR is a draft */
  draft: boolean;
  /** Assigned reviewers */
  reviewers: string[];
  /** Labels on the PR */
  labels: string[];
  /** Number of comments */
  comments_count: number;
  /** Number of commits */
  commits_count: number;
  /** Number of changed files */
  changed_files: number;
  /** Lines added */
  additions: number;
  /** Lines deleted */
  deletions: number;
  /** Creation timestamp */
  created_at: string;
  /** Last update timestamp */
  updated_at: string;
  /** Merged timestamp */
  merged_at?: string;
  /** User who merged the PR */
  merged_by?: string;
  /** Closed timestamp */
  closed_at?: string;
}

export interface CreatePullRequestRequest {
  /** Pull request title */
  title: string;
  /** Pull request body/description */
  body?: string;
  /** Source branch */
  source_branch: string;
  /** Target branch */
  target_branch: string;
  /** Whether the PR is a draft */
  draft?: boolean;
}

export interface UpdatePullRequestRequest {
  /** Pull request title */
  title?: string;
  /** Pull request body/description */
  body?: string;
  /** Pull request state */
  state?: 'open' | 'closed';
  /** Target branch */
  target_branch?: string;
}

export interface ListPullRequestsOptions {
  /** Filter by state */
  state?: PullRequestState | 'all';
  /** Filter by source branch */
  source_branch?: string;
  /** Filter by target branch */
  target_branch?: string;
  /** Sort field */
  sort?: 'created' | 'updated';
  /** Sort direction */
  direction?: 'asc' | 'desc';
  /** Page number */
  page?: number;
  /** Items per page */
  per_page?: number;
}
