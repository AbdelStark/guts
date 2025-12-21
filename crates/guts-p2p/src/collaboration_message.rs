//! P2P protocol messages for collaboration replication (PRs, Issues, Comments, Reviews).

use bytes::{Buf, BufMut, Bytes, BytesMut};
use guts_collaboration::{
    Comment, CommentTarget, Issue, IssueState, Label, PullRequest, PullRequestState, Review,
    ReviewState,
};
use guts_storage::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{P2PError, Result};

/// Collaboration message type discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CollaborationMessageType {
    /// Pull request created.
    PullRequestCreated = 10,
    /// Pull request updated.
    PullRequestUpdated = 11,
    /// Issue created.
    IssueCreated = 12,
    /// Issue updated.
    IssueUpdated = 13,
    /// Comment created.
    CommentCreated = 14,
    /// Review created.
    ReviewCreated = 15,
    /// Request collaboration data sync.
    SyncCollaborationRequest = 16,
    /// Response with collaboration data.
    SyncCollaborationResponse = 17,
}

impl CollaborationMessageType {
    /// Parse a message type from a byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            10 => Ok(CollaborationMessageType::PullRequestCreated),
            11 => Ok(CollaborationMessageType::PullRequestUpdated),
            12 => Ok(CollaborationMessageType::IssueCreated),
            13 => Ok(CollaborationMessageType::IssueUpdated),
            14 => Ok(CollaborationMessageType::CommentCreated),
            15 => Ok(CollaborationMessageType::ReviewCreated),
            16 => Ok(CollaborationMessageType::SyncCollaborationRequest),
            17 => Ok(CollaborationMessageType::SyncCollaborationResponse),
            _ => Err(P2PError::InvalidMessage(format!(
                "unknown collaboration message type: {}",
                b
            ))),
        }
    }
}

/// Serializable version of a pull request for P2P transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializablePullRequest {
    pub id: u64,
    pub repo_key: String,
    pub number: u32,
    pub title: String,
    pub description: String,
    pub author: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub source_commit: String,
    pub target_commit: String,
    pub labels: Vec<SerializableLabel>,
    pub created_at: u64,
    pub updated_at: u64,
    pub merged_at: Option<u64>,
    pub merged_by: Option<String>,
}

impl From<PullRequest> for SerializablePullRequest {
    fn from(pr: PullRequest) -> Self {
        Self {
            id: pr.id,
            repo_key: pr.repo_key,
            number: pr.number,
            title: pr.title,
            description: pr.description,
            author: pr.author,
            state: pr.state.to_string(),
            source_branch: pr.source_branch,
            target_branch: pr.target_branch,
            source_commit: pr.source_commit.to_hex(),
            target_commit: pr.target_commit.to_hex(),
            labels: pr.labels.into_iter().map(Into::into).collect(),
            created_at: pr.created_at,
            updated_at: pr.updated_at,
            merged_at: pr.merged_at,
            merged_by: pr.merged_by,
        }
    }
}

impl SerializablePullRequest {
    /// Convert back to a PullRequest.
    pub fn into_pull_request(self) -> Result<PullRequest> {
        let source_commit = ObjectId::from_hex(&self.source_commit)
            .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
        let target_commit = ObjectId::from_hex(&self.target_commit)
            .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;

        let state = match self.state.as_str() {
            "open" => PullRequestState::Open,
            "closed" => PullRequestState::Closed,
            "merged" => PullRequestState::Merged,
            s => {
                return Err(P2PError::InvalidMessage(format!(
                    "invalid PR state: {}",
                    s
                )))
            }
        };

        let mut pr = PullRequest::new(
            self.id,
            self.repo_key,
            self.number,
            self.title,
            self.description,
            self.author,
            self.source_branch,
            self.target_branch,
            source_commit,
            target_commit,
        );

        // Set the stored values
        pr.id = self.id;
        pr.number = self.number;
        pr.state = state;
        pr.created_at = self.created_at;
        pr.updated_at = self.updated_at;
        pr.merged_at = self.merged_at;
        pr.merged_by = self.merged_by;

        for label in self.labels {
            pr.labels.push(label.into_label());
        }

        Ok(pr)
    }
}

