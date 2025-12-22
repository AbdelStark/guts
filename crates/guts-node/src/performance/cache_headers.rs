//! CDN-friendly cache headers.
//!
//! Provides utilities for adding appropriate cache headers to responses
//! to enable efficient CDN caching of static and semi-static content.

use axum::{
    body::Body,
    http::{header, HeaderValue, Request, Response},
    middleware::Next,
};
use std::time::Duration;

/// Cache control directives.
#[derive(Debug, Clone)]
pub enum CacheControl {
    /// Content that never changes (Git objects).
    Immutable,
    /// Publicly cacheable for a duration.
    Public(Duration),
    /// Private cache only, for a duration.
    Private(Duration),
    /// Must revalidate on every request.
    MustRevalidate(Duration),
    /// No caching at all.
    NoCache,
    /// No storing at all (sensitive data).
    NoStore,
}

impl CacheControl {
    /// Converts to a Cache-Control header value.
    pub fn to_header_value(&self) -> HeaderValue {
        let value = match self {
            CacheControl::Immutable => "public, max-age=31536000, immutable".to_string(),
            CacheControl::Public(dur) => format!("public, max-age={}", dur.as_secs()),
            CacheControl::Private(dur) => format!("private, max-age={}", dur.as_secs()),
            CacheControl::MustRevalidate(dur) => {
                format!("public, max-age={}, must-revalidate", dur.as_secs())
            }
            CacheControl::NoCache => "no-cache".to_string(),
            CacheControl::NoStore => "no-store, no-cache, must-revalidate".to_string(),
        };
        HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static("no-cache"))
    }

    /// Returns appropriate cache control for Git objects.
    pub fn for_git_object() -> Self {
        // Git objects are immutable (content-addressed)
        CacheControl::Immutable
    }

    /// Returns appropriate cache control for repository metadata.
    pub fn for_repo_metadata() -> Self {
        // Short cache for metadata that may change
        CacheControl::Public(Duration::from_secs(60))
    }

    /// Returns appropriate cache control for refs.
    pub fn for_refs() -> Self {
        // Very short cache for refs (can change frequently)
        CacheControl::MustRevalidate(Duration::from_secs(10))
    }

    /// Returns appropriate cache control for static assets.
    pub fn for_static_assets() -> Self {
        // Long cache for versioned static assets
        CacheControl::Public(Duration::from_secs(86400)) // 1 day
    }

    /// Returns appropriate cache control for API responses.
    pub fn for_api_response() -> Self {
        // Short cache for API responses
        CacheControl::Public(Duration::from_secs(30))
    }

    /// Returns appropriate cache control for authenticated responses.
    pub fn for_authenticated() -> Self {
        // Private cache for authenticated responses
        CacheControl::Private(Duration::from_secs(60))
    }
}

/// Adds cache headers to a response.
pub fn add_cache_headers<B>(response: &mut Response<B>, cache_control: CacheControl) {
    let headers = response.headers_mut();

    // Set Cache-Control
    headers.insert(header::CACHE_CONTROL, cache_control.to_header_value());

    // Set Vary header for proper CDN behavior
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
}

/// Axum middleware layer for automatic cache headers.
pub async fn cache_control_layer(request: Request<Body>, next: Next) -> Response<Body> {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    // Only add cache headers to successful responses
    if !response.status().is_success() {
        return response;
    }

    // Determine appropriate cache control based on path
    let cache_control = determine_cache_control(&path);
    add_cache_headers(&mut response, cache_control);

    response
}

/// Determines appropriate cache control based on request path.
fn determine_cache_control(path: &str) -> CacheControl {
    if path.contains("/objects/") || path.contains("/git-upload-pack") {
        // Git objects are immutable
        CacheControl::Immutable
    } else if path.contains("/info/refs") {
        // Refs can change, short cache with revalidation
        CacheControl::MustRevalidate(Duration::from_secs(10))
    } else if path.starts_with("/static/") || path.starts_with("/assets/") {
        // Static assets
        CacheControl::for_static_assets()
    } else if path.starts_with("/api/") {
        // API responses
        CacheControl::for_api_response()
    } else if path == "/health" || path.starts_with("/health/") {
        // Health checks shouldn't be cached
        CacheControl::NoCache
    } else if path.starts_with("/git/") {
        // General git operations
        CacheControl::MustRevalidate(Duration::from_secs(30))
    } else {
        // Default: short cache
        CacheControl::Public(Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_immutable() {
        let cc = CacheControl::Immutable;
        let value = cc.to_header_value();
        assert_eq!(
            value.to_str().unwrap(),
            "public, max-age=31536000, immutable"
        );
    }

    #[test]
    fn test_cache_control_public() {
        let cc = CacheControl::Public(Duration::from_secs(3600));
        let value = cc.to_header_value();
        assert_eq!(value.to_str().unwrap(), "public, max-age=3600");
    }

    #[test]
    fn test_cache_control_no_store() {
        let cc = CacheControl::NoStore;
        let value = cc.to_header_value();
        assert!(value.to_str().unwrap().contains("no-store"));
    }

    #[test]
    fn test_determine_cache_control_git_objects() {
        let cc = determine_cache_control("/git/owner/repo/objects/abc123");
        assert!(matches!(cc, CacheControl::Immutable));
    }

    #[test]
    fn test_determine_cache_control_refs() {
        let cc = determine_cache_control("/git/owner/repo/info/refs");
        assert!(matches!(cc, CacheControl::MustRevalidate(_)));
    }

    #[test]
    fn test_determine_cache_control_api() {
        let cc = determine_cache_control("/api/repos");
        assert!(matches!(cc, CacheControl::Public(_)));
    }

    #[test]
    fn test_determine_cache_control_health() {
        let cc = determine_cache_control("/health");
        assert!(matches!(cc, CacheControl::NoCache));
    }
}
