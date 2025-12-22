//! # Repository Tree View
//!
//! Directory browser within a repository.

use dioxus::prelude::*;

use crate::api::{ContentEntry, ContentType, ContentsResponse};
use crate::router::Route;
use crate::state::AppState;

/// Repository tree (directory) view.
///
/// Displays the contents of a directory within a repository.
#[component]
pub fn RepositoryTree(owner: String, name: String, branch: String, path: String) -> Element {
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
                                p { class: "text-secondary text-center", "This directory is empty" }
                            }
                        },
                        Some(Ok(ContentsResponse::File(_))) => rsx! {
                            div { class: "error", "Expected a directory, got a file" }
                        },
                        Some(Err(err)) => rsx! {
                            div { class: "error", "Error loading contents: {err}" }
                        },
                        None => rsx! {
                            div { class: "loading", "Loading files..." }
                        },
                    }
                }
            }
        }
    }
}

/// Sort entries: directories first, then alphabetically.
pub fn sort_entries(entries: &[ContentEntry]) -> Vec<ContentEntry> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| match (a.is_dir(), b.is_dir()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    sorted
}

/// Breadcrumb navigation component.
#[component]
pub fn Breadcrumb(owner: String, name: String, branch: String, path: String) -> Element {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();

    rsx! {
        div {
            class: "breadcrumb",

            // Root link
            Link {
                to: Route::Repository { owner: owner.clone(), name: name.clone() },
                class: "breadcrumb-item",
                "{name}"
            }

            // Branch indicator
            span { class: "breadcrumb-branch", "@ {branch}" }

            // Path segments
            for (i, part) in parts.iter().enumerate() {
                span { class: "breadcrumb-separator", "/" }

                if i < parts.len() - 1 {
                    Link {
                        to: Route::RepositoryTree {
                            owner: owner.clone(),
                            name: name.clone(),
                            branch: branch.clone(),
                            path: parts[..=i].join("/"),
                        },
                        class: "breadcrumb-item",
                        "{part}"
                    }
                } else {
                    span { class: "breadcrumb-item breadcrumb-current", "{part}" }
                }
            }
        }
    }
}

/// Single file/directory entry row.
#[component]
pub fn FileEntry(entry: ContentEntry, owner: String, name: String, branch: String) -> Element {
    let icon_class = match entry.content_type {
        ContentType::Dir => "folder-icon",
        ContentType::File => "file-icon",
        ContentType::Symlink => "symlink-icon",
        ContentType::Submodule => "submodule-icon",
    };

    let route = if entry.is_dir() {
        Route::RepositoryTree {
            owner: owner.clone(),
            name: name.clone(),
            branch: branch.clone(),
            path: entry.path.clone(),
        }
    } else {
        Route::RepositoryBlob {
            owner: owner.clone(),
            name: name.clone(),
            branch: branch.clone(),
            path: entry.path.clone(),
        }
    };

    rsx! {
        Link {
            to: route,
            class: "file-entry",

            span { class: "file-icon {icon_class}" }
            span { class: "file-name", "{entry.name}" }

            if entry.is_file() {
                span { class: "file-size", "{format_size(entry.size)}" }
            }
        }
    }
}

/// Format file size for display.
pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