/// Serializable version of an issue for P2P transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableIssue {
    pub id: u64,
    pub repo_key: String,
    pub number: u32,
    pub title: String,
    pub description: String,
    pub author: String,
    pub state: String,
    pub labels: Vec<SerializableLabel>,
    pub created_at: u64,
    pub updated_at: u64,
    pub closed_at: Option<u64>,
    pub closed_by: Option<String>,
}

impl From<Issue> for SerializableIssue {
    fn from(issue: Issue) -> Self {
        Self {
            id: issue.id,
            repo_key: issue.repo_key,
            number: issue.number,
            title: issue.title,
            description: issue.description,
            author: issue.author,
            state: issue.state.to_string(),
            labels: issue.labels.into_iter().map(Into::into).collect(),
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            closed_at: issue.closed_at,
            closed_by: issue.closed_by,
        }
    }
}

impl SerializableIssue {
    /// Convert back to an Issue.
    pub fn into_issue(self) -> Result<Issue> {
        let state = match self.state.as_str() {
            "open" => IssueState::Open,
            "closed" => IssueState::Closed,
            s => {
                return Err(P2PError::InvalidMessage(format!(
                    "invalid issue state: {}",
                    s
                )))
            }
        };

        let mut issue = Issue::new(
            self.id,
            self.repo_key,
            self.number,
            self.title,
            self.description,
            self.author,
        );

        // Set the stored values
        issue.id = self.id;
        issue.number = self.number;
        issue.state = state;
        issue.created_at = self.created_at;
        issue.updated_at = self.updated_at;
        issue.closed_at = self.closed_at;
        issue.closed_by = self.closed_by;

        for label in self.labels {
            issue.labels.push(label.into_label());
        }

        Ok(issue)
    }
}

/// Serializable version of a label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableLabel {
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

impl From<Label> for SerializableLabel {
    fn from(label: Label) -> Self {
        Self {
            name: label.name,
            color: label.color,
            description: label.description,
        }
    }
}

impl SerializableLabel {
    /// Convert back to a Label.
    pub fn into_label(self) -> Label {
        let mut label = Label::new(self.name, self.color);
        if let Some(desc) = self.description {
            label = label.with_description(desc);
        }
        label
    }
}

/// Serializable version of a comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableComment {
    pub id: u64,
    pub target_type: String,
    pub repo_key: String,
    pub number: u32,
    pub author: String,
    pub body: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<Comment> for SerializableComment {
    fn from(comment: Comment) -> Self {
        let (target_type, repo_key, number) = match &comment.target {
            CommentTarget::PullRequest { repo_key, number } => {
                ("pull_request".to_string(), repo_key.clone(), *number)
            }
            CommentTarget::Issue { repo_key, number } => {
                ("issue".to_string(), repo_key.clone(), *number)
            }
        };

        Self {
            id: comment.id,
            target_type,
            repo_key,
            number,
            author: comment.author,
            body: comment.body,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        }
    }
}

impl SerializableComment {
    /// Convert back to a Comment.
    pub fn into_comment(self) -> Result<Comment> {
        let target = match self.target_type.as_str() {
            "pull_request" => CommentTarget::pull_request(&self.repo_key, self.number),
            "issue" => CommentTarget::issue(&self.repo_key, self.number),
            t => {
                return Err(P2PError::InvalidMessage(format!(
                    "invalid comment target type: {}",
                    t
                )))
            }
        };

        let mut comment = Comment::new(self.id, target, self.author, self.body);
        comment.id = self.id;
        comment.created_at = self.created_at;
        comment.updated_at = self.updated_at;

        Ok(comment)
    }
}

/// Serializable version of a review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableReview {
    pub id: u64,
    pub repo_key: String,
    pub pr_number: u32,
    pub author: String,
    pub state: String,
    pub body: Option<String>,
    pub commit_id: String,
    pub created_at: u64,
}

