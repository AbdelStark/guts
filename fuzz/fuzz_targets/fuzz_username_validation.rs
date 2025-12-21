//! Fuzz target for username validation.
//!
//! Tests that the username validator handles arbitrary input without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to interpret the input as a UTF-8 string
    if let Ok(s) = std::str::from_utf8(data) {
        // Validate the username - should never panic
        let _ = guts_compat::User::validate_username(s);
    }

    // Also try with lossy conversion (includes invalid UTF-8 bytes as replacement chars)
    let lossy = String::from_utf8_lossy(data);
    let _ = guts_compat::User::validate_username(&lossy);
});
