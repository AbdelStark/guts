//! # Create Repository View
//!
//! Form for creating new repositories.

use dioxus::prelude::*;

use crate::router::Route;
use crate::state::AppState;

/// Create repository view component.
///
/// Provides a form to create a new repository with name and owner fields.
#[component]
pub fn CreateRepository() -> Element {
    let state = use_context::<AppState>();
    let navigator = use_navigator();

    // Form state
    let mut name_input = use_signal(String::new);
    let mut owner_input = use_signal(|| "alice".to_string());
    let mut creating = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let name = name_input.read().trim().to_string();
        let owner = owner_input.read().trim().to_string();

        // Client-side validation
        if name.is_empty() {
            error_msg.set(Some("Repository name is required".to_string()));
            return;
        }
        if owner.is_empty() {
            error_msg.set(Some("Owner is required".to_string()));
            return;
        }

        let client = state.client();
        creating.set(true);
        error_msg.set(None);

        spawn(async move {
            match client.create_repository(&name, &owner).await {
                Ok(repo) => {
                    // Navigate to the new repository
                    navigator.push(Route::Repository {
                        owner: repo.owner,
                        name: repo.name,
                    });
                }
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                    creating.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "create-repo-view",

            Link {
                to: Route::Home {},
                class: "back-link",
                "← Back to Home"
            }

            h2 { class: "mb-lg", "Create New Repository" }

            form {
                class: "create-repo-form glass-panel-static",
                onsubmit: on_submit,

                div {
                    class: "form-field mb-md",

                    label { r#for: "owner", "Owner" }
                    input {
                        id: "owner",
                        r#type: "text",
                        value: "{owner_input}",
                        placeholder: "Owner name",
                        oninput: move |evt| owner_input.set(evt.value().clone()),
                        disabled: *creating.read(),
                    }
                }

                div {
                    class: "form-field mb-md",

                    label { r#for: "name", "Repository Name" }
                    input {
                        id: "name",
                        r#type: "text",
                        value: "{name_input}",
                        placeholder: "my-awesome-repo",
                        oninput: move |evt| name_input.set(evt.value().clone()),
                        disabled: *creating.read(),
                        autofocus: true,
                    }
                }

                // Error message
                if let Some(err) = error_msg.read().as_ref() {
                    div {
                        class: "alert alert-error mb-md",
                        "{err}"
                    }
                }

                div {
                    class: "btn-group",

                    button {
                        class: "btn-primary",
                        r#type: "submit",
                        disabled: *creating.read(),
                        if *creating.read() { "Creating..." } else { "Create Repository" }
                    }

                    Link {
                        to: Route::Home {},
                        class: "btn-glass",
                        "Cancel"
                    }
                }
            }
        }
    }
}
