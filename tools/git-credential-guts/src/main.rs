//! Git Credential Helper for Guts
//!
//! This credential helper integrates Guts authentication with Git, allowing
//! seamless clone, push, and pull operations using Guts personal access tokens.
//!
//! ## Installation
//!
//! ```bash
//! cargo install --path tools/git-credential-guts
//! git config --global credential.helper guts
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Store a token
//! git-credential-guts store
//! # Input: protocol=https
//! # Input: host=guts.network
//! # Input: username=token
//! # Input: password=guts_xxx
//!
//! # Or configure directly
//! git-credential-guts configure --token guts_xxx
//! ```

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Git credential helper for Guts authentication.
#[derive(Parser, Debug)]
#[command(name = "git-credential-guts")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get credentials for a Guts host
    Get,

    /// Store credentials for a Guts host
    Store,

    /// Erase credentials for a Guts host
    Erase,

    /// Configure Guts credentials interactively
    Configure {
        /// Guts personal access token
        #[arg(short, long)]
        token: Option<String>,

        /// Guts host (default: guts.network)
        #[arg(long, default_value = "guts.network")]
        host: String,
    },

    /// List configured Guts hosts
    List,

    /// Remove configuration for a host
    Remove {
        /// Host to remove
        host: String,
    },
}

/// Configuration for Guts credential helper.
#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    /// Map of host to token
    hosts: HashMap<String, HostConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HostConfig {
    /// Username (usually "token" for token auth)
    username: String,
    /// Whether to use keyring for secure storage
    use_keyring: bool,
}

const SERVICE_NAME: &str = "git-credential-guts";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Get => handle_get()?,
        Commands::Store => handle_store()?,
        Commands::Erase => handle_erase()?,
        Commands::Configure { token, host } => handle_configure(token, &host)?,
        Commands::List => handle_list()?,
        Commands::Remove { host } => handle_remove(&host)?,
    }

    Ok(())
}

/// Handle the "get" command - retrieve credentials.
fn handle_get() -> anyhow::Result<()> {
    let input = read_credential_input()?;

    let host = input.get("host").cloned().unwrap_or_default();
    let protocol = input.get("protocol").cloned().unwrap_or_default();

    // Only handle Guts hosts
    if !is_guts_host(&host) {
        return Ok(());
    }

    // Try to get token from keyring
    if let Some(token) = get_token_from_keyring(&host)? {
        println!("protocol={protocol}");
        println!("host={host}");
        println!("username=token");
        println!("password={token}");
        return Ok(());
    }

    // Try to get token from config file
    let config = load_config()?;
    if let Some(host_config) = config.hosts.get(&host) {
        if let Some(token) = get_token_from_keyring(&host)? {
            println!("protocol={protocol}");
            println!("host={host}");
            println!("username={}", host_config.username);
            println!("password={token}");
        }
    }

    Ok(())
}

/// Handle the "store" command - store credentials.
fn handle_store() -> anyhow::Result<()> {
    let input = read_credential_input()?;

    let host = input.get("host").cloned().unwrap_or_default();
    let password = input.get("password").cloned().unwrap_or_default();
    let username = input.get("username").cloned().unwrap_or_else(|| "token".to_string());

    // Only handle Guts hosts
    if !is_guts_host(&host) {
        return Ok(());
    }

    if password.is_empty() {
        return Ok(());
    }

    // Store token in keyring
    store_token_in_keyring(&host, &password)?;

    // Update config
    let mut config = load_config()?;
    config.hosts.insert(
        host,
        HostConfig {
            username,
            use_keyring: true,
        },
    );
    save_config(&config)?;

    Ok(())
}

/// Handle the "erase" command - remove credentials.
fn handle_erase() -> anyhow::Result<()> {
    let input = read_credential_input()?;

    let host = input.get("host").cloned().unwrap_or_default();

    // Only handle Guts hosts
    if !is_guts_host(&host) {
        return Ok(());
    }

    // Remove from keyring
    delete_token_from_keyring(&host)?;

    // Update config
    let mut config = load_config()?;
    config.hosts.remove(&host);
    save_config(&config)?;

    Ok(())
}

