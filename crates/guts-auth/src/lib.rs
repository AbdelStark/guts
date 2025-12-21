//! Authorization and governance for Guts.
//!
//! This crate provides:
//! - **Permissions**: Granular access control (Read, Write, Admin)
//! - **Organizations**: Multi-user repository ownership
//! - **Teams**: Group-based permission management
//! - **Collaborators**: Direct repository access grants
//! - **Branch Protection**: Rules for protecting important branches
//! - **Webhooks**: Event notifications for CI/CD integration
//!
//! # Example
//!
//! ```
//! use guts_auth::{AuthStore, Permission, OrgMember, OrgRole};
//!
//! // Create a store
//! let store = AuthStore::new();
//!
//! // Create an organization
//! let org = store.create_organization(
//!     "acme".into(),
//!     "Acme Corporation".into(),
//!     "owner_pubkey".into(),
//! ).unwrap();
//!
//! // Create a team with write access
//! let team = store.create_team(
//!     org.id,
//!     "backend".into(),
//!     Permission::Write,
//!     "owner_pubkey".into(),
//! ).unwrap();
//!
//! // Add a member to the team
//! store.add_team_member(team.id, "developer_pubkey".into()).unwrap();
//!
//! // Add a repository to the team
//! store.add_team_repo(team.id, "acme/api".into()).unwrap();
//!
//! // Check permissions
//! assert!(store.check_permission("developer_pubkey", "acme/api", Permission::Write));
//! ```

mod branch_protection;
mod collaborator;
mod error;
mod organization;
mod permission;
mod store;
mod team;
mod webhook;

pub use branch_protection::{BranchProtection, BranchProtectionRequest};
pub use collaborator::{Collaborator, CollaboratorRequest, CollaboratorResponse};
pub use error::{AuthError, Result};
pub use organization::{OrgMember, OrgRole, Organization};
pub use permission::{Permission, PermissionGrant};
pub use store::AuthStore;
pub use team::Team;
pub use webhook::{
    CreateWebhookRequest, UpdateWebhookRequest, Webhook, WebhookEvent, WebhookPayload,
    WebhookRepository,
};
