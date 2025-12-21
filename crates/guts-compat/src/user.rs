//! User account types and management.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for a user.
pub type UserId = u64;

/// A user account in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID.
    pub id: UserId,
    /// Unique username (lowercase, alphanumeric with hyphens).
    pub username: String,
    /// Display name (can contain any characters).
    pub display_name: Option<String>,
    /// Email address (optional).
    pub email: Option<String>,
    /// Short biography.
    pub bio: Option<String>,
    /// Location string.
    pub location: Option<String>,
    /// Personal website URL.
    pub website: Option<String>,
    /// Avatar URL.
    pub avatar_url: Option<String>,
    /// Ed25519 public key (hex-encoded identity).
    pub public_key: String,
    /// Whether the user's email is public.
    pub email_public: bool,
    /// Unix timestamp when created.
    pub created_at: u64,
    /// Unix timestamp when last updated.
    pub updated_at: u64,
}

impl User {
    /// Create a new user.
    pub fn new(id: UserId, username: String, public_key: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            username,
            display_name: None,
            email: None,
            bio: None,
            location: None,
            website: None,
            avatar_url: None,
            public_key,
            email_public: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate a username format.
    ///
    /// Usernames must:
    /// - Be 1-39 characters long
    /// - Start with an alphanumeric character
    /// - Contain only lowercase alphanumeric characters and hyphens
    /// - Not contain consecutive hyphens
    /// - Not end with a hyphen
    pub fn validate_username(username: &str) -> Result<(), String> {
        if username.is_empty() {
            return Err("username cannot be empty".to_string());
        }

        if username.len() > 39 {
            return Err("username must be 39 characters or less".to_string());
        }

        let chars: Vec<char> = username.chars().collect();

        // Must start with alphanumeric
        if !chars[0].is_ascii_alphanumeric() {
            return Err("username must start with a letter or number".to_string());
        }

        // Must end with alphanumeric
        if !chars.last().unwrap().is_ascii_alphanumeric() {
            return Err("username must end with a letter or number".to_string());
        }

        // Check each character
        for (i, c) in chars.iter().enumerate() {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-' {
                if c.is_ascii_uppercase() {
                    return Err("username must be lowercase".to_string());
                }
                return Err(format!("invalid character in username: {}", c));
            }

            // No consecutive hyphens
            if *c == '-' && i > 0 && chars[i - 1] == '-' {
                return Err("username cannot contain consecutive hyphens".to_string());
            }
        }

        // Reserved usernames
        let reserved = [
            "admin",
            "api",
            "git",
            "guts",
            "help",
            "login",
            "logout",
            "new",
            "organizations",
            "repos",
            "settings",
            "signup",
            "user",
            "users",
        ];
        if reserved.contains(&username) {
            return Err(format!("username '{}' is reserved", username));
        }

        Ok(())
    }

    /// Update the updated_at timestamp.
    pub fn touch(&mut self) {
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Convert to a public profile (for API responses).
    pub fn to_profile(&self, public_repos: u64, followers: u64, following: u64) -> UserProfile {
        UserProfile {
            login: self.username.clone(),
            id: self.id,
            avatar_url: self.avatar_url.clone(),
            name: self.display_name.clone(),
            email: if self.email_public {
                self.email.clone()
            } else {
                None
            },
            bio: self.bio.clone(),
            location: self.location.clone(),
            blog: self.website.clone(),
            public_repos,
            followers,
            following,
            created_at: format_timestamp(self.created_at),
            updated_at: format_timestamp(self.updated_at),
        }
    }
}

/// User profile for public API responses (GitHub-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Username (GitHub calls this "login").
    pub login: String,
    /// User ID.
    pub id: u64,
    /// Avatar URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Public email (if enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Biography.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// Location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Website/blog URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blog: Option<String>,
    /// Number of public repositories.
    pub public_repos: u64,
    /// Number of followers.
    pub followers: u64,
    /// Number of users following.
    pub following: u64,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last update timestamp.
    pub updated_at: String,
}

/// Request to create a new user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// Desired username.
    pub username: String,
    /// Ed25519 public key (hex-encoded).
    pub public_key: String,
    /// Optional email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Optional display name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Request to update a user profile.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUserRequest {
    /// New display name.
    #[serde(default)]
    pub name: Option<String>,
    /// New email address.
    #[serde(default)]
    pub email: Option<String>,
    /// New biography.
    #[serde(default)]
    pub bio: Option<String>,
    /// New location.
    #[serde(default)]
    pub location: Option<String>,
    /// New website/blog URL.
    #[serde(default)]
    pub blog: Option<String>,
    /// Whether email is public.
    #[serde(default)]
    pub email_public: Option<bool>,
}

