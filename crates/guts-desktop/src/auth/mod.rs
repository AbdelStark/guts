//! # Authentication Module
//!
//! Handles user identity and credentials for the desktop client.
//!
//! ## Components
//!
//! - [`Identity`] - Ed25519 keypair generation and storage
//! - [`Credentials`] - User credentials including username, key, and token

mod credentials;
mod identity;

pub use credentials::Credentials;
pub use identity::Identity;
