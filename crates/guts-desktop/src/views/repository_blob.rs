//! # Repository Blob View
//!
//! Single file viewer within a repository.

use dioxus::prelude::*;

use crate::api::{ContentEntry, ContentsResponse};
use crate::router::Route;
use crate::state::AppState;

use super::repository_tree::{format_size, Breadcrumb};

/// Repository blob (file) view.
///
/// Displays the contents of a single file within a repository.
#[component]
pub fn RepositoryBlob(owner: String, name: String, branch: String, path: String) -> Element {
    let state = use_context::<AppState>();
    let client = state.client();

    let owner_clone = owner.clone();
    let name_clone = name.clone();
    let path_clone = path.clone();
    let branch_clone = branch.clone();

    let contents = use_resource(move || {
        let client = client.clone();
        let owner = owner_clone.clone();
        let name = name_clone.clone();
        let path = path_clone.clone();
        let branch = branch_clone.clone();
        async move {
            client
                .get_contents(&owner, &name, &path, Some(&branch))
                .await
        }
    });

    rsx! {
        div {
            class: "repository-view",

            Link {
                to: Route::Repository { owner: owner.clone(), name: name.clone() },
                class: "back-link",
                "← Back to {owner}/{name}"
            }

            div {
                class: "file-browser glass-panel-static",

                Breadcrumb {
                    owner: owner.clone(),
                    name: name.clone(),
                    branch: branch.clone(),
                    path: path.clone(),
                }

                match &*contents.read() {
                    Some(Ok(ContentsResponse::File(entry))) => rsx! {
                        FileViewer { entry: entry.clone() }
                    },
                    Some(Ok(ContentsResponse::Directory(_))) => rsx! {
                        div { class: "error", "Expected a file, got a directory" }
                    },
                    Some(Err(err)) => rsx! {
                        div { class: "error", "Error loading file: {err}" }
                    },
                    None => rsx! {
                        div { class: "loading", "Loading file..." }
                    },
                }
            }
        }
    }
}

/// File content viewer component.
#[component]
fn FileViewer(entry: ContentEntry) -> Element {
    let content = entry
        .decode_content()
        .unwrap_or_else(|| "[Binary file or decoding error]".to_string());

    rsx! {
        div {
            class: "file-viewer",

            div {
                class: "file-viewer-header",
                span { class: "file-name", "{entry.name}" }
                span { class: "file-size", "{format_size(entry.size)}" }
            }

            pre {
                class: "file-content mono",
                code { "{content}" }
            }
        }
    }
}
