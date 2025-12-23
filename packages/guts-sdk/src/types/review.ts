/**
 * Review types.
 */

export type ReviewState = 'pending' | 'commented' | 'approved' | 'changes_requested' | 'dismissed';

export interface Review {
  /** Review ID */
  id: string;
  /** Pull request number */
  pr_number: number;
  /** Review author */
  author: string;
  /** Review state */
  state: ReviewState;
  /** Review body */
  body?: string;
  /** Commit SHA the review was made on */
  commit_id: string;
  /** Creation timestamp */
  created_at: string;
  /** Submission timestamp */
  submitted_at?: string;
}

export interface CreateReviewRequest {
  /** Review body */
  body?: string;
  /** Review state */
  state: 'approve' | 'request_changes' | 'comment';
  /** Commit SHA to review */
  commit_id?: string;
  /** Inline comments */
  comments?: ReviewComment[];
}

export interface ReviewComment {
  /** File path */
  path: string;
  /** Line number */
  line: number;
  /** Comment body */
  body: string;
}
