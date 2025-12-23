//! # Settings View
//!
//! Application settings for node URL and preferences.

use dioxus::prelude::*;

use crate::state::AppState;

/// Settings view component.
///
/// Allows users to configure the node URL and test the connection.
#[component]
pub fn Settings() -> Element {
    let mut state = use_context::<AppState>();
    let mut url_input = use_signal(|| state.node_url.read().clone());
    let mut testing = use_signal(|| false);
    let mut test_result = use_signal(|| Option::<Result<(), String>>::None);

    let on_save = move |_| {
        let url = url_input.read().clone();
        state.set_node_url(url);
        state.save_config();
        test_result.set(None);
    };

    let on_test = move |_| {
        let client = state.client();
        testing.set(true);
        test_result.set(None);

        spawn(async move {
            match client.health().await {
                Ok(true) => {
                    state.connected.set(true);
                    test_result.set(Some(Ok(())));
                }
                Ok(false) => {
                    state.connected.set(false);
                    test_result.set(Some(Err("Node returned unhealthy status".to_string())));
                }
                Err(e) => {
                    state.connected.set(false);
                    test_result.set(Some(Err(e.to_string())));
                }
            }
            testing.set(false);
        });
    };

    rsx! {
        div {
            class: "settings-view",

            h2 { class: "mb-lg", "Settings" }

            div {
                class: "settings-section",

                h3 { class: "mb-md", "Node Connection" }

                div {
                    class: "mb-md",

                    label { "Node URL" }

                    input {
                        r#type: "text",
                        value: "{url_input}",
                        oninput: move |evt| url_input.set(evt.value().clone()),
                    }
                }

                div {
                    class: "btn-group",

                    button {
                        class: "btn-primary",
                        onclick: on_save,
                        "Save"
                    }

                    button {
                        class: "btn-success",
                        onclick: on_test,
                        disabled: *testing.read(),
                        if *testing.read() { "Testing..." } else { "Test Connection" }
                    }
                }

                // Test result
                if let Some(result) = test_result.read().as_ref() {
                    match result {
                        Ok(()) => rsx! {
                            div {
                                class: "alert alert-success",
                                "Connection successful!"
                            }
                        },
                        Err(msg) => rsx! {
                            div {
                                class: "alert alert-error",
                                "Connection failed: {msg}"
                            }
                        },
                    }
                }
            }

            // Current state display
            div {
                class: "current-state",

                h3 { class: "mb-md", "Current State" }

                div {
                    div {
                        strong { "Node URL: " }
                        span { class: "mono", "{state.node_url.read()}" }
                    }

                    div {
                        strong { "Connected: " }
                        if *state.connected.read() {
                            span { class: "text-success", "Yes" }
                        } else {
                            span { class: "text-error", "No" }
                        }
                    }
                }
            }
        }
    }
}
