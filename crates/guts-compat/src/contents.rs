//! Repository contents API types.

use serde::{Deserialize, Serialize};

/// Content type for repository entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Dir => write!(f, "dir"),
            Self::Symlink => write!(f, "symlink"),
            Self::Submodule => write!(f, "submodule"),
        }
    }
}

/// A content entry (file or directory) in a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    /// Entry type.
    #[serde(rename = "type")]
    pub content_type: ContentType,
    /// Encoding (e.g., "base64" for file content).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    /// Size in bytes (0 for directories).
    pub size: u64,
    /// Entry name (filename or directory name).
    pub name: String,
    /// Full path from repository root.
    pub path: String,
    /// Base64-encoded content (only for files when requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Git object SHA.
    pub sha: String,
    /// URL to download raw content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// URL to view in web UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// API URL for this entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Symlink target (only for symlinks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Submodule URL (only for submodules).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submodule_git_url: Option<String>,
}

impl ContentEntry {
    /// Create a new file entry.
    pub fn file(name: String, path: String, sha: String, size: u64) -> Self {
        Self {
            content_type: ContentType::File,
            encoding: None,
            size,
            name,
            path,
            content: None,
            sha,
            download_url: None,
            html_url: None,
            url: None,
            target: None,
            submodule_git_url: None,
        }
    }

    /// Create a new directory entry.
    pub fn dir(name: String, path: String, sha: String) -> Self {
        Self {
            content_type: ContentType::Dir,
            encoding: None,
            size: 0,
            name,
            path,
            content: None,
            sha,
            download_url: None,
            html_url: None,
            url: None,
            target: None,
            submodule_git_url: None,
        }
    }

    /// Create a new symlink entry.
    pub fn symlink(name: String, path: String, sha: String, target: String) -> Self {
        Self {
            content_type: ContentType::Symlink,
            encoding: None,
            size: target.len() as u64,
            name,
            path,
            content: None,
            sha,
            download_url: None,
            html_url: None,
            url: None,
            target: Some(target),
            submodule_git_url: None,
        }
    }

    /// Create a new submodule entry.
    pub fn submodule(name: String, path: String, sha: String, git_url: String) -> Self {
        Self {
            content_type: ContentType::Submodule,
            encoding: None,
            size: 0,
            name,
            path,
            content: None,
            sha,
            download_url: None,
            html_url: None,
            url: None,
            target: None,
            submodule_git_url: Some(git_url),
        }
    }

    /// Set the content (base64-encoded).
    pub fn with_content(mut self, content: String) -> Self {
        self.encoding = Some("base64".to_string());
        self.content = Some(content);
        self
    }

    /// Set URLs.
    pub fn with_urls(mut self, download_url: String, html_url: String, api_url: String) -> Self {
        self.download_url = Some(download_url);
        self.html_url = Some(html_url);
        self.url = Some(api_url);
        self
    }
}

/// README file response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeResponse {
    /// Content type (always "file").
    #[serde(rename = "type")]
    pub content_type: ContentType,
    /// Encoding (usually "base64").
    pub encoding: String,
    /// Size in bytes.
    pub size: u64,
    /// Filename.
    pub name: String,
    /// Path.
    pub path: String,
    /// Base64-encoded content.
    pub content: String,
    /// Git SHA.
    pub sha: String,
    /// Download URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// HTML view URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
}

/// License file response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseResponse {
    /// License name (e.g., "MIT", "Apache-2.0").
    pub name: String,
    /// License path in repository.
    pub path: String,
    /// SPDX license ID (if recognized).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spdx_id: Option<String>,
    /// Git SHA.
    pub sha: String,
    /// Size in bytes.
    pub size: u64,
    /// Download URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// HTML view URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// Base64-encoded content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Encoding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

