//! HTTP middleware for GitHub API compatibility.

use serde::{Deserialize, Serialize};

use crate::pagination::PaginationLinks;
use crate::rate_limit::{RateLimitHeaders, RateLimitResource, RateLimitState};
use crate::token::TokenScope;
use crate::user::UserId;

/// Authentication context extracted from request.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// User ID if authenticated.
    pub user_id: Option<UserId>,
    /// Username if authenticated.
    pub username: Option<String>,
    /// Token scopes if authenticated via token.
    pub scopes: Vec<TokenScope>,
    /// Whether the user is authenticated.
    pub authenticated: bool,
    /// Client IP for rate limiting.
    pub client_ip: String,
}

impl AuthContext {
    /// Create an unauthenticated context.
    pub fn anonymous(client_ip: String) -> Self {
        Self {
            user_id: None,
            username: None,
            scopes: Vec::new(),
            authenticated: false,
            client_ip,
        }
    }

    /// Create an authenticated context.
    pub fn authenticated(
        user_id: UserId,
        username: String,
        scopes: Vec<TokenScope>,
        client_ip: String,
    ) -> Self {
        Self {
            user_id: Some(user_id),
            username: Some(username),
            scopes,
            authenticated: true,
            client_ip,
        }
    }

    /// Check if a scope is granted.
    pub fn has_scope(&self, scope: TokenScope) -> bool {
        if !self.authenticated {
            return false;
        }

        // Admin scope grants all
        if self.scopes.contains(&TokenScope::Admin) {
            return true;
        }

        self.scopes.contains(&scope)
    }

    /// Get the rate limit key (user ID or IP).
    pub fn rate_limit_key(&self) -> String {
        if let Some(id) = self.user_id {
            format!("user:{}", id)
        } else {
            format!("ip:{}", self.client_ip)
        }
    }
}

/// GitHub-compatible error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message.
    pub message: String,
    /// Documentation URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    /// Validation errors (for 422 responses).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ValidationError>>,
}

impl ErrorResponse {
    /// Create a simple error response.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            documentation_url: None,
            errors: None,
        }
    }

    /// Create an error with documentation URL.
    pub fn with_docs(message: impl Into<String>, docs_url: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            documentation_url: Some(docs_url.into()),
            errors: None,
        }
    }

    /// Create a validation error response.
    pub fn validation(message: impl Into<String>, errors: Vec<ValidationError>) -> Self {
        Self {
            message: message.into(),
            documentation_url: None,
            errors: Some(errors),
        }
    }

    /// Standard "Not Found" error.
    pub fn not_found() -> Self {
        Self::new("Not Found")
    }

    /// Standard "Bad credentials" error.
    pub fn bad_credentials() -> Self {
        Self::new("Bad credentials")
    }

    /// Standard "Forbidden" error.
    pub fn forbidden() -> Self {
        Self::new("Forbidden")
    }

    /// Standard rate limit error.
    pub fn rate_limited(reset: u64) -> Self {
        Self::with_docs(
            format!(
                "API rate limit exceeded. Rate limit will reset at {}",
                reset
            ),
            "https://docs.guts.network/rest/rate-limiting",
        )
    }
}

/// Validation error detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Resource type.
    pub resource: String,
    /// Field name.
    pub field: String,
    /// Error code.
    pub code: ValidationErrorCode,
    /// Additional message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ValidationError {
    /// Create a new validation error.
    pub fn new(
        resource: impl Into<String>,
        field: impl Into<String>,
        code: ValidationErrorCode,
    ) -> Self {
        Self {
            resource: resource.into(),
            field: field.into(),
            code,
            message: None,
        }
    }

    /// Create with a message.
    pub fn with_message(
        resource: impl Into<String>,
        field: impl Into<String>,
        code: ValidationErrorCode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            field: field.into(),
            code,
            message: Some(message.into()),
        }
    }
}

/// Validation error codes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationErrorCode {
    /// Field is missing.
    Missing,
    /// Field value is missing (null).
    MissingField,
    /// Field value is invalid.
    Invalid,
    /// Resource already exists.
    AlreadyExists,
    /// Value is not unique.
    NotUnique,
    /// Value is too long.
    TooLong,
    /// Value is too short.
    TooShort,
    /// Custom error.
    Custom,
}

/// Response headers builder.
#[derive(Debug, Clone, Default)]
pub struct ResponseHeaders {
    /// Rate limit headers.
    pub rate_limit: Option<RateLimitHeaders>,
    /// Pagination Link header.
    pub link: Option<String>,
    /// ETag header.
    pub etag: Option<String>,
    /// Last-Modified header.
    pub last_modified: Option<String>,
    /// Cache-Control header.
    pub cache_control: Option<String>,
}

impl ResponseHeaders {
    /// Create a new response headers builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add rate limit headers.
    pub fn with_rate_limit(mut self, state: &RateLimitState) -> Self {
        self.rate_limit = Some(RateLimitHeaders::from(state));
        self
    }

    /// Add pagination Link header.
    pub fn with_pagination(mut self, links: &PaginationLinks) -> Self {
        self.link = links.to_header_value();
        self
    }

    /// Add ETag header.
    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(format!("\"{}\"", etag.into()));
        self
    }

    /// Add cache control.
    pub fn with_cache_control(mut self, value: impl Into<String>) -> Self {
        self.cache_control = Some(value.into());
        self
    }

    /// No cache.
    pub fn no_cache(mut self) -> Self {
        self.cache_control = Some("private, max-age=60, s-maxage=60".to_string());
        self
    }
}

