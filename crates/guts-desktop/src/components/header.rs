//! # Header Component
//!
//! Application header with branding and status.

use dioxus::prelude::*;

use crate::router::Route;
use crate::state::AppState;

/// Application header component.
///
/// Displays the application title, connection status, and user menu.
#[component]
pub fn Header() -> Element {
    let mut state = use_context::<AppState>();
    let connected = state.connected.read();

    let status_class = if *connected {
        "status-indicator connected"
    } else {
        "status-indicator disconnected"
    };

    let on_logout = move |_| {
        state.logout();
    };

    rsx! {
        header {
            class: "app-header",

            h1 { "Guts Desktop" }

            div { class: "header-right",
                // Connection status
                div {
                    class: "connection-status",

                    span {
                        class: "{status_class}",
                    }

                    span {
                        if *connected { "Connected" } else { "Disconnected" }
                    }
                }

                // User menu or login link
                if state.is_logged_in() {
                    div { class: "user-menu",
                        if let Some(username) = state.current_user.read().as_ref() {
                            div { class: "user-avatar",
                                "{username.chars().next().unwrap_or('?').to_uppercase()}"
                            }
                            span { class: "username", "{username}" }
                        }
                        button {
                            class: "btn-sm btn-ghost",
                            onclick: on_logout,
                            "Logout"
                        }
                    }
                } else {
                    Link {
                        class: "btn-sm btn-primary",
                        to: Route::Login {},
                        "Login"
                    }
                }
            }
        }
    }
}
