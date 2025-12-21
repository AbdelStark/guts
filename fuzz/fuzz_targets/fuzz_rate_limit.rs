#![no_main]

use libfuzzer_sys::fuzz_target;
use std::net::{IpAddr, Ipv4Addr};

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    // Create rate limiter with fuzzed config
    let config = guts_security::RateLimitConfig {
        unauthenticated_limit: u32::from_le_bytes([data[0], data[1], data[2], data[3]]) % 1000 + 1,
        authenticated_limit: u32::from_le_bytes([data[4], data[5], data[6], data[7]]) % 10000 + 1,
        window_secs: 60,
        adaptive_enabled: data.get(8).map(|b| b % 2 == 0).unwrap_or(true),
        suspicious_threshold: 10,
        block_duration_secs: 60,
    };

    let limiter = guts_security::EnhancedRateLimiter::new(config);

    // Generate IPs from fuzzed data
    for chunk in data.chunks(4) {
        if chunk.len() < 4 {
            continue;
        }

        let ip = IpAddr::V4(Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]));
        let path = String::from_utf8_lossy(&data[data.len() / 2..]);
        let method = if data.get(0).map(|b| b % 2 == 0).unwrap_or(false) {
            "GET"
        } else {
            "POST"
        };

        let ctx = guts_security::RequestContext::new(ip, path.to_string(), method);

        // Check rate limit
        let _ = limiter.check(&ctx);

        // Record some failures
        if chunk[0] % 3 == 0 {
            limiter.record_auth_failure(ip);
        }

        // Record suspicious patterns
        if chunk[1] % 5 == 0 {
            limiter.record_suspicious(ip, guts_security::SuspiciousPattern::RapidRequests);
        }
    }

    // Cleanup
    limiter.cleanup();
});
