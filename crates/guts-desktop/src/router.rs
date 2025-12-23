//! # Routing
//!
//! Defines the application routes and navigation structure.

use dioxus::prelude::*;

use crate::components::Layout;
use crate::views::{
    CreateRepository, Home, Login, Repository, RepositoryBlob, RepositoryTree, Settings,
};

/// Application routes.
///
/// All routes are wrapped in the [`Layout`] component which provides
/// consistent navigation and structure.
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    /// Main layout wrapper for all routes.
    #[layout(Layout)]
    /// Home dashboard showing repositories and node status.
    #[route("/")]
    Home {},

    /// User login/registration.
    #[route("/login")]
    Login {},

    /// Create new repository form.
    #[route("/new")]
    CreateRepository {},

    /// Single repository view with details.
    ///
    /// # Parameters
    ///
    /// * `owner` - The repository owner
    /// * `name` - The repository name
    #[route("/repo/:owner/:name")]
    Repository { owner: String, name: String },

    /// Directory browser within a repository.
    ///
    /// # Parameters
    ///
    /// * `owner` - The repository owner
    /// * `name` - The repository name
    /// * `branch` - The branch name
    /// * `path` - Path within the repository (can contain slashes)
    #[route("/repo/:owner/:name/tree/:branch/*path")]
    RepositoryTree {
        owner: String,
        name: String,
        branch: String,
        path: String,
    },

    /// Single file viewer within a repository.
    ///
    /// # Parameters
    ///
    /// * `owner` - The repository owner
    /// * `name` - The repository name
    /// * `branch` - The branch name
    /// * `path` - Path to the file (can contain slashes)
    #[route("/repo/:owner/:name/blob/:branch/*path")]
    RepositoryBlob {
        owner: String,
        name: String,
        branch: String,
        path: String,
    },

    /// Application settings (node URL, preferences).
    #[route("/settings")]
    Settings {},
}
