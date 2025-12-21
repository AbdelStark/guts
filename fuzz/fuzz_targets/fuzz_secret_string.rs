#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test SecretString with arbitrary data
    if let Ok(s) = std::str::from_utf8(data) {
        let secret = guts_security::SecretString::new(s);

        // Verify basic properties
        assert_eq!(secret.len(), s.len());
        assert_eq!(secret.is_empty(), s.is_empty());

        // Verify expose returns original
        assert_eq!(secret.expose(), s);

        // Verify Debug doesn't leak
        let debug = format!("{:?}", secret);
        if !s.is_empty() && s.len() > 1 {
            assert!(!debug.contains(s));
        }
        assert!(debug.contains("REDACTED"));

        // Verify Display doesn't leak
        let display = format!("{}", secret);
        if !s.is_empty() && s.len() > 1 {
            assert!(!display.contains(s));
        }

        // Test equality
        let secret2 = guts_security::SecretString::new(s);
        assert_eq!(secret, secret2);

        // Test serialization
        if let Ok(json) = serde_json::to_string(&secret) {
            // Deserialize back
            if let Ok(deserialized) = serde_json::from_str::<guts_security::SecretString>(&json) {
                assert_eq!(deserialized.expose(), secret.expose());
            }
        }
    }

    // Test with binary data (should fail UTF-8 conversion gracefully)
    let secret = guts_security::SecretString::new(String::from_utf8_lossy(data).to_string());
    let _ = secret.expose();
    let _ = format!("{:?}", secret);
});
