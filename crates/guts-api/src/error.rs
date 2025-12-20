//! API error types.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Errors that can occur in the API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Bad request.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// Unauthorized.
    #[error("unauthorized")]
    Unauthorized,

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// A specialized Result type for API operations.
pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        });

        (status, body).into_response()
    }
}
