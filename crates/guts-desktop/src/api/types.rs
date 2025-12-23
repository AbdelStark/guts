//! # API Types
//!
//! Types for API requests and responses.

use serde::{Deserialize, Serialize};

/// Request to create a new repository.
#[derive(Debug, Clone, Serialize)]
pub struct CreateRepoRequest {
    /// The repository name.
    pub name: String,
    /// The repository owner.
    pub owner: String,
}

/// Response after creating a repository (minimal info from API).
#[derive(Debug, Clone, Deserialize)]
pub struct RepoInfo {
    /// The repository name.
    pub name: String,
    /// The repository owner.
    pub owner: String,
}

/// Repository visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Public repository.
    #[default]
    Public,
    /// Private repository.
    Private,
}

/// A repository with all display fields.
///
/// The API only returns `name` and `owner`, so we provide defaults
/// for the other fields needed by the UI.
#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    /// The repository name.
    pub name: String,
    /// The repository owner.
    pub owner: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Default branch name (defaults to "main").
    #[serde(default = "default_branch")]
    pub default_branch: String,
    /// Repository visibility (defaults to Public).
    #[serde(default)]
    pub visibility: Visibility,
}

fn default_branch() -> String {
    "main".to_string()
}

/// Content type for repository entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// Regular file.
    File,
    /// Directory.
    Dir,
    /// Symbolic link.
    Symlink,
    /// Git submodule.
    Submodule,
}

/// A content entry (file or directory) in a repository.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ContentEntry {
    /// Entry type.
    #[serde(rename = "type")]
    pub content_type: ContentType,
    /// Encoding (e.g., "base64" for file content).
    pub encoding: Option<String>,
    /// Size in bytes (0 for directories).
    pub size: u64,
    /// Entry name (filename or directory name).
    pub name: String,
    /// Full path from repository root.
    pub path: String,
    /// Base64-encoded content (only for files when requested).
    pub content: Option<String>,
    /// Git object SHA.
    pub sha: String,
}

impl ContentEntry {
    /// Returns true if this is a directory.
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.content_type == ContentType::Dir
    }

    /// Returns true if this is a file.
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.content_type == ContentType::File
    }

    /// Decodes base64 content to a string (for text files).
    #[must_use]
    pub fn decode_content(&self) -> Option<String> {
        self.content.as_ref().and_then(|c| base64_decode(c).ok())
    }
}

/// Decode base64 string to UTF-8 text.
fn base64_decode(input: &str) -> Result<String, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = input.trim().replace(['\n', '\r'], "");
    let bytes: Vec<u8> = input
        .chars()
        .filter(|&c| c != '=')
        .map(|c| ALPHABET.iter().position(|&b| b == c as u8).unwrap_or(0) as u8)
        .collect();

    let mut result = Vec::new();
    for chunk in bytes.chunks(4) {
        if chunk.len() >= 2 {
            result.push((chunk[0] << 2) | (chunk[1] >> 4));
        }
        if chunk.len() >= 3 {
            result.push((chunk[1] << 4) | (chunk[2] >> 2));
        }
        if chunk.len() >= 4 {
            result.push((chunk[2] << 6) | chunk[3]);
        }
    }

    String::from_utf8(result).map_err(|e| e.to_string())
}

/// Contents response - can be a single file or array of entries.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ContentsResponse {
    /// Single file entry with content.
    File(ContentEntry),
    /// Directory listing.
    Directory(Vec<ContentEntry>),
}

// ==================== Authentication Types ====================

/// Request to create a new user account.
#[derive(Debug, Clone, Serialize)]
pub struct CreateUserRequest {
    /// Desired username.
    pub username: String,
    /// Ed25519 public key (hex-encoded).
    pub public_key: String,
}

/// User profile response from the API.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct UserProfile {
    /// User ID.
    pub id: u64,
    /// Username (login).
    pub login: String,
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Public email (if enabled).
    #[serde(default)]
    pub email: Option<String>,
    /// Avatar URL.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// Biography.
    #[serde(default)]
    pub bio: Option<String>,
}

/// Request to create a personal access token.
#[derive(Debug, Clone, Serialize)]
pub struct CreateTokenRequest {
    /// Token name for identification.
    pub name: String,
    /// Permission scopes (e.g., "repo:read", "repo:write").
    pub scopes: Vec<String>,
}

/// Response when creating a personal access token.
///
/// The `token` field is only present on creation and should be
/// stored securely - it cannot be retrieved again.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TokenResponse {
    /// Token ID.
    pub id: u64,
    /// Token name.
    pub name: String,
    /// Token prefix (for identification).
    pub token_prefix: String,
    /// The full token value (only returned on creation).
    ///
    /// Format: `guts_<prefix>_<secret>`
    #[serde(default)]
    pub token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode_simple() {
        // "hello" in base64
        let encoded = "aGVsbG8=";
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn test_base64_decode_multiline() {
        // "hello world" split across lines
        let encoded = "aGVsbG8g\nd29ybGQ=";
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, "hello world");
    }
}
