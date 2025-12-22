//! Performance optimizations for Guts node.
//!
//! This module contains:
//! - Connection pooling for efficient resource management
//! - Request coalescing to deduplicate concurrent identical requests
//! - CDN-friendly cache headers
//! - Archive pre-generation

mod cache_headers;
mod coalescing;
mod pool;

pub use cache_headers::{add_cache_headers, cache_control_layer, CacheControl};
pub use coalescing::{CoalescerConfig, CoalescerStats, RequestCoalescer};
pub use pool::{ConnectionPool, PoolConfig, PoolStats, PooledConnection};
