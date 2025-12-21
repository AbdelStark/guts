#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz audit event JSON parsing
    if let Ok(s) = std::str::from_utf8(data) {
        // Try to parse as AuditEvent JSON
        let _ = serde_json::from_str::<guts_security::AuditEvent>(s);

        // Try to parse as AuditEntry JSON
        let _ = serde_json::from_str::<guts_security::AuditEntry>(s);

        // Try to parse as AuditQuery JSON
        let _ = serde_json::from_str::<serde_json::Value>(s).map(|v| {
            // Simulate query building from JSON
            if let Some(actor) = v.get("actor").and_then(|a| a.as_str()) {
                let _ = guts_security::AuditQueryBuilder::new()
                    .actor(actor)
                    .build();
            }
        });
    }

    // Test audit log with arbitrary events
    let log = guts_security::AuditLog::with_capacity(100);

    // Create events from fuzzed data
    if data.len() >= 4 {
        let event_type_idx = data[0] % 30;
        let event_type = match event_type_idx {
            0 => guts_security::AuditEventType::Login,
            1 => guts_security::AuditEventType::Logout,
            2 => guts_security::AuditEventType::LoginFailed,
            3 => guts_security::AuditEventType::TokenCreated,
            4 => guts_security::AuditEventType::TokenRevoked,
            5 => guts_security::AuditEventType::PermissionGranted,
            6 => guts_security::AuditEventType::PermissionDenied,
            7 => guts_security::AuditEventType::RepoCreated,
            8 => guts_security::AuditEventType::KeyRotated,
            _ => guts_security::AuditEventType::Login,
        };

        let actor = String::from_utf8_lossy(&data[1..data.len().min(20)]);
        let resource = String::from_utf8_lossy(&data[data.len().min(20)..data.len().min(40)]);

        let event = guts_security::AuditEvent::new(event_type, actor, resource, "fuzz_test");

        // Record and query
        let entry = log.record(event);
        let _ = log.get(entry.id);
        let _ = log.query(&guts_security::AuditQuery::default());
        let _ = log.recent(10);
        let _ = log.export_json();
    }
});