impl From<Review> for SerializableReview {
    fn from(review: Review) -> Self {
        Self {
            id: review.id,
            repo_key: review.repo_key,
            pr_number: review.pr_number,
            author: review.author,
            state: review.state.to_string(),
            body: review.body,
            commit_id: review.commit_id,
            created_at: review.created_at,
        }
    }
}

impl SerializableReview {
    /// Convert back to a Review.
    pub fn into_review(self) -> Result<Review> {
        let state = match self.state.as_str() {
            "approved" => ReviewState::Approved,
            "changes_requested" => ReviewState::ChangesRequested,
            "commented" => ReviewState::Commented,
            "dismissed" => ReviewState::Dismissed,
            s => {
                return Err(P2PError::InvalidMessage(format!(
                    "invalid review state: {}",
                    s
                )))
            }
        };

        let mut review = Review::new(
            self.id,
            self.repo_key,
            self.pr_number,
            self.author,
            state,
            self.commit_id,
        );

        if let Some(body) = self.body {
            review = review.with_body(body);
        }

        review.id = self.id;
        review.created_at = self.created_at;

        Ok(review)
    }
}

/// Collaboration message for P2P transmission.
#[derive(Debug, Clone)]
pub enum CollaborationMessage {
    /// A new pull request was created.
    PullRequestCreated(SerializablePullRequest),
    /// A pull request was updated.
    PullRequestUpdated(SerializablePullRequest),
    /// A new issue was created.
    IssueCreated(SerializableIssue),
    /// An issue was updated.
    IssueUpdated(SerializableIssue),
    /// A new comment was created.
    CommentCreated(SerializableComment),
    /// A new review was created.
    ReviewCreated(SerializableReview),
    /// Request sync of collaboration data.
    SyncCollaborationRequest { repo_key: String },
    /// Response with collaboration data.
    SyncCollaborationResponse {
        repo_key: String,
        pull_requests: Vec<SerializablePullRequest>,
        issues: Vec<SerializableIssue>,
        comments: Vec<SerializableComment>,
        reviews: Vec<SerializableReview>,
    },
}

impl CollaborationMessage {
    /// Encode the message to bytes.
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();

        match self {
            CollaborationMessage::PullRequestCreated(pr) => {
                buf.put_u8(CollaborationMessageType::PullRequestCreated as u8);
                let json = serde_json::to_vec(pr).unwrap();
                buf.put_u32(json.len() as u32);
                buf.put_slice(&json);
            }
            CollaborationMessage::PullRequestUpdated(pr) => {
                buf.put_u8(CollaborationMessageType::PullRequestUpdated as u8);
                let json = serde_json::to_vec(pr).unwrap();
                buf.put_u32(json.len() as u32);
                buf.put_slice(&json);
            }
            CollaborationMessage::IssueCreated(issue) => {
                buf.put_u8(CollaborationMessageType::IssueCreated as u8);
                let json = serde_json::to_vec(issue).unwrap();
                buf.put_u32(json.len() as u32);
                buf.put_slice(&json);
            }
            CollaborationMessage::IssueUpdated(issue) => {
                buf.put_u8(CollaborationMessageType::IssueUpdated as u8);
                let json = serde_json::to_vec(issue).unwrap();
                buf.put_u32(json.len() as u32);
                buf.put_slice(&json);
            }
            CollaborationMessage::CommentCreated(comment) => {
                buf.put_u8(CollaborationMessageType::CommentCreated as u8);
                let json = serde_json::to_vec(comment).unwrap();
                buf.put_u32(json.len() as u32);
                buf.put_slice(&json);
            }
            CollaborationMessage::ReviewCreated(review) => {
                buf.put_u8(CollaborationMessageType::ReviewCreated as u8);
                let json = serde_json::to_vec(review).unwrap();
                buf.put_u32(json.len() as u32);
                buf.put_slice(&json);
            }
            CollaborationMessage::SyncCollaborationRequest { repo_key } => {
                buf.put_u8(CollaborationMessageType::SyncCollaborationRequest as u8);
                let repo_bytes = repo_key.as_bytes();
                buf.put_u16(repo_bytes.len() as u16);
                buf.put_slice(repo_bytes);
            }
            CollaborationMessage::SyncCollaborationResponse {
                repo_key,
                pull_requests,
                issues,
                comments,
                reviews,
            } => {
                buf.put_u8(CollaborationMessageType::SyncCollaborationResponse as u8);

                // Repo key
                let repo_bytes = repo_key.as_bytes();
                buf.put_u16(repo_bytes.len() as u16);
                buf.put_slice(repo_bytes);

                // PRs
                let pr_json = serde_json::to_vec(pull_requests).unwrap();
                buf.put_u32(pr_json.len() as u32);
                buf.put_slice(&pr_json);

                // Issues
                let issue_json = serde_json::to_vec(issues).unwrap();
                buf.put_u32(issue_json.len() as u32);
                buf.put_slice(&issue_json);

                // Comments
                let comment_json = serde_json::to_vec(comments).unwrap();
                buf.put_u32(comment_json.len() as u32);
                buf.put_slice(&comment_json);

                // Reviews
                let review_json = serde_json::to_vec(reviews).unwrap();
                buf.put_u32(review_json.len() as u32);
                buf.put_slice(&review_json);
            }
        }

