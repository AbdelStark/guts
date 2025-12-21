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

    #[test]
    fn test_message_kind_all_invalid_values() {
        // Test all values from 3 to 255 are invalid
        for i in 3..=255u8 {
            assert!(
                MessageKind::from_u8(i).is_none(),
                "Expected None for value {}",
                i
            );
        }
    }

    #[test]
    fn test_message_kind_to_hex() {
        assert_eq!(MessageKind::RepoUpdate.to_hex(), hex(&[0]));
        assert_eq!(MessageKind::PullRequest.to_hex(), hex(&[1]));
        assert_eq!(MessageKind::Issue.to_hex(), hex(&[2]));
    }

    #[test]
    fn test_namespace_constant() {
        assert_eq!(NAMESPACE, b"_GUTS");
        assert_eq!(NAMESPACE.len(), 5);
    }

    #[test]
    fn test_epoch_constants() {
        assert_eq!(EPOCH, 0);
        assert_eq!(EPOCH_LENGTH, u64::MAX);
    }
}
