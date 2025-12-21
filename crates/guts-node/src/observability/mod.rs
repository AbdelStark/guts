//! # Observability Module
//!
//! Production-grade observability for the Guts node including:
//!
//! - **Structured Logging**: JSON-formatted logs with request IDs and context
//! - **Prometheus Metrics**: HTTP, P2P, storage, and business metrics
//! - **Request Tracing**: Request ID propagation across all operations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use axum::Router;
//! use guts_node::observability::{init_logging, MetricsState, request_id_layer};
//!
//! // Initialize logging
//! init_logging("info", true);
//!
//! // Create metrics state
//! let metrics = MetricsState::new();
//!
//! // Add request ID layer to router
//! let app: Router<()> = Router::new()
//!     .layer(request_id_layer());
//! ```

mod logging;
mod metrics;
pub mod middleware;

pub use logging::{init_logging, LogFormat};
pub use metrics::MetricsState;
pub use middleware::{metrics_layer, request_id_layer, MiddlewareLayer, REQUEST_ID_HEADER};

/// Re-export commonly used types
pub use uuid::Uuid;
