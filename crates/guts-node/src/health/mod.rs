//! # Health Check Module
//!
//! Comprehensive health checks for production deployments including:
//!
//! - **Liveness Probe**: Is the process running?
//! - **Readiness Probe**: Is the service ready to accept traffic?
//! - **Startup Probe**: Has initial startup completed?
//!
//! ## Usage
//!
//! ```rust,ignore
//! use axum::Router;
//! use guts_node::health::{health_routes, HealthState};
//!
//! let health_state = HealthState::new();
//! health_state.set_ready(true);
//!
//! let app: Router<()> = Router::new()
//!     .merge(health_routes(health_state));
//! ```

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use parking_lot::RwLock;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Health status values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Component is healthy.
    Up,
    /// Component is unhealthy.
    Down,
    /// Component status is unknown.
    Unknown,
}

/// Individual component health.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    /// Component status.
    pub status: HealthStatus,
    /// Optional latency in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    /// Optional additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ComponentHealth {
    /// Create a healthy component.
    pub fn up() -> Self {
        Self {
            status: HealthStatus::Up,
            latency_ms: None,
            details: None,
        }
    }

    /// Create a healthy component with latency.
    pub fn up_with_latency(latency: Duration) -> Self {
        Self {
            status: HealthStatus::Up,
            latency_ms: Some(latency.as_millis() as u64),
            details: None,
        }
    }

    /// Create an unhealthy component.
    pub fn down() -> Self {
        Self {
            status: HealthStatus::Down,
            latency_ms: None,
            details: None,
        }
    }

    /// Create an unhealthy component with reason.
    pub fn down_with_reason(reason: &str) -> Self {
        Self {
            status: HealthStatus::Down,
            latency_ms: None,
            details: Some(serde_json::json!({ "reason": reason })),
        }
    }
}

/// Liveness probe response.
#[derive(Debug, Clone, Serialize)]
pub struct LivenessResponse {
    /// Overall status.
    pub status: HealthStatus,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
}

/// Readiness probe response.
#[derive(Debug, Clone, Serialize)]
pub struct ReadinessResponse {
    /// Overall status.
    pub status: HealthStatus,
    /// Component health checks.
    pub checks: ReadinessChecks,
}

/// Readiness component checks.
#[derive(Debug, Clone, Serialize)]
pub struct ReadinessChecks {
    /// Storage subsystem health.
    pub storage: ComponentHealth,
    /// P2P network health.
    pub p2p: ComponentHealth,
    /// Real-time (WebSocket) health.
    pub realtime: ComponentHealth,
}

/// Startup probe response.
#[derive(Debug, Clone, Serialize)]
pub struct StartupResponse {
    /// Overall status.
    pub status: HealthStatus,
    /// Startup duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_duration_ms: Option<u64>,
}

/// Overall health response.
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    /// Overall status.
    pub status: HealthStatus,
    /// Version info.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
    /// Component checks.
    pub checks: ReadinessChecks,
}

/// Health state for tracking component health.
#[derive(Clone)]
pub struct HealthState {
    /// When the service started.
    start_time: Instant,
    /// Whether startup is complete.
    startup_complete: Arc<AtomicBool>,
    /// Whether the service is ready.
    ready: Arc<AtomicBool>,
    /// Component health states.
    components: Arc<RwLock<ComponentStates>>,
}

