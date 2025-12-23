//! Operator commands for Guts node administration.
//!
//! This module provides commands for:
//! - Key generation and management
//! - Backup and restore operations
//! - Diagnostics collection
//! - Storage maintenance

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Result of a backup operation
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub created_at: u64,
    pub node_version: String,
    pub data_dir: PathBuf,
    pub output_path: PathBuf,
    pub size_bytes: u64,
    pub checksum: String,
}

/// Result of diagnostics collection
#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsInfo {
    pub collected_at: u64,
    pub node_version: String,
    pub output_path: PathBuf,
    pub components: Vec<String>,
}

/// Generate a new Ed25519 keypair for node identity.
///
/// Returns the private and public keys as hex-encoded strings.
pub fn keygen() -> Result<(String, String)> {
    use commonware_cryptography::{PrivateKeyExt, Signer};

    // Generate a random seed
    let seed: u64 = rand::random();
    let private_key = commonware_cryptography::ed25519::PrivateKey::from_seed(seed);
    let public_key = private_key.public_key();

    let private_hex = hex::encode(private_key.as_ref());
    let public_hex = hex::encode(public_key.as_ref());

    Ok((private_hex, public_hex))
}

/// Generate a keypair and write to file.
pub fn keygen_to_file(output_path: &Path) -> Result<String> {
    let (private_key, public_key) = keygen()?;

    let mut file = File::create(output_path).context("Failed to create key file")?;

    // Write private key on first line, public key on second
    writeln!(file, "{}", private_key)?;
    writeln!(file, "{}", public_key)?;

    Ok(public_key)
}

