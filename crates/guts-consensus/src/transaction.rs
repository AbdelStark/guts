//! Consensus transaction types.
//!
//! All state-changing operations in Guts are represented as transactions
//! that go through the consensus layer for total ordering.

use guts_storage::ObjectId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Serializable public key (hex-encoded Ed25519 public key).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerializablePublicKey(pub String);

impl SerializablePublicKey {
    /// Creates from a hex string.
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// Creates from a commonware public key.
    pub fn from_pubkey(pk: &commonware_cryptography::ed25519::PublicKey) -> Self {
        Self(hex::encode(pk.as_ref()))
    }

    /// Converts to a commonware public key.
    pub fn to_pubkey(
        &self,
    ) -> Result<commonware_cryptography::ed25519::PublicKey, hex::FromHexError> {
        let bytes = hex::decode(&self.0)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        // Create via ed25519_consensus VerificationKey which implements Into<PublicKey>
        let vk = ed25519_consensus::VerificationKey::try_from(arr)
            .map_err(|_| hex::FromHexError::InvalidStringLength)?;
        Ok(commonware_cryptography::ed25519::PublicKey::from(vk))
    }

    /// Returns the hex string as a reference.
    pub fn as_hex(&self) -> &str {
        &self.0
    }

    /// Returns the hex string as an owned String.
    pub fn to_hex(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for SerializablePublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Serializable signature (hex-encoded Ed25519 signature).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializableSignature(pub String);

impl SerializableSignature {
    /// Creates from a hex string.
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// Creates from a commonware signature.
    pub fn from_signature(sig: &commonware_cryptography::ed25519::Signature) -> Self {
        Self(hex::encode(sig.as_ref()))
    }

    /// Converts to a commonware signature.
    pub fn to_signature(
        &self,
    ) -> Result<commonware_cryptography::ed25519::Signature, hex::FromHexError> {
        let bytes = hex::decode(&self.0)?;
        if bytes.len() != 64 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        // Create via ed25519_consensus Signature which implements Into<Signature>
        let sig = ed25519_consensus::Signature::from(arr);
        Ok(commonware_cryptography::ed25519::Signature::from(sig))
    }

    /// Returns the hex string.
    pub fn as_hex(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SerializableSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Update to a pull request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestUpdate {
    /// New title (if changed).
    pub title: Option<String>,
    /// New description (if changed).
    pub description: Option<String>,
    /// New state (open, closed, merged).
    pub state: Option<String>,
    /// Labels to add.
    pub labels_add: Vec<String>,
    /// Labels to remove.
    pub labels_remove: Vec<String>,
    /// Merged by (for merge operations).
    pub merged_by: Option<String>,
}

/// Update to an issue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueUpdate {
    /// New title (if changed).
    pub title: Option<String>,
    /// New description (if changed).
    pub description: Option<String>,
    /// New state (open, closed).
    pub state: Option<String>,
    /// Labels to add.
    pub labels_add: Vec<String>,
    /// Labels to remove.
    pub labels_remove: Vec<String>,
    /// Closed by (for close operations).
    pub closed_by: Option<String>,
}

/// Update to an organization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrgUpdate {
    /// New display name.
    pub display_name: Option<String>,
    /// New description.
    pub description: Option<String>,
}

/// Comment target specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommentTargetSpec {
    /// Comment on a pull request.
    PullRequest { repo_key: String, number: u32 },
    /// Comment on an issue.
    Issue { repo_key: String, number: u32 },
}

/// Serializable branch protection rules for consensus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchProtectionSpec {
    /// Require pull request for changes.
    pub require_pr: bool,
    /// Number of required reviews.
    pub required_reviews: u32,
    /// Require signed commits.
    pub require_signed_commits: bool,
    /// Allow force pushes.
    pub allow_force_push: bool,
    /// Allow deletions.
    pub allow_deletions: bool,
    /// Required status checks.
    pub required_status_checks: Vec<String>,
}

