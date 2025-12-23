# Runbook: Node Not Syncing

**Severity:** P2
**Impact:** Node cannot serve current data, may serve stale content
**On-Call Action:** Investigate within 30 minutes

## Symptoms

- [ ] Node reports sync status as "syncing" for extended period (>5 min)
- [ ] Block height not increasing
- [ ] API returns stale data compared to other nodes
- [ ] `guts_consensus_block_height` metric is stalled
- [ ] Users report seeing old data

## Detection

**Alert Name:** `GutsNodeNotSyncing`

**Query:**
```promql
time() - guts_last_block_time > 60
```

**Dashboard:** Guts Node Overview → Sync Status panel

## Diagnosis

### Step 1: Check Sync Status

```bash
curl -s http://localhost:8080/api/consensus/status | jq
```

Expected output (healthy):
```json
{
  "status": "synced",
  "block_height": 12345,
  "latest_block_time": "2025-01-15T10:30:00Z",
  "peers": 5
}
```

If issue present:
```json
{
  "status": "syncing",
  "block_height": 12300,
  "sync_target": 12345,
  "peers": 2
}
```

### Step 2: Check Peer Connectivity

```bash
# Check connected peers
curl -s http://localhost:8080/api/consensus/validators | jq '.peers | length'

# List peers
curl -s http://localhost:8080/api/consensus/validators | jq '.peers'
```

If fewer than 3 peers:
- Check firewall rules (port 9000)
- Verify bootstrap nodes are reachable
- Check network connectivity

### Step 3: Check Disk Space

```bash
df -h /var/lib/guts
```

If usage > 90%, see [Disk Full Runbook](disk-full.md)

### Step 4: Check Memory

```bash
free -h
ps aux | grep guts-node | grep -v grep
```

If memory exhausted, see [High Memory Runbook](high-memory.md)

### Step 5: Check for Errors in Logs

```bash
# Recent errors
journalctl -u guts-node --since "10 min ago" | grep -iE "error|panic|failed"

# Sync-specific logs
journalctl -u guts-node --since "10 min ago" | grep -i sync
```

### Step 6: Compare with Other Nodes

```bash
# Check block heights across nodes
for node in node1 node2 node3; do
  echo "$node: $(curl -s http://$node:8080/api/consensus/status | jq .block_height)"
done
```

If all nodes are at same height but not progressing, see [Consensus Stuck](consensus-stuck.md)

### Step 7: Check Network Latency

```bash
# Test connectivity to peers
for peer in $(curl -s http://localhost:8080/api/consensus/validators | jq -r '.peers[].addr'); do
  echo "Ping to $peer:"
  ping -c 3 "$peer" | tail -1
done
```

High latency (>500ms) can cause sync issues.

## Resolution

### Option A: Restart Node (Try First)

Most sync issues resolve with a restart:

```bash
# Graceful restart
sudo systemctl restart guts-node

# Wait for startup
sleep 30

# Check sync status
curl -s http://localhost:8080/api/consensus/status | jq
```

### Option B: Force Peer Reconnection

If peers are stale:

```bash
# Clear peer cache and restart
sudo systemctl stop guts-node
rm -f /var/lib/guts/peers.db
sudo systemctl start guts-node
```

### Option C: Reset Sync State

If sync state is corrupted (preserves data):

```bash
# Stop node
sudo systemctl stop guts-node

# Clear sync state
guts-node sync reset --data-dir /var/lib/guts

# Restart
sudo systemctl start guts-node

# Monitor sync progress
watch -n 5 'curl -s http://localhost:8080/api/consensus/status | jq'
```

### Option D: Resync from Snapshot

For faster recovery with large data:

```bash
# Stop node
sudo systemctl stop guts-node

# Download latest snapshot
guts-node snapshot download \
  --url https://snapshots.guts.network/latest.tar.gz \
  --output /tmp/snapshot.tar.gz

# Restore snapshot
guts-node backup restore /tmp/snapshot.tar.gz --target /var/lib/guts

# Restart
sudo systemctl start guts-node
```

### Option E: Full Resync from Network

Last resort - complete resync:

```bash
# Stop node
sudo systemctl stop guts-node

# Backup current data (just in case)
tar -czf /tmp/guts-backup.tar.gz -C /var/lib/guts .

# Clear data directory
rm -rf /var/lib/guts/*

# Restart - node will sync from scratch
sudo systemctl start guts-node

# Monitor (may take hours for large networks)
watch -n 30 'curl -s http://localhost:8080/api/consensus/status | jq'
```

## Escalation

If none of the above works after 1 hour:

1. **Collect diagnostics:**
   ```bash
   guts-node diagnostics --output /tmp/sync-issue-$(date +%Y%m%d).tar.gz
   ```

2. **Check if network-wide:**
   - Are other nodes also stuck?
   - Is this a consensus issue?

3. **Contact core team:**
   - Include: Node ID, diagnostic bundle, timeline
   - Join: #ops-emergency channel

## Post-Incident

- [ ] Verify node fully caught up
- [ ] Check for any data inconsistencies
- [ ] Monitor for recurrence (next 24 hours)
- [ ] Update monitoring thresholds if alert was too sensitive
- [ ] Document root cause in incident report

## Related Runbooks

- [Consensus Stuck](consensus-stuck.md)
- [Network Partition](network-partition.md)
- [High Memory](high-memory.md)
- [Disk Full](disk-full.md)