/// Mutable component states.
#[derive(Default)]
struct ComponentStates {
    storage_healthy: bool,
    p2p_connected: bool,
    p2p_peer_count: usize,
    realtime_healthy: bool,
    websocket_connections: usize,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthState {
    /// Create a new health state.
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            startup_complete: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(false)),
            components: Arc::new(RwLock::new(ComponentStates::default())),
        }
    }

    /// Get uptime in seconds.
    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Mark startup as complete.
    pub fn set_startup_complete(&self, complete: bool) {
        self.startup_complete.store(complete, Ordering::SeqCst);
    }

    /// Check if startup is complete.
    pub fn is_startup_complete(&self) -> bool {
        self.startup_complete.load(Ordering::SeqCst)
    }

    /// Set readiness state.
    pub fn set_ready(&self, ready: bool) {
        self.ready.store(ready, Ordering::SeqCst);
    }

    /// Check if service is ready.
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    /// Update storage health.
    pub fn set_storage_healthy(&self, healthy: bool) {
        self.components.write().storage_healthy = healthy;
    }

    /// Update P2P health.
    pub fn set_p2p_connected(&self, connected: bool, peer_count: usize) {
        let mut components = self.components.write();
        components.p2p_connected = connected;
        components.p2p_peer_count = peer_count;
    }

    /// Update realtime health.
    pub fn set_realtime_healthy(&self, healthy: bool, connection_count: usize) {
        let mut components = self.components.write();
        components.realtime_healthy = healthy;
        components.websocket_connections = connection_count;
    }

    /// Get storage component health.
    fn storage_health(&self) -> ComponentHealth {
        let components = self.components.read();
        if components.storage_healthy {
            ComponentHealth::up()
        } else {
            ComponentHealth::down()
        }
    }

    /// Get P2P component health.
    fn p2p_health(&self) -> ComponentHealth {
        let components = self.components.read();
        if components.p2p_connected {
            ComponentHealth {
                status: HealthStatus::Up,
                latency_ms: None,
                details: Some(serde_json::json!({
                    "peer_count": components.p2p_peer_count
                })),
            }
        } else {
            // P2P might not be enabled, so unknown is acceptable
            ComponentHealth {
                status: HealthStatus::Unknown,
                latency_ms: None,
                details: Some(serde_json::json!({
                    "reason": "P2P not connected or not enabled"
                })),
            }
        }
    }

    /// Get realtime component health.
    fn realtime_health(&self) -> ComponentHealth {
        let components = self.components.read();
        ComponentHealth {
            status: if components.realtime_healthy {
                HealthStatus::Up
            } else {
                HealthStatus::Down
            },
            latency_ms: None,
            details: Some(serde_json::json!({
                "connections": components.websocket_connections
            })),
        }
    }

    /// Get readiness checks.
    fn readiness_checks(&self) -> ReadinessChecks {
        ReadinessChecks {
            storage: self.storage_health(),
            p2p: self.p2p_health(),
            realtime: self.realtime_health(),
        }
    }
}

/// Create health check routes.
pub fn health_routes<S>(state: HealthState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/health", get(health_handler))
        .route("/health/live", get(liveness_handler))
        .route("/health/ready", get(readiness_handler))
        .route("/health/startup", get(startup_handler))
        .with_state(state)
}

/// Overall health handler.
async fn health_handler(State(state): State<HealthState>) -> Response {
    let checks = state.readiness_checks();
    let overall_status = if state.is_ready()
        && checks.storage.status == HealthStatus::Up
        && checks.realtime.status == HealthStatus::Up
    {
        HealthStatus::Up
    } else {
        HealthStatus::Down
    };

    let response = HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.uptime(),
        checks,
    };

    let status_code = match overall_status {
        HealthStatus::Up => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response)).into_response()
}

/// Liveness probe handler.
async fn liveness_handler(State(state): State<HealthState>) -> Response {
    let response = LivenessResponse {
        status: HealthStatus::Up,
        uptime_seconds: state.uptime(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Readiness probe handler.
async fn readiness_handler(State(state): State<HealthState>) -> Response {
    if !state.is_ready() {
        let response = ReadinessResponse {
            status: HealthStatus::Down,
            checks: state.readiness_checks(),
        };
        return (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response();
    }

    let checks = state.readiness_checks();
    let overall_status = if checks.storage.status == HealthStatus::Up {
        HealthStatus::Up
    } else {
        HealthStatus::Down
    };

    let response = ReadinessResponse {
        status: overall_status,
        checks,
    };

    let status_code = match overall_status {
        HealthStatus::Up => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response)).into_response()
}

/// Startup probe handler.
async fn startup_handler(State(state): State<HealthState>) -> Response {
    if state.is_startup_complete() {
        let response = StartupResponse {
            status: HealthStatus::Up,
            startup_duration_ms: None,
        };
        (StatusCode::OK, Json(response)).into_response()
    } else {
        let response = StartupResponse {
            status: HealthStatus::Down,
            startup_duration_ms: None,
        };
        (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_state() {
        let state = HealthState::new();

        assert!(!state.is_startup_complete());
        assert!(!state.is_ready());

        state.set_startup_complete(true);
        state.set_ready(true);

        assert!(state.is_startup_complete());
        assert!(state.is_ready());
    }

    #[test]
    fn test_component_health() {
        let up = ComponentHealth::up();
        assert_eq!(up.status, HealthStatus::Up);

        let down = ComponentHealth::down_with_reason("test failure");
        assert_eq!(down.status, HealthStatus::Down);
        assert!(down.details.is_some());
    }
}
