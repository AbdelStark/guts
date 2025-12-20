//! API request handlers.

use crate::types::{ApiResponse, HealthResponse};
use axum::Json;

/// Health check handler.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        node_id: None,
    })
}

/// Root handler.
pub async fn root() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("Guts API".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check() {
        let response = health().await;
        assert_eq!(response.status, "healthy");
    }
}