        buf.freeze()
    }

    /// Decode a message from bytes.
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(P2PError::InvalidMessage("empty collaboration message".into()));
        }

        let msg_type = CollaborationMessageType::from_byte(data[0])?;
        let mut payload = &data[1..];

        match msg_type {
            CollaborationMessageType::PullRequestCreated => {
                let len = read_u32(&mut payload)?;
                let pr: SerializablePullRequest = serde_json::from_slice(&payload[..len as usize])
                    .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                Ok(CollaborationMessage::PullRequestCreated(pr))
            }
            CollaborationMessageType::PullRequestUpdated => {
                let len = read_u32(&mut payload)?;
                let pr: SerializablePullRequest = serde_json::from_slice(&payload[..len as usize])
                    .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                Ok(CollaborationMessage::PullRequestUpdated(pr))
            }
            CollaborationMessageType::IssueCreated => {
                let len = read_u32(&mut payload)?;
                let issue: SerializableIssue = serde_json::from_slice(&payload[..len as usize])
                    .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                Ok(CollaborationMessage::IssueCreated(issue))
            }
            CollaborationMessageType::IssueUpdated => {
                let len = read_u32(&mut payload)?;
                let issue: SerializableIssue = serde_json::from_slice(&payload[..len as usize])
                    .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                Ok(CollaborationMessage::IssueUpdated(issue))
            }
            CollaborationMessageType::CommentCreated => {
                let len = read_u32(&mut payload)?;
                let comment: SerializableComment =
                    serde_json::from_slice(&payload[..len as usize])
                        .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                Ok(CollaborationMessage::CommentCreated(comment))
            }
            CollaborationMessageType::ReviewCreated => {
                let len = read_u32(&mut payload)?;
                let review: SerializableReview = serde_json::from_slice(&payload[..len as usize])
                    .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                Ok(CollaborationMessage::ReviewCreated(review))
            }
            CollaborationMessageType::SyncCollaborationRequest => {
                let repo_len = read_u16(&mut payload)?;
                let repo_key = String::from_utf8(payload[..repo_len as usize].to_vec())
                    .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                Ok(CollaborationMessage::SyncCollaborationRequest { repo_key })
            }
            CollaborationMessageType::SyncCollaborationResponse => {
                // Repo key
                let repo_len = read_u16(&mut payload)?;
                let repo_key = String::from_utf8(payload[..repo_len as usize].to_vec())
                    .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                payload = &payload[repo_len as usize..];

                // PRs
                let pr_len = read_u32(&mut payload)?;
                let pull_requests: Vec<SerializablePullRequest> =
                    serde_json::from_slice(&payload[..pr_len as usize])
                        .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                payload = &payload[pr_len as usize..];

                // Issues
                let issue_len = read_u32(&mut payload)?;
                let issues: Vec<SerializableIssue> =
                    serde_json::from_slice(&payload[..issue_len as usize])
                        .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                payload = &payload[issue_len as usize..];

                // Comments
                let comment_len = read_u32(&mut payload)?;
                let comments: Vec<SerializableComment> =
                    serde_json::from_slice(&payload[..comment_len as usize])
                        .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;
                payload = &payload[comment_len as usize..];

                // Reviews
                let review_len = read_u32(&mut payload)?;
                let reviews: Vec<SerializableReview> =
                    serde_json::from_slice(&payload[..review_len as usize])
                        .map_err(|e| P2PError::InvalidMessage(e.to_string()))?;

                Ok(CollaborationMessage::SyncCollaborationResponse {
                    repo_key,
                    pull_requests,
                    issues,
                    comments,
                    reviews,
                })
            }
        }
    }
}

