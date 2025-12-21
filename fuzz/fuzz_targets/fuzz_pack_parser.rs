//! Fuzz target for Git pack file parsing.
//!
//! Tests that the pack parser handles arbitrary input without panicking.

#![no_main]

use guts_storage::ObjectStore;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Create a fresh object store for each fuzz iteration
    let store = ObjectStore::new();

    // Try to parse the data as a pack file
    let mut parser = guts_git::PackParser::new(data);
    let _ = parser.parse(&store);
});
