//! Common types used throughout `guts`.
//!
//! This crate provides the core types for the Guts decentralized
//! code collaboration platform.

mod identity;
mod repository;

pub use identity::{Identity, PublicKey, Signature};
pub use repository::{Repository, RepositoryId};

use commonware_utils::hex;

/// The unique namespace prefix used in all signing operations to prevent signature replay attacks.
pub const NAMESPACE: &[u8] = b"_GUTS";

/// The epoch number used in consensus.
///
/// Because Guts does not yet implement reconfiguration (validator set changes),
/// we hardcode the epoch to 0.
pub const EPOCH: u64 = 0;

/// The epoch length used in consensus.
///
/// Because Guts does not yet implement reconfiguration,
/// we hardcode the epoch length to u64::MAX.
pub const EPOCH_LENGTH: u64 = u64::MAX;

/// Message types for the Guts protocol.
#[repr(u8)]
pub enum MessageKind {
    /// Repository update announcement.
    RepoUpdate = 0,
    /// Pull request created.
    PullRequest = 1,
    /// Issue created.
    Issue = 2,
}

impl MessageKind {
    /// Converts a u8 to a MessageKind.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::RepoUpdate),
            1 => Some(Self::PullRequest),
            2 => Some(Self::Issue),
            _ => None,
        }
    }

    /// Converts the MessageKind to a hex string.
    pub fn to_hex(&self) -> String {
        match self {
            Self::RepoUpdate => hex(&[0]),
            Self::PullRequest => hex(&[1]),
            Self::Issue => hex(&[2]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_kind_roundtrip() {
        assert!(matches!(
            MessageKind::from_u8(0),
            Some(MessageKind::RepoUpdate)
        ));
        assert!(matches!(
            MessageKind::from_u8(1),
            Some(MessageKind::PullRequest)
        ));
        assert!(matches!(MessageKind::from_u8(2), Some(MessageKind::Issue)));
        assert!(MessageKind::from_u8(255).is_none());
    }
}
