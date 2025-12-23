"""
Exception types for the Guts Python SDK.
"""

from typing import Any, Optional


class GutsError(Exception):
    """Base exception for Guts SDK errors."""

    def __init__(
        self,
        message: str,
        status_code: int = 0,
        details: Optional[Any] = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.status_code = status_code
        self.details = details

    def __str__(self) -> str:
        if self.status_code:
            return f"[{self.status_code}] {self.message}"
        return self.message


class NotFoundError(GutsError):
    """Resource not found (404)."""

    def __init__(self, message: str = "Resource not found", details: Optional[Any] = None) -> None:
        super().__init__(message, status_code=404, details=details)


class UnauthorizedError(GutsError):
    """Authentication required or invalid credentials (401)."""

    def __init__(
        self, message: str = "Authentication required", details: Optional[Any] = None
    ) -> None:
        super().__init__(message, status_code=401, details=details)


class ForbiddenError(GutsError):
    """Permission denied (403)."""

    def __init__(self, message: str = "Permission denied", details: Optional[Any] = None) -> None:
        super().__init__(message, status_code=403, details=details)


class RateLimitError(GutsError):
    """Rate limit exceeded (429)."""

    def __init__(
        self,
        message: str = "Rate limit exceeded",
        retry_after: Optional[int] = None,
        details: Optional[Any] = None,
    ) -> None:
        super().__init__(message, status_code=429, details=details)
        self.retry_after = retry_after


class ValidationError(GutsError):
    """Validation error (422)."""

    def __init__(
        self, message: str = "Validation error", details: Optional[Any] = None
    ) -> None:
        super().__init__(message, status_code=422, details=details)


class ServerError(GutsError):
    """Server error (5xx)."""

    def __init__(
        self,
        message: str = "Internal server error",
        status_code: int = 500,
        details: Optional[Any] = None,
    ) -> None:
        super().__init__(message, status_code=status_code, details=details)


def raise_for_status(status_code: int, message: str, details: Optional[Any] = None) -> None:
    """Raise appropriate exception based on status code."""
    if status_code == 401:
        raise UnauthorizedError(message, details)
    elif status_code == 403:
        raise ForbiddenError(message, details)
    elif status_code == 404:
        raise NotFoundError(message, details)
    elif status_code == 422:
        raise ValidationError(message, details)
    elif status_code == 429:
        raise RateLimitError(message, details=details)
    elif status_code >= 500:
        raise ServerError(message, status_code, details)
    elif status_code >= 400:
        raise GutsError(message, status_code, details)