/// Format a Unix timestamp as ISO 8601.
fn format_timestamp(timestamp: u64) -> String {
    use std::fmt::Write;
    // Simple ISO 8601 format: 2024-01-15T12:00:00Z
    // For a proper implementation, use chrono or time crate
    let secs_per_day = 86400;
    let secs_per_hour = 3600;
    let secs_per_min = 60;

    // Days since epoch
    let mut days = timestamp / secs_per_day;
    let remaining = timestamp % secs_per_day;
    let hours = remaining / secs_per_hour;
    let remaining = remaining % secs_per_hour;
    let minutes = remaining / secs_per_min;
    let seconds = remaining % secs_per_min;

    // Calculate year/month/day (simplified, doesn't handle leap seconds)
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

    let mut s = String::with_capacity(20);
    write!(
        s,
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
    .unwrap();
    s
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(User::validate_username("alice").is_ok());
        assert!(User::validate_username("bob123").is_ok());
        assert!(User::validate_username("my-project").is_ok());
        assert!(User::validate_username("a").is_ok());
        assert!(User::validate_username("a1").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(User::validate_username("").is_err());
        assert!(User::validate_username("-alice").is_err());
        assert!(User::validate_username("alice-").is_err());
        assert!(User::validate_username("alice--bob").is_err());
        assert!(User::validate_username("Alice").is_err());
        assert!(User::validate_username("alice_bob").is_err());
        assert!(User::validate_username("admin").is_err());

        // Too long
        let long_name = "a".repeat(40);
        assert!(User::validate_username(&long_name).is_err());
    }

    #[test]
    fn test_validate_username_reserved() {
        // Test all reserved usernames
        let reserved = [
            "admin",
            "api",
            "git",
            "guts",
            "help",
            "login",
            "logout",
            "new",
            "organizations",
            "repos",
            "settings",
            "signup",
            "user",
            "users",
        ];
        for name in reserved {
            let result = User::validate_username(name);
            assert!(result.is_err(), "Expected '{}' to be reserved", name);
            assert!(
                result.unwrap_err().contains("reserved"),
                "Error should mention 'reserved'"
            );
        }
    }

    #[test]
    fn test_validate_username_max_length() {
        // 39 chars should be valid
        let max_valid = "a".repeat(39);
        assert!(User::validate_username(&max_valid).is_ok());

        // 40 chars should be invalid
        let too_long = "a".repeat(40);
        assert!(User::validate_username(&too_long).is_err());
    }

    #[test]
    fn test_validate_username_special_chars() {
        // Invalid special characters
        assert!(User::validate_username("user@name").is_err());
        assert!(User::validate_username("user.name").is_err());
        assert!(User::validate_username("user#name").is_err());
        assert!(User::validate_username("user$name").is_err());
        assert!(User::validate_username("user%name").is_err());
        assert!(User::validate_username("user name").is_err());
        assert!(User::validate_username("user\tname").is_err());
        assert!(User::validate_username("user\nname").is_err());
    }

    #[test]
    fn test_validate_username_unicode() {
        // Unicode should be rejected
        assert!(User::validate_username("αβγ").is_err());
        assert!(User::validate_username("日本語").is_err());
        assert!(User::validate_username("émoji").is_err());
        assert!(User::validate_username("user🔥").is_err());
    }

    #[test]
    fn test_validate_username_edge_cases() {
        // Single character at boundaries
        assert!(User::validate_username("0").is_ok()); // digit
        assert!(User::validate_username("z").is_ok()); // letter

        // Hyphen placement
        assert!(User::validate_username("a-b").is_ok());
        assert!(User::validate_username("a-b-c").is_ok());
        assert!(User::validate_username("-a").is_err()); // starts with hyphen
        assert!(User::validate_username("a-").is_err()); // ends with hyphen
        assert!(User::validate_username("a--b").is_err()); // consecutive hyphens
        assert!(User::validate_username("---").is_err()); // all hyphens
    }

    #[test]
    fn test_create_user() {
        let user = User::new(1, "alice".to_string(), "abc123".to_string());
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "alice");
        assert_eq!(user.public_key, "abc123");
        assert!(user.display_name.is_none());
    }

    #[test]
    fn test_user_profile() {
        let mut user = User::new(1, "alice".to_string(), "abc123".to_string());
        user.display_name = Some("Alice Smith".to_string());
        user.email = Some("alice@example.com".to_string());
        user.email_public = true;

        let profile = user.to_profile(5, 10, 3);
        assert_eq!(profile.login, "alice");
        assert_eq!(profile.name, Some("Alice Smith".to_string()));
        assert_eq!(profile.email, Some("alice@example.com".to_string()));
        assert_eq!(profile.public_repos, 5);
        assert_eq!(profile.followers, 10);
        assert_eq!(profile.following, 3);
    }

    #[test]
    fn test_user_profile_private_email() {
        let mut user = User::new(1, "alice".to_string(), "abc123".to_string());
        user.email = Some("alice@example.com".to_string());
        user.email_public = false;

        let profile = user.to_profile(0, 0, 0);
        assert!(profile.email.is_none());
    }

    #[test]
    fn test_format_timestamp() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let ts = format_timestamp(1704067200);
        assert_eq!(ts, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_format_timestamp_epoch() {
        let ts = format_timestamp(0);
        assert_eq!(ts, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_format_timestamp_leap_year() {
        // Feb 29, 2024 12:00:00 UTC (2024 is a leap year)
        let ts = format_timestamp(1709208000);
        assert_eq!(ts, "2024-02-29T12:00:00Z");
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000)); // divisible by 400
        assert!(!is_leap_year(1900)); // divisible by 100 but not 400
        assert!(is_leap_year(2024)); // divisible by 4 but not 100
        assert!(!is_leap_year(2023)); // not divisible by 4
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for generating valid usernames
    fn valid_username_strategy() -> impl Strategy<Value = String> {
        // Generate usernames with pattern: alphanumeric, optionally with single hyphens
        prop::collection::vec(
            prop_oneof![
                4 => prop::char::ranges(vec!['a'..='z', '0'..='9'].into_iter().collect()),
                1 => Just('-'),
            ],
            1..=39,
        )
        .prop_filter_map("filter valid usernames", |chars| {
            let s: String = chars.into_iter().collect();
            // Must start and end with alphanumeric, no consecutive hyphens
            if s.is_empty() {
                return None;
            }
            let chars: Vec<char> = s.chars().collect();
            if !chars[0].is_ascii_alphanumeric() {
                return None;
            }
            if !chars.last().unwrap().is_ascii_alphanumeric() {
                return None;
            }
            // Check for consecutive hyphens
            for i in 1..chars.len() {
                if chars[i] == '-' && chars[i - 1] == '-' {
                    return None;
                }
            }
            // Skip reserved names
            let reserved = [
                "admin",
                "api",
                "git",
                "guts",
                "help",
                "login",
                "logout",
                "new",
                "organizations",
                "repos",
                "settings",
                "signup",
                "user",
                "users",
            ];
            if reserved.contains(&s.as_str()) {
                return None;
            }
            Some(s)
        })
    }

    proptest! {
        /// Property: Valid usernames should always be accepted
        #[test]
        fn prop_valid_usernames_accepted(username in valid_username_strategy()) {
            prop_assert!(
                User::validate_username(&username).is_ok(),
                "Username '{}' should be valid", username
            );
        }

        /// Property: Empty strings are always rejected
        #[test]
        fn prop_empty_string_rejected(_seed in 0u32..1000) {
            prop_assert!(User::validate_username("").is_err());
        }

        /// Property: Strings > 39 chars are always rejected
        #[test]
        fn prop_long_usernames_rejected(len in 40usize..200) {
            let long_name: String = (0..len).map(|_| 'a').collect();
            prop_assert!(
                User::validate_username(&long_name).is_err(),
                "Username of length {} should be rejected", len
            );
        }

        /// Property: Uppercase letters are always rejected
        #[test]
        fn prop_uppercase_rejected(prefix in "[a-z]{0,5}", upper in "[A-Z]", suffix in "[a-z]{0,5}") {
            let username = format!("{}{}{}", prefix, upper, suffix);
            if !username.is_empty() && username.len() <= 39 {
                prop_assert!(User::validate_username(&username).is_err());
            }
        }

        /// Property: Starting with hyphen is rejected
        #[test]
        fn prop_hyphen_start_rejected(rest in "[a-z0-9-]{0,10}") {
            let username = format!("-{}", rest);
            prop_assert!(User::validate_username(&username).is_err());
        }

        /// Property: Ending with hyphen is rejected
        #[test]
        fn prop_hyphen_end_rejected(prefix in "[a-z0-9]{1,10}") {
            let username = format!("{}-", prefix);
            prop_assert!(User::validate_username(&username).is_err());
        }

        /// Property: Consecutive hyphens are rejected
        #[test]
        fn prop_consecutive_hyphens_rejected(prefix in "[a-z0-9]{1,5}", suffix in "[a-z0-9]{1,5}") {
            let username = format!("{}--{}", prefix, suffix);
            prop_assert!(User::validate_username(&username).is_err());
        }

        /// Property: Underscores are rejected
        #[test]
        fn prop_underscore_rejected(prefix in "[a-z]{1,5}", suffix in "[a-z]{1,5}") {
            let username = format!("{}_{}", prefix, suffix);
            prop_assert!(User::validate_username(&username).is_err());
        }

        /// Property: Spaces are rejected
        #[test]
        fn prop_space_rejected(prefix in "[a-z]{1,5}", suffix in "[a-z]{1,5}") {
            let username = format!("{} {}", prefix, suffix);
            prop_assert!(User::validate_username(&username).is_err());
        }

        /// Property: Arbitrary Unicode is rejected
        #[test]
        fn prop_unicode_rejected(s in "\\PC{1,10}") {
            // If the string contains any non-ASCII characters, it should be rejected
            if !s.is_ascii() {
                prop_assert!(User::validate_username(&s).is_err());
            }
        }

        /// Property: Validation is consistent (idempotent)
        #[test]
        fn prop_validation_consistent(s in ".*") {
            let result1 = User::validate_username(&s);
            let result2 = User::validate_username(&s);
            prop_assert_eq!(result1.is_ok(), result2.is_ok());
        }
    }
}
