//! # Sidebar Component
//!
//! Navigation sidebar for the application.

use dioxus::prelude::*;

use crate::router::Route;

/// Navigation sidebar component.
///
/// Provides navigation links to the main application routes.
#[component]
pub fn Sidebar() -> Element {
    rsx! {
        nav {
            class: "sidebar",

            div {
                class: "sidebar-brand",
                "Guts"
            }

            div {
                class: "nav-links",

                Link {
                    to: Route::Home {},
                    class: "nav-link",
                    "Home"
                }

                Link {
                    to: Route::Settings {},
                    class: "nav-link",
                    "Settings"
                }
            }
        }
    }
}
