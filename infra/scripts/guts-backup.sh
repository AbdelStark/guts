#!/usr/bin/env bash
#
# Guts Node Backup Script
# Creates compressed backups of the Guts node data directory
#
# Usage:
#   ./guts-backup.sh [options]
#
# Options:
#   --data-dir DIR      Data directory to backup (default: /var/lib/guts)
#   --output-dir DIR    Directory to store backups (default: /var/backups/guts)
#   --retention DAYS    Number of days to keep backups (default: 7)
#   --s3-bucket BUCKET  Upload to S3 bucket (optional)
#   --verify            Verify backup after creation
#   --quiet             Suppress non-error output
#   --help              Show this help message

set -euo pipefail

# Default configuration
DATA_DIR="${GUTS_DATA_DIR:-/var/lib/guts}"
OUTPUT_DIR="${GUTS_BACKUP_DIR:-/var/backups/guts}"
RETENTION_DAYS="${GUTS_BACKUP_RETENTION:-7}"
S3_BUCKET="${GUTS_S3_BUCKET:-}"
VERIFY=false
QUIET=false

# Logging
log() {
    if [[ "$QUIET" != "true" ]]; then
        echo "[$(date -Iseconds)] $*"
    fi
}

log_error() {
    echo "[$(date -Iseconds)] ERROR: $*" >&2
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --data-dir)
            DATA_DIR="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --retention)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --s3-bucket)
            S3_BUCKET="$2"
            shift 2
            ;;
        --verify)
            VERIFY=true
            shift
            ;;
        --quiet)
            QUIET=true
            shift
            ;;
        --help)
            head -25 "$0" | tail -20
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate data directory
if [[ ! -d "$DATA_DIR" ]]; then
    log_error "Data directory does not exist: $DATA_DIR"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Generate backup filename
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
HOSTNAME=$(hostname -s 2>/dev/null || echo "guts")
BACKUP_FILE="guts-${HOSTNAME}-${TIMESTAMP}.tar.gz"
BACKUP_PATH="${OUTPUT_DIR}/${BACKUP_FILE}"

log "Starting Guts backup..."
log "Data directory: $DATA_DIR"
log "Output file: $BACKUP_PATH"

# Create backup
log "Creating compressed backup..."
if ! tar -czf "$BACKUP_PATH" -C "$DATA_DIR" .; then
    log_error "Failed to create backup"
    rm -f "$BACKUP_PATH"
    exit 1
fi

# Calculate checksum
CHECKSUM=$(sha256sum "$BACKUP_PATH" | cut -d' ' -f1)
echo "$CHECKSUM  $BACKUP_FILE" > "${BACKUP_PATH}.sha256"

# Get backup size
SIZE=$(stat -c%s "$BACKUP_PATH" 2>/dev/null || stat -f%z "$BACKUP_PATH" 2>/dev/null || echo "unknown")
SIZE_HUMAN=$(numfmt --to=iec "$SIZE" 2>/dev/null || echo "$SIZE bytes")

log "Backup created: $BACKUP_FILE ($SIZE_HUMAN)"
log "Checksum: $CHECKSUM"

# Verify backup if requested
if [[ "$VERIFY" == "true" ]]; then
    log "Verifying backup..."
    if tar -tzf "$BACKUP_PATH" > /dev/null 2>&1; then
        log "Backup verification: PASSED"
    else
        log_error "Backup verification: FAILED"
        exit 1
    fi
fi

# Upload to S3 if configured
if [[ -n "$S3_BUCKET" ]]; then
    log "Uploading to S3: s3://${S3_BUCKET}/${BACKUP_FILE}"
    if aws s3 cp "$BACKUP_PATH" "s3://${S3_BUCKET}/${BACKUP_FILE}"; then
        aws s3 cp "${BACKUP_PATH}.sha256" "s3://${S3_BUCKET}/${BACKUP_FILE}.sha256"
        log "S3 upload complete"
    else
        log_error "S3 upload failed"
        exit 1
    fi
fi

# Clean up old backups
log "Cleaning up backups older than ${RETENTION_DAYS} days..."
find "$OUTPUT_DIR" -name "guts-*.tar.gz" -mtime +"$RETENTION_DAYS" -delete
find "$OUTPUT_DIR" -name "guts-*.sha256" -mtime +"$RETENTION_DAYS" -delete

# Clean up old S3 backups
if [[ -n "$S3_BUCKET" ]]; then
    CUTOFF_DATE=$(date -d "-${RETENTION_DAYS} days" +%Y-%m-%d 2>/dev/null || date -v-${RETENTION_DAYS}d +%Y-%m-%d 2>/dev/null || echo "")
    if [[ -n "$CUTOFF_DATE" ]]; then
        aws s3 ls "s3://${S3_BUCKET}/" | while read -r line; do
            FILE_DATE=$(echo "$line" | awk '{print $1}')
            FILE_NAME=$(echo "$line" | awk '{print $4}')
            if [[ -n "$FILE_NAME" && "$FILE_DATE" < "$CUTOFF_DATE" ]]; then
                log "Deleting old S3 backup: $FILE_NAME"
                aws s3 rm "s3://${S3_BUCKET}/${FILE_NAME}" 2>/dev/null || true
            fi
        done
    fi
fi

# Summary
log "Backup completed successfully"
log "  File: $BACKUP_PATH"
log "  Size: $SIZE_HUMAN"
log "  Checksum: $CHECKSUM"

# Output for monitoring
if [[ "$QUIET" != "true" ]]; then
    cat << EOF
{
  "status": "success",
  "timestamp": "$(date -Iseconds)",
  "file": "$BACKUP_PATH",
  "size_bytes": $SIZE,
  "checksum": "$CHECKSUM"
}
EOF
fi
