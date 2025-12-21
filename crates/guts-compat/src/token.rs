//! Personal Access Token types and authentication.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{CompatError, Result};
use crate::user::UserId;

/// Unique identifier for a token.
pub type TokenId = u64;

/// Token format: guts_<prefix>_<secret>
/// Prefix: 8 lowercase alphanumeric characters
/// Secret: 32 alphanumeric characters (mixed case)
const TOKEN_PREFIX_LEN: usize = 8;
const TOKEN_SECRET_LEN: usize = 32;

/// A personal access token for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccessToken {
    /// Unique token ID.
    pub id: TokenId,
    /// User who owns this token.
    pub user_id: UserId,
    /// User-provided name/description.
    pub name: String,
    /// Argon2id hash of the full token (prefix + secret).
    pub token_hash: String,
    /// First 8 characters for quick lookup.
    pub token_prefix: String,
    /// Scopes granted to this token.
    pub scopes: Vec<TokenScope>,
    /// Optional expiration timestamp.
    pub expires_at: Option<u64>,
    /// Last time the token was used.
    pub last_used_at: Option<u64>,
    /// When the token was created.
    pub created_at: u64,
}

impl PersonalAccessToken {
    /// Generate a new token with a random value.
    ///
    /// Returns the token struct and the plaintext token value (only shown once).
    pub fn generate(
        id: TokenId,
        user_id: UserId,
        name: String,
        scopes: Vec<TokenScope>,
        expires_at: Option<u64>,
    ) -> Result<(Self, String)> {
        let token_value = TokenValue::generate();
        let plaintext = token_value.to_string();

        // Hash the secret part
        let token_hash = hash_token_secret(&token_value.secret)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let token = Self {
            id,
            user_id,
            name,
            token_hash,
            token_prefix: token_value.prefix,
            scopes,
            expires_at,
            last_used_at: None,
            created_at: now,
        };

        Ok((token, plaintext))
    }

    /// Verify a token secret against the stored hash.
    pub fn verify(&self, secret: &str) -> Result<()> {
        verify_token_secret(secret, &self.token_hash)
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    /// Check if the token has a specific scope.
    pub fn has_scope(&self, required: TokenScope) -> bool {
        // Admin scope grants all permissions
        if self.scopes.contains(&TokenScope::Admin) {
            return true;
        }

        // Check for exact scope match
        if self.scopes.contains(&required) {
            return true;
        }

        // Check for parent scope (e.g., RepoWrite includes RepoRead)
        match required {
            TokenScope::RepoRead => {
                self.scopes.contains(&TokenScope::RepoWrite)
                    || self.scopes.contains(&TokenScope::RepoAdmin)
            }
            TokenScope::RepoWrite => self.scopes.contains(&TokenScope::RepoAdmin),
            TokenScope::UserRead => self.scopes.contains(&TokenScope::UserWrite),
            TokenScope::OrgRead => {
                self.scopes.contains(&TokenScope::OrgWrite)
                    || self.scopes.contains(&TokenScope::OrgAdmin)
            }
            TokenScope::OrgWrite => self.scopes.contains(&TokenScope::OrgAdmin),
            TokenScope::SshKeyRead => self.scopes.contains(&TokenScope::SshKeyWrite),
            TokenScope::WorkflowRead => self.scopes.contains(&TokenScope::WorkflowWrite),
            TokenScope::WebhookRead => self.scopes.contains(&TokenScope::WebhookWrite),
            _ => false,
        }
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

    /// Convert to a response (without the hash).
    pub fn to_response(&self, plaintext: Option<&str>) -> TokenResponse {
        TokenResponse {
            id: self.id,
            name: self.name.clone(),
            scopes: self.scopes.clone(),
            token_prefix: self.token_prefix.clone(),
            token: plaintext.map(|s| s.to_string()),
            expires_at: self.expires_at.map(format_timestamp),
            last_used_at: self.last_used_at.map(format_timestamp),
            created_at: format_timestamp(self.created_at),
        }
    }
}

/// Token scopes for fine-grained permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenScope {
    // Repository
    /// Read access to repositories.
    RepoRead,
    /// Write (push) access to repositories.
    RepoWrite,
    /// Admin access to repositories (settings, collaborators).
    RepoAdmin,
    /// Delete repositories.
    RepoDelete,

    // User
    /// Read user profile.
    UserRead,
    /// Update user profile.
    UserWrite,
    /// Access email addresses.
    UserEmail,

    // Organization
    /// Read organization info.
    OrgRead,
    /// Manage organization (members, teams).
    OrgWrite,
    /// Admin organization operations.
    OrgAdmin,

    // SSH Keys
    /// List SSH keys.
    SshKeyRead,
    /// Add/remove SSH keys.
    SshKeyWrite,

    // Workflow (CI/CD)
    /// Read workflows and runs.
    WorkflowRead,
    /// Trigger and manage workflows.
    WorkflowWrite,

    // Webhooks
    /// Read webhooks.
    WebhookRead,
    /// Manage webhooks.
    WebhookWrite,

    // Admin (superuser)
    /// Full admin access (all permissions).
    Admin,
}

impl TokenScope {
    /// Get all available scopes.
    pub fn all() -> Vec<Self> {
        vec![
            Self::RepoRead,
            Self::RepoWrite,
            Self::RepoAdmin,
            Self::RepoDelete,
            Self::UserRead,
            Self::UserWrite,
            Self::UserEmail,
            Self::OrgRead,
            Self::OrgWrite,
            Self::OrgAdmin,
            Self::SshKeyRead,
            Self::SshKeyWrite,
            Self::WorkflowRead,
            Self::WorkflowWrite,
            Self::WebhookRead,
            Self::WebhookWrite,
            Self::Admin,
        ]
    }