/// Parse Authorization header.
///
/// Supports:
/// - `Bearer <token>` - Personal access token
/// - `Basic <base64>` - username:token as password
/// - `token <token>` - GitHub-style token header
pub fn parse_authorization_header(header: &str) -> Option<AuthorizationValue> {
    let header = header.trim();

    if let Some(token) = header.strip_prefix("Bearer ") {
        return Some(AuthorizationValue::Bearer(token.trim().to_string()));
    }

    if let Some(token) = header.strip_prefix("token ") {
        return Some(AuthorizationValue::Token(token.trim().to_string()));
    }

    if let Some(encoded) = header.strip_prefix("Basic ") {
        if let Some((username, password)) = decode_basic_auth(encoded.trim()) {
            return Some(AuthorizationValue::Basic { username, password });
        }
    }

    None
}

/// Authorization header value.
#[derive(Debug, Clone)]
pub enum AuthorizationValue {
    /// Bearer token.
    Bearer(String),
    /// Token (GitHub-style).
    Token(String),
    /// Basic auth (username:password).
    Basic { username: String, password: String },
}

impl AuthorizationValue {
    /// Get the token string regardless of format.
    pub fn token(&self) -> Option<&str> {
        match self {
            Self::Bearer(t) | Self::Token(t) => Some(t),
            Self::Basic { password, .. } => {
                // In Basic auth, the token is used as the password
                if password.starts_with("guts_") {
                    Some(password)
                } else {
                    None
                }
            }
        }
    }

    /// Get the username for Basic auth.
    pub fn username(&self) -> Option<&str> {
        match self {
            Self::Basic { username, .. } => Some(username),
            _ => None,
        }
    }
}

/// Decode Basic auth header value.
fn decode_basic_auth(encoded: &str) -> Option<(String, String)> {
    // Simple base64 decode
    let decoded = base64_decode(encoded)?;
    let text = String::from_utf8(decoded).ok()?;

    let (username, password) = text.split_once(':')?;
    Some((username.to_string(), password.to_string()))
}

/// Base64 decode.
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    fn char_to_value(c: u8) -> Option<u8> {
        if let Some(pos) = ALPHABET.iter().position(|&x| x == c) {
            Some(pos as u8)
        } else if c == b'=' {
            Some(0)
        } else {
            None
        }
    }

    let input = input.trim();
    if input.is_empty() || input.len() % 4 != 0 {
        return None;
    }

    let bytes: Vec<u8> = input.bytes().collect();
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

    Some(result)
}

/// Get resource type from request path.
pub fn resource_from_path(path: &str) -> RateLimitResource {
    if path.contains("/search") {
        RateLimitResource::Search
    } else if path.contains("/graphql") {
        RateLimitResource::Graphql
    } else if path.starts_with("/git/") {
        RateLimitResource::Git
    } else {
        RateLimitResource::Core
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context_anonymous() {
        let ctx = AuthContext::anonymous("127.0.0.1".to_string());

        assert!(!ctx.authenticated);
        assert!(ctx.user_id.is_none());
        assert!(!ctx.has_scope(TokenScope::RepoRead));
    }

    #[test]
    fn test_auth_context_authenticated() {
        let ctx = AuthContext::authenticated(
            1,
            "alice".to_string(),
            vec![TokenScope::RepoRead],
            "127.0.0.1".to_string(),
        );

        assert!(ctx.authenticated);
        assert_eq!(ctx.user_id, Some(1));
        assert!(ctx.has_scope(TokenScope::RepoRead));
        assert!(!ctx.has_scope(TokenScope::RepoWrite));
    }

    #[test]
    fn test_rate_limit_key() {
        let anon = AuthContext::anonymous("10.0.0.1".to_string());
        assert_eq!(anon.rate_limit_key(), "ip:10.0.0.1");

        let auth = AuthContext::authenticated(42, "bob".to_string(), vec![], "10.0.0.1".to_string());
        assert_eq!(auth.rate_limit_key(), "user:42");
    }

    #[test]
    fn test_parse_authorization_bearer() {
        let auth = parse_authorization_header("Bearer guts_abc12345_secret").unwrap();
        match auth {
            AuthorizationValue::Bearer(token) => {
                assert_eq!(token, "guts_abc12345_secret");
            }
            _ => panic!("Expected Bearer"),
        }
    }

    #[test]
    fn test_parse_authorization_token() {
        let auth = parse_authorization_header("token guts_abc12345_secret").unwrap();
        match auth {
            AuthorizationValue::Token(token) => {
                assert_eq!(token, "guts_abc12345_secret");
            }
            _ => panic!("Expected Token"),
        }
    }

    #[test]
    fn test_parse_authorization_basic() {
        // "user:pass" in base64 = "dXNlcjpwYXNz"
        let auth = parse_authorization_header("Basic dXNlcjpwYXNz").unwrap();
        match auth {
            AuthorizationValue::Basic { username, password } => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("Expected Basic"),
        }
    }

    #[test]
    fn test_error_response() {
        let err = ErrorResponse::not_found();
        assert_eq!(err.message, "Not Found");
        assert!(err.errors.is_none());
    }

    #[test]
    fn test_validation_error() {
        let err = ValidationError::new("User", "username", ValidationErrorCode::AlreadyExists);
        assert_eq!(err.resource, "User");
        assert_eq!(err.field, "username");
    }

    #[test]
    fn test_resource_from_path() {
        assert_eq!(
            resource_from_path("/api/search/repositories"),
            RateLimitResource::Search
        );
        assert_eq!(
            resource_from_path("/git/owner/repo/info/refs"),
            RateLimitResource::Git
        );
        assert_eq!(
            resource_from_path("/api/repos/owner/repo"),
            RateLimitResource::Core
        );
    }
}
