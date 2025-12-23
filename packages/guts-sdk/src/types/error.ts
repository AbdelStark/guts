/**
 * Error types.
 */

export interface ApiError {
  /** HTTP status code */
  status: number;
  /** Error message */
  message: string;
  /** Additional error details */
  details?: unknown;
}

export class GutsError extends Error {
  /** HTTP status code */
  status: number;
  /** Additional error details */
  details?: unknown;

  constructor(status: number, message: string, details?: unknown) {
    super(message);
    this.name = 'GutsError';
    this.status = status;
    this.details = details;
  }

  /** Check if this is a not found error */
  isNotFound(): boolean {
    return this.status === 404;
  }

  /** Check if this is an unauthorized error */
  isUnauthorized(): boolean {
    return this.status === 401;
  }

  /** Check if this is a forbidden error */
  isForbidden(): boolean {
    return this.status === 403;
  }

  /** Check if this is a rate limit error */
  isRateLimited(): boolean {
    return this.status === 429;
  }

  /** Check if this is a validation error */
  isValidationError(): boolean {
    return this.status === 422;
  }

  /** Check if this is a server error */
  isServerError(): boolean {
    return this.status >= 500;
  }
}
