//! CLI command implementations.

use commonware_cryptography::{ed25519::PrivateKey, PrivateKeyExt, Signer};
use std::path::Path;
use thiserror::Error;

/// CLI errors.
#[derive(Debug, Error)]
pub enum CliError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, CliError>;

/// Initialize a new repository.
pub fn init(name: &str, path: Option<&str>) -> Result<()> {
    let base_path = path.unwrap_or(".");
    let repo_path = Path::new(base_path).join(name);

    tracing::info!(name = %name, path = %repo_path.display(), "Initializing repository");

    std::fs::create_dir_all(&repo_path)?;

    // Create .guts directory
    let guts_dir = repo_path.join(".guts");
    std::fs::create_dir_all(&guts_dir)?;

    println!(
        "Initialized empty Guts repository in {}",
        repo_path.display()
    );

    Ok(())
}

/// Clone a repository.
pub fn clone(url: &str, _path: Option<&str>) -> Result<()> {
    tracing::info!(url = %url, "Cloning repository");

    // TODO: Implement actual clone logic using P2P
    Err(CliError::NotImplemented("clone".to_string()))
}

/// Generate a new identity.
pub fn identity_generate(output: Option<&str>) -> Result<()> {
    use rand::rngs::OsRng;

    let private_key = PrivateKey::from_rng(&mut OsRng);
    let public_key = private_key.public_key();

    println!("Generated new identity:");
    println!(
        "  Public Key: {}",
        commonware_utils::hex(public_key.as_ref())
    );

    if let Some(output_path) = output {
        let secret_hex = commonware_utils::hex(private_key.as_ref());
        std::fs::write(output_path, &secret_hex)?;
        println!("\nSecret key saved to: {output_path}");
        println!("WARNING: Keep this file secure and never share it!");
    }

    Ok(())
}

/// Show current identity.
pub fn identity_show() -> Result<()> {
    println!("No identity configured. Use 'guts identity generate' to create one.");
    Ok(())
}

/// Show status.
pub fn status() -> Result<()> {
    println!("Guts Status");
    println!("===========");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Node:    Not connected");
    println!("Peers:   0");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generate() {
        // Just test that it doesn't panic
        identity_generate(None).unwrap();
    }
}
