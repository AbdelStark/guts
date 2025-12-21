//! Prometheus metrics collection.
//!
//! Provides comprehensive metrics for:
//! - HTTP request latency and counts
//! - P2P message statistics
//! - Storage operations
//! - Business metrics (repos, PRs, issues)

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;
use std::sync::Arc;

/// HTTP request labels.
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HttpLabels {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path pattern
    pub path: String,
    /// Response status code
    pub status: u16,
}

/// P2P message labels.
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct P2pLabels {
    /// Message type
    pub message_type: String,
    /// Direction (sent/received)
    pub direction: String,
}

/// Storage operation labels.
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct StorageLabels {
    /// Operation type (read, write, delete)
    pub operation: String,
    /// Object type (blob, tree, commit)
    pub object_type: String,
}

/// Global metrics state.
pub static METRICS: Lazy<MetricsState> = Lazy::new(MetricsState::new);

/// Metrics state container.
#[derive(Clone)]
pub struct MetricsState {
    /// Prometheus registry.
    pub registry: Arc<RwLock<Registry>>,
    /// HTTP request counter.
    pub http_requests_total: Family<HttpLabels, Counter>,
    /// HTTP request duration histogram (seconds).
    pub http_request_duration_seconds: Family<HttpLabels, Histogram>,
    /// HTTP active connections gauge.
    pub http_active_connections: Gauge,
    /// P2P connected peers gauge.
    pub p2p_peers_connected: Gauge,
    /// P2P messages counter.
    pub p2p_messages_total: Family<P2pLabels, Counter>,
    /// P2P message latency histogram.
    pub p2p_message_latency_seconds: Family<P2pLabels, Histogram>,
    /// Storage objects gauge by type.
    pub storage_objects_total: Family<StorageLabels, Gauge>,
    /// Storage operation duration histogram.
    pub storage_operation_duration_seconds: Family<StorageLabels, Histogram>,
    /// Total repositories gauge.
    pub repositories_total: Gauge,
    /// Pull requests by state.
    pub pull_requests_total: Gauge,
    /// Issues by state.
    pub issues_total: Gauge,
    /// Total users.
    pub users_total: Gauge,
    /// Total organizations.
    pub organizations_total: Gauge,
    /// WebSocket active connections.
    pub websocket_connections: Gauge,
}

impl Default for MetricsState {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsState {
    /// Create a new metrics state with all metrics registered.
    pub fn new() -> Self {
        let mut registry = Registry::default();

        // HTTP metrics
        let http_requests_total = Family::<HttpLabels, Counter>::default();
        registry.register(
            "guts_http_requests",
            "Total HTTP requests",
            http_requests_total.clone(),
        );

        let http_request_duration_seconds =
            Family::<HttpLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(exponential_buckets(0.001, 2.0, 16))
            });
        registry.register(
            "guts_http_request_duration_seconds",
            "HTTP request duration in seconds",
            http_request_duration_seconds.clone(),
        );

        let http_active_connections = Gauge::default();
        registry.register(
            "guts_http_active_connections",
            "Number of active HTTP connections",
            http_active_connections.clone(),
        );

        // P2P metrics
        let p2p_peers_connected = Gauge::default();
        registry.register(
            "guts_p2p_peers_connected",
            "Number of connected P2P peers",
            p2p_peers_connected.clone(),
        );

        let p2p_messages_total = Family::<P2pLabels, Counter>::default();
        registry.register(
            "guts_p2p_messages",
            "Total P2P messages",
            p2p_messages_total.clone(),
        );

