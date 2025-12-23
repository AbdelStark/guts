//! # Guts Desktop
//!
//! Native desktop client for the Guts decentralized code collaboration platform.
//!
//! ## Architecture
//!
//! This application connects to a running `guts-node` instance via HTTP API
//! and provides a graphical interface for repository management and
//! collaboration features.
//!
//! ## Modules
//!
//! - [`api`] - HTTP client for communicating with guts-node
//! - [`auth`] - User identity and credentials management
//! - [`components`] - Reusable UI components
//! - [`router`] - Application routes
//! - [`state`] - Global application state
//! - [`views`] - Page-level view components

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod api;
mod auth;
mod components;
mod config;
mod router;
mod state;
mod views;

use router::Route;
use state::AppState;

fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    tracing::info!("Starting Guts Desktop");

    // Configure desktop window
    let cfg = Config::new().with_window(
        WindowBuilder::new()
            .with_title("Guts")
            .with_inner_size(LogicalSize::new(1200.0, 800.0))
            .with_min_inner_size(LogicalSize::new(900.0, 600.0)),
    );

    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

/// Root application component.
///
/// Initializes global state, loads the Liquid Glass stylesheet,
/// and renders the router.
#[component]
fn App() -> Element {
    // Provide global application state
    use_context_provider(AppState::new);

    rsx! {
        // Load Liquid Glass design system stylesheet
        document::Stylesheet { href: asset!("/assets/styles.css") }
        Router::<Route> {}
    }
}
