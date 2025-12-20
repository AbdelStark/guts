//! Git protocol implementation for Guts.
//!
//! This crate implements the git pack file format and smart HTTP protocol,
//! enabling standard git clients to push and pull from Guts repositories.

mod error;
mod pack;
mod pktline;
mod protocol;

pub use error::GitError;
pub use pack::{PackBuilder, PackParser};
pub use pktline::{PktLine, PktLineReader, PktLineWriter};
pub use protocol::{
    advertise_refs, receive_pack, upload_pack, Command, RefAdvertisement, WantHave,
};

/// Result type for git protocol operations.
pub type Result<T> = std::result::Result<T, GitError>;
