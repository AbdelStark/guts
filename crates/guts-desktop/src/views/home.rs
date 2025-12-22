//! # Home View
//!
//! Dashboard showing repository list and node status.

use dioxus::prelude::*;

use crate::router::Route;
use crate::state::AppState;

/// Home dashboard view.
///
/// Displays a list of repositories fetched from the connected guts-node.
/// When logged in, only shows repositories owned by the current user.
#[component]
pub fn Home() -> Element {
    let state = use_context::<AppState>();
    let client = state.client();

    // Get current user for filtering
    let current_user = state.current_user.read().clone();

    // Fetch repositories
    let repos = use_resource(move || {
        let client = client.clone();
        async move { client.list_repositories().await }
    });

    rsx! {
        div {
            class: "home-view",

            div {
                class: "home-header",

                h2 {
                    if current_user.is_some() {
                        "My Repositories"
                    } else {
                        "Repositories"
                    }
                }

                Link {
                    to: Route::CreateRepository {},
                    class: "btn-primary",
                    "+ New Repository"
                }
            }

            match &*repos.read() {
                Some(Ok(items)) => {
                    // Filter by owner if logged in
                    let filtered: Vec<_> = if let Some(ref user) = current_user {
                        items.iter().filter(|r| r.owner == *user).collect()
                    } else {
                        items.iter().collect()
                    };

                    rsx! {
                        div {
                            class: "repo-list",

                            if filtered.is_empty() {
                                p { class: "text-secondary",
                                    if current_user.is_some() {
                                        "You don't have any repositories yet. Create one to get started!"
                                    } else {
                                        "No repositories found. Create one to get started!"
                                    }
                                }
                            } else {
                                for repo in filtered {
                                    Link {
                                        to: Route::Repository {
                                            owner: repo.owner.clone(),
                                            name: repo.name.clone(),
                                        },

                                        div {
                                            class: "repo-card",

                                            h3 { "{repo.owner}/{repo.name}" }

                                            if let Some(desc) = &repo.description {
                                                p { "{desc}" }
                                            }

                                            div {
                                                class: "meta",
                                                "Branch: {repo.default_branch}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    div {
                        class: "error",
                        "Error loading repositories: {err}"
                    }
                },
                None => rsx! {
                    div {
                        class: "loading",
                        "Loading repositories..."
                    }
                },
            }
        }
    }
}
