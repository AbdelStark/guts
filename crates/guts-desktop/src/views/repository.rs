//! # Repository View
//!
//! Single repository details view with file browsing.

use dioxus::prelude::*;

use crate::api::ContentsResponse;
use crate::router::Route;
use crate::state::AppState;

use super::repository_tree::{sort_entries, Breadcrumb, FileEntry};

/// Repository details view.
///
/// Shows repository info and root directory contents.
///
/// # Props
///
/// * `owner` - The repository owner
/// * `name` - The repository name
#[component]
pub fn Repository(owner: String, name: String) -> Element {
    let state = use_context::<AppState>();
    let client = state.client();
    let nav = use_navigator();

    let owner_clone = owner.clone();
    let name_clone = name.clone();

    // Delete confirmation state
    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let mut delete_error = use_signal(|| Option::<String>::None);

    // Fetch single repository
    let repo = use_resource(move || {
        let client = client.clone();
        let owner = owner_clone.clone();
        let name = name_clone.clone();
        async move { client.get_repository(&owner, &name).await }
    });

    // Delete handler
    let owner_for_delete = owner.clone();
    let name_for_delete = name.clone();
    let mut do_delete = move || {
        deleting.set(true);
        delete_error.set(None);

        let client = state.client();
        let owner = owner_for_delete.clone();
        let name = name_for_delete.clone();

        spawn(async move {
            match client.delete_repository(&owner, &name).await {
                Ok(()) => {
                    // Navigate back to home on success
                    nav.push(Route::Home {});
                }
                Err(e) => {
                    delete_error.set(Some(format!("Failed to delete: {}", e)));
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "repository-view",

            Link {
                to: Route::Home {},
                class: "back-link",
                "← Back to Home"
            }

            match &*repo.read() {
                Some(Ok(r)) => rsx! {
                    div {
                        class: "repo-header mb-lg",

                        div {
                            class: "repo-header-content",

                            div {
                                h2 { "{r.owner}/{r.name}" }

                                if let Some(desc) = &r.description {
                                    p { class: "text-secondary", "{desc}" }
                                }

                                div {
                                    class: "repo-meta-inline",

                                    span { class: "badge", "{r.default_branch}" }
                                    span { class: "badge badge-secondary", "{r.visibility:?}" }
                                }
                            }

                            button {
                                class: "btn-danger btn-sm",
                                onclick: move |_| show_delete_confirm.set(true),
                                "Delete"
                            }
                        }
                    }

                    // Delete confirmation modal
                    if *show_delete_confirm.read() {
                        div {
                            class: "modal-overlay",
                            onclick: move |_| show_delete_confirm.set(false),

                            div {
                                class: "modal-content glass-panel",
                                onclick: move |evt| evt.stop_propagation(),

                                h3 { "Delete Repository" }
                                p { class: "text-secondary",
                                    "Are you sure you want to delete "
                                    strong { "{r.owner}/{r.name}" }
                                    "? This action cannot be undone."
                                }

                                div {
                                    class: "modal-actions",

                                    button {
                                        class: "btn-ghost",
                                        disabled: *deleting.read(),
                                        onclick: move |_| show_delete_confirm.set(false),
                                        "Cancel"
                                    }

                                    button {
                                        class: "btn-danger",
                                        disabled: *deleting.read(),
                                        onclick: move |_| do_delete(),
                                        if *deleting.read() {
                                            "Deleting..."
                                        } else {
                                            "Delete Repository"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Error message
                    if let Some(err) = delete_error.read().as_ref() {
                        div { class: "alert alert-error mb-lg", "{err}" }
                    }

                    // File browser
                    FileBrowser {
                        owner: owner.clone(),
                        name: name.clone(),
                        branch: r.default_branch.clone(),
                    }
                },
                Some(Err(err)) => rsx! {
                    div {
                        class: "error mt-lg",
                        "Error loading repository: {err}"
                    }
                },
                None => rsx! {
                    div {
                        class: "loading mt-lg",
                        "Loading repository..."
                    }
                },
            }
        }
    }
}

/// File browser component for repository root contents.
#[component]
fn FileBrowser(owner: String, name: String, branch: String) -> Element {
    let state = use_context::<AppState>();
    let client = state.client();

    let owner_clone = owner.clone();
    let name_clone = name.clone();
    let branch_clone = branch.clone();

    // Fetch root contents
    let contents = use_resource(move || {
        let client = client.clone();
        let owner = owner_clone.clone();
        let name = name_clone.clone();
        let branch = branch_clone.clone();
        async move { client.get_contents(&owner, &name, "", Some(&branch)).await }
    });

    rsx! {
        div {
            class: "file-browser glass-panel-static",

            Breadcrumb {
                owner: owner.clone(),
                name: name.clone(),
                branch: branch.clone(),
                path: String::new(),
            }

            div {
                class: "file-list",

                match &*contents.read() {
                    Some(Ok(ContentsResponse::Directory(entries))) => rsx! {
                        for entry in sort_entries(entries) {
                            FileEntry {
                                entry: entry.clone(),
                                owner: owner.clone(),
                                name: name.clone(),
                                branch: branch.clone(),
                            }
                        }

                        if entries.is_empty() {
                            p { class: "text-secondary text-center", "This repository is empty" }
                        }
                    },
                    Some(Ok(ContentsResponse::File(_))) => rsx! {
                        div { class: "error", "Unexpected file at root" }
                    },
                    Some(Err(_)) => rsx! {
                        div { class: "text-secondary text-center", "No files yet (push some code to get started)" }
                    },
                    None => rsx! {
                        div { class: "loading", "Loading files..." }
                    },
                }
            }
        }
    }
}
