//! # Views
//!
//! Page-level view components for the Guts desktop application.
//!
//! - [`Home`] - Repository list and dashboard
//! - [`CreateRepository`] - Create new repository form
//! - [`Login`] - User authentication
//! - [`Repository`] - Single repository details
//! - [`RepositoryTree`] - Directory browser
//! - [`RepositoryBlob`] - File viewer
//! - [`Settings`] - Application settings

mod create_repo;
mod home;
mod login;
mod repository;
mod repository_blob;
mod repository_tree;
mod settings;

pub use create_repo::CreateRepository;
pub use home::Home;
pub use login::Login;
pub use repository::Repository;
pub use repository_blob::RepositoryBlob;
pub use repository_tree::RepositoryTree;
pub use settings::Settings;