    /// Get the display name for this scope.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::RepoRead => "repo:read",
            Self::RepoWrite => "repo:write",
            Self::RepoAdmin => "repo:admin",
            Self::RepoDelete => "repo:delete",
            Self::UserRead => "user:read",
            Self::UserWrite => "user:write",
            Self::UserEmail => "user:email",
            Self::OrgRead => "org:read",
            Self::OrgWrite => "org:write",
            Self::OrgAdmin => "org:admin",
            Self::SshKeyRead => "ssh_key:read",
            Self::SshKeyWrite => "ssh_key:write",
            Self::WorkflowRead => "workflow:read",
            Self::WorkflowWrite => "workflow:write",
            Self::WebhookRead => "webhook:read",
            Self::WebhookWrite => "webhook:write",
            Self::Admin => "admin",
        }
    }
}

/// The plaintext token value (prefix + secret).
#[derive(Debug, Clone)]
pub struct TokenValue {
    /// First 8 characters for lookup.
    pub prefix: String,
    /// Secret part (32 characters).
    pub secret: String,
}

impl TokenValue {
    /// Generate a new random token value.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        // Generate prefix (lowercase alphanumeric)
        let prefix: String = (0..TOKEN_PREFIX_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'a' + idx - 10) as char
                }
            })
            .collect();

        // Generate secret (mixed case alphanumeric)
        let secret: String = (0..TOKEN_SECRET_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                if idx < 10 {
                    (b'0' + idx) as char
                } else if idx < 36 {
                    (b'a' + idx - 10) as char
                } else {
                    (b'A' + idx - 36) as char
                }
            })
            .collect();

        Self { prefix, secret }
    }

    /// Parse a token string into prefix and secret.
    pub fn parse(token: &str) -> Result<Self> {
        // Format: guts_<prefix>_<secret>
        let parts: Vec<&str> = token.split('_').collect();
        if parts.len() != 3 || parts[0] != "guts" {
            return Err(CompatError::InvalidTokenFormat);
        }

        let prefix = parts[1];
        let secret = parts[2];

        if prefix.len() != TOKEN_PREFIX_LEN || secret.len() != TOKEN_SECRET_LEN {
            return Err(CompatError::InvalidTokenFormat);
        }

        Ok(Self {
            prefix: prefix.to_string(),
            secret: secret.to_string(),
        })
    }
}

impl std::fmt::Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "guts_{}_{}", self.prefix, self.secret)
    }
}

/// Hash a token secret using Argon2id.
fn hash_token_secret(secret: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(secret.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| CompatError::Crypto(e.to_string()))
}

/// Verify a token secret against a hash.
fn verify_token_secret(secret: &str, hash: &str) -> Result<()> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| CompatError::Crypto(e.to_string()))?;

    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed_hash)
        .map_err(|_| CompatError::InvalidToken)
}

/// Token response for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Token ID.
    pub id: TokenId,
    /// User-provided name.
    pub name: String,
    /// Granted scopes.
    pub scopes: Vec<TokenScope>,
    /// Token prefix for identification.
    pub token_prefix: String,
    /// Full token (only included on creation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Last used timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    /// Creation timestamp.
    pub created_at: String,
}

