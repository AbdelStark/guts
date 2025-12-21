//! Structured logging initialization.
//!
//! Provides production-ready logging with:
//! - JSON or pretty format
//! - Request ID tracking
//! - Configurable log levels
//! - Automatic context propagation

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable pretty format (for development).
    Pretty,
    /// JSON format (for production log aggregation).
    Json,
}

impl LogFormat {
    /// Parse log format from string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => LogFormat::Json,
            _ => LogFormat::Pretty,
        }
    }
}

/// Initialize the logging system.
///
/// # Arguments
///
/// * `level` - Log level (trace, debug, info, warn, error)
/// * `json_format` - If true, output logs in JSON format
///
/// # Example
///
/// ```rust,no_run
/// use guts_node::observability::init_logging;
///
/// init_logging("info", true);
/// ```
pub fn init_logging(level: &str, json_format: bool) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
            "guts={level},tower_http=debug,axum::rejection=trace",
            level = level
        )
        .into()
    });

    let registry = tracing_subscriber::registry().with(env_filter);

    if json_format {
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(false)
                    .with_file(true)
                    .with_line_number(true)
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false),
            )
            .init();
    } else {
        registry.with(fmt::layer().pretty()).init();
    }

    tracing::info!(
        level = %level,
        format = if json_format { "json" } else { "pretty" },
        "Logging initialized"
    );
}
