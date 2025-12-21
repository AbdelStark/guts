//! Observability middleware for request tracking and metrics.
//!
//! Provides:
//! - Request ID generation and propagation
//! - HTTP metrics collection
//! - Request/response logging

use axum::{
    body::Body,
    extract::Request,
    http::{header::HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;
use uuid::Uuid;

use super::metrics::METRICS;

/// Header name for request ID.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Type alias for the middleware future.
type MiddlewareFuture = Pin<Box<dyn Future<Output = Response> + Send>>;

/// Type alias for middleware function pointer.
type MiddlewareFn = fn(Request, Next) -> MiddlewareFuture;

/// Type alias for the middleware layer.
pub type MiddlewareLayer = axum::middleware::FromFnLayer<MiddlewareFn, (), Request>;

/// Create a request ID layer function.
pub fn request_id_layer() -> MiddlewareLayer {
    axum::middleware::from_fn(request_id_middleware_fn)
}

fn request_id_middleware_fn(request: Request, next: Next) -> MiddlewareFuture {
    Box::pin(async move {
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create span with request ID
        let span = tracing::info_span!(
            "request",
            request_id = %request_id,
            method = %request.method(),
            uri = %request.uri(),
        );

        let _guard = span.enter();

        let mut response = next.run(request).await;

        // Add request ID to response headers
        if let Ok(header_value) = HeaderValue::from_str(&request_id) {
            response
                .headers_mut()
                .insert(HeaderName::from_static("x-request-id"), header_value);
        }

        response
    })
}

/// Request ID middleware - adds request ID to all requests.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Get or generate request ID
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Insert request ID into extensions for handlers to access
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Create span with request ID
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
    );

    let _guard = span.enter();

    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), header_value);
    }

    response
}

/// Request ID extension type.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Metrics middleware - records HTTP request metrics.
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Increment active connections
    METRICS.http_active_connections.inc();

    let response = next.run(request).await;

    // Decrement active connections
    METRICS.http_active_connections.dec();

    // Record metrics
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();

    METRICS.record_http_request(&method, &path, status, duration);

    tracing::debug!(
        method = %method,
        path = %path,
        status = %status,
        duration_ms = %format!("{:.2}", duration * 1000.0),
        "Request completed"
    );

    response
}

/// Create a metrics layer.
pub fn metrics_layer() -> MiddlewareLayer {
    axum::middleware::from_fn(metrics_middleware_fn)
}

fn metrics_middleware_fn(request: Request, next: Next) -> MiddlewareFuture {
    Box::pin(async move {
        let start = Instant::now();
        let method = request.method().to_string();
        let path = request.uri().path().to_string();

        METRICS.http_active_connections.inc();

        let response = next.run(request).await;

        METRICS.http_active_connections.dec();

        let duration = start.elapsed().as_secs_f64();
        let status = response.status().as_u16();

        METRICS.record_http_request(&method, &path, status, duration);

        tracing::debug!(
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %format!("{:.2}", duration * 1000.0),
            "Request completed"
        );

        response
    })
}

/// Alias for request_id_layer
pub type RequestIdLayer = axum::middleware::FromFnLayer<
    fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
    (),
    Request,
>;

/// Alias for metrics_layer
pub type MetricsLayer = axum::middleware::FromFnLayer<
    fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
    (),
    Request,
>;

/// Get metrics endpoint handler.
pub async fn metrics_handler() -> Response<Body> {
    let metrics_output = METRICS.encode();

    Response::builder()
        .status(200)
        .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
        .body(Body::from(metrics_output))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body(Body::from("Failed to encode metrics"))
                .expect("Failed to build error response")
        })
}
