# Runbook: Data Corruption

**Severity:** P1
**Impact:** Data integrity compromised, potential data loss
**On-Call Action:** Respond immediately (15 min)

## Symptoms

- [ ] Checksum verification failures
- [ ] Git operations return errors
- [ ] Database read errors in logs
- [ ] Inconsistent data between nodes
- [ ] Users report missing or incorrect data

## Detection

**Alert Name:** `GutsDataIntegrityError`

**Log patterns:**
```
ERROR guts_storage: checksum mismatch
ERROR guts_git: corrupted object
ERROR rocksdb: corruption detected
```

**Manual check:**
```bash
guts-node verify --data-dir /var/lib/guts
```

## Diagnosis

### Step 1: Identify Scope of Corruption

```bash
# Run full verification
guts-node verify --data-dir /var/lib/guts --full 2>&1 | tee /tmp/verify-output.log

# Count errors
grep -c "ERROR\|CORRUPT\|MISMATCH" /tmp/verify-output.log

# Identify affected objects
grep "CORRUPT\|MISMATCH" /tmp/verify-output.log | head -20
```

### Step 2: Check Recent Events

```bash
# Recent errors
journalctl -u guts-node --since "1 hour ago" | grep -iE "corrupt|error|failed"

# Check for disk errors
dmesg | grep -iE "error|i/o|sector"

# Check SMART status
sudo smartctl -a /dev/sda | grep -E "Reallocated|Pending|Uncorrectable"
```

### Step 3: Determine Cause

| Symptom | Likely Cause |
|---------|--------------|
| Disk I/O errors | Hardware failure |
| After power loss | Incomplete write |
| After upgrade | Software bug |
| Random objects | Bit rot |
| Specific repo | User/application issue |

### Step 4: Assess Impact

```bash
# Which repositories are affected?
guts-node verify --data-dir /var/lib/guts --list-affected

# How much data?
du -sh /var/lib/guts
wc -l /tmp/verify-output.log
```

## Resolution

### Option A: Repair from Pack Files

If loose objects are corrupted but pack files are intact:

```bash
# Repack to regenerate objects
guts-node storage repack --data-dir /var/lib/guts
```

### Option B: Recover from Network

If other nodes have correct data:

```bash
# Stop local node
sudo systemctl stop guts-node

# Clear corrupted objects (keeps refs)
guts-node storage clear-corrupted --data-dir /var/lib/guts

# Restart and resync
sudo systemctl start guts-node

# Monitor sync
watch -n 5 'curl -s http://localhost:8080/api/consensus/status | jq'
```

### Option C: Restore from Backup

If corruption is widespread:

```bash
# Stop node
sudo systemctl stop guts-node

# List available backups
ls -la /var/backups/guts/

# Verify backup integrity
guts-node backup verify /var/backups/guts/guts-latest.tar.gz

# Restore
guts-node backup restore /var/backups/guts/guts-latest.tar.gz \
  --target /var/lib/guts \
  --verify

# Start node
sudo systemctl start guts-node
```

### Option D: Full Resync

Last resort if no good backup:

```bash
# Stop node
sudo systemctl stop guts-node

# Backup corrupted data (for investigation)
mv /var/lib/guts /var/lib/guts-corrupted-$(date +%Y%m%d)

# Create fresh data directory
mkdir -p /var/lib/guts
chown guts:guts /var/lib/guts

# Start node (will sync from network)
sudo systemctl start guts-node

# Monitor full resync
watch -n 30 'curl -s http://localhost:8080/api/consensus/status | jq'
```

## Specific Corruption Types

### Git Object Corruption

```bash
# Identify corrupted objects
find /var/lib/guts/objects -type f -exec sh -c '
  OBJ=$(basename {} | sed "s/.*/\0/")
  if ! git cat-file -e "$OBJ" 2>/dev/null; then
    echo "Corrupted: $OBJ"
  fi
' \;

# Remove corrupted and let git fetch from pack
rm /var/lib/guts/objects/<corrupted-hash>
```

### RocksDB Corruption

```bash
# Try repair
guts-node storage repair-db --data-dir /var/lib/guts

# If repair fails, restore from backup
```

### Consensus State Corruption

```bash
# Reset consensus state (preserves data)
guts-node consensus reset --data-dir /var/lib/guts

# Restart to rebuild from network
sudo systemctl restart guts-node
```

## Prevention

### Enable Checksums

```yaml
# config.yaml
storage:
  rocksdb:
    verify_checksums: true
  git:
    verify_objects: true
```

### Use ECC Memory

For production validators, use ECC RAM to prevent bit flips.

### RAID Storage

Use RAID-1 or RAID-10 for redundancy:

```bash
# Check RAID status
cat /proc/mdstat
mdadm --detail /dev/md0
```

### Regular Verification

```bash
# Cron job for weekly verification
0 3 * * 0 guts /usr/local/bin/guts-node verify --data-dir /var/lib/guts >> /var/log/guts-verify.log 2>&1
```

### ZFS with Scrub

If using ZFS:

```bash
# Schedule regular scrubs
zpool scrub tank

# Check for errors
zpool status tank
```

## Investigation

### Collect Evidence

```bash
# Full diagnostic bundle
guts-node diagnostics --output /tmp/corruption-diag.tar.gz

# Disk health
sudo smartctl -a /dev/sda > /tmp/smart.log

# System logs
journalctl -b > /tmp/syslog.txt

# Memory errors
grep -i "memory\|ecc\|mce" /var/log/syslog > /tmp/memory.log
```

### Root Cause Analysis

| Evidence | Probable Cause | Action |
|----------|---------------|--------|
| SMART errors | Failing disk | Replace disk |
| Memory errors | Bad RAM | Replace/test RAM |
| No hardware errors | Software bug | Report to developers |
| After crash | Incomplete write | Improve UPS/shutdown |

## Escalation

If data cannot be recovered:

1. **Stop all write operations**
2. **Preserve evidence** for investigation
3. **Contact core team** with diagnostic bundle
4. **Communicate** data loss to affected users

## Post-Incident

- [ ] Verify all data integrity
- [ ] Document data loss (if any)
- [ ] Identify and address root cause
- [ ] Review backup procedures
- [ ] Update monitoring for earlier detection
- [ ] Schedule hardware replacement if needed

## Related Runbooks

- [Disk Full](disk-full.md)
- [Backup Failed](backup-failed.md)
- [Emergency Shutdown](emergency-shutdown.md)
