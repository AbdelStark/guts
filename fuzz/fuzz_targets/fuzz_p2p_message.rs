//! Fuzz target for P2P message parsing.
//!
//! Tests that the P2P message decoder handles arbitrary input without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the main Message::decode function
    let _ = guts_p2p::Message::decode(data);

    // Also fuzz individual message type decoders if there's enough data
    if !data.is_empty() {
        // RepoAnnounce expects payload without message type byte
        let _ = guts_p2p::RepoAnnounce::decode(data);

        // SyncRequest expects payload without message type byte
        let _ = guts_p2p::SyncRequest::decode(data);

        // ObjectData expects payload without message type byte
        let _ = guts_p2p::ObjectData::decode(data);

        // RefUpdate expects payload without message type byte
        let _ = guts_p2p::RefUpdate::decode(data);
    }
});