/// Create a backup of the node's data directory.
///
/// This creates a compressed tarball of the data directory.
pub fn create_backup(data_dir: &Path, output_path: &Path) -> Result<BackupInfo> {
    use sha2::{Digest, Sha256};

    if !data_dir.exists() {
        anyhow::bail!("Data directory does not exist: {}", data_dir.display());
    }

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create the tarball
    let tar_file = File::create(output_path).context("Failed to create backup file")?;
    let encoder = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    // Add the data directory to the tarball
    tar.append_dir_all(".", data_dir)
        .context("Failed to add data directory to backup")?;

    tar.finish().context("Failed to finalize backup")?;
    drop(tar);

    // Calculate checksum
    let mut file = File::open(output_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let checksum = hex::encode(hasher.finalize());
    let metadata = fs::metadata(output_path)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(BackupInfo {
        created_at: timestamp,
        node_version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: data_dir.to_path_buf(),
        output_path: output_path.to_path_buf(),
        size_bytes: metadata.len(),
        checksum,
    })
}

/// Verify backup integrity.
pub fn verify_backup(backup_path: &Path) -> Result<bool> {
    if !backup_path.exists() {
        anyhow::bail!("Backup file does not exist: {}", backup_path.display());
    }

    // Try to open and read the archive
    let file = File::open(backup_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    // Verify all entries can be read
    let mut entry_count = 0;
    for entry in archive.entries()? {
        let entry = entry.context("Failed to read archive entry")?;
        let _path = entry.path()?;
        entry_count += 1;
    }

    tracing::info!(entries = entry_count, "Backup verification complete");

    Ok(true)
}

/// Restore data from a backup.
pub fn restore_backup(backup_path: &Path, target_dir: &Path, verify: bool) -> Result<()> {
    if !backup_path.exists() {
        anyhow::bail!("Backup file does not exist: {}", backup_path.display());
    }

    // Verify backup first if requested
    if verify {
        verify_backup(backup_path)?;
    }

    // Create target directory
    fs::create_dir_all(target_dir)?;

    // Extract the archive
    let file = File::open(backup_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(target_dir)
        .context("Failed to extract backup")?;

    tracing::info!(
        backup = %backup_path.display(),
        target = %target_dir.display(),
        "Backup restored successfully"
    );

    Ok(())
}

/// Collect node diagnostics for troubleshooting.
pub fn collect_diagnostics(
    data_dir: &Path,
    output_path: &Path,
    include_logs: bool,
    include_metrics: bool,
) -> Result<DiagnosticsInfo> {
    use std::process::Command;

    // Create temp directory for diagnostics
    let temp_dir = tempfile::tempdir()?;
    let diag_dir = temp_dir.path();

    let mut components = Vec::new();

    // Collect system information
    let sys_info_path = diag_dir.join("system-info.txt");
    let mut sys_file = File::create(&sys_info_path)?;

    writeln!(sys_file, "=== Guts Node Diagnostics ===")?;
    writeln!(sys_file, "Timestamp: {:?}", SystemTime::now())?;
    writeln!(sys_file, "Node Version: {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(sys_file, "Data Directory: {}", data_dir.display())?;
    writeln!(sys_file)?;

    // System info
    writeln!(sys_file, "=== System Information ===")?;
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("uname").arg("-a").output() {
            writeln!(sys_file, "OS: {}", String::from_utf8_lossy(&output.stdout))?;
        }
        if let Ok(output) = Command::new("free").arg("-h").output() {
            writeln!(
                sys_file,
                "Memory:\n{}",
                String::from_utf8_lossy(&output.stdout)
            )?;
        }
        if let Ok(output) = Command::new("df").arg("-h").output() {
            writeln!(
                sys_file,
                "Disk:\n{}",
                String::from_utf8_lossy(&output.stdout)
            )?;
        }
    }
    components.push("system-info".to_string());

    // Collect configuration (if exists)
    let config_paths = ["config.yaml", "config.yml", "config.json"];
    for config_name in &config_paths {
        let config_path = data_dir.parent().unwrap_or(data_dir).join(config_name);
        if config_path.exists() {
            let dest = diag_dir.join(config_name);
            fs::copy(&config_path, &dest)?;
            components.push(format!("config:{}", config_name));
        }
    }

    // Collect data directory info
    let data_info_path = diag_dir.join("data-dir-info.txt");
    let mut data_file = File::create(&data_info_path)?;
    writeln!(data_file, "=== Data Directory Structure ===")?;

    fn list_dir(dir: &Path, prefix: &str, file: &mut File, depth: usize) -> Result<()> {
        if depth > 3 {
            return Ok(());
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();

                if path.is_dir() {
                    writeln!(file, "{}{}/", prefix, name)?;
                    list_dir(&path, &format!("{}  ", prefix), file, depth + 1)?;
                } else {
                    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    writeln!(file, "{}{} ({} bytes)", prefix, name, size)?;
                }
            }
        }
        Ok(())
    }

    if data_dir.exists() {
        list_dir(data_dir, "", &mut data_file, 0)?;
    }
    components.push("data-dir-info".to_string());

    // Collect logs if requested
    if include_logs {
        // Try to get recent journal logs
        #[cfg(unix)]
        {
            if let Ok(output) = Command::new("journalctl")
                .args(["-u", "guts-node", "--since", "1 hour ago", "-n", "1000"])
                .output()
            {
                let log_path = diag_dir.join("journal.log");
                fs::write(&log_path, &output.stdout)?;
                components.push("journal-logs".to_string());
            }
        }
    }

    // Collect metrics if requested
    if include_metrics {
        // Try to fetch metrics from the local endpoint
        let metrics_content = "# Metrics collection placeholder\n# Connect to http://localhost:9090/metrics to fetch live metrics\n";
        let metrics_path = diag_dir.join("metrics.txt");
        fs::write(&metrics_path, metrics_content)?;
        components.push("metrics".to_string());
    }

    // Create the output tarball
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tar_file = File::create(output_path)?;
    let encoder = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    tar.append_dir_all("diagnostics", diag_dir)?;
    tar.finish()?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(DiagnosticsInfo {
        collected_at: timestamp,
        node_version: env!("CARGO_PKG_VERSION").to_string(),
        output_path: output_path.to_path_buf(),
        components,
    })
}

/// Node status information
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeStatus {
    pub version: String,
    pub uptime_secs: u64,
    pub data_dir: PathBuf,
    pub storage: StorageStatus,
    pub consensus: Option<ConsensusStatus>,
    pub p2p: Option<P2pStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageStatus {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusStatus {
    pub enabled: bool,
    pub block_height: u64,
    pub synced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2pStatus {
    pub peer_count: usize,
    pub listening_addr: String,
}

/// Get current node status (offline mode - reads from disk)
pub fn get_status(data_dir: &Path) -> Result<NodeStatus> {
    // Check storage usage
    let storage = if data_dir.exists() {
        let dir_size = calculate_dir_size(data_dir)?;
        StorageStatus {
            total_bytes: dir_size,
            available_bytes: 0, // Would need statvfs for accurate filesystem info
            usage_percent: 0.0,
        }
    } else {
        StorageStatus {
            total_bytes: 0,
            available_bytes: 0,
            usage_percent: 0.0,
        }
    };

    Ok(NodeStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0, // Can't determine in offline mode
        data_dir: data_dir.to_path_buf(),
        storage,
        consensus: None,
        p2p: None,
    })
}

/// Calculate total size of a directory recursively
fn calculate_dir_size(dir: &Path) -> Result<u64> {
    let mut total = 0u64;

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                total += calculate_dir_size(&path)?;
            } else {
                total += fs::metadata(&path)?.len();
            }
        }
    }

    Ok(total)
}

/// Verify data integrity
pub fn verify_data(data_dir: &Path, full: bool) -> Result<VerifyResult> {
    if !data_dir.exists() {
        anyhow::bail!("Data directory does not exist: {}", data_dir.display());
    }

    let errors: Vec<String> = Vec::new();
    let mut warnings = Vec::new();
    let mut objects_checked = 0u64;

    // Check for required subdirectories
    let required_dirs = ["objects", "refs"];
    for dir_name in &required_dirs {
        let dir_path = data_dir.join(dir_name);
        if !dir_path.exists() {
            warnings.push(format!("Missing directory: {}", dir_name));
        }
    }

    // If full verification, check all objects
    if full {
        let objects_dir = data_dir.join("objects");
        if objects_dir.exists() {
            for entry in walkdir::WalkDir::new(&objects_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    objects_checked += 1;
                    // In a real implementation, would verify object checksums
                }
            }
        }
    }

    Ok(VerifyResult {
        valid: errors.is_empty(),
        objects_checked,
        errors,
        warnings,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResult {
    pub valid: bool,
    pub objects_checked: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_keygen() {
        let (private, public) = keygen().unwrap();

        // Keys should be hex-encoded
        assert!(!private.is_empty());
        assert!(!public.is_empty());

        // Private key should be different from public
        assert_ne!(private, public);
    }

    #[test]
    fn test_keygen_to_file() {
        let temp = tempdir().unwrap();
        let key_path = temp.path().join("node.key");

        let public_key = keygen_to_file(&key_path).unwrap();

        assert!(key_path.exists());
        assert!(!public_key.is_empty());

        // Read and verify file contents
        let contents = fs::read_to_string(&key_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(!lines[0].is_empty()); // Private key
        assert_eq!(lines[1], public_key); // Public key
    }
}
