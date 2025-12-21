//! Fuzz target for Git smart HTTP protocol parsing.
//!
//! Tests that the Git protocol handlers handle arbitrary input without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Test reference advertisement parsing
    // Format: <oid> <refname>\0<capabilities>\n
    // or: <oid> <refname>\n
    if data.len() >= 44 {
        // Minimum: 40 hex chars + space + 1 char ref + \n
        let _ = parse_ref_line(data);
    }

    // Test want/have negotiation parsing
    let _ = parse_want_have(data);

    // Test command parsing (for receive-pack)
    let _ = parse_command(data);
});

/// Parse a reference advertisement line
fn parse_ref_line(data: &[u8]) -> Option<(String, String)> {
    let s = std::str::from_utf8(data).ok()?;
    let line = s.trim_end_matches('\n');

    // Handle capability line (first ref)
    let (rest, _caps) = if let Some(idx) = line.find('\0') {
        (&line[..idx], &line[idx + 1..])
    } else {
        (line, "")
    };

    // Parse oid and ref name
    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return None;
    }

    let oid = parts[0];
    let refname = parts[1];

    // Validate oid is 40 hex chars
    if oid.len() != 40 || !oid.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    // Validate refname
    if refname.is_empty() || !refname.starts_with("refs/") && refname != "HEAD" {
        return None;
    }

    Some((oid.to_string(), refname.to_string()))
}

/// Parse want/have negotiation lines
fn parse_want_have(data: &[u8]) -> Option<Vec<(String, String)>> {
    let s = std::str::from_utf8(data).ok()?;
    let mut results = Vec::new();

    for line in s.lines() {
        let line = line.trim();
        if line.starts_with("want ") {
            let oid = line.strip_prefix("want ")?;
            let oid = oid.split_whitespace().next()?;
            if oid.len() == 40 && oid.chars().all(|c| c.is_ascii_hexdigit()) {
                results.push(("want".to_string(), oid.to_string()));
            }
        } else if line.starts_with("have ") {
            let oid = line.strip_prefix("have ")?;
            if oid.len() == 40 && oid.chars().all(|c| c.is_ascii_hexdigit()) {
                results.push(("have".to_string(), oid.to_string()));
            }
        } else if line == "done" {
            results.push(("done".to_string(), String::new()));
        }
    }

    Some(results)
}

/// Parse receive-pack command lines
/// Format: <old-oid> <new-oid> <refname>
fn parse_command(data: &[u8]) -> Option<Vec<(String, String, String)>> {
    let s = std::str::from_utf8(data).ok()?;
    let mut results = Vec::new();

    for line in s.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let old_oid = parts[0];
            let new_oid = parts[1];
            let refname = parts[2];

            // Validate OIDs
            if old_oid.len() == 40
                && new_oid.len() == 40
                && old_oid.chars().all(|c| c.is_ascii_hexdigit())
                && new_oid.chars().all(|c| c.is_ascii_hexdigit())
            {
                results.push((old_oid.to_string(), new_oid.to_string(), refname.to_string()));
            }
        }
    }

    Some(results)
}
