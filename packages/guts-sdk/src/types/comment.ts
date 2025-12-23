/**
 * Comment types.
 */

export interface Comment {
  /** Comment ID */
  id: string;
  /** Comment body */
  body: string;
  /** Comment author */
  author: string;
  /** Creation timestamp */
  created_at: string;
  /** Last update timestamp */
  updated_at: string;
}

export interface CreateCommentRequest {
  /** Comment body */
  body: string;
}

export interface UpdateCommentRequest {
  /** Comment body */
  body: string;
}
