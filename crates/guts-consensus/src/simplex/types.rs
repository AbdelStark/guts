//! Type definitions for Simplex BFT consensus.
//!
//! This module provides the core type aliases used throughout the
//! Simplex consensus implementation.

use commonware_consensus::simplex::signing_scheme::ed25519;
use commonware_consensus::simplex::types::{
    Activity as CActivity, Finalization as CFinalization, Notarization as CNotarization,
};
use commonware_cryptography::sha256::Digest;

/// The signing scheme used for consensus.
///
/// We use ed25519 for simplicity and HSM compatibility.
/// For production with threshold signatures, consider bls12381_threshold.
pub type Scheme = ed25519::Scheme;

/// Notarization certificate type.
pub type Notarization = CNotarization<Scheme, Digest>;

/// Finalization certificate type.
pub type Finalization = CFinalization<Scheme, Digest>;

/// Activity report type.
pub type Activity = CActivity<Scheme, Digest>;

/// Re-export the public key type.
pub use commonware_cryptography::ed25519::PublicKey as ValidatorPublicKey;

/// Re-export the private key type.
pub use commonware_cryptography::ed25519::PrivateKey as ValidatorPrivateKey;
