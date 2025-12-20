//! CLI command implementations.

use anyhow::{anyhow, Result};
use guts_identity::Keypair;
use std::path::Path;

/// Initialize a new repository.
pub fn init(name: &str, path: Option<&str>) -> Result<()> {
    let path = path.unwrap_or(".");
    let repo_path = Path::new(path).join(name);

    tracing::info!(name = %name, path = %repo_path.display(), "Initializing repository");

    std::fs::create_dir_all(&repo_path)?;

    // Create .guts directory
    let guts_dir = repo_path.join(".guts");
    std::fs::create_dir_all(&guts_dir)?;

    println!("Initialized empty Guts repository in {}", repo_path.display());

    Ok(())
}

/// Clone a repository.
pub async fn clone(url: &str, path: Option<&str>) -> Result<()> {
    let dest = path.unwrap_or_else(|| {
        url.rsplit('/').next().unwrap_or("repo")
    });

    tracing::info!(url = %url, dest = %dest, "Cloning repository");

    // TODO: Implement actual clone logic
    println!("Cloning {} into {}", url, dest);

    Err(anyhow!("Clone not yet implemented"))
}

/// Generate a new identity.
pub fn identity_generate(output: Option<&str>) -> Result<()> {
    let keypair = Keypair::generate();
    let public_key = keypair.public_key();

    println!("Generated new identity:");
    println!("  Public Key: {}", public_key);
    println!("  Short ID:   {}", public_key.short_id());

    if let Some(output_path) = output {
        let secret_bytes = keypair.secret_bytes();
        let hex_secret = hex::encode(&*secret_bytes);

        std::fs::write(output_path, &hex_secret)?;
        println!("\nSecret key saved to: {}", output_path);
        println!("WARNING: Keep this file secure and never share it!");
    }

    Ok(())
}

/// Show current identity.
pub fn identity_show() -> Result<()> {
    // TODO: Load identity from config
    println!("No identity configured. Use 'guts identity generate' to create one.");
    Ok(())
}

/// Export identity.
pub fn identity_export(output: &str) -> Result<()> {
    // TODO: Implement identity export
    println!("Exporting identity to: {}", output);
    Err(anyhow!("Export not yet implemented"))
}

/// Show node status.
pub async fn status() -> Result<()> {
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
    use tempfile::TempDir;

    #[test]
    fn test_init() {
        let temp = TempDir::new().unwrap();
        init("test-repo", Some(temp.path().to_str().unwrap())).unwrap();

        let repo_path = temp.path().join("test-repo");
        assert!(repo_path.exists());
        assert!(repo_path.join(".guts").exists());
    }

    #[test]
    fn test_identity_generate() {
        // Just test that it doesn't panic
        identity_generate(None).unwrap();
    }
}
