/**
 * Pagination types.
 */

export interface PaginationParams {
  /** Page number (1-indexed) */
  page?: number;
  /** Items per page */
  per_page?: number;
}

export interface PaginatedResponse<T> {
  /** Items on this page */
  items: T[];
  /** Total number of items */
  total_count: number;
  /** Current page number */
  page: number;
  /** Items per page */
  per_page: number;
  /** Total number of pages */
  total_pages: number;
  /** Whether there is a next page */
  has_next: boolean;
  /** Whether there is a previous page */
  has_prev: boolean;
}

export interface PaginationLinks {
  /** First page URL */
  first?: string;
  /** Previous page URL */
  prev?: string;
  /** Next page URL */
  next?: string;
  /** Last page URL */
  last?: string;
}
