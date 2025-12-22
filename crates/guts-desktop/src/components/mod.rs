//! # UI Components
//!
//! Reusable UI components for the Guts desktop application.
//!
//! This module provides the main layout components:
//! - [`Layout`] - Main application layout wrapper
//! - [`Sidebar`] - Navigation sidebar
//! - [`Header`] - Application header

mod header;
mod layout;
mod sidebar;

pub use header::Header;
pub use layout::Layout;
pub use sidebar::Sidebar;
