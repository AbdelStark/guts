//! SSH key management types.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{CompatError, Result};
use crate::user::UserId;

/// Unique identifier for an SSH key.
pub type SshKeyId = u64;

/// An SSH public key for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    /// Unique key ID.
    pub id: SshKeyId,
    /// User who owns this key.
    pub user_id: UserId,
    /// User-provided title/name.
    pub title: String,
    /// Key type (ed25519, rsa, ecdsa).
    pub key_type: SshKeyType,
    /// Full public key string.
    pub public_key: String,
    /// SHA256 fingerprint.
    pub fingerprint: String,
    /// When the key was added.
    pub created_at: u64,
    /// Last time the key was used.
    pub last_used_at: Option<u64>,
}

impl SshKey {
    /// Create a new SSH key from a public key string.
    pub fn new(id: SshKeyId, user_id: UserId, title: String, public_key: String) -> Result<Self> {
        let (key_type, fingerprint) = parse_and_validate_key(&public_key)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            id,
            user_id,
            title,
            key_type,
            public_key,
            fingerprint,
            created_at: now,
            last_used_at: None,
        })
    }

    /// Update the last_used_at timestamp.
    pub fn touch(&mut self) {
        self.last_used_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// Convert to API response.
    pub fn to_response(&self) -> SshKeyResponse {
        SshKeyResponse {
            id: self.id,
            title: self.title.clone(),
            key_type: self.key_type,
            key: self.public_key.clone(),
            fingerprint: self.fingerprint.clone(),
            created_at: format_timestamp(self.created_at),
            last_used_at: self.last_used_at.map(format_timestamp),
        }
    }
}

/// SSH key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SshKeyType {
    /// Ed25519 key (preferred).
    Ed25519,
    /// RSA key.
    Rsa,
    /// ECDSA key.
    Ecdsa,
}

impl std::fmt::Display for SshKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "ssh-ed25519"),
            Self::Rsa => write!(f, "ssh-rsa"),
            Self::Ecdsa => write!(f, "ecdsa-sha2-nistp256"),
        }
    }
}

/// Parse and validate an SSH public key string.
///
/// Returns the key type and SHA256 fingerprint.
fn parse_and_validate_key(key: &str) -> Result<(SshKeyType, String)> {
    let parts: Vec<&str> = key.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(CompatError::InvalidSshKey(
            "key must have at least type and data parts".to_string(),
        ));
    }

    let key_type = match parts[0] {
        "ssh-ed25519" => SshKeyType::Ed25519,
        "ssh-rsa" => SshKeyType::Rsa,
        "ecdsa-sha2-nistp256" | "ecdsa-sha2-nistp384" | "ecdsa-sha2-nistp521" => SshKeyType::Ecdsa,
        other => {
            return Err(CompatError::InvalidSshKey(format!(
                "unsupported key type: {}",
                other
            )));
        }
    };

    // Validate base64 encoding and calculate fingerprint
    let key_data = parts[1];
    let decoded = base64_decode(key_data).map_err(|e| {
        CompatError::InvalidSshKey(format!("invalid base64 encoding: {}", e))
    })?;

    // Validate key data has correct structure
    validate_key_data(&decoded, key_type)?;

    // Calculate SHA256 fingerprint
    let fingerprint = calculate_fingerprint(&decoded);

    Ok((key_type, fingerprint))
}

/// Basic base64 decoding.
fn base64_decode(input: &str) -> std::result::Result<Vec<u8>, &'static str> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    fn char_to_value(c: u8) -> std::result::Result<u8, &'static str> {
        if let Some(pos) = ALPHABET.iter().position(|&x| x == c) {
            Ok(pos as u8)
        } else if c == b'=' {
            Ok(0) // Padding
        } else {
            Err("invalid base64 character")
        }
    }

    let input = input.trim();
    if input.is_empty() {
        return Err("empty input");
    }

    let bytes: Vec<u8> = input.bytes().filter(|b| *b != b'\n' && *b != b'\r').collect();

    if bytes.len() % 4 != 0 {
        return Err("invalid base64 length");
    }

    let mut result = Vec::with_capacity(bytes.len() * 3 / 4);

    for chunk in bytes.chunks(4) {
        let a = char_to_value(chunk[0])?;
        let b = char_to_value(chunk[1])?;
        let c = char_to_value(chunk[2])?;
        let d = char_to_value(chunk[3])?;

        result.push((a << 2) | (b >> 4));

        if chunk[2] != b'=' {
            result.push((b << 4) | (c >> 2));
        }
        if chunk[3] != b'=' {
            result.push((c << 6) | d);
        }
    }

    Ok(result)
}

