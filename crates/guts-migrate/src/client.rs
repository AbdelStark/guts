//! Guts API client for migration operations.

use crate::error::{MigrationError, Result};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Client for interacting with the Guts API during migration.
pub struct GutsClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateRepoRequest {
    name: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RepoResponse {
    pub name: String,
    pub owner: String,
    pub clone_url: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assignees: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct IssueResponse {
    pub number: u64,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub body: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestResponse {
    pub number: u64,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct CreateReleaseRequest {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseResponse {
    pub id: String,
    pub tag_name: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

impl GutsClient {
    /// Create a new Guts client.
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent("guts-migrate")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            token,
        })
    }

    /// Get authorization headers.
    fn auth_headers(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    /// Make a GET request.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let mut request = self.client.get(&url);

        if let Some(auth) = self.auth_headers() {
            request = request.header("Authorization", auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MigrationError::ApiError(format!(
                "Request failed with status {status}: {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| MigrationError::ApiError(e.to_string()))
    }

    /// Make a POST request.
    async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let mut request = self.client.post(&url).json(body);

        if let Some(auth) = self.auth_headers() {
            request = request.header("Authorization", auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MigrationError::ApiError(format!(
                "Request failed with status {status}: {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| MigrationError::ApiError(e.to_string()))
    }

    /// Make a PATCH request.
    async fn patch<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let mut request = self.client.patch(&url).json(body);

        if let Some(auth) = self.auth_headers() {
            request = request.header("Authorization", auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MigrationError::ApiError(format!(
                "Request failed with status {status}: {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| MigrationError::ApiError(e.to_string()))
    }

    /// Create a repository on Guts.
    pub async fn create_repo(
        &self,
        name: &str,
        description: Option<&str>,
        private: bool,
    ) -> Result<RepoResponse> {
        self.post(
            "/api/repos",
            &CreateRepoRequest {
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
                private: Some(private),
            },
        )
        .await
    }

    /// Get repository information.
    pub async fn get_repo(&self, owner: &str, name: &str) -> Result<RepoResponse> {
        self.get(&format!("/api/repos/{owner}/{name}")).await
    }

    /// Create an issue.
    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        request: &CreateIssueRequest,
    ) -> Result<IssueResponse> {
        self.post(&format!("/api/repos/{owner}/{repo}/issues"), request)
            .await
    }

    /// Close an issue.
    pub async fn close_issue(&self, owner: &str, repo: &str, number: u64) -> Result<IssueResponse> {
        #[derive(Serialize)]
        struct CloseRequest {
            state: String,
        }

        self.patch(
            &format!("/api/repos/{owner}/{repo}/issues/{number}"),
            &CloseRequest {
                state: "closed".to_string(),
            },
        )
        .await
    }

    /// Create a comment on an issue.
    pub async fn create_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<()> {
        let _: serde_json::Value = self
            .post(
                &format!("/api/repos/{owner}/{repo}/issues/{number}/comments"),
                &CreateCommentRequest {
                    body: body.to_string(),
                },
            )
            .await?;
        Ok(())
    }

    /// Create a pull request.
    pub async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        request: &CreatePullRequestRequest,
    ) -> Result<PullRequestResponse> {
        self.post(&format!("/api/repos/{owner}/{repo}/pulls"), request)
            .await
    }

    /// Create a comment on a pull request.
    pub async fn create_pr_comment(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<()> {
        let _: serde_json::Value = self
            .post(
                &format!("/api/repos/{owner}/{repo}/pulls/{number}/comments"),
                &CreateCommentRequest {
                    body: body.to_string(),
                },
            )
            .await?;
        Ok(())
    }

    /// Create a release.
    pub async fn create_release(
        &self,
        owner: &str,
        repo: &str,
        request: &CreateReleaseRequest,
    ) -> Result<ReleaseResponse> {
        self.post(&format!("/api/repos/{owner}/{repo}/releases"), request)
            .await
    }

    /// Upload a release asset.
    pub async fn upload_release_asset(
        &self,
        owner: &str,
        repo: &str,
        release_id: &str,
        name: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        let url = format!(
            "{}/api/repos/{owner}/{repo}/releases/{release_id}/assets?name={name}",
            self.base_url
        );

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", content_type)
            .body(data);

        if let Some(auth) = self.auth_headers() {
            request = request.header("Authorization", auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MigrationError::ApiError(format!(
                "Asset upload failed with status {status}: {body}"
            )));
        }

        Ok(())
    }

    /// Create a label.
    pub async fn create_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        color: &str,
        description: Option<&str>,
    ) -> Result<()> {
        let _: serde_json::Value = self
            .post(
                &format!("/api/repos/{owner}/{repo}/labels"),
                &CreateLabelRequest {
                    name: name.to_string(),
                    color: color.to_string(),
                    description: description.map(|s| s.to_string()),
                },
            )
            .await?;
        Ok(())
    }

    /// Check if the Guts node is healthy.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health/ready", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| MigrationError::NetworkError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GutsClient::new("http://localhost:8080", None);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_token() {
        let client =
            GutsClient::new("http://localhost:8080", Some("guts_test_token".to_string())).unwrap();
        assert!(client.auth_headers().is_some());
    }
}
