//! Release and tag management types.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for a release.
pub type ReleaseId = u64;

/// Unique identifier for a release asset.
pub type AssetId = u64;

/// A release (tagged version) in a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Unique release ID.
    pub id: ReleaseId,
    /// Repository key (owner/name).
    pub repo_key: String,
    /// Tag name (e.g., "v1.0.0").
    pub tag_name: String,
    /// Target branch or commit SHA.
    pub target_commitish: String,
    /// Release title (optional).
    pub name: Option<String>,
    /// Markdown body (changelog, notes).
    pub body: Option<String>,
    /// Whether this is a draft release.
    pub draft: bool,
    /// Whether this is a prerelease.
    pub prerelease: bool,
    /// Username of the author.
    pub author: String,
    /// Attached assets.
    pub assets: Vec<ReleaseAsset>,
    /// When the release was created.
    pub created_at: u64,
    /// When the release was published (None if draft).
    pub published_at: Option<u64>,
}

impl Release {
    /// Create a new release.
    pub fn new(
        id: ReleaseId,
        repo_key: String,
        tag_name: String,
        target_commitish: String,
        author: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            repo_key,
            tag_name,
            target_commitish,
            name: None,
            body: None,
            draft: false,
            prerelease: false,
            author,
            assets: Vec::new(),
            created_at: now,
            published_at: Some(now),
        }
    }

    /// Check if this is the latest non-prerelease, non-draft release.
    pub fn is_publishable(&self) -> bool {
        !self.draft && !self.prerelease
    }

    /// Publish a draft release.
    pub fn publish(&mut self) {
        self.draft = false;
        self.published_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// Add an asset to this release.
    pub fn add_asset(&mut self, asset: ReleaseAsset) {
        self.assets.push(asset);
    }

    /// Remove an asset by ID.
    pub fn remove_asset(&mut self, asset_id: AssetId) -> Option<ReleaseAsset> {
        if let Some(pos) = self.assets.iter().position(|a| a.id == asset_id) {
            Some(self.assets.remove(pos))
        } else {
            None
        }
    }

    /// Convert to API response.
    pub fn to_response(&self) -> ReleaseResponse {
        ReleaseResponse {
            id: self.id,
            tag_name: self.tag_name.clone(),
            target_commitish: self.target_commitish.clone(),
            name: self.name.clone(),
            body: self.body.clone(),
            draft: self.draft,
            prerelease: self.prerelease,
            author: AuthorInfo {
                login: self.author.clone(),
            },
            assets: self.assets.iter().map(|a| a.to_response()).collect(),
            created_at: format_timestamp(self.created_at),
            published_at: self.published_at.map(format_timestamp),
        }
    }
}

/// An asset attached to a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    /// Unique asset ID.
    pub id: AssetId,
    /// Release this asset belongs to.
    pub release_id: ReleaseId,
    /// Filename.
    pub name: String,
    /// Optional label for display.
    pub label: Option<String>,
    /// MIME content type.
    pub content_type: String,
    /// Size in bytes.
    pub size: u64,
    /// Download count.
    pub download_count: u64,
    /// SHA-256 hash of content.
    pub content_hash: String,
    /// When the asset was uploaded.
    pub created_at: u64,
    /// Username of uploader.
    pub uploader: String,
}

impl ReleaseAsset {
    /// Create a new asset.
    pub fn new(
        id: AssetId,
        release_id: ReleaseId,
        name: String,
        content_type: String,
        size: u64,
        content_hash: String,
        uploader: String,
    ) -> Self {
        Self {
            id,
            release_id,
            name,
            label: None,
            content_type,
            size,
            download_count: 0,
            content_hash,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            uploader,
        }
    }

    /// Increment the download count.
    pub fn increment_downloads(&mut self) {
        self.download_count += 1;
    }

    /// Convert to API response.
    pub fn to_response(&self) -> AssetResponse {
        AssetResponse {
            id: self.id,
            name: self.name.clone(),
            label: self.label.clone(),
            content_type: self.content_type.clone(),
            size: self.size,
            download_count: self.download_count,
            created_at: format_timestamp(self.created_at),
            uploader: AuthorInfo {
                login: self.uploader.clone(),
            },
        }
    }
}

