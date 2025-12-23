//! # Layout Component
//!
//! Main application layout wrapper providing consistent structure.

use dioxus::prelude::*;

use super::{Header, Sidebar};
use crate::router::Route;
use crate::state::AppState;

/// Main layout wrapper component.
///
/// Provides the application shell with sidebar navigation and header.
/// All routed views are rendered inside the main content area via `Outlet`.
///
/// # Structure
///
/// ```text
/// +---------------------------------------------+
/// | Sidebar |         Header                    |
/// |         |------------------------------------|
/// |  Nav    |                                   |
/// |  Items  |         Main Content              |
/// |         |         (Outlet)                  |
/// |         |                                   |
/// +---------------------------------------------+
/// ```
#[component]
pub fn Layout() -> Element {
    let mut state = use_context::<AppState>();

    // Auto-connect on startup if user is logged in
    use_effect(move || {
        if state.is_logged_in() {
            let client = state.client();
            spawn(async move {
                match client.health().await {
                    Ok(true) => {
                        state.connected.set(true);
                        tracing::info!("Auto-connected to node");
                    }
                    Ok(false) => {
                        state.connected.set(false);
                        tracing::warn!("Node is unhealthy");
                    }
                    Err(e) => {
                        state.connected.set(false);
                        tracing::warn!("Failed to auto-connect: {}", e);
                    }
                }
            });
        }
    });

    rsx! {
        div {
            class: "app-layout",

            Sidebar {}

            div {
                class: "main-panel",

                Header {}

                main {
                    class: "content",

                    Outlet::<Route> {}
                }
            }
        }
    }
}
