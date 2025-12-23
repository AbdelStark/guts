# Backup & Recovery Guide

> Protect your Guts node data with comprehensive backup procedures.

## Overview

Guts nodes store several types of data that should be backed up:

| Data Type | Location | Frequency | Priority |
|-----------|----------|-----------|----------|
| Node Key | `/etc/guts/node.key` | Once | **Critical** |
| Configuration | `/etc/guts/config.yaml` | On change | High |
| Data Directory | `/var/lib/guts/` | Daily | High |
| Consensus State | `/var/lib/guts/consensus/` | Hourly | Medium |

## What to Backup

### Critical: Node Identity

The node key is your node's identity. **Loss means losing your node identity.**

```bash
# Backup node key
cp /etc/guts/node.key /backup/node.key
chmod 600 /backup/node.key
```

### High Priority: Configuration

```bash
# Backup configuration
cp /etc/guts/config.yaml /backup/config.yaml
```

### High Priority: Data Directory

Contains all Git objects, repository metadata, and collaboration data.

```bash
# Full data backup
tar -czf /backup/guts-data-$(date +%Y%m%d).tar.gz -C /var/lib/guts .
```

## Backup Methods

### Method 1: Snapshot Backup (Recommended)

Uses built-in backup command for consistent snapshots:

```bash
# Create backup
guts-node backup create \
  --output /backup/guts-$(date +%Y%m%d-%H%M%S).tar.gz

# Verify backup integrity
guts-node backup verify /backup/guts-20250101-120000.tar.gz
```

### Method 2: File System Backup

For LVM or ZFS environments:

```bash
# 1. Enter maintenance mode (pause writes)
guts-node maintenance enter

# 2. Create LVM snapshot
lvcreate -L 10G -s -n guts-snap /dev/vg0/guts-data

# 3. Exit maintenance mode
guts-node maintenance exit

# 4. Mount and backup snapshot
mount /dev/vg0/guts-snap /mnt/snap -o ro
tar -czf /backup/guts-$(date +%Y%m%d).tar.gz -C /mnt/snap .
umount /mnt/snap

# 5. Remove snapshot
lvremove -f /dev/vg0/guts-snap
```

For ZFS:

```bash
# Create snapshot
zfs snapshot tank/guts@backup-$(date +%Y%m%d)

# Send to backup location
zfs send tank/guts@backup-20250101 | gzip > /backup/guts-20250101.zfs.gz
```

### Method 3: Hot Backup (No Downtime)

```bash
# RocksDB checkpoint (consistent point-in-time copy)
guts-node backup create --hot \
  --output /backup/guts-hot-$(date +%Y%m%d).tar.gz
```

### Method 4: Continuous Replication

For near-zero RPO requirements:

```bash
# Configure WAL archiving
guts-node config set wal.archive_command "aws s3 cp %f s3://guts-wal/%p"
guts-node config set wal.restore_command "aws s3 cp s3://guts-wal/%f %p"

# Restart to apply
systemctl restart guts-node
```

## Automated Backup Script

Create `/usr/local/bin/guts-backup.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/var/backups/guts}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"
S3_BUCKET="${S3_BUCKET:-}"
NOTIFY_URL="${NOTIFY_URL:-}"

# Setup
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="guts-${DATE}.tar.gz"
LOG_FILE="/var/log/guts-backup.log"

log() {
    echo "[$(date -Iseconds)] $*" | tee -a "$LOG_FILE"
}

notify() {
    if [[ -n "$NOTIFY_URL" ]]; then
        curl -s -X POST "$NOTIFY_URL" -d "message=$*" || true
    fi
}

# Create backup directory
mkdir -p "$BACKUP_DIR"

log "Starting backup..."

# Create backup
if guts-node backup create --output "$BACKUP_DIR/$BACKUP_FILE"; then
    log "Backup created: $BACKUP_FILE"

    # Verify backup
    if guts-node backup verify "$BACKUP_DIR/$BACKUP_FILE"; then
        log "Backup verified successfully"
    else
        log "ERROR: Backup verification failed"
        notify "Guts backup verification failed: $BACKUP_FILE"
        exit 1
    fi

    # Upload to S3 if configured
    if [[ -n "$S3_BUCKET" ]]; then
        log "Uploading to S3..."
        if aws s3 cp "$BACKUP_DIR/$BACKUP_FILE" "s3://$S3_BUCKET/$BACKUP_FILE"; then
            log "Uploaded to s3://$S3_BUCKET/$BACKUP_FILE"
        else
            log "ERROR: S3 upload failed"
            notify "Guts backup S3 upload failed: $BACKUP_FILE"
        fi
    fi
else
    log "ERROR: Backup creation failed"
    notify "Guts backup creation failed"
    exit 1
fi

# Cleanup old backups
log "Cleaning up old backups..."
find "$BACKUP_DIR" -name "guts-*.tar.gz" -mtime +$RETENTION_DAYS -delete
if [[ -n "$S3_BUCKET" ]]; then
    aws s3 ls "s3://$S3_BUCKET/" | while read -r line; do
        file=$(echo "$line" | awk '{print $4}')
        date_str=$(echo "$file" | sed 's/guts-\([0-9]*\)-.*/\1/')
        if [[ -n "$date_str" ]]; then
            file_date=$(date -d "$date_str" +%s 2>/dev/null || echo 0)
            cutoff_date=$(date -d "-$RETENTION_DAYS days" +%s)
            if [[ $file_date -lt $cutoff_date ]]; then
                aws s3 rm "s3://$S3_BUCKET/$file"
                log "Deleted old S3 backup: $file"
            fi
        fi
    done
fi

log "Backup completed successfully"
notify "Guts backup completed: $BACKUP_FILE"
```

