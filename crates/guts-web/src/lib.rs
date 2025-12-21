//! Guts Web Gateway
//!
//! Provides browser-based access to Guts repositories, including:
//! - Repository browsing with file tree navigation
//! - Pull request and issue viewing
//! - Markdown rendering for README files
//! - Syntax-highlighted code viewing
//! - Search functionality
//! - API documentation with Swagger UI

#![recursion_limit = "512"]

pub mod error;
pub mod markdown;
pub mod routes;
pub mod templates;

pub use error::WebError;
pub use markdown::render_markdown;
pub use routes::{web_routes, WebState};
pub use templates::*;