/// Serializable team specification for consensus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamSpec {
    /// Team name (slug).
    pub name: String,
    /// Team description.
    pub description: Option<String>,
    /// Team permission level.
    pub permission: String,
}

/// A unique transaction identifier (SHA-256 hash of the transaction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId([u8; 32]);

impl TransactionId {
    /// Creates a transaction ID from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the hex representation.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Creates a transaction ID from a hex string.
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Transactions that require consensus ordering.
///
/// Every state-changing operation in Guts is represented as a transaction.
/// Transactions are ordered by the consensus layer and applied atomically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Transaction {
    // ==================== Git Operations ====================
    /// Push operation updating references.
    GitPush {
        /// Repository key (owner/name).
        repo_key: String,
        /// Reference being updated (e.g., "refs/heads/main").
        ref_name: String,
        /// Old object ID (for optimistic locking).
        old_oid: ObjectId,
        /// New object ID.
        new_oid: ObjectId,
        /// List of new objects being added.
        objects: Vec<ObjectId>,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature over transaction data.
        signature: SerializableSignature,
    },

    // ==================== Repository Management ====================
    /// Create a new repository.
    CreateRepository {
        /// Owner (user or organization).
        owner: String,
        /// Repository name.
        name: String,
        /// Description.
        description: String,
        /// Default branch name.
        default_branch: String,
        /// Visibility (public/private).
        visibility: String,
        /// Creator's public key.
        creator: SerializablePublicKey,
        /// Signature over transaction data.
        signature: SerializableSignature,
    },

    /// Delete a repository.
    DeleteRepository {
        /// Repository key (owner/name).
        repo_key: String,
        /// Deleter's public key.
        deleter: SerializablePublicKey,
        /// Signature over transaction data.
        signature: SerializableSignature,
    },

