//! Guts Web Gateway
//!
//! Provides browser-based access to Guts repositories, including:
//! - Repository browsing with file tree navigation
//! - Pull request and issue viewing
//! - Markdown rendering for README files
//! - Syntax-highlighted code viewing

pub mod error;
pub mod routes;
pub mod templates;

// Markdown module is available but not currently used in routes
#[allow(dead_code)]
pub mod markdown;

pub use error::WebError;
pub use routes::{web_routes, WebState};
pub use templates::*;
