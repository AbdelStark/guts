//! Fuzz target for RepoAnnounce message parsing.
//!
//! Tests that the P2P RepoAnnounce message decoder handles arbitrary input without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Prepend the message type byte for RepoAnnounce (0x01)
    let mut message = vec![0x01];
    message.extend_from_slice(data);

    // Try to decode as a complete P2P message
    let _ = guts_p2p::Message::decode(&message);
});
