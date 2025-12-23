/**
 * Issue types.
 */

import type { Label } from './label';
import type { User } from './user';

export type IssueState = 'open' | 'closed';

export interface Issue {
  /** Issue ID */
  id: string;
  /** Issue number */
  number: number;
  /** Issue title */
  title: string;
  /** Issue body/description */
  body?: string;
  /** Issue state */
  state: IssueState;
  /** Issue author */
  author: string;
  /** Assigned users */
  assignees: string[];
  /** Labels on the issue */
  labels: string[];
  /** Milestone ID */
  milestone?: string;
  /** Number of comments */
  comments_count: number;
  /** Creation timestamp */
  created_at: string;
  /** Last update timestamp */
  updated_at: string;
  /** Closed timestamp */
  closed_at?: string;
  /** User who closed the issue */
  closed_by?: string;
}

export interface CreateIssueRequest {
  /** Issue title */
  title: string;
  /** Issue body/description */
  body?: string;
  /** Labels to add */
  labels?: string[];
  /** Users to assign */
  assignees?: string[];
  /** Milestone ID */
  milestone?: string;
}

export interface UpdateIssueRequest {
  /** Issue title */
  title?: string;
  /** Issue body/description */
  body?: string;
  /** Issue state */
  state?: IssueState;
  /** Labels to set (replaces existing) */
  labels?: string[];
  /** Users to assign (replaces existing) */
  assignees?: string[];
  /** Milestone ID */
  milestone?: string | null;
}

export interface ListIssuesOptions {
  /** Filter by state */
  state?: IssueState | 'all';
  /** Filter by labels (comma-separated) */
  labels?: string;
  /** Filter by assignee username */
  assignee?: string;
  /** Filter by author username */
  author?: string;
  /** Sort field */
  sort?: 'created' | 'updated' | 'comments';
  /** Sort direction */
  direction?: 'asc' | 'desc';
  /** Page number */
  page?: number;
  /** Items per page */
  per_page?: number;
}
