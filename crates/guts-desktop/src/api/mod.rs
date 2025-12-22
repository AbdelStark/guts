//! # API Client
//!
//! HTTP client for communicating with `guts-node`.
//!
//! This module provides the [`GutsClient`] for making API requests
//! to a running guts-node instance.

mod client;
mod error;
mod types;

pub use client::GutsClient;
#[allow(unused_imports)]
pub use error::{ApiError, ApiResult};
#[allow(unused_imports)]
pub use types::{
    ContentEntry, ContentType, ContentsResponse, CreateRepoRequest, CreateTokenRequest,
    CreateUserRequest, RepoInfo, Repository, TokenResponse, UserProfile, Visibility,
};
