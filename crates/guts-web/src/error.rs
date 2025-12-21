//! Error types for the web gateway.

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

/// Web gateway errors.
#[derive(Debug, Error)]
pub enum WebError {
    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Template rendering error.
    #[error("template error: {0}")]
    Template(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            WebError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            WebError::Template(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            WebError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Error - Guts</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen flex items-center justify-center">
    <div class="text-center">
        <h1 class="text-6xl font-bold text-red-500 mb-4">{}</h1>
        <p class="text-xl text-gray-400 mb-8">{}</p>
        <a href="/" class="text-blue-400 hover:text-blue-300 underline">Back to Home</a>
    </div>
</body>
</html>"#,
            status.as_u16(),
            message
        );

        (status, Html(html)).into_response()
    }
}

impl From<askama::Error> for WebError {
    fn from(err: askama::Error) -> Self {
        WebError::Template(err.to_string())
    }
}

impl From<guts_storage::StorageError> for WebError {
    fn from(err: guts_storage::StorageError) -> Self {
        match err {
            guts_storage::StorageError::ObjectNotFound(id) => {
                WebError::NotFound(format!("Object '{}' not found", id))
            }
            guts_storage::StorageError::RepoNotFound(key) => {
                WebError::NotFound(format!("Repository '{}' not found", key))
            }
            other => WebError::Internal(other.to_string()),
        }
    }
}