fn read_u16(buf: &mut &[u8]) -> Result<u16> {
    if buf.remaining() < 2 {
        return Err(P2PError::InvalidMessage("truncated u16".into()));
    }
    Ok(buf.get_u16())
}

fn read_u32(buf: &mut &[u8]) -> Result<u32> {
    if buf.remaining() < 4 {
        return Err(P2PError::InvalidMessage("truncated u32".into()));
    }
    Ok(buf.get_u32())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_message_roundtrip() {
        let pr = SerializablePullRequest {
            id: 1,
            repo_key: "alice/repo".to_string(),
            number: 1,
            title: "Add feature".to_string(),
            description: "Description".to_string(),
            author: "alice".to_string(),
            state: "open".to_string(),
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            source_commit: "0".repeat(40),
            target_commit: "1".repeat(40),
            labels: vec![],
            created_at: 12345,
            updated_at: 12345,
            merged_at: None,
            merged_by: None,
        };

        let msg = CollaborationMessage::PullRequestCreated(pr.clone());
        let encoded = msg.encode();
        let decoded = CollaborationMessage::decode(&encoded).unwrap();

        match decoded {
            CollaborationMessage::PullRequestCreated(decoded_pr) => {
                assert_eq!(decoded_pr.id, pr.id);
                assert_eq!(decoded_pr.title, pr.title);
                assert_eq!(decoded_pr.number, pr.number);
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_issue_message_roundtrip() {
        let issue = SerializableIssue {
            id: 2,
            repo_key: "bob/project".to_string(),
            number: 5,
            title: "Bug report".to_string(),
            description: "Steps to reproduce".to_string(),
            author: "bob".to_string(),
            state: "open".to_string(),
            labels: vec![SerializableLabel {
                name: "bug".to_string(),
                color: "ff0000".to_string(),
                description: Some("A bug".to_string()),
            }],
            created_at: 54321,
            updated_at: 54321,
            closed_at: None,
            closed_by: None,
        };

        let msg = CollaborationMessage::IssueCreated(issue.clone());
        let encoded = msg.encode();
        let decoded = CollaborationMessage::decode(&encoded).unwrap();

        match decoded {
            CollaborationMessage::IssueCreated(decoded_issue) => {
                assert_eq!(decoded_issue.id, issue.id);
                assert_eq!(decoded_issue.title, issue.title);
                assert_eq!(decoded_issue.labels.len(), 1);
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_sync_request_roundtrip() {
        let msg = CollaborationMessage::SyncCollaborationRequest {
            repo_key: "carol/test".to_string(),
        };

        let encoded = msg.encode();
        let decoded = CollaborationMessage::decode(&encoded).unwrap();

        match decoded {
            CollaborationMessage::SyncCollaborationRequest { repo_key } => {
                assert_eq!(repo_key, "carol/test");
            }
            _ => panic!("wrong message type"),
        }
    }
}