/// Validate key data structure.
fn validate_key_data(data: &[u8], key_type: SshKeyType) -> Result<()> {
    if data.len() < 4 {
        return Err(CompatError::InvalidSshKey("key data too short".to_string()));
    }

    // First 4 bytes are length of key type string
    let type_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if data.len() < 4 + type_len {
        return Err(CompatError::InvalidSshKey(
            "key data truncated".to_string(),
        ));
    }

    // Verify key type matches
    let type_str = std::str::from_utf8(&data[4..4 + type_len])
        .map_err(|_| CompatError::InvalidSshKey("invalid key type encoding".to_string()))?;

    let expected_type = match key_type {
        SshKeyType::Ed25519 => "ssh-ed25519",
        SshKeyType::Rsa => "ssh-rsa",
        SshKeyType::Ecdsa => {
            // ECDSA can have different curve names
            if !type_str.starts_with("ecdsa-sha2-") {
                return Err(CompatError::InvalidSshKey(format!(
                    "expected ecdsa key type, got: {}",
                    type_str
                )));
            }
            return Ok(());
        }
    };

    if type_str != expected_type {
        return Err(CompatError::InvalidSshKey(format!(
            "key type mismatch: expected {}, got {}",
            expected_type, type_str
        )));
    }

    Ok(())
}

/// Calculate SHA256 fingerprint of key data.
fn calculate_fingerprint(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    format!("SHA256:{}", base64_encode(&hash))
}

/// Basic base64 encoding (without padding).
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        }
    }

    result
}

/// SSH key response for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyResponse {
    /// Key ID.
    pub id: SshKeyId,
    /// User-provided title.
    pub title: String,
    /// Key type.
    pub key_type: SshKeyType,
    /// Full public key.
    pub key: String,
    /// SHA256 fingerprint.
    pub fingerprint: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Last used timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
}

/// Request to add an SSH key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSshKeyRequest {
    /// Title/name for the key.
    pub title: String,
    /// Full public key string.
    pub key: String,
}

/// Format a Unix timestamp as ISO 8601.
fn format_timestamp(timestamp: u64) -> String {
    let secs_per_day = 86400;
    let secs_per_hour = 3600;
    let secs_per_min = 60;

    let mut days = timestamp / secs_per_day;
    let remaining = timestamp % secs_per_day;
    let hours = remaining / secs_per_hour;
    let remaining = remaining % secs_per_hour;
    let minutes = remaining / secs_per_min;
    let seconds = remaining % secs_per_min;

    let mut year = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let days_in_month = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0;
    for (i, &dim) in days_in_month.iter().enumerate() {
        if days < dim as u64 {
            month = i + 1;
            break;
        }
        days -= dim as u64;
    }
    let day = days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A valid Ed25519 public key for testing
    const TEST_ED25519_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl test@example.com";

    #[test]
    fn test_parse_ed25519_key() {
        let result = parse_and_validate_key(TEST_ED25519_KEY);
        assert!(result.is_ok());

        let (key_type, fingerprint) = result.unwrap();
        assert_eq!(key_type, SshKeyType::Ed25519);
        assert!(fingerprint.starts_with("SHA256:"));
    }

    #[test]
    fn test_invalid_key_format() {
        assert!(parse_and_validate_key("invalid").is_err());
        assert!(parse_and_validate_key("unknown-type AAAAB3NzaC1").is_err());
    }

    #[test]
    fn test_ssh_key_creation() {
        let key = SshKey::new(1, 1, "My Key".to_string(), TEST_ED25519_KEY.to_string()).unwrap();

        assert_eq!(key.id, 1);
        assert_eq!(key.user_id, 1);
        assert_eq!(key.title, "My Key");
        assert_eq!(key.key_type, SshKeyType::Ed25519);
        assert!(key.fingerprint.starts_with("SHA256:"));
    }

    #[test]
    fn test_ssh_key_response() {
        let key = SshKey::new(1, 1, "My Key".to_string(), TEST_ED25519_KEY.to_string()).unwrap();
        let response = key.to_response();

        assert_eq!(response.id, 1);
        assert_eq!(response.title, "My Key");
        assert_eq!(response.key_type, SshKeyType::Ed25519);
    }

    #[test]
    fn test_key_type_display() {
        assert_eq!(SshKeyType::Ed25519.to_string(), "ssh-ed25519");
        assert_eq!(SshKeyType::Rsa.to_string(), "ssh-rsa");
        assert_eq!(SshKeyType::Ecdsa.to_string(), "ecdsa-sha2-nistp256");
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World!";
        let encoded = base64_encode(data);
        // Note: Our encode doesn't add padding, so we add it for decode
        let padded = format!(
            "{}{}",
            encoded,
            match data.len() % 3 {
                1 => "==",
                2 => "=",
                _ => "",
            }
        );
        let decoded = base64_decode(&padded).unwrap();
        assert_eq!(decoded, data);
    }
}