/// Author information for responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// Username.
    pub login: String,
}

/// Release response for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResponse {
    /// Release ID.
    pub id: ReleaseId,
    /// Tag name.
    pub tag_name: String,
    /// Target branch/commit.
    pub target_commitish: String,
    /// Release title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Markdown body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Whether this is a draft.
    pub draft: bool,
    /// Whether this is a prerelease.
    pub prerelease: bool,
    /// Author information.
    pub author: AuthorInfo,
    /// Attached assets.
    pub assets: Vec<AssetResponse>,
    /// Creation timestamp.
    pub created_at: String,
    /// Publication timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
}

/// Asset response for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    /// Asset ID.
    pub id: AssetId,
    /// Filename.
    pub name: String,
    /// Optional label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// MIME content type.
    pub content_type: String,
    /// Size in bytes.
    pub size: u64,
    /// Download count.
    pub download_count: u64,
    /// Upload timestamp.
    pub created_at: String,
    /// Uploader information.
    pub uploader: AuthorInfo,
}

/// Request to create a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReleaseRequest {
    /// Tag name (required).
    pub tag_name: String,
    /// Target branch or commit (default: default branch).
    #[serde(default)]
    pub target_commitish: Option<String>,
    /// Release title.
    #[serde(default)]
    pub name: Option<String>,
    /// Markdown body.
    #[serde(default)]
    pub body: Option<String>,
    /// Create as draft.
    #[serde(default)]
    pub draft: bool,
    /// Mark as prerelease.
    #[serde(default)]
    pub prerelease: bool,
}

/// Request to update a release.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateReleaseRequest {
    /// New tag name.
    #[serde(default)]
    pub tag_name: Option<String>,
    /// New target.
    #[serde(default)]
    pub target_commitish: Option<String>,
    /// New title.
    #[serde(default)]
    pub name: Option<String>,
    /// New body.
    #[serde(default)]
    pub body: Option<String>,
    /// Update draft status.
    #[serde(default)]
    pub draft: Option<bool>,
    /// Update prerelease status.
    #[serde(default)]
    pub prerelease: Option<bool>,
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

    #[test]
    fn test_release_creation() {
        let release = Release::new(
            1,
            "alice/repo".to_string(),
            "v1.0.0".to_string(),
            "main".to_string(),
            "alice".to_string(),
        );

        assert_eq!(release.id, 1);
        assert_eq!(release.tag_name, "v1.0.0");
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert!(release.published_at.is_some());
    }

    #[test]
    fn test_draft_release() {
        let mut release = Release::new(
            1,
            "alice/repo".to_string(),
            "v1.0.0".to_string(),
            "main".to_string(),
            "alice".to_string(),
        );

        release.draft = true;
        release.published_at = None;

        assert!(!release.is_publishable());

        release.publish();
        assert!(release.is_publishable());
        assert!(release.published_at.is_some());
    }

    #[test]
    fn test_asset_management() {
        let mut release = Release::new(
            1,
            "alice/repo".to_string(),
            "v1.0.0".to_string(),
            "main".to_string(),
            "alice".to_string(),
        );

        let asset = ReleaseAsset::new(
            1,
            1,
            "app-v1.0.0.tar.gz".to_string(),
            "application/gzip".to_string(),
            1024,
            "abc123".to_string(),
            "alice".to_string(),
        );

        release.add_asset(asset);
        assert_eq!(release.assets.len(), 1);

        let removed = release.remove_asset(1);
        assert!(removed.is_some());
        assert_eq!(release.assets.len(), 0);
    }

    #[test]
    fn test_asset_downloads() {
        let mut asset = ReleaseAsset::new(
            1,
            1,
            "app.tar.gz".to_string(),
            "application/gzip".to_string(),
            1024,
            "abc123".to_string(),
            "alice".to_string(),
        );

        assert_eq!(asset.download_count, 0);
        asset.increment_downloads();
        asset.increment_downloads();
        assert_eq!(asset.download_count, 2);
    }
}
