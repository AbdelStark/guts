# Runbook: Disk Full

**Severity:** P1
**Impact:** Node cannot write data, may crash, data loss possible
**On-Call Action:** Respond immediately (15 min)

## Symptoms

- [ ] Disk usage > 90%
- [ ] Write operations failing
- [ ] Node logs show "No space left on device"
- [ ] Git push operations fail
- [ ] Database write errors

## Detection

**Alert Name:** `GutsDiskSpaceCritical`

**Query:**
```promql
guts_storage_available_bytes / guts_storage_total_bytes < 0.1
```

Or system-level:
```promql
(node_filesystem_avail_bytes{mountpoint="/var/lib/guts"} / node_filesystem_size_bytes{mountpoint="/var/lib/guts"}) < 0.1
```

**Dashboard:** Node Exporter → Disk Usage

## Diagnosis

### Step 1: Check Disk Usage

```bash
# Overall disk usage
df -h /var/lib/guts

# What's using space
du -sh /var/lib/guts/*

# Detailed breakdown
du -h --max-depth=2 /var/lib/guts | sort -rh | head -20
```

### Step 2: Identify Large Files

```bash
# Find largest files
find /var/lib/guts -type f -size +100M -exec ls -lh {} \; | sort -k5 -rh | head -20

# Find old log files
find /var/lib/guts -name "*.log" -mtime +7 -size +10M
```

### Step 3: Check for Runaway Growth

```bash
# Check inode usage (too many small files)
df -i /var/lib/guts

# Recent file creation
find /var/lib/guts -type f -mmin -60 | wc -l
```

### Step 4: Identify Cause

Common causes:
- **Git objects accumulation:** Large repos, many pushes
- **Log files:** Debug logging enabled
- **Pack files:** Git garbage collection not running
- **Consensus logs:** WAL files not archived
- **Backups:** Old backups not cleaned up

## Resolution

### Option A: Clean Up Safe-to-Delete Files

```bash
# Remove old log files
find /var/lib/guts -name "*.log" -mtime +7 -delete

# Remove temporary files
find /var/lib/guts -name "*.tmp" -delete
find /var/lib/guts -name "*.temp" -delete

# Remove old pack files (after gc)
find /var/lib/guts -name "*.old" -delete
```

### Option B: Run Git Garbage Collection

```bash
# Compact Git objects
guts-node storage gc --aggressive

# Or manually for each repo
for repo in /var/lib/guts/repos/*; do
  git -C "$repo" gc --aggressive --prune=now
done
```

### Option C: Archive Old Data

```bash
# Archive old consensus logs
guts-node storage archive \
  --older-than 7d \
  --output /backup/archive-$(date +%Y%m%d).tar.gz

# Remove archived data
guts-node storage prune --older-than 7d --archived
```

### Option D: Expand Storage (Cloud)

#### AWS EBS
```bash
# Resize EBS volume in AWS Console or CLI
aws ec2 modify-volume --volume-id vol-xxx --size 500

# Grow filesystem
sudo growpart /dev/xvda 1
sudo resize2fs /dev/xvda1
```

#### GCP Persistent Disk
```bash
# Resize in GCP Console or CLI
gcloud compute disks resize guts-data --size 500GB

# Grow filesystem
sudo resize2fs /dev/sdb
```

### Option E: Add Additional Storage

```bash
# Mount additional volume
sudo mkfs.ext4 /dev/xvdb
sudo mkdir -p /var/lib/guts-overflow
sudo mount /dev/xvdb /var/lib/guts-overflow

# Move large data
sudo mv /var/lib/guts/archive /var/lib/guts-overflow/
sudo ln -s /var/lib/guts-overflow/archive /var/lib/guts/archive
```

### Option F: Emergency: Remove Non-Critical Data

**⚠️ Use only if node is completely stuck:**

```bash
# Stop node
sudo systemctl stop guts-node

# Remove oldest pack files (can be re-fetched from network)
find /var/lib/guts/objects/pack -name "*.pack" -mtime +30 | head -5 | xargs rm -f

# Start node (will resync missing data)
sudo systemctl start guts-node
```

## Prevention

### Configure Monitoring

Add alerts at 80% and 90%:

```yaml
groups:
  - name: disk
    rules:
      - alert: GutsDiskSpaceWarning
        expr: guts_storage_available_bytes / guts_storage_total_bytes < 0.2
        for: 5m
        labels:
          severity: warning

      - alert: GutsDiskSpaceCritical
        expr: guts_storage_available_bytes / guts_storage_total_bytes < 0.1
        for: 1m
        labels:
          severity: critical
```

### Set Up Automatic Cleanup

```bash
# Add cron job for regular cleanup
cat > /etc/cron.d/guts-cleanup << 'EOF'
# Run git gc weekly
0 3 * * 0 guts /usr/local/bin/guts-node storage gc >> /var/log/guts-gc.log 2>&1

# Clean old logs daily
0 4 * * * root find /var/lib/guts -name "*.log" -mtime +7 -delete

# Archive old data monthly
0 2 1 * * guts /usr/local/bin/guts-node storage archive --older-than 30d
EOF
```

### Configure Log Rotation

```bash
cat > /etc/logrotate.d/guts << 'EOF'
/var/lib/guts/logs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    maxsize 100M
}
EOF
```

## Escalation

If unable to free space quickly:

1. **Notify users:** Write operations may fail
2. **Consider failover:** If multi-node, direct traffic elsewhere
3. **Emergency storage:** Provision emergency volume

## Post-Incident

- [ ] Verify node is healthy
- [ ] Document what consumed space
- [ ] Implement prevention measures
- [ ] Review storage sizing
- [ ] Update capacity planning

## Related Runbooks

- [High Memory](high-memory.md)
- [Data Corruption](data-corruption.md)