### Schedule with Cron

```bash
# /etc/cron.d/guts-backup
# Daily backup at 2 AM
0 2 * * * root /usr/local/bin/guts-backup.sh >> /var/log/guts-backup.log 2>&1

# Hourly consensus state backup
0 * * * * root guts-node backup create --consensus-only --output /var/backups/guts/consensus-$(date +\%H).tar.gz
```

### Schedule with Systemd Timer

```ini
# /etc/systemd/system/guts-backup.service
[Unit]
Description=Guts Backup

[Service]
Type=oneshot
User=root
ExecStart=/usr/local/bin/guts-backup.sh
Environment=BACKUP_DIR=/var/backups/guts
Environment=S3_BUCKET=my-guts-backups
```

```ini
# /etc/systemd/system/guts-backup.timer
[Unit]
Description=Daily Guts Backup

[Timer]
OnCalendar=*-*-* 02:00:00
Persistent=true
RandomizedDelaySec=30min

[Install]
WantedBy=timers.target
```

```bash
sudo systemctl enable guts-backup.timer
sudo systemctl start guts-backup.timer
```

## Restore Procedures

### Scenario 1: Same Server, New Disk

```bash
# 1. Install Guts
curl -sSL https://get.guts.network | sh

# 2. Stop service
sudo systemctl stop guts-node

# 3. Restore data
sudo guts-node backup restore /backup/guts-latest.tar.gz

# 4. Restore node key (if not in backup)
sudo cp /backup/node.key /etc/guts/node.key
sudo chmod 600 /etc/guts/node.key
sudo chown guts:guts /etc/guts/node.key

# 5. Start service
sudo systemctl start guts-node

# 6. Verify
curl http://localhost:8080/health/ready
```

### Scenario 2: New Server, Same Identity

```bash
# 1. Install Guts on new server
curl -sSL https://get.guts.network | sh

# 2. Transfer node key
scp backup-server:/backup/node.key /etc/guts/node.key
sudo chmod 600 /etc/guts/node.key
sudo chown guts:guts /etc/guts/node.key

# 3. Transfer configuration
scp backup-server:/backup/config.yaml /etc/guts/config.yaml

# 4. Restore data
scp backup-server:/backup/guts-latest.tar.gz /tmp/
sudo guts-node backup restore /tmp/guts-latest.tar.gz

# 5. Update configuration (new IP, etc.)
sudo vim /etc/guts/config.yaml

# 6. Start service
sudo systemctl start guts-node
```

### Scenario 3: Disaster Recovery (Full Resync)

When backup is unavailable, resync from the network:

```bash
# 1. Install Guts
curl -sSL https://get.guts.network | sh

# 2. Generate new identity (or restore old one)
guts-node keygen > /etc/guts/node.key

# 3. Configure with bootstrap nodes
cat > /etc/guts/config.yaml << EOF
api:
  addr: "0.0.0.0:8080"
p2p:
  addr: "0.0.0.0:9000"
  bootstrap_nodes:
    - "/dns4/bootstrap.guts.network/tcp/9000/p2p/..."
storage:
  data_dir: "/var/lib/guts"
EOF

# 4. Start and wait for sync
sudo systemctl start guts-node
guts-node status --wait-sync
```

## Backup Verification

### Verify Backup Integrity

```bash
# Verify backup file
guts-node backup verify /backup/guts-20250101.tar.gz

# Output:
# Backup file: guts-20250101.tar.gz
# Created: 2025-01-01T02:00:00Z
# Size: 1.2 GB
# Checksum: SHA256:abc123...
# Objects: 50,000 blobs, 10,000 trees, 5,000 commits
# Repositories: 100
# Status: VALID
```

### Test Restore

Periodically test restores to a temporary location:

```bash
# Test restore
mkdir -p /tmp/guts-test
guts-node backup restore /backup/guts-latest.tar.gz \
  --target /tmp/guts-test \
  --verify

# Run verification
guts-node verify --data-dir /tmp/guts-test --full

# Cleanup
rm -rf /tmp/guts-test
```

## Recovery Time Objectives

| Scenario | RTO Target | Procedure |
|----------|------------|-----------|
| Node restart | < 1 min | `systemctl restart guts-node` |
| Restore from local backup | < 30 min | Full restore |
| Restore from S3 | < 1 hour | Download + restore |
| Full resync from network | < 4 hours | Depends on data size |

## Recovery Point Objectives

| Backup Frequency | RPO | Storage Cost |
|------------------|-----|--------------|
| Daily | 24 hours | Low |
| Hourly | 1 hour | Medium |
| Continuous (WAL) | Minutes | High |

## Monitoring Backups

### Alert on Backup Failure

```yaml
# Prometheus alert
groups:
  - name: guts-backup
    rules:
      - alert: GutsBackupMissing
        expr: time() - guts_last_backup_timestamp > 86400  # 24 hours
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "No backup in 24 hours"
          runbook_url: "https://docs.guts.network/runbooks/backup-failed"
```

### Backup Metrics

```bash
# Add to backup script
echo "guts_last_backup_timestamp $(date +%s)" | curl --data-binary @- http://pushgateway:9091/metrics/job/guts-backup
echo "guts_backup_size_bytes $(stat -c%s $BACKUP_FILE)" | curl --data-binary @- http://pushgateway:9091/metrics/job/guts-backup
```

## Related Documentation

- [Upgrade Procedures](upgrades.md)
- [Disaster Recovery Runbook](../runbooks/data-corruption.md)
- [Monitoring Guide](monitoring.md)