/// Request to create a new token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    /// Name/description for the token.
    pub name: String,
    /// Scopes to grant.
    pub scopes: Vec<TokenScope>,
    /// Optional expiration in days.
    #[serde(default)]
    pub expires_in_days: Option<u32>,
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
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let (token, plaintext) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::RepoRead], None)
                .unwrap();

        assert_eq!(token.id, 1);
        assert_eq!(token.user_id, 1);
        assert_eq!(token.name, "test");
        assert!(plaintext.starts_with("guts_"));

        // Verify the token
        let parsed = TokenValue::parse(&plaintext).unwrap();
        assert!(token.verify(&parsed.secret).is_ok());
    }

    #[test]
    fn test_token_value_format() {
        let token = TokenValue::generate();
        let s = token.to_string();

        assert!(s.starts_with("guts_"));
        let parts: Vec<&str> = s.split('_').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "guts");
        assert_eq!(parts[1].len(), 8);
        assert_eq!(parts[2].len(), 32);
    }

    #[test]
    fn test_token_parse() {
        let token = TokenValue::generate();
        let s = token.to_string();
        let parsed = TokenValue::parse(&s).unwrap();

        assert_eq!(parsed.prefix, token.prefix);
        assert_eq!(parsed.secret, token.secret);
    }

    #[test]
    fn test_token_parse_invalid() {
        assert!(TokenValue::parse("invalid").is_err());
        assert!(TokenValue::parse("guts_short_secret").is_err());
        assert!(TokenValue::parse("github_abc12345_12345678901234567890123456789012").is_err());
    }

    #[test]
    fn test_token_parse_wrong_prefix() {
        // Wrong starting word
        assert!(TokenValue::parse("github_abc12345_12345678901234567890123456789012").is_err());
        assert!(TokenValue::parse("pat_12345678_12345678901234567890123456789012").is_err());
    }

    #[test]
    fn test_token_parse_wrong_part_count() {
        // Too few parts
        assert!(TokenValue::parse("guts_12345678901234567890123456789012").is_err());
        // Too many parts
        assert!(TokenValue::parse("guts_abc12345_12345678901234567890123456789012_extra").is_err());
    }

    #[test]
    fn test_token_parse_wrong_prefix_length() {
        // Prefix too short
        assert!(TokenValue::parse("guts_abc_12345678901234567890123456789012").is_err());
        // Prefix too long
        assert!(TokenValue::parse("guts_abc123456789_12345678901234567890123456789012").is_err());
    }

    #[test]
    fn test_token_parse_wrong_secret_length() {
        // Secret too short
        assert!(TokenValue::parse("guts_abc12345_short").is_err());
        // Secret too long
        assert!(
            TokenValue::parse("guts_abc12345_123456789012345678901234567890123456789012345")
                .is_err()
        );
    }

    #[test]
    fn test_token_expiration() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (mut token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![], Some(now - 1)).unwrap();
        assert!(token.is_expired());

        token.expires_at = Some(now + 3600);
        assert!(!token.is_expired());

        token.expires_at = None;
        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_expiration_boundary() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Exactly at expiration time should be expired
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![], Some(now)).unwrap();
        assert!(token.is_expired());
    }

    #[test]
    fn test_token_scope_hierarchy() {
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::RepoAdmin], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::RepoAdmin));
        assert!(token.has_scope(TokenScope::RepoWrite));
        assert!(token.has_scope(TokenScope::RepoRead));
        assert!(!token.has_scope(TokenScope::OrgRead));
    }

    #[test]
    fn test_token_scope_user_hierarchy() {
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::UserWrite], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::UserWrite));
        assert!(token.has_scope(TokenScope::UserRead));
        assert!(!token.has_scope(TokenScope::UserEmail));
    }

    #[test]
    fn test_token_scope_org_hierarchy() {
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::OrgAdmin], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::OrgAdmin));
        assert!(token.has_scope(TokenScope::OrgWrite));
        assert!(token.has_scope(TokenScope::OrgRead));
    }

    #[test]
    fn test_token_scope_ssh_hierarchy() {
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::SshKeyWrite], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::SshKeyWrite));
        assert!(token.has_scope(TokenScope::SshKeyRead));
    }

    #[test]
    fn test_token_scope_workflow_hierarchy() {
        let (token, _) = PersonalAccessToken::generate(
            1,
            1,
            "test".into(),
            vec![TokenScope::WorkflowWrite],
            None,
        )
        .unwrap();

        assert!(token.has_scope(TokenScope::WorkflowWrite));
        assert!(token.has_scope(TokenScope::WorkflowRead));
    }

    #[test]
    fn test_token_scope_webhook_hierarchy() {
        let (token, _) = PersonalAccessToken::generate(
            1,
            1,
            "test".into(),
            vec![TokenScope::WebhookWrite],
            None,
        )
        .unwrap();

        assert!(token.has_scope(TokenScope::WebhookWrite));
        assert!(token.has_scope(TokenScope::WebhookRead));
    }

    #[test]
    fn test_admin_scope_grants_all() {
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::Admin], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::RepoRead));
        assert!(token.has_scope(TokenScope::UserWrite));
        assert!(token.has_scope(TokenScope::OrgAdmin));
        assert!(token.has_scope(TokenScope::Admin));
    }

    #[test]
    fn test_scope_display_names() {
        assert_eq!(TokenScope::RepoRead.display_name(), "repo:read");
        assert_eq!(TokenScope::Admin.display_name(), "admin");
    }

    #[test]
    fn test_all_scopes() {
        let scopes = TokenScope::all();
        assert_eq!(scopes.len(), 17);
        assert!(scopes.contains(&TokenScope::RepoRead));
        assert!(scopes.contains(&TokenScope::Admin));
    }

    #[test]
    fn test_all_scope_display_names() {
        // Every scope should have a unique display name
        let scopes = TokenScope::all();
        let display_names: Vec<_> = scopes.iter().map(|s| s.display_name()).collect();
        let unique: std::collections::HashSet<_> = display_names.iter().collect();
        assert_eq!(unique.len(), scopes.len());
    }

    #[test]
    fn test_token_verify_wrong_secret() {
        let (token, _plaintext) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::RepoRead], None)
                .unwrap();

        // Verify with wrong secret should fail
        assert!(token.verify("wrongsecret").is_err());
        assert!(token.verify("12345678901234567890123456789012").is_err());
    }

    #[test]
    fn test_token_touch() {
        let (mut token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![], None).unwrap();

        assert!(token.last_used_at.is_none());
        token.touch();
        assert!(token.last_used_at.is_some());
    }

    #[test]
    fn test_token_to_response() {
        let (token, plaintext) = PersonalAccessToken::generate(
            1,
            1,
            "My Token".into(),
            vec![TokenScope::RepoRead],
            None,
        )
        .unwrap();

        // With plaintext
        let response = token.to_response(Some(&plaintext));
        assert_eq!(response.id, 1);
        assert_eq!(response.name, "My Token");
        assert!(response.token.is_some());
        assert_eq!(response.token.as_ref().unwrap(), &plaintext);

        // Without plaintext
        let response = token.to_response(None);
        assert!(response.token.is_none());
    }

    #[test]
    fn test_token_response_timestamps() {
        let (token, _) = PersonalAccessToken::generate(1, 1, "test".into(), vec![], None).unwrap();

        let response = token.to_response(None);
        assert!(!response.created_at.is_empty());
        assert!(response.created_at.contains('T'));
        assert!(response.created_at.ends_with('Z'));
    }

    #[test]
    fn test_token_uniqueness() {
        // Generate multiple tokens and ensure they're unique
        let mut tokens = Vec::new();
        for _ in 0..10 {
            let token = TokenValue::generate();
            tokens.push(token.to_string());
        }

        let unique: std::collections::HashSet<_> = tokens.iter().collect();
        assert_eq!(unique.len(), tokens.len());
    }

    #[test]
    fn test_token_prefix_format() {
        // Generate multiple tokens and verify prefix format
        for _ in 0..10 {
            let token = TokenValue::generate();
            // Prefix should be lowercase alphanumeric
            assert!(token
                .prefix
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
            assert_eq!(token.prefix.len(), 8);
        }
    }

    #[test]
    fn test_token_secret_format() {
        // Generate multiple tokens and verify secret format
        for _ in 0..10 {
            let token = TokenValue::generate();
            // Secret should be alphanumeric (mixed case)
            assert!(token.secret.chars().all(|c| c.is_ascii_alphanumeric()));
            assert_eq!(token.secret.len(), 32);
        }
    }

    #[test]
    fn test_token_scope_no_read_without_write() {
        // RepoWrite grants RepoRead
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::RepoWrite], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::RepoRead));
        assert!(token.has_scope(TokenScope::RepoWrite));
        assert!(!token.has_scope(TokenScope::RepoAdmin));
    }

    #[test]
    fn test_token_scope_exact_match() {
        // Token with only RepoRead should only have RepoRead
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::RepoRead], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::RepoRead));
        assert!(!token.has_scope(TokenScope::RepoWrite));
        assert!(!token.has_scope(TokenScope::RepoAdmin));
    }

    #[test]
    fn test_token_multiple_scopes() {
        let (token, _) = PersonalAccessToken::generate(
            1,
            1,
            "test".into(),
            vec![TokenScope::RepoRead, TokenScope::UserRead],
            None,
        )
        .unwrap();

        assert!(token.has_scope(TokenScope::RepoRead));
        assert!(token.has_scope(TokenScope::UserRead));
        assert!(!token.has_scope(TokenScope::RepoWrite));
        assert!(!token.has_scope(TokenScope::UserWrite));
    }

    #[test]
    fn test_format_timestamp_epoch() {
        let ts = format_timestamp(0);
        assert_eq!(ts, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_format_timestamp_2024() {
        let ts = format_timestamp(1704067200);
        assert_eq!(ts, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_token_scope_delete() {
        // RepoDelete is standalone
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::RepoDelete], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::RepoDelete));
        assert!(!token.has_scope(TokenScope::RepoRead));
        assert!(!token.has_scope(TokenScope::RepoWrite));
    }

    #[test]
    fn test_token_scope_email() {
        // UserEmail is standalone
        let (token, _) =
            PersonalAccessToken::generate(1, 1, "test".into(), vec![TokenScope::UserEmail], None)
                .unwrap();

        assert!(token.has_scope(TokenScope::UserEmail));
        assert!(!token.has_scope(TokenScope::UserRead));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Token generation always produces valid parseable tokens
        #[test]
        fn prop_token_generation_parseable(
            id in 0u64..1000,
            user_id in 0u64..1000,
            name in "[a-zA-Z0-9 ]{1,50}"
        ) {
            let (_, plaintext) = PersonalAccessToken::generate(
                id,
                user_id,
                name,
                vec![TokenScope::RepoRead],
                None,
            ).unwrap();

            let parsed = TokenValue::parse(&plaintext);
            prop_assert!(parsed.is_ok());
        }

        /// Property: Token verification succeeds with correct secret
        #[test]
        fn prop_token_verification_correct(
            id in 0u64..100,
            user_id in 0u64..100
        ) {
            let (token, plaintext) = PersonalAccessToken::generate(
                id,
                user_id,
                "test".to_string(),
                vec![TokenScope::RepoRead],
                None,
            ).unwrap();

            let parsed = TokenValue::parse(&plaintext).unwrap();
            let result = token.verify(&parsed.secret);
            prop_assert!(result.is_ok());
        }

        /// Property: Token verification fails with wrong secret
        #[test]
        fn prop_token_verification_wrong_secret(
            wrong_secret in "[a-zA-Z0-9]{32}"
        ) {
            let (token, plaintext) = PersonalAccessToken::generate(
                1,
                1,
                "test".to_string(),
                vec![TokenScope::RepoRead],
                None,
            ).unwrap();

            let parsed = TokenValue::parse(&plaintext).unwrap();

            // Only test if the wrong secret is actually different
            if wrong_secret != parsed.secret {
                let result = token.verify(&wrong_secret);
                prop_assert!(result.is_err());
            }
        }

        /// Property: Admin scope grants all other scopes
        #[test]
        fn prop_admin_grants_all(_seed in 0u32..100) {
            let (token, _) = PersonalAccessToken::generate(
                1,
                1,
                "test".to_string(),
                vec![TokenScope::Admin],
                None,
            ).unwrap();

            for scope in TokenScope::all() {
                prop_assert!(token.has_scope(scope), "Admin should grant {:?}", scope);
            }
        }

        /// Property: Token prefix is always 8 lowercase alphanumeric chars
        #[test]
        fn prop_token_prefix_format(_seed in 0u32..100) {
            let token = TokenValue::generate();
            prop_assert_eq!(token.prefix.len(), 8);
            prop_assert!(token.prefix.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
        }

        /// Property: Token secret is always 32 alphanumeric chars
        #[test]
        fn prop_token_secret_format(_seed in 0u32..100) {
            let token = TokenValue::generate();
            prop_assert_eq!(token.secret.len(), 32);
            prop_assert!(token.secret.chars().all(|c| c.is_ascii_alphanumeric()));
        }

        /// Property: Tokens are always unique
        #[test]
        fn prop_token_uniqueness(_seed in 0u32..100) {
            let token1 = TokenValue::generate();
            let token2 = TokenValue::generate();
            // Extremely unlikely to be the same
            prop_assert!(token1.to_string() != token2.to_string());
        }

        /// Property: Invalid token formats are always rejected
        #[test]
        fn prop_invalid_token_rejected(s in ".*") {
            // Unless it happens to be a valid format (extremely unlikely)
            if !s.starts_with("guts_") || s.split('_').count() != 3 {
                let result = TokenValue::parse(&s);
                prop_assert!(result.is_err());
            }
        }
    }
}
