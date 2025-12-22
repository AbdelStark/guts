//! # API Client
//!
//! HTTP client for communicating with `guts-node`.

use reqwest::Client;

use super::error::{ApiError, ApiResult};
use super::types::{
    ContentsResponse, CreateRepoRequest, CreateTokenRequest, CreateUserRequest, RepoInfo,
    Repository, TokenResponse, UserProfile,
};

/// HTTP client for the Guts node API.
///
/// Provides methods to interact with a running `guts-node` instance.
/// The client is cheaply cloneable and can be shared across components.
///
/// # Examples
///
/// ```rust,ignore
/// use guts_desktop::api::GutsClient;
///
/// let client = GutsClient::new("http://127.0.0.1:8080");
///
/// // Check node health
/// if client.health().await? {
///     let repos = client.list_repositories().await?;
///     println!("Found {} repositories", repos.len());
/// }
/// ```
#[derive(Clone)]
pub struct GutsClient {
    base_url: String,
    http: Client,
}

impl GutsClient {
    /// Creates a new client connected to the specified node URL.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the guts-node API
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let client = GutsClient::new("http://localhost:8080");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to create HTTP client"),
        }
    }

    /// Returns the configured base URL.
    #[must_use]
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Checks if the node is reachable and healthy.
    ///
    /// # Returns
    ///
    /// `true` if the node responds with a successful status.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::Network`] if the request fails.
    pub async fn health(&self) -> ApiResult<bool> {
        let res = self
            .http
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        Ok(res.status().is_success())
    }

    /// Retrieves all repositories from the node.
    ///
    /// # Returns
    ///
    /// A vector of [`Repository`] structs.
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::InvalidResponse`] - Response could not be parsed
    pub async fn list_repositories(&self) -> ApiResult<Vec<Repository>> {
        let res = self
            .http
            .get(format!("{}/api/repos", self.base_url))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        res.json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }

    /// Retrieves a single repository by owner and name.
    ///
    /// # Arguments
    ///
    /// * `owner` - The repository owner
    /// * `name` - The repository name
    ///
    /// # Returns
    ///
    /// The [`Repository`] if found.
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::NodeError`] - Repository not found (404)
    pub async fn get_repository(&self, owner: &str, name: &str) -> ApiResult<Repository> {
        let res = self
            .http
            .get(format!("{}/api/repos/{}/{}", self.base_url, owner, name))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        res.json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }

    /// Creates a new repository.
    ///
    /// # Arguments
    ///
    /// * `name` - The repository name
    /// * `owner` - The repository owner
    ///
    /// # Returns
    ///
    /// The created repository info.
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::NodeError`] - Repository already exists (409) or validation error (422)
    pub async fn create_repository(&self, name: &str, owner: &str) -> ApiResult<RepoInfo> {
        let req = CreateRepoRequest {
            name: name.to_string(),
            owner: owner.to_string(),
        };

        let res = self
            .http
            .post(format!("{}/api/repos", self.base_url))
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        res.json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }

    /// Deletes a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - The repository owner
    /// * `name` - The repository name
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::NodeError`] - Repository not found (404)
    pub async fn delete_repository(&self, owner: &str, name: &str) -> ApiResult<()> {
        let res = self
            .http
            .delete(format!("{}/api/repos/{}/{}", self.base_url, owner, name))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    /// Gets repository contents at a path.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `name` - Repository name
    /// * `path` - Path within the repository (empty string for root)
    /// * `git_ref` - Optional branch/tag/SHA (defaults to HEAD)
    ///
    /// # Returns
    ///
    /// Either a single file entry or a directory listing.
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::NodeError`] - Path not found (404)
    pub async fn get_contents(
        &self,
        owner: &str,
        name: &str,
        path: &str,
        git_ref: Option<&str>,
    ) -> ApiResult<ContentsResponse> {
        let url = if path.is_empty() {
            format!("{}/api/repos/{}/{}/contents", self.base_url, owner, name)
        } else {
            format!(
                "{}/api/repos/{}/{}/contents/{}",
                self.base_url, owner, name, path
            )
        };

        let mut request = self.http.get(&url);

        if let Some(ref_name) = git_ref {
            request = request.query(&[("ref", ref_name)]);
        }

        let res = request.send().await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        res.json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }

    // ==================== Authentication Methods ====================

    /// Registers a new user account.
    ///
    /// # Arguments
    ///
    /// * `username` - The desired username
    /// * `public_key` - Hex-encoded Ed25519 public key
    ///
    /// # Returns
    ///
    /// The created user profile.
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::NodeError`] - Username taken (409) or validation error (422)
    pub async fn register_user(&self, username: &str, public_key: &str) -> ApiResult<UserProfile> {
        let req = CreateUserRequest {
            username: username.to_string(),
            public_key: public_key.to_string(),
        };

        let res = self
            .http
            .post(format!("{}/api/users", self.base_url))
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        res.json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }

    /// Creates a personal access token for a user.
    ///
    /// This method uses the `X-Guts-Identity` header for initial authentication
    /// before a token exists. After registration, use this to create a token
    /// that can be used for subsequent API calls.
    ///
    /// # Arguments
    ///
    /// * `name` - A name to identify the token
    /// * `scopes` - Permission scopes (e.g., `["repo:read", "repo:write"]`)
    /// * `username` - The username for the X-Guts-Identity header
    ///
    /// # Returns
    ///
    /// The token response including the plaintext token value.
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::NodeError`] - User not found or unauthorized
    pub async fn create_token_with_identity(
        &self,
        name: &str,
        scopes: &[&str],
        username: &str,
    ) -> ApiResult<TokenResponse> {
        let req = CreateTokenRequest {
            name: name.to_string(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
        };

        let res = self
            .http
            .post(format!("{}/api/user/tokens", self.base_url))
            .header("X-Guts-Identity", username)
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        res.json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }

    /// Gets the current authenticated user's profile.
    ///
    /// # Arguments
    ///
    /// * `token` - Personal access token for authentication
    ///
    /// # Returns
    ///
    /// The authenticated user's profile.
    ///
    /// # Errors
    ///
    /// * [`ApiError::Network`] - Network request failed
    /// * [`ApiError::NodeError`] - Unauthorized (401) or token invalid
    #[allow(dead_code)]
    pub async fn get_current_user(&self, token: &str) -> ApiResult<UserProfile> {
        let res = self
            .http
            .get(format!("{}/api/user", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(ApiError::NodeError {
                status: res.status().as_u16(),
                message: res.text().await.unwrap_or_default(),
            });
        }

        res.json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_health_returns_true_when_node_healthy() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = GutsClient::new(mock_server.uri());
        let result = client.health().await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_health_returns_false_when_node_unhealthy() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let client = GutsClient::new(mock_server.uri());
        let result = client.health().await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_list_repositories_handles_empty_array() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/repos"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let client = GutsClient::new(mock_server.uri());
        let repos = client.list_repositories().await.unwrap();

        assert!(repos.is_empty());
    }
}