/// Handle the "configure" command - interactive setup.
fn handle_configure(token: Option<String>, host: &str) -> anyhow::Result<()> {
    let token = match token {
        Some(t) => t,
        None => {
            print!("Enter your Guts personal access token: ");
            io::stdout().flush()?;
            let mut token = String::new();
            io::stdin().read_line(&mut token)?;
            token.trim().to_string()
        }
    };

    if token.is_empty() {
        eprintln!("Error: Token cannot be empty");
        std::process::exit(1);
    }

    // Validate token format
    if !token.starts_with("guts_") {
        eprintln!("Warning: Token doesn't start with 'guts_'. Are you sure this is correct?");
    }

    // Store token
    store_token_in_keyring(host, &token)?;

    // Update config
    let mut config = load_config()?;
    config.hosts.insert(
        host.to_string(),
        HostConfig {
            username: "token".to_string(),
            use_keyring: true,
        },
    );
    save_config(&config)?;

    println!("Credentials stored for {host}");
    println!();
    println!("Git is now configured to use Guts authentication.");
    println!("You can clone repositories using:");
    println!("  git clone https://{host}/owner/repo.git");

    Ok(())
}

/// Handle the "list" command - show configured hosts.
fn handle_list() -> anyhow::Result<()> {
    let config = load_config()?;

    if config.hosts.is_empty() {
        println!("No Guts hosts configured.");
        println!();
        println!("Run 'git-credential-guts configure' to set up authentication.");
        return Ok(());
    }

    println!("Configured Guts hosts:");
    for (host, host_config) in &config.hosts {
        let storage = if host_config.use_keyring {
            "keyring"
        } else {
            "config"
        };
        println!("  {host} (username: {}, storage: {storage})", host_config.username);
    }

    Ok(())
}

/// Handle the "remove" command - remove a host configuration.
fn handle_remove(host: &str) -> anyhow::Result<()> {
    // Remove from keyring
    delete_token_from_keyring(host)?;

    // Update config
    let mut config = load_config()?;
    if config.hosts.remove(host).is_some() {
        save_config(&config)?;
        println!("Removed configuration for {host}");
    } else {
        println!("No configuration found for {host}");
    }

    Ok(())
}

/// Read credential input from stdin (Git credential helper protocol).
fn read_credential_input() -> anyhow::Result<HashMap<String, String>> {
    let mut input = HashMap::new();

    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }

        if let Some((key, value)) = line.split_once('=') {
            input.insert(key.to_string(), value.to_string());
        }
    }

    Ok(input)
}

/// Check if a host is a Guts host.
fn is_guts_host(host: &str) -> bool {
    host.ends_with(".guts.network")
        || host == "guts.network"
        || host == "localhost"
        || host.starts_with("localhost:")
        || host.starts_with("127.0.0.1")
}

/// Get config file path.
fn config_path() -> anyhow::Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    Ok(config_dir.join("guts").join("credentials.toml"))
}

/// Load configuration from file.
fn load_config() -> anyhow::Result<Config> {
    let path = config_path()?;

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

/// Save configuration to file.
fn save_config(config: &Config) -> anyhow::Result<()> {
    let path = config_path()?;

    // Create parent directories
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(config)?;
    std::fs::write(&path, contents)?;

    Ok(())
}

/// Get token from system keyring.
fn get_token_from_keyring(host: &str) -> anyhow::Result<Option<String>> {
    let entry = keyring::Entry::new(SERVICE_NAME, host)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(keyring::Error::NoStorageAccess(_)) => {
            // Keyring not available, fall back to config
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Store token in system keyring.
fn store_token_in_keyring(host: &str, token: &str) -> anyhow::Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, host)?;
    entry.set_password(token)?;
    Ok(())
}

/// Delete token from system keyring.
fn delete_token_from_keyring(host: &str) -> anyhow::Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, host)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_guts_host() {
        assert!(is_guts_host("guts.network"));
        assert!(is_guts_host("api.guts.network"));
        assert!(is_guts_host("localhost"));
        assert!(is_guts_host("localhost:8080"));
        assert!(is_guts_host("127.0.0.1:8080"));

        assert!(!is_guts_host("github.com"));
        assert!(!is_guts_host("gitlab.com"));
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.hosts.insert(
            "guts.network".to_string(),
            HostConfig {
                username: "token".to_string(),
                use_keyring: true,
            },
        );

        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();

        assert!(parsed.hosts.contains_key("guts.network"));
    }
}
