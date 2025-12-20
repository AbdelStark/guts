//! # Guts Protocol
//!
//! Network protocol definitions for the Guts P2P network.
//!
//! This crate defines the message types and serialization formats
//! used for communication between Guts nodes.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod messages;
mod version;

pub use error::{ProtocolError, Result};
pub use messages::{Message, MessageKind};
pub use version::{Version, PROTOCOL_VERSION};

/// Magic bytes identifying Guts protocol messages.
pub const MAGIC: [u8; 4] = *b"GUTS";

/// Maximum message size in bytes (16 MB).
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;
