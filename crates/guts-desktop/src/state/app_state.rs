//! # Application State
//!
//! Global state management using Dioxus signals and context.

use dioxus::prelude::*;

use crate::api::GutsClient;
use crate::auth::Credentials;
use crate::config::Config;

/// Global application state.
///
/// Shared across all components via Dioxus context.
/// Use `use_context::<AppState>()` to access in components.
///
/// # Examples
///
/// ```rust,ignore
/// #[component]
/// fn MyComponent() -> Element {
///     let state = use_context::<AppState>();
///     let url = state.node_url.read();
///
///     rsx! {
///         p { "Connected to: {url}" }
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct AppState {
    /// URL of the connected guts-node.
    pub node_url: Signal<String>,

    /// Whether we're currently connected to the node.
    pub connected: Signal<bool>,

    /// Last error message, if any.
    /// Reserved for critical errors (not used by Settings test connection).
    #[allow(dead_code)]
    pub last_error: Signal<Option<String>>,

    // ==================== Authentication State ====================
    /// Current user's username (if logged in).
    pub current_user: Signal<Option<String>>,

    /// Whether the user is authenticated.
    pub is_authenticated: Signal<bool>,

    /// Stored credentials (username, public key, token).
    pub credentials: Signal<Option<Credentials>>,
}

impl AppState {
    /// Creates a new application state, loading persisted config from disk.
    ///
    /// Falls back to default node URL `http://127.0.0.1:8080` if no config exists.
    /// If credentials exist in config, restores authentication state.
    #[must_use]
    pub fn new() -> Self {
        let config = Config::load();

        // Restore auth state from credentials if present
        let (current_user, is_authenticated, credentials) =
            if let Some(ref creds) = config.credentials {
                if creds.token.is_some() {
                    (Some(creds.username.clone()), true, Some(creds.clone()))
                } else {
                    (None, false, None)
                }
            } else {
                (None, false, None)
            };

        Self {
            node_url: Signal::new(config.node_url),
            connected: Signal::new(false),
            last_error: Signal::new(None),
            current_user: Signal::new(current_user),
            is_authenticated: Signal::new(is_authenticated),
            credentials: Signal::new(credentials),
        }
    }

    /// Saves the current configuration to disk (including credentials and accounts).
    pub fn save_config(&self) {
        // Load existing config to preserve accounts list
        let mut config = Config::load();
        config.node_url = self.node_url.read().clone();
        config.credentials = self.credentials.read().clone();
        // accounts are managed separately via save_account/remove_account
        if let Err(e) = config.save() {
            tracing::warn!("Failed to save config: {}", e);
        }
    }

    /// Creates a [`GutsClient`] from the current node URL.
    #[must_use]
    pub fn client(&self) -> GutsClient {
        GutsClient::new(self.node_url.read().clone())
    }

    /// Updates the node URL and resets connection state only if URL changed.
    pub fn set_node_url(&mut self, url: String) {
        // Only reset connection if URL actually changed
        if *self.node_url.read() != url {
            self.node_url.set(url);
            self.connected.set(false);
        }
    }

    /// Records an error message.
    /// Reserved for critical errors (not used by Settings test connection).
    #[allow(dead_code)]
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.last_error.set(Some(msg.into()));
    }

    /// Clears the last error.
    /// Reserved for critical errors (not used by Settings test connection).
    #[allow(dead_code)]
    pub fn clear_error(&mut self) {
        self.last_error.set(None);
    }

    // ==================== Authentication Methods ====================

    /// Log in with credentials.
    ///
    /// This sets the auth state and persists credentials to disk.
    pub fn login(&mut self, creds: Credentials) {
        let username = creds.username.clone();
        self.credentials.set(Some(creds));
        self.current_user.set(Some(username));
        self.is_authenticated.set(true);
        self.save_config();
    }

    /// Log out and clear all credentials.
    ///
    /// This clears auth state and removes credentials from disk.
    pub fn logout(&mut self) {
        self.credentials.set(None);
        self.current_user.set(None);
        self.is_authenticated.set(false);
        self.save_config();
    }

    /// Check if user is logged in.
    #[must_use]
    pub fn is_logged_in(&self) -> bool {
        *self.is_authenticated.read()
    }

    /// Get the current user's token (if authenticated).
    #[must_use]
    #[allow(dead_code)]
    pub fn token(&self) -> Option<String> {
        self.credentials
            .read()
            .as_ref()
            .and_then(|c| c.token.clone())
    }

    // ==================== Account Management ====================

    /// Save credentials to the accounts list.
    ///
    /// Updates existing account if username matches, otherwise adds new.
    /// Also sets as current credentials.
    pub fn save_account(&mut self, creds: &Credentials) {
        let mut config = Config::load();

        // Update or add to accounts list
        if let Some(existing) = config
            .accounts
            .iter_mut()
            .find(|a| a.username == creds.username)
        {
            *existing = creds.clone();
        } else {
            config.accounts.push(creds.clone());
        }

        // Also set as current credentials
        config.credentials = Some(creds.clone());

        if let Err(e) = config.save() {
            tracing::warn!("Failed to save account: {}", e);
        }
    }

    /// Switch to a saved account by username.
    ///
    /// Returns true if account was found and switched, false otherwise.
    pub fn switch_account(&mut self, username: &str) -> bool {
        let config = Config::load();
        if let Some(account) = config.accounts.iter().find(|a| a.username == username) {
            self.login(account.clone());
            true
        } else {
            false
        }
    }

    /// Get all saved accounts.
    #[must_use]
    pub fn saved_accounts(&self) -> Vec<Credentials> {
        Config::load().accounts
    }

    /// Remove an account from the saved list.
    #[allow(dead_code)]
    pub fn remove_account(&mut self, username: &str) {
        let mut config = Config::load();
        config.accounts.retain(|a| a.username != username);

        // If removing current user, also clear credentials
        if self
            .current_user
            .read()
            .as_ref()
            .is_some_and(|u| u == username)
        {
            config.credentials = None;
            self.logout();
        }

        if let Err(e) = config.save() {
            tracing::warn!("Failed to remove account: {}", e);
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
