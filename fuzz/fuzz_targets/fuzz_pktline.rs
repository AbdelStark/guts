//! Fuzz target for Git pkt-line protocol parsing.
//!
//! Tests that the pkt-line reader handles arbitrary input without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Create a cursor over the input data
    let cursor = Cursor::new(data);

    // Create a pkt-line reader and try to read packets
    let mut reader = guts_git::PktLineReader::new(cursor);

    // Try to read up to 100 packets (prevent infinite loops on crafted input)
    for _ in 0..100 {
        match reader.read() {
            Ok(Some(_)) => continue,
            Ok(None) => break, // Flush packet
            Err(_) => break,   // Error is expected for malformed input
        }
    }
});