/// Recognize license from filename.
pub fn recognize_license_file(filename: &str) -> Option<&'static str> {
    let lower = filename.to_lowercase();
    if lower == "license" || lower == "license.txt" || lower == "license.md" {
        Some("LICENSE")
    } else if lower == "copying" || lower == "copying.txt" {
        Some("COPYING")
    } else if lower == "unlicense" {
        Some("UNLICENSE")
    } else {
        None
    }
}

/// Detect SPDX license ID from content.
pub fn detect_spdx_id(content: &str) -> Option<&'static str> {
    let content_lower = content.to_lowercase();

    // Check for common license signatures
    if content_lower.contains("mit license")
        || content_lower.contains("permission is hereby granted, free of charge")
    {
        Some("MIT")
    } else if content_lower.contains("apache license") && content_lower.contains("version 2.0") {
        Some("Apache-2.0")
    } else if content_lower.contains("gnu general public license") {
        if content_lower.contains("version 3") {
            Some("GPL-3.0")
        } else if content_lower.contains("version 2") {
            Some("GPL-2.0")
        } else {
            Some("GPL")
        }
    } else if content_lower.contains("bsd 3-clause") || content_lower.contains("new bsd license") {
        Some("BSD-3-Clause")
    } else if content_lower.contains("bsd 2-clause") || content_lower.contains("simplified bsd") {
        Some("BSD-2-Clause")
    } else if content_lower.contains("mozilla public license") && content_lower.contains("2.0") {
        Some("MPL-2.0")
    } else if content_lower.contains("isc license") {
        Some("ISC")
    } else if content_lower.contains("unlicense") || content_lower.contains("public domain") {
        Some("Unlicense")
    } else {
        None
    }
}

/// Recognize README file from filename.
pub fn is_readme_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower == "readme"
        || lower == "readme.md"
        || lower == "readme.txt"
        || lower == "readme.rst"
        || lower == "readme.markdown"
        || lower == "readme.rdoc"
        || lower == "readme.org"
        || lower == "readme.adoc"
}

/// Query parameters for contents API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentsQuery {
    /// Git ref (branch, tag, or SHA).
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

/// Base64 encode bytes.
pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_entry_file() {
        let entry = ContentEntry::file(
            "main.rs".to_string(),
            "src/main.rs".to_string(),
            "abc123".to_string(),
            1024,
        );

        assert_eq!(entry.content_type, ContentType::File);
        assert_eq!(entry.name, "main.rs");
        assert_eq!(entry.path, "src/main.rs");
        assert_eq!(entry.size, 1024);
    }

    #[test]
    fn test_content_entry_with_content() {
        let entry = ContentEntry::file(
            "test.txt".to_string(),
            "test.txt".to_string(),
            "abc123".to_string(),
            11,
        )
        .with_content(base64_encode(b"Hello World"));

        assert_eq!(entry.encoding, Some("base64".to_string()));
        assert!(entry.content.is_some());
    }

    #[test]
    fn test_is_readme_file() {
        assert!(is_readme_file("README"));
        assert!(is_readme_file("README.md"));
        assert!(is_readme_file("readme.txt"));
        assert!(!is_readme_file("main.rs"));
        assert!(!is_readme_file("READMEE"));
    }

    #[test]
    fn test_recognize_license() {
        assert_eq!(recognize_license_file("LICENSE"), Some("LICENSE"));
        assert_eq!(recognize_license_file("license.txt"), Some("LICENSE"));
        assert_eq!(recognize_license_file("COPYING"), Some("COPYING"));
        assert_eq!(recognize_license_file("main.rs"), None);
    }

    #[test]
    fn test_detect_spdx_id() {
        assert_eq!(
            detect_spdx_id("MIT License\n\nPermission is hereby granted, free of charge"),
            Some("MIT")
        );
        assert_eq!(
            detect_spdx_id("Apache License Version 2.0"),
            Some("Apache-2.0")
        );
        assert_eq!(detect_spdx_id("Random text"), None);
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
        assert_eq!(base64_encode(b"Hi"), "SGk=");
        assert_eq!(base64_encode(b"A"), "QQ==");
    }
}