    // ==================== Collaboration - Pull Requests ====================
    /// Create a new pull request.
    CreatePullRequest {
        /// Repository key.
        repo_key: String,
        /// PR title.
        title: String,
        /// PR description.
        description: String,
        /// Author username.
        author: String,
        /// Source branch.
        source_branch: String,
        /// Target branch.
        target_branch: String,
        /// Source commit hash.
        source_commit: ObjectId,
        /// Target commit hash.
        target_commit: ObjectId,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Update an existing pull request.
    UpdatePullRequest {
        /// Repository key.
        repo_key: String,
        /// PR number.
        pr_number: u32,
        /// Update details.
        update: PullRequestUpdate,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Merge a pull request.
    MergePullRequest {
        /// Repository key.
        repo_key: String,
        /// PR number.
        pr_number: u32,
        /// Merge commit object ID.
        merge_commit: ObjectId,
        /// Person performing the merge.
        merged_by: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    // ==================== Collaboration - Issues ====================
    /// Create a new issue.
    CreateIssue {
        /// Repository key.
        repo_key: String,
        /// Issue title.
        title: String,
        /// Issue description.
        description: String,
        /// Author username.
        author: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Update an existing issue.
    UpdateIssue {
        /// Repository key.
        repo_key: String,
        /// Issue number.
        issue_number: u32,
        /// Update details.
        update: IssueUpdate,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    // ==================== Collaboration - Comments & Reviews ====================
    /// Create a comment on a PR or issue.
    CreateComment {
        /// Comment target (PR or issue).
        target: CommentTargetSpec,
        /// Author username.
        author: String,
        /// Comment body.
        body: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Create a review on a pull request.
    CreateReview {
        /// Repository key.
        repo_key: String,
        /// PR number.
        pr_number: u32,
        /// Reviewer username.
        author: String,
        /// Review state (approved, changes_requested, commented).
        state: String,
        /// Optional review body.
        body: Option<String>,
        /// Commit ID being reviewed.
        commit_id: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    // ==================== Governance - Organizations ====================
    /// Create a new organization.
    CreateOrganization {
        /// Organization slug (unique name).
        name: String,
        /// Display name.
        display_name: String,
        /// Creator's public key.
        creator: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Update an organization.
    UpdateOrganization {
        /// Organization name.
        org_name: String,
        /// Update details.
        update: OrgUpdate,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Add a member to an organization.
    AddOrgMember {
        /// Organization name.
        org_name: String,
        /// Member's public key.
        member: SerializablePublicKey,
        /// Role (owner, admin, member).
        role: String,
        /// Signer's public key (admin performing action).
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Remove a member from an organization.
    RemoveOrgMember {
        /// Organization name.
        org_name: String,
        /// Member's public key.
        member: SerializablePublicKey,
        /// Signer's public key (admin performing action).
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    // ==================== Governance - Teams ====================
    /// Create a new team.
    CreateTeam {
        /// Organization name.
        org_name: String,
        /// Team specification.
        team: TeamSpec,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Delete a team.
    DeleteTeam {
        /// Organization name.
        org_name: String,
        /// Team name.
        team_name: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Add a member to a team.
    AddTeamMember {
        /// Organization name.
        org_name: String,
        /// Team name.
        team_name: String,
        /// Member's public key.
        member: SerializablePublicKey,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Remove a member from a team.
    RemoveTeamMember {
        /// Organization name.
        org_name: String,
        /// Team name.
        team_name: String,
        /// Member's public key.
        member: SerializablePublicKey,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Add a repository to a team.
    AddTeamRepo {
        /// Organization name.
        org_name: String,
        /// Team name.
        team_name: String,
        /// Repository key.
        repo_key: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    // ==================== Governance - Permissions ====================
    /// Add or update a collaborator on a repository.
    SetCollaborator {
        /// Repository key.
        repo_key: String,
        /// Collaborator's username or public key.
        collaborator: String,
        /// Permission level (read, write, admin).
        permission: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Remove a collaborator from a repository.
    RemoveCollaborator {
        /// Repository key.
        repo_key: String,
        /// Collaborator's username or public key.
        collaborator: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    // ==================== Governance - Branch Protection ====================
    /// Set branch protection rules.
    SetBranchProtection {
        /// Repository key.
        repo_key: String,
        /// Branch name (pattern).
        branch: String,
        /// Protection rules.
        protection: BranchProtectionSpec,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },

    /// Remove branch protection rules.
    RemoveBranchProtection {
        /// Repository key.
        repo_key: String,
        /// Branch name (pattern).
        branch: String,
        /// Signer's public key.
        signer: SerializablePublicKey,
        /// Signature.
        signature: SerializableSignature,
    },
}

impl Transaction {
    /// Computes the unique transaction ID (SHA-256 hash of serialized transaction).
    pub fn id(&self) -> TransactionId {
        let bytes = serde_json::to_vec(self).expect("transaction serialization should not fail");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        TransactionId(id)
    }

    /// Returns the signer's public key for this transaction.
    pub fn signer(&self) -> &SerializablePublicKey {
        match self {
            Transaction::GitPush { signer, .. } => signer,
            Transaction::CreateRepository { creator, .. } => creator,
            Transaction::DeleteRepository { deleter, .. } => deleter,
            Transaction::CreatePullRequest { signer, .. } => signer,
            Transaction::UpdatePullRequest { signer, .. } => signer,
            Transaction::MergePullRequest { signer, .. } => signer,
            Transaction::CreateIssue { signer, .. } => signer,
            Transaction::UpdateIssue { signer, .. } => signer,
            Transaction::CreateComment { signer, .. } => signer,
            Transaction::CreateReview { signer, .. } => signer,
            Transaction::CreateOrganization { creator, .. } => creator,
            Transaction::UpdateOrganization { signer, .. } => signer,
            Transaction::AddOrgMember { signer, .. } => signer,
            Transaction::RemoveOrgMember { signer, .. } => signer,
            Transaction::CreateTeam { signer, .. } => signer,
            Transaction::DeleteTeam { signer, .. } => signer,
            Transaction::AddTeamMember { signer, .. } => signer,
            Transaction::RemoveTeamMember { signer, .. } => signer,
            Transaction::AddTeamRepo { signer, .. } => signer,
            Transaction::SetCollaborator { signer, .. } => signer,
            Transaction::RemoveCollaborator { signer, .. } => signer,
            Transaction::SetBranchProtection { signer, .. } => signer,
            Transaction::RemoveBranchProtection { signer, .. } => signer,
        }
    }

    /// Returns the signature for this transaction.
    pub fn signature(&self) -> &SerializableSignature {
        match self {
            Transaction::GitPush { signature, .. } => signature,
            Transaction::CreateRepository { signature, .. } => signature,
            Transaction::DeleteRepository { signature, .. } => signature,
            Transaction::CreatePullRequest { signature, .. } => signature,
            Transaction::UpdatePullRequest { signature, .. } => signature,
            Transaction::MergePullRequest { signature, .. } => signature,
            Transaction::CreateIssue { signature, .. } => signature,
            Transaction::UpdateIssue { signature, .. } => signature,
            Transaction::CreateComment { signature, .. } => signature,
            Transaction::CreateReview { signature, .. } => signature,
            Transaction::CreateOrganization { signature, .. } => signature,
            Transaction::UpdateOrganization { signature, .. } => signature,
            Transaction::AddOrgMember { signature, .. } => signature,
            Transaction::RemoveOrgMember { signature, .. } => signature,
            Transaction::CreateTeam { signature, .. } => signature,
            Transaction::DeleteTeam { signature, .. } => signature,
            Transaction::AddTeamMember { signature, .. } => signature,
            Transaction::RemoveTeamMember { signature, .. } => signature,
            Transaction::AddTeamRepo { signature, .. } => signature,
            Transaction::SetCollaborator { signature, .. } => signature,
            Transaction::RemoveCollaborator { signature, .. } => signature,
            Transaction::SetBranchProtection { signature, .. } => signature,
            Transaction::RemoveBranchProtection { signature, .. } => signature,
        }
    }

    /// Returns the affected repository key, if any.
    pub fn repo_key(&self) -> Option<&str> {
        match self {
            Transaction::GitPush { repo_key, .. } => Some(repo_key),
            Transaction::CreateRepository { .. } => None, // Key is owner/name, returned separately
            Transaction::DeleteRepository { repo_key, .. } => Some(repo_key),
            Transaction::CreatePullRequest { repo_key, .. } => Some(repo_key),
            Transaction::UpdatePullRequest { repo_key, .. } => Some(repo_key),
            Transaction::MergePullRequest { repo_key, .. } => Some(repo_key),
            Transaction::CreateIssue { repo_key, .. } => Some(repo_key),
            Transaction::UpdateIssue { repo_key, .. } => Some(repo_key),
            Transaction::CreateComment { target, .. } => match target {
                CommentTargetSpec::PullRequest { repo_key, .. } => Some(repo_key),
                CommentTargetSpec::Issue { repo_key, .. } => Some(repo_key),
            },
            Transaction::CreateReview { repo_key, .. } => Some(repo_key),
            Transaction::SetCollaborator { repo_key, .. } => Some(repo_key),
            Transaction::RemoveCollaborator { repo_key, .. } => Some(repo_key),
            Transaction::SetBranchProtection { repo_key, .. } => Some(repo_key),
            Transaction::RemoveBranchProtection { repo_key, .. } => Some(repo_key),
            Transaction::AddTeamRepo { repo_key, .. } => Some(repo_key),
            _ => None, // Org/Team transactions don't have a repo_key
        }
    }

    /// Returns a human-readable description of the transaction type.
    pub fn kind(&self) -> &'static str {
        match self {
            Transaction::GitPush { .. } => "git_push",
            Transaction::CreateRepository { .. } => "create_repository",
            Transaction::DeleteRepository { .. } => "delete_repository",
            Transaction::CreatePullRequest { .. } => "create_pull_request",
            Transaction::UpdatePullRequest { .. } => "update_pull_request",
            Transaction::MergePullRequest { .. } => "merge_pull_request",
            Transaction::CreateIssue { .. } => "create_issue",
            Transaction::UpdateIssue { .. } => "update_issue",
            Transaction::CreateComment { .. } => "create_comment",
            Transaction::CreateReview { .. } => "create_review",
            Transaction::CreateOrganization { .. } => "create_organization",
            Transaction::UpdateOrganization { .. } => "update_organization",
            Transaction::AddOrgMember { .. } => "add_org_member",
            Transaction::RemoveOrgMember { .. } => "remove_org_member",
            Transaction::CreateTeam { .. } => "create_team",
            Transaction::DeleteTeam { .. } => "delete_team",
            Transaction::AddTeamMember { .. } => "add_team_member",
            Transaction::RemoveTeamMember { .. } => "remove_team_member",
            Transaction::AddTeamRepo { .. } => "add_team_repo",
            Transaction::SetCollaborator { .. } => "set_collaborator",
            Transaction::RemoveCollaborator { .. } => "remove_collaborator",
            Transaction::SetBranchProtection { .. } => "set_branch_protection",
            Transaction::RemoveBranchProtection { .. } => "remove_branch_protection",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::{ed25519, PrivateKeyExt, Signer};

    fn test_keypair() -> (SerializablePublicKey, SerializableSignature) {
        let key = ed25519::PrivateKey::from_seed(42);
        let sig = key.sign(Some(b"_GUTS"), b"test");
        (
            SerializablePublicKey::from_pubkey(&key.public_key()),
            SerializableSignature::from_signature(&sig),
        )
    }

    #[test]
    fn test_transaction_id_roundtrip() {
        let bytes = [0xab; 32];
        let id = TransactionId::from_bytes(bytes);
        assert_eq!(id.as_bytes(), &bytes);

        let hex = id.to_hex();
        let parsed = TransactionId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_transaction_id_display() {
        let id = TransactionId::from_bytes([0; 32]);
        assert_eq!(format!("{}", id), "0".repeat(64));
    }

    #[test]
    fn test_transaction_kind() {
        let (signer, signature) = test_keypair();

        let tx = Transaction::CreateRepository {
            owner: "alice".into(),
            name: "test".into(),
            description: "A test repo".into(),
            default_branch: "main".into(),
            visibility: "public".into(),
            creator: signer,
            signature,
        };

        assert_eq!(tx.kind(), "create_repository");
    }

    #[test]
    fn test_transaction_id_unique() {
        let (signer, signature) = test_keypair();

        let tx1 = Transaction::CreateRepository {
            owner: "alice".into(),
            name: "test1".into(),
            description: "A test repo".into(),
            default_branch: "main".into(),
            visibility: "public".into(),
            creator: signer.clone(),
            signature: signature.clone(),
        };

        let tx2 = Transaction::CreateRepository {
            owner: "alice".into(),
            name: "test2".into(), // Different name
            description: "A test repo".into(),
            default_branch: "main".into(),
            visibility: "public".into(),
            creator: signer,
            signature,
        };

        assert_ne!(tx1.id(), tx2.id());
    }

    #[test]
    fn test_serializable_pubkey_roundtrip() {
        let key = ed25519::PrivateKey::from_seed(42);
        let pk = key.public_key();
        let serializable = SerializablePublicKey::from_pubkey(&pk);
        let recovered = serializable.to_pubkey().unwrap();
        assert_eq!(pk, recovered);
    }

    #[test]
    fn test_serializable_signature_roundtrip() {
        let key = ed25519::PrivateKey::from_seed(42);
        let sig = key.sign(Some(b"_GUTS"), b"test");
        let serializable = SerializableSignature::from_signature(&sig);
        let recovered = serializable.to_signature().unwrap();
        assert_eq!(sig, recovered);
    }

    #[test]
    fn test_transaction_serialization() {
        let (signer, signature) = test_keypair();

        let tx = Transaction::CreateRepository {
            owner: "alice".into(),
            name: "test".into(),
            description: "A test repo".into(),
            default_branch: "main".into(),
            visibility: "public".into(),
            creator: signer,
            signature,
        };

        let json = serde_json::to_string(&tx).unwrap();
        let parsed: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(tx.id(), parsed.id());
    }
}