        let p2p_message_latency_seconds =
            Family::<P2pLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(exponential_buckets(0.001, 2.0, 16))
            });
        registry.register(
            "guts_p2p_message_latency_seconds",
            "P2P message latency in seconds",
            p2p_message_latency_seconds.clone(),
        );

        // Storage metrics
        let storage_objects_total = Family::<StorageLabels, Gauge>::default();
        registry.register(
            "guts_storage_objects",
            "Total storage objects by type",
            storage_objects_total.clone(),
        );

        let storage_operation_duration_seconds =
            Family::<StorageLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(exponential_buckets(0.0001, 2.0, 16))
            });
        registry.register(
            "guts_storage_operation_duration_seconds",
            "Storage operation duration in seconds",
            storage_operation_duration_seconds.clone(),
        );

        // Business metrics
        let repositories_total = Gauge::default();
        registry.register(
            "guts_repositories",
            "Total number of repositories",
            repositories_total.clone(),
        );

        let pull_requests_total = Gauge::default();
        registry.register(
            "guts_pull_requests",
            "Total number of pull requests",
            pull_requests_total.clone(),
        );

        let issues_total = Gauge::default();
        registry.register(
            "guts_issues",
            "Total number of issues",
            issues_total.clone(),
        );

        let users_total = Gauge::default();
        registry.register("guts_users", "Total number of users", users_total.clone());

        let organizations_total = Gauge::default();
        registry.register(
            "guts_organizations",
            "Total number of organizations",
            organizations_total.clone(),
        );

        let websocket_connections = Gauge::default();
        registry.register(
            "guts_websocket_connections",
            "Active WebSocket connections",
            websocket_connections.clone(),
        );

        Self {
            registry: Arc::new(RwLock::new(registry)),
            http_requests_total,
            http_request_duration_seconds,
            http_active_connections,
            p2p_peers_connected,
            p2p_messages_total,
            p2p_message_latency_seconds,
            storage_objects_total,
            storage_operation_duration_seconds,
            repositories_total,
            pull_requests_total,
            issues_total,
            users_total,
            organizations_total,
            websocket_connections,
        }
    }

    /// Record an HTTP request.
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        let labels = HttpLabels {
            method: method.to_string(),
            path: normalize_path(path),
            status,
        };

        self.http_requests_total.get_or_create(&labels).inc();
        self.http_request_duration_seconds
            .get_or_create(&labels)
            .observe(duration_secs);
    }

    /// Record a P2P message.
    pub fn record_p2p_message(&self, message_type: &str, direction: &str, latency_secs: f64) {
        let labels = P2pLabels {
            message_type: message_type.to_string(),
            direction: direction.to_string(),
        };

        self.p2p_messages_total.get_or_create(&labels).inc();
        if latency_secs > 0.0 {
            self.p2p_message_latency_seconds
                .get_or_create(&labels)
                .observe(latency_secs);
        }
    }

    /// Encode metrics for Prometheus scraping.
    pub fn encode(&self) -> String {
        let mut buffer = String::new();
        let registry = self.registry.read();
        prometheus_client::encoding::text::encode(&mut buffer, &registry)
            .expect("Failed to encode metrics");
        buffer
    }
}

/// Normalize path for metrics (replace dynamic segments).
fn normalize_path(path: &str) -> String {
    // Replace common dynamic path segments with placeholders
    let parts: Vec<&str> = path.split('/').collect();
    let normalized: Vec<&str> = parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            // Skip empty parts and keep static paths
            if part.is_empty() {
                return *part;
            }
            // Detect dynamic segments (UUIDs, numbers, owner/repo patterns)
            if is_dynamic_segment(part, i, &parts) {
                ":param"
            } else {
                *part
            }
        })
        .collect();
    normalized.join("/")
}

/// Check if a path segment is dynamic.
fn is_dynamic_segment(segment: &str, index: usize, parts: &[&str]) -> bool {
    // UUID pattern
    if segment.len() == 36 && segment.contains('-') {
        return true;
    }
    // Pure numeric
    if segment.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    // After /repos or /git, next two segments are owner/name
    if index >= 2 {
        if let Some(parent) = parts.get(index - 2) {
            if *parent == "repos" || *parent == "git" {
                return true;
            }
        }
    }
    if index >= 1 {
        if let Some(parent) = parts.get(index - 1) {
            if *parent == "repos" || *parent == "git" {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/health"), "/health");
        assert_eq!(normalize_path("/api/repos"), "/api/repos");
        assert_eq!(
            normalize_path("/api/repos/alice/myrepo"),
            "/api/repos/:param/:param"
        );
        assert_eq!(
            normalize_path("/git/alice/myrepo/info/refs"),
            "/git/:param/:param/info/refs"
        );
    }

    #[test]
    fn test_metrics_state_creation() {
        let metrics = MetricsState::new();
        metrics.record_http_request("GET", "/health", 200, 0.001);
        let encoded = metrics.encode();
        assert!(encoded.contains("guts_http_requests"));
    }
}
