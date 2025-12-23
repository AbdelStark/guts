//! # Login View
//!
//! User authentication view for registration and login.

use dioxus::prelude::*;

use crate::api::ApiError;
use crate::auth::{Credentials, Identity};
use crate::router::Route;
use crate::state::AppState;

/// Login/Register view component.
///
/// Allows users to create a new account or login with existing credentials.
/// Shows saved accounts for quick switching, or create a new account.
#[component]
pub fn Login() -> Element {
    let mut state = use_context::<AppState>();
    let nav = use_navigator();

    let mut username = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);

    // Load saved accounts
    let saved_accounts = state.saved_accounts();

    // If already authenticated, redirect to home
    if state.is_logged_in() {
        nav.push(Route::Home {});
    }

    // Connect to existing account
    let mut do_connect = move |account_username: String| {
        if state.switch_account(&account_username) {
            nav.push(Route::Home {});
        } else {
            error.set(Some("Failed to switch account".to_string()));
        }
    };

    let mut do_register = move || {
        // Auto-lowercase the username
        let username_val = username.read().trim().to_lowercase();

        // Validate username
        if username_val.is_empty() {
            error.set(Some("Username is required".to_string()));
            return;
        }

        if username_val.len() < 3 {
            error.set(Some("Username must be at least 3 characters".to_string()));
            return;
        }

        if username_val.len() > 39 {
            error.set(Some("Username must be 39 characters or less".to_string()));
            return;
        }

        // Check for valid characters (alphanumeric and hyphens)
        if !username_val
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            error.set(Some(
                "Username can only contain lowercase letters, numbers, and hyphens".to_string(),
            ));
            return;
        }

        // Check for consecutive hyphens
        if username_val.contains("--") {
            error.set(Some(
                "Username cannot contain consecutive hyphens".to_string(),
            ));
            return;
        }

        // Check start/end with hyphen
        if username_val.starts_with('-') || username_val.ends_with('-') {
            error.set(Some(
                "Username cannot start or end with a hyphen".to_string(),
            ));
            return;
        }

        loading.set(true);
        error.set(None);

        let client = state.client();

        spawn(async move {
            // 1. Generate Ed25519 identity
            let identity = Identity::generate();
            let public_key = identity.public_key_hex();

            // 2. Register user with the node
            match client.register_user(&username_val, &public_key).await {
                Ok(user_profile) => {
                    // 3. Create personal access token (use snake_case scopes!)
                    match client
                        .create_token_with_identity(
                            "Desktop App",
                            &["repo_read", "repo_write", "user_read"],
                            &username_val,
                        )
                        .await
                    {
                        Ok(token_resp) => {
                            // 4. Build credentials and login
                            let mut creds = Credentials::new(username_val, &identity);
                            creds.token = token_resp.token;
                            creds.user_id = Some(user_profile.id);

                            // Save to accounts list and set as current
                            state.save_account(&creds);
                            state.login(creds);

                            // Navigate to home
                            nav.push(Route::Home {});
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to create token: {}", e)));
                        }
                    }
                }
                Err(ApiError::NodeError { status: 409, .. }) => {
                    error.set(Some("Username already taken".to_string()));
                }
                Err(e) => {
                    error.set(Some(format!("Registration failed: {}", e)));
                }
            }

            loading.set(false);
        });
    };

    rsx! {
        div { class: "login-view",
            div { class: "login-card glass-panel",
                div { class: "login-header",
                    h1 { "Guts" }
                    p { class: "text-secondary", "Decentralized Code Collaboration" }
                }

                // Saved accounts section
                if !saved_accounts.is_empty() {
                    div { class: "saved-accounts",
                        h3 { class: "section-label", "Saved Accounts" }

                        for account in saved_accounts.iter() {
                            div { class: "account-item",
                                div { class: "account-info",
                                    div { class: "account-avatar", "{account.username.chars().next().unwrap_or('?').to_ascii_uppercase()}" }
                                    span { class: "account-username", "{account.username}" }
                                }
                                button {
                                    class: "btn-primary btn-sm",
                                    onclick: {
                                        let username = account.username.clone();
                                        move |_| do_connect(username.clone())
                                    },
                                    "Connect"
                                }
                            }
                        }
                    }

                    div { class: "divider",
                        span { "or create new" }
                    }
                }

                div { class: "login-form",
                    div { class: "form-group",
                        label { r#for: "username", "Username" }
                        input {
                            id: "username",
                            r#type: "text",
                            placeholder: "Enter your username",
                            value: "{username}",
                            disabled: *loading.read(),
                            oninput: move |evt| username.set(evt.value().clone()),
                            onkeypress: move |evt| {
                                if evt.key() == Key::Enter && !*loading.read() {
                                    do_register();
                                }
                            },
                        }
                    }

                    if let Some(err) = error.read().as_ref() {
                        div { class: "alert alert-error", "{err}" }
                    }

                    button {
                        class: "btn-primary btn-lg btn-block",
                        disabled: *loading.read(),
                        onclick: move |_| do_register(),
                        if *loading.read() {
                            "Creating Account..."
                        } else {
                            "Create Account"
                        }
                    }

                    div { class: "login-hints",
                        p { class: "login-hint text-secondary",
                            "Username: 3-39 chars, lowercase letters, numbers, hyphens"
                        }
                        p { class: "login-hint text-tertiary",
                            "A new Ed25519 identity will be generated for you."
                        }
                    }
                }
            }
        }
    }
}
