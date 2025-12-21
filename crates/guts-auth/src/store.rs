//! In-memory authorization store.

use crate::{
    branch_protection::BranchProtection,
    collaborator::Collaborator,
    error::{AuthError, Result},
    organization::{OrgMember, OrgRole, Organization},
    permission::Permission,
    team::Team,
    webhook::Webhook,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe in-memory store for authorization data.
#[derive(Debug, Default)]
pub struct AuthStore {
    /// Next available ID for new entities.
    next_id: AtomicU64,

    /// Organizations by ID.
    organizations: RwLock<HashMap<u64, Organization>>,

    /// Organization name to ID mapping.
    org_name_index: RwLock<HashMap<String, u64>>,

    /// Teams by ID.
    teams: RwLock<HashMap<u64, Team>>,

    /// Collaborators by (repo_key, user) pair.
    collaborators: RwLock<HashMap<(String, String), Collaborator>>,

    /// Collaborator repo index: repo_key -> list of (user, permission).
    collaborator_index: RwLock<HashMap<String, Vec<(String, Permission)>>>,

    /// Branch protection rules by (repo_key, pattern) pair.
    branch_protections: RwLock<HashMap<(String, String), BranchProtection>>,

    /// Webhooks by ID.
    webhooks: RwLock<HashMap<u64, Webhook>>,

    /// Webhook repo index: repo_key -> webhook IDs.
    webhook_index: RwLock<HashMap<String, Vec<u64>>>,
}

impl AuthStore {
    /// Create a new empty auth store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a new unique ID.
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst) + 1
    }

    // ==================== Organizations ====================

    /// Create a new organization.
    pub fn create_organization(&self, name: String, display_name: String, creator: String) -> Result<Organization> {
        // Check if name already exists
        if self.org_name_index.read().contains_key(&name) {
            return Err(AuthError::AlreadyExists(format!("organization '{}'", name)));
        }

        let id = self.next_id();
        let org = Organization::new(id, name.clone(), display_name, creator);

        self.organizations.write().insert(id, org.clone());
        self.org_name_index.write().insert(name, id);

        Ok(org)
    }

    /// Get an organization by ID.
    pub fn get_organization(&self, id: u64) -> Option<Organization> {
        self.organizations.read().get(&id).cloned()
    }

    /// Get an organization by name.
    pub fn get_organization_by_name(&self, name: &str) -> Option<Organization> {
        let id = self.org_name_index.read().get(name).copied()?;
        self.get_organization(id)
    }

    /// List all organizations.
    pub fn list_organizations(&self) -> Vec<Organization> {
        self.organizations.read().values().cloned().collect()
    }

    /// List organizations a user belongs to.
    pub fn list_user_organizations(&self, user: &str) -> Vec<Organization> {
        self.organizations
            .read()
            .values()
            .filter(|org| org.is_member(user))
            .cloned()
            .collect()
    }

    /// Update an organization.
    pub fn update_organization(
        &self,
        id: u64,
        display_name: Option<String>,
        description: Option<String>,
    ) -> Result<Organization> {
        let mut orgs = self.organizations.write();
        let org = orgs.get_mut(&id).ok_or_else(|| AuthError::NotFound(format!("organization {}", id)))?;

        if let Some(dn) = display_name {
            org.display_name = dn;
        }
        if let Some(desc) = description {
            org.description = Some(desc);
        }
        org.updated_at = Self::now();

        Ok(org.clone())
    }

    /// Delete an organization.
    pub fn delete_organization(&self, id: u64) -> Result<()> {
        let mut orgs = self.organizations.write();
        let org = orgs.remove(&id).ok_or_else(|| AuthError::NotFound(format!("organization {}", id)))?;

        // Remove from name index
        self.org_name_index.write().remove(&org.name);

        // Remove all teams
        let team_ids: Vec<u64> = org.teams.iter().copied().collect();
        for team_id in team_ids {
            self.teams.write().remove(&team_id);
        }

        Ok(())
    }

    /// Add a member to an organization.
    pub fn add_org_member(&self, org_id: u64, member: OrgMember) -> Result<()> {
        let mut orgs = self.organizations.write();
        let org = orgs.get_mut(&org_id).ok_or_else(|| AuthError::NotFound(format!("organization {}", org_id)))?;

        if !org.add_member(member.clone()) {
            return Err(AuthError::AlreadyExists(format!("member '{}'", member.user)));
        }

        Ok(())
    }

    /// Remove a member from an organization.
    pub fn remove_org_member(&self, org_id: u64, user: &str) -> Result<()> {
        let mut orgs = self.organizations.write();
        let org = orgs.get_mut(&org_id).ok_or_else(|| AuthError::NotFound(format!("organization {}", org_id)))?;

        org.remove_member(user).map_err(|_| AuthError::LastOwner)?;

        Ok(())
    }

    /// Update a member's role in an organization.
    pub fn update_org_member_role(&self, org_id: u64, user: &str, role: OrgRole) -> Result<()> {
        let mut orgs = self.organizations.write();
        let org = orgs.get_mut(&org_id).ok_or_else(|| AuthError::NotFound(format!("organization {}", org_id)))?;

        org.update_member_role(user, role).map_err(|_| AuthError::LastOwner)?;

        Ok(())
    }

    // ==================== Teams ====================

    /// Create a new team.
    pub fn create_team(
        &self,
        org_id: u64,
        name: String,
        permission: Permission,
        created_by: String,
    ) -> Result<Team> {
        // Verify org exists
        let mut orgs = self.organizations.write();
        let org = orgs.get_mut(&org_id).ok_or_else(|| AuthError::NotFound(format!("organization {}", org_id)))?;

        // Check for duplicate name
        let teams = self.teams.read();
        if teams.values().any(|t| t.org_id == org_id && t.name == name) {
            return Err(AuthError::AlreadyExists(format!("team '{}' in organization", name)));
        }
        drop(teams);

        let id = self.next_id();
        let team = Team::new(id, org_id, name, permission, created_by);

        org.add_team(id);
        self.teams.write().insert(id, team.clone());

        Ok(team)
    }

    /// Get a team by ID.
    pub fn get_team(&self, id: u64) -> Option<Team> {
        self.teams.read().get(&id).cloned()
    }

    /// Get a team by org and name.
    pub fn get_team_by_name(&self, org_id: u64, name: &str) -> Option<Team> {
        self.teams
            .read()
            .values()
            .find(|t| t.org_id == org_id && t.name == name)
            .cloned()
    }

    /// List teams in an organization.
    pub fn list_teams(&self, org_id: u64) -> Vec<Team> {
        self.teams
            .read()
            .values()
            .filter(|t| t.org_id == org_id)
            .cloned()
            .collect()
    }

    /// List teams a user belongs to.
    pub fn list_user_teams(&self, user: &str) -> Vec<Team> {
        self.teams
            .read()
            .values()
            .filter(|t| t.is_member(user))
            .cloned()
            .collect()
    }

    /// Update a team.
    pub fn update_team(
        &self,
        id: u64,
        name: Option<String>,
        description: Option<String>,
        permission: Option<Permission>,
    ) -> Result<Team> {
        let mut teams = self.teams.write();
        let team = teams.get_mut(&id).ok_or_else(|| AuthError::NotFound(format!("team {}", id)))?;

        if let Some(n) = name {
            team.name = n;
        }
        if let Some(d) = description {
            team.description = Some(d);
        }
        if let Some(p) = permission {
            team.permission = p;
        }
        team.updated_at = Self::now();

        Ok(team.clone())
    }

    /// Delete a team.
    pub fn delete_team(&self, id: u64) -> Result<()> {
        let team = self.teams.write().remove(&id).ok_or_else(|| AuthError::NotFound(format!("team {}", id)))?;

        // Remove from org
        if let Some(org) = self.organizations.write().get_mut(&team.org_id) {
            org.remove_team(id);
        }

        Ok(())
    }

    /// Add a member to a team.
    pub fn add_team_member(&self, team_id: u64, user: String) -> Result<()> {
        let mut teams = self.teams.write();
        let team = teams.get_mut(&team_id).ok_or_else(|| AuthError::NotFound(format!("team {}", team_id)))?;

        if !team.add_member(user.clone()) {
            return Err(AuthError::AlreadyExists(format!("member '{}' in team", user)));
        }

        Ok(())
    }

    /// Remove a member from a team.
    pub fn remove_team_member(&self, team_id: u64, user: &str) -> Result<()> {
        let mut teams = self.teams.write();
        let team = teams.get_mut(&team_id).ok_or_else(|| AuthError::NotFound(format!("team {}", team_id)))?;

        if !team.remove_member(user) {
            return Err(AuthError::NotFound(format!("member '{}' in team", user)));
        }

        Ok(())
    }

    /// Add a repository to a team.
    pub fn add_team_repo(&self, team_id: u64, repo_key: String) -> Result<()> {
        let mut teams = self.teams.write();
        let team = teams.get_mut(&team_id).ok_or_else(|| AuthError::NotFound(format!("team {}", team_id)))?;

        team.add_repo(repo_key);
        Ok(())
    }

    /// Remove a repository from a team.
    pub fn remove_team_repo(&self, team_id: u64, repo_key: &str) -> Result<()> {
        let mut teams = self.teams.write();
        let team = teams.get_mut(&team_id).ok_or_else(|| AuthError::NotFound(format!("team {}", team_id)))?;

        if !team.remove_repo(repo_key) {
            return Err(AuthError::NotFound(format!("repo '{}' in team", repo_key)));
        }

        Ok(())
    }

    // ==================== Collaborators ====================

    /// Add or update a collaborator.
    pub fn set_collaborator(&self, repo_key: String, user: String, permission: Permission, added_by: String) -> Collaborator {
        let key = (repo_key.clone(), user.clone());
        let mut collabs = self.collaborators.write();

        if let Some(existing) = collabs.get_mut(&key) {
            existing.permission = permission;
            existing.updated_at = Self::now();
            return existing.clone();
        }

        let id = self.next_id();
        let collab = Collaborator::new(id, repo_key.clone(), user.clone(), permission, added_by);
        collabs.insert(key, collab.clone());

        // Update index
        let mut index = self.collaborator_index.write();
        index.entry(repo_key).or_default().push((user, permission));

        collab
    }

    /// Get a collaborator.
    pub fn get_collaborator(&self, repo_key: &str, user: &str) -> Option<Collaborator> {
        let key = (repo_key.to_string(), user.to_string());
        self.collaborators.read().get(&key).cloned()
    }

    /// List collaborators for a repository.
    pub fn list_collaborators(&self, repo_key: &str) -> Vec<Collaborator> {
        self.collaborators
            .read()
            .values()
            .filter(|c| c.repo_key == repo_key)
            .cloned()
            .collect()
    }

    /// Remove a collaborator.
    pub fn remove_collaborator(&self, repo_key: &str, user: &str) -> Result<()> {
        let key = (repo_key.to_string(), user.to_string());
        if self.collaborators.write().remove(&key).is_none() {
            return Err(AuthError::NotFound(format!("collaborator '{}' on '{}'", user, repo_key)));
        }

        // Update index
        let mut index = self.collaborator_index.write();
        if let Some(users) = index.get_mut(repo_key) {
            users.retain(|(u, _)| u != user);
        }

        Ok(())
    }

    // ==================== Branch Protection ====================

    /// Set branch protection for a pattern.
    pub fn set_branch_protection(&self, repo_key: String, pattern: String) -> BranchProtection {
        let key = (repo_key.clone(), pattern.clone());
        let mut protections = self.branch_protections.write();

        if let Some(existing) = protections.get(&key) {
            return existing.clone();
        }

        let id = self.next_id();
        let protection = BranchProtection::new(id, repo_key, pattern);
        protections.insert(key, protection.clone());
        protection
    }

    /// Get branch protection for a pattern.
    pub fn get_branch_protection(&self, repo_key: &str, pattern: &str) -> Option<BranchProtection> {
        let key = (repo_key.to_string(), pattern.to_string());
        self.branch_protections.read().get(&key).cloned()
    }

    /// Find branch protection that matches a branch name.
    pub fn find_branch_protection(&self, repo_key: &str, branch: &str) -> Option<BranchProtection> {
        self.branch_protections
            .read()
            .values()
            .filter(|p| p.repo_key == repo_key && p.matches(branch))
            .max_by_key(|p| p.pattern.len()) // Most specific match
            .cloned()
    }

    /// List all branch protections for a repository.
    pub fn list_branch_protections(&self, repo_key: &str) -> Vec<BranchProtection> {
        self.branch_protections
            .read()
            .values()
            .filter(|p| p.repo_key == repo_key)
            .cloned()
            .collect()
    }

    /// Update branch protection.
    pub fn update_branch_protection(&self, repo_key: &str, pattern: &str, update: impl FnOnce(&mut BranchProtection)) -> Result<BranchProtection> {
        let key = (repo_key.to_string(), pattern.to_string());
        let mut protections = self.branch_protections.write();
        let protection = protections.get_mut(&key).ok_or_else(|| AuthError::NotFound(format!("branch protection for '{}'", pattern)))?;

        update(protection);
        protection.updated_at = Self::now();

        Ok(protection.clone())
    }

    /// Remove branch protection.
    pub fn remove_branch_protection(&self, repo_key: &str, pattern: &str) -> Result<()> {
        let key = (repo_key.to_string(), pattern.to_string());
        if self.branch_protections.write().remove(&key).is_none() {
            return Err(AuthError::NotFound(format!("branch protection for '{}'", pattern)));
        }
        Ok(())
    }

    // ==================== Webhooks ====================

    /// Create a webhook.
    pub fn create_webhook(&self, repo_key: String, url: String, events: std::collections::HashSet<crate::webhook::WebhookEvent>) -> Webhook {
        let id = self.next_id();
        let webhook = Webhook::new(id, repo_key.clone(), url, events);

        self.webhooks.write().insert(id, webhook.clone());

        // Update index
        self.webhook_index.write().entry(repo_key).or_default().push(id);

        webhook
    }

    /// Get a webhook by ID.
    pub fn get_webhook(&self, id: u64) -> Option<Webhook> {
        self.webhooks.read().get(&id).cloned()
    }

    /// List webhooks for a repository.
    pub fn list_webhooks(&self, repo_key: &str) -> Vec<Webhook> {
        self.webhooks
            .read()
            .values()
            .filter(|w| w.repo_key == repo_key)
            .cloned()
            .collect()
    }

    /// Update a webhook.
    pub fn update_webhook(&self, id: u64, update: impl FnOnce(&mut Webhook)) -> Result<Webhook> {
        let mut webhooks = self.webhooks.write();
        let webhook = webhooks.get_mut(&id).ok_or_else(|| AuthError::NotFound(format!("webhook {}", id)))?;

        update(webhook);
        webhook.updated_at = Self::now();

        Ok(webhook.clone())
    }

    /// Delete a webhook.
    pub fn delete_webhook(&self, id: u64) -> Result<()> {
        let webhook = self.webhooks.write().remove(&id).ok_or_else(|| AuthError::NotFound(format!("webhook {}", id)))?;

        // Update index
        if let Some(ids) = self.webhook_index.write().get_mut(&webhook.repo_key) {
            ids.retain(|&wid| wid != id);
        }

        Ok(())
    }

    /// Find webhooks that should fire for an event.
    pub fn find_webhooks_for_event(&self, repo_key: &str, event: crate::webhook::WebhookEvent) -> Vec<Webhook> {
        self.webhooks
            .read()
            .values()
            .filter(|w| w.repo_key == repo_key && w.should_fire(event))
            .cloned()
            .collect()
    }

    // ==================== Permission Resolution ====================

    /// Check if a user has at least the required permission on a repository.
    ///
    /// Permission resolution order:
    /// 1. Repository owner always has Admin
    /// 2. Direct collaborator permission
    /// 3. Team permission (highest wins)
    /// 4. Organization membership (if org owns repo)
    pub fn check_permission(&self, user: &str, repo_key: &str, required: Permission) -> bool {
        // Check if user is repo owner
        if let Some((owner, _)) = repo_key.split_once('/') {
            if owner == user {
                return true; // Owner has Admin access
            }

            // Check if owner is an org
            if let Some(org) = self.get_organization_by_name(owner) {
                // Org owners and admins have Admin access to all org repos
                if org.is_admin(user) {
                    return true;
                }
                // Org members have Read access by default
                if org.is_member(user) && required == Permission::Read {
                    return true;
                }
            }
        }

        // Check direct collaborator permission
        if let Some(collab) = self.get_collaborator(repo_key, user) {
            if collab.has_permission(required) {
                return true;
            }
        }

        // Check team permissions
        let user_teams = self.list_user_teams(user);
        for team in user_teams {
            if let Some(perm) = team.get_repo_permission(repo_key) {
                if perm.has(required) {
                    return true;
                }
            }
        }

        false
    }

    /// Get the effective permission for a user on a repository.
    pub fn get_effective_permission(&self, user: &str, repo_key: &str) -> Option<Permission> {
        let mut best: Option<Permission> = None;

        // Check if user is repo owner
        if let Some((owner, _)) = repo_key.split_once('/') {
            if owner == user {
                return Some(Permission::Admin);
            }

            // Check if owner is an org
            if let Some(org) = self.get_organization_by_name(owner) {
                if org.is_admin(user) {
                    return Some(Permission::Admin);
                }
                if org.is_member(user) {
                    best = Some(Permission::Read);
                }
            }
        }

        // Check direct collaborator permission
        if let Some(collab) = self.get_collaborator(repo_key, user) {
            if best.is_none() || collab.permission > best.unwrap() {
                best = Some(collab.permission);
            }
        }

        // Check team permissions
        let user_teams = self.list_user_teams(user);
        for team in user_teams {
            if let Some(perm) = team.get_repo_permission(repo_key) {
                if best.is_none() || perm > best.unwrap() {
                    best = Some(perm);
                }
            }
        }

        best
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_crud() {
        let store = AuthStore::new();

        // Create
        let org = store.create_organization("acme".into(), "Acme Corp".into(), "owner".into()).unwrap();
        assert_eq!(org.name, "acme");

        // Duplicate fails
        assert!(store.create_organization("acme".into(), "Another".into(), "other".into()).is_err());

        // Get by ID
        let org2 = store.get_organization(org.id).unwrap();
        assert_eq!(org2.name, "acme");

        // Get by name
        let org3 = store.get_organization_by_name("acme").unwrap();
        assert_eq!(org3.id, org.id);

        // Update
        let org4 = store.update_organization(org.id, Some("ACME Inc".into()), Some("Description".into())).unwrap();
        assert_eq!(org4.display_name, "ACME Inc");
        assert_eq!(org4.description, Some("Description".into()));

        // Delete
        store.delete_organization(org.id).unwrap();
        assert!(store.get_organization(org.id).is_none());
    }

    #[test]
    fn test_team_crud() {
        let store = AuthStore::new();
        let org = store.create_organization("acme".into(), "Acme Corp".into(), "owner".into()).unwrap();

        // Create team
        let team = store.create_team(org.id, "backend".into(), Permission::Write, "owner".into()).unwrap();
        assert_eq!(team.name, "backend");

        // List teams
        let teams = store.list_teams(org.id);
        assert_eq!(teams.len(), 1);

        // Add members
        store.add_team_member(team.id, "user1".into()).unwrap();
        store.add_team_member(team.id, "user2".into()).unwrap();

        let team = store.get_team(team.id).unwrap();
        assert!(team.is_member("user1"));
        assert!(team.is_member("user2"));

        // Add repos
        store.add_team_repo(team.id, "acme/api".into()).unwrap();
        let team = store.get_team(team.id).unwrap();
        assert!(team.has_repo("acme/api"));

        // Delete team
        store.delete_team(team.id).unwrap();
        assert!(store.get_team(team.id).is_none());
    }

    #[test]
    fn test_permission_resolution() {
        let store = AuthStore::new();

        // Owner always has admin
        assert!(store.check_permission("alice", "alice/repo", Permission::Admin));

        // Non-owner has no access by default
        assert!(!store.check_permission("bob", "alice/repo", Permission::Read));

        // Add collaborator
        store.set_collaborator("alice/repo".into(), "bob".into(), Permission::Write, "alice".into());
        assert!(store.check_permission("bob", "alice/repo", Permission::Read));
        assert!(store.check_permission("bob", "alice/repo", Permission::Write));
        assert!(!store.check_permission("bob", "alice/repo", Permission::Admin));

        // Create org with team
        let org = store.create_organization("acme".into(), "Acme".into(), "owner".into()).unwrap();
        let team = store.create_team(org.id, "devs".into(), Permission::Write, "owner".into()).unwrap();
        store.add_team_member(team.id, "charlie".into()).unwrap();
        store.add_team_repo(team.id, "acme/api".into()).unwrap();

        // Charlie has team access
        assert!(store.check_permission("charlie", "acme/api", Permission::Write));

        // Org admin has full access
        store.add_org_member(org.id, OrgMember::new("dave".into(), OrgRole::Admin, "owner".into())).unwrap();
        assert!(store.check_permission("dave", "acme/api", Permission::Admin));
    }

    #[test]
    fn test_branch_protection() {
        let store = AuthStore::new();

        let protection = store.set_branch_protection("alice/repo".into(), "main".into());
        assert!(protection.matches("main"));

        // Find matching protection
        let found = store.find_branch_protection("alice/repo", "main");
        assert!(found.is_some());

        let not_found = store.find_branch_protection("alice/repo", "develop");
        assert!(not_found.is_none());

        // Update protection
        let updated = store.update_branch_protection("alice/repo", "main", |p| {
            p.required_reviews = 2;
            p.require_code_owner_review = true;
        }).unwrap();
        assert_eq!(updated.required_reviews, 2);
        assert!(updated.require_code_owner_review);

        // Remove
        store.remove_branch_protection("alice/repo", "main").unwrap();
        assert!(store.find_branch_protection("alice/repo", "main").is_none());
    }

    #[test]
    fn test_webhooks() {
        use crate::webhook::WebhookEvent;
        use std::collections::HashSet;

        let store = AuthStore::new();

        let mut events = HashSet::new();
        events.insert(WebhookEvent::Push);
        events.insert(WebhookEvent::PullRequest);

        let webhook = store.create_webhook("alice/repo".into(), "https://example.com/hook".into(), events);
        assert_eq!(webhook.id, 1);

        // Find webhooks for event
        let push_hooks = store.find_webhooks_for_event("alice/repo", WebhookEvent::Push);
        assert_eq!(push_hooks.len(), 1);

        let issue_hooks = store.find_webhooks_for_event("alice/repo", WebhookEvent::Issue);
        assert_eq!(issue_hooks.len(), 0);

        // Update
        store.update_webhook(webhook.id, |w| w.disable()).unwrap();
        let disabled_hooks = store.find_webhooks_for_event("alice/repo", WebhookEvent::Push);
        assert_eq!(disabled_hooks.len(), 0);

        // Delete
        store.delete_webhook(webhook.id).unwrap();
        assert!(store.get_webhook(webhook.id).is_none());
    }
}
