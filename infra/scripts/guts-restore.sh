#!/usr/bin/env bash
#
# Guts Node Restore Script
# Restores Guts node data from a backup
#
# Usage:
#   ./guts-restore.sh [options] <backup-file>
#
# Options:
#   --target-dir DIR    Target directory for restore (default: /var/lib/guts)
#   --verify            Verify backup before restoring
#   --force             Overwrite existing data without confirmation
#   --dry-run           Show what would be done without making changes
#   --s3-bucket BUCKET  Download backup from S3
#   --help              Show this help message

set -euo pipefail

# Default configuration
TARGET_DIR="${GUTS_DATA_DIR:-/var/lib/guts}"
VERIFY=false
FORCE=false
DRY_RUN=false
S3_BUCKET=""
BACKUP_FILE=""

# Logging
log() {
    echo "[$(date -Iseconds)] $*"
}

log_error() {
    echo "[$(date -Iseconds)] ERROR: $*" >&2
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --target-dir)
            TARGET_DIR="$2"
            shift 2
            ;;
        --verify)
            VERIFY=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --s3-bucket)
            S3_BUCKET="$2"
            shift 2
            ;;
        --help)
            head -20 "$0" | tail -16
            exit 0
            ;;
        -*)
            log_error "Unknown option: $1"
            exit 1
            ;;
        *)
            BACKUP_FILE="$1"
            shift
            ;;
    esac
done

# Validate backup file argument
if [[ -z "$BACKUP_FILE" ]]; then
    log_error "Backup file not specified"
    echo "Usage: $0 [options] <backup-file>"
    exit 1
fi

# Download from S3 if specified
if [[ -n "$S3_BUCKET" ]]; then
    log "Downloading backup from S3: s3://${S3_BUCKET}/${BACKUP_FILE}"
    TEMP_BACKUP="/tmp/${BACKUP_FILE}"
    if ! aws s3 cp "s3://${S3_BUCKET}/${BACKUP_FILE}" "$TEMP_BACKUP"; then
        log_error "Failed to download backup from S3"
        exit 1
    fi
    BACKUP_FILE="$TEMP_BACKUP"
fi

# Validate backup file exists
if [[ ! -f "$BACKUP_FILE" ]]; then
    log_error "Backup file does not exist: $BACKUP_FILE"
    exit 1
fi

log "Guts restore starting..."
log "Backup file: $BACKUP_FILE"
log "Target directory: $TARGET_DIR"

# Verify backup checksum if available
CHECKSUM_FILE="${BACKUP_FILE}.sha256"
if [[ -f "$CHECKSUM_FILE" ]]; then
    log "Verifying backup checksum..."
    EXPECTED=$(cat "$CHECKSUM_FILE" | cut -d' ' -f1)
    ACTUAL=$(sha256sum "$BACKUP_FILE" | cut -d' ' -f1)
    if [[ "$EXPECTED" != "$ACTUAL" ]]; then
        log_error "Checksum verification failed!"
        log_error "  Expected: $EXPECTED"
        log_error "  Actual:   $ACTUAL"
        exit 1
    fi
    log "Checksum verified: $EXPECTED"
fi

# Verify backup integrity
if [[ "$VERIFY" == "true" ]]; then
    log "Verifying backup integrity..."
    if ! tar -tzf "$BACKUP_FILE" > /dev/null 2>&1; then
        log_error "Backup file is corrupted or invalid"
        exit 1
    fi
    log "Backup integrity verified"
fi

# Show backup contents in dry-run mode
if [[ "$DRY_RUN" == "true" ]]; then
    log "DRY RUN - Would restore the following:"
    tar -tzf "$BACKUP_FILE" | head -50
    echo "..."
    log "DRY RUN complete (no changes made)"
    exit 0
fi

# Check if target directory has data
if [[ -d "$TARGET_DIR" && "$(ls -A "$TARGET_DIR" 2>/dev/null)" ]]; then
    if [[ "$FORCE" != "true" ]]; then
        log "Target directory is not empty: $TARGET_DIR"
        read -p "Overwrite existing data? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log "Restore cancelled"
            exit 1
        fi
    fi
    log "Removing existing data..."
    rm -rf "${TARGET_DIR:?}"/*
fi

# Create target directory
mkdir -p "$TARGET_DIR"

# Check if guts-node is running
if systemctl is-active --quiet guts-node 2>/dev/null; then
    log "WARNING: guts-node service is running"
    if [[ "$FORCE" != "true" ]]; then
        read -p "Stop the service before restoring? [Y/n] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            log "Stopping guts-node service..."
            sudo systemctl stop guts-node
        fi
    fi
fi

# Restore backup
log "Extracting backup..."
if ! tar -xzf "$BACKUP_FILE" -C "$TARGET_DIR"; then
    log_error "Failed to extract backup"
    exit 1
fi

# Set permissions
if id -u guts >/dev/null 2>&1; then
    log "Setting ownership to guts:guts..."
    chown -R guts:guts "$TARGET_DIR"
fi

# Verify restoration
RESTORED_FILES=$(find "$TARGET_DIR" -type f | wc -l)
log "Restored $RESTORED_FILES files"

# Summary
log "Restore completed successfully"
log "  Target: $TARGET_DIR"
log "  Files: $RESTORED_FILES"

# Suggest next steps
echo ""
echo "Next steps:"
echo "  1. Review the restored data: ls -la $TARGET_DIR"
echo "  2. Start the node: sudo systemctl start guts-node"
echo "  3. Verify node health: curl http://localhost:8080/health/ready"
