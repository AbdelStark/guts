# Runbook: Consensus Stuck

**Severity:** P1
**Impact:** No new blocks being produced, writes fail, network halted
**On-Call Action:** Respond immediately (15 min)

## Symptoms

- [ ] No new blocks produced for >2 minutes
- [ ] `guts_consensus_commits_total` not increasing
- [ ] Write operations fail with timeout
- [ ] All validators show same block height
- [ ] API returns errors for write operations

## Detection

**Alert Name:** `GutsConsensusStalled`

**Query:**
```promql
rate(guts_consensus_commits_total[5m]) == 0
```

**Dashboard:** Guts Consensus → Block Production Rate

## Diagnosis

### Step 1: Check Consensus Status on All Validators

```bash
# Check all validators
for i in 1 2 3 4; do
  echo "Validator $i:"
  curl -s http://validator$i:8080/api/consensus/status | jq '{height: .block_height, round: .round, role: .role}'
done
```

Expected (healthy):
```json
{"height": 12345, "round": 0, "role": "validator"}
{"height": 12345, "round": 0, "role": "leader"}
{"height": 12345, "round": 0, "role": "validator"}
{"height": 12345, "round": 0, "role": "validator"}
```

Problematic indicators:
- Heights differ by >1
- Round number > 0 (indicates proposal failures)
- Missing validators

### Step 2: Count Active Validators

```bash
# Check validator count
curl -s http://localhost:8080/api/consensus/validators | jq '.validators | length'

# Check active (responding) validators
for i in 1 2 3 4; do
  if curl -s --connect-timeout 2 http://validator$i:8080/health/ready >/dev/null 2>&1; then
    echo "Validator $i: UP"
  else
    echo "Validator $i: DOWN"
  fi
done
```

**Byzantine Fault Tolerance:**
- 4 validators: Need 3 up (can tolerate 1 failure)
- 7 validators: Need 5 up (can tolerate 2 failures)
- Formula: Need > 2/3 of validators

### Step 3: Check for Leader Issues

```bash
# Find current leader
curl -s http://localhost:8080/api/consensus/status | jq '.leader'

# Check leader's status
LEADER=$(curl -s http://localhost:8080/api/consensus/status | jq -r '.leader.addr')
curl -s "http://$LEADER/health/ready"
```

If leader is down or unreachable:
- Network should elect new leader within ~30 seconds
- If not, there may be a quorum issue

### Step 4: Check Network Connectivity Between Validators

```bash
# From each validator, test connectivity to others
for src in 1 2 3 4; do
  for dst in 1 2 3 4; do
    if [ $src != $dst ]; then
      echo "validator$src -> validator$dst: $(ssh validator$src "nc -zv validator$dst 9000 2>&1 | tail -1")"
    fi
  done
done
```

Look for:
- Timeouts
- Connection refused
- One-way connectivity (partitions)

### Step 5: Check for Byzantine Behavior

```bash
# Check for conflicting proposals
journalctl -u guts-node --since "10 min ago" | grep -iE "conflicting|equivocation|invalid"

# Check vote counts
curl -s http://localhost:8080/api/consensus/status | jq '.votes'
```

### Step 6: Check Mempool

```bash
# Check if transactions are pending
curl -s http://localhost:8080/api/consensus/mempool | jq

# If mempool is full, may indicate processing issues
```

## Resolution

### Option A: Wait for Automatic Recovery (First Try)

Simplex BFT has built-in recovery mechanisms. Wait 2-3 minutes for:
- Leader rotation
- View change
- Timeout-based recovery

```bash
# Monitor for automatic recovery
watch -n 5 'curl -s http://localhost:8080/api/consensus/status | jq "{height: .block_height, round: .round}"'
```

### Option B: Restart Stalled Validator

If one validator is causing issues:

```bash
# Identify problematic validator
# (One that's not voting or has different height)

# Restart it
ssh validator2 "sudo systemctl restart guts-node"

# Wait for rejoin
sleep 30

# Check if consensus resumed
curl -s http://localhost:8080/api/consensus/status | jq
```

### Option C: Force Leader Rotation

If leader is stuck but reachable:

```bash
# This triggers view change
guts-node consensus force-view-change --node http://validator1:8080
```

### Option D: Restore Quorum

If too many validators are down:

```bash
# Check which validators are down
for i in 1 2 3 4; do
  curl -s --connect-timeout 2 http://validator$i:8080/health/ready || echo "validator$i: DOWN"
done

# Bring them back up
ssh validator2 "sudo systemctl start guts-node"
ssh validator3 "sudo systemctl start guts-node"

# Wait for quorum
sleep 60

# Check consensus
curl -s http://localhost:8080/api/consensus/status | jq
```

### Option E: Emergency Consensus Reset

**⚠️ DANGER: Only use if consensus is completely broken and other options failed**

```bash
# 1. Stop all validators
for i in 1 2 3 4; do
  ssh validator$i "sudo systemctl stop guts-node"
done

# 2. Identify validator with highest consistent block
for i in 1 2 3 4; do
  echo "Validator $i: $(ssh validator$i "guts-node status --offline | jq .block_height")"
done

# 3. Copy state from highest to others
BEST=validator1  # The one with highest height
for i in 2 3 4; do
  rsync -av $BEST:/var/lib/guts/consensus/ validator$i:/var/lib/guts/consensus/
done

# 4. Restart all validators
for i in 1 2 3 4; do
  ssh validator$i "sudo systemctl start guts-node"
done

# 5. Monitor recovery
watch -n 5 'for i in 1 2 3 4; do echo -n "validator$i: "; curl -s http://validator$i:8080/api/consensus/status | jq -c "{h: .block_height, r: .round}"; done'
```

## Escalation

If consensus doesn't resume within 15 minutes:

1. **Declare P1 Incident:**
   - Update status page
   - Notify stakeholders

2. **Collect diagnostics from ALL validators:**
   ```bash
   for i in 1 2 3 4; do
     ssh validator$i "guts-node diagnostics --output /tmp/diag-validator$i.tar.gz"
     scp validator$i:/tmp/diag-validator$i.tar.gz ./
   done
   ```

3. **Contact core team:**
   - Emergency contact: [on-call rotation]
   - Include: Timeline, diagnostics, actions taken

## Post-Incident

- [ ] Verify all validators are in sync
- [ ] Check for data loss (compare block heights)
- [ ] Review logs for root cause
- [ ] Update monitoring if detection was slow
- [ ] Schedule post-mortem if >5 min downtime
- [ ] Consider adding redundant validators if quorum was close

## Root Cause Categories

| Symptom | Likely Cause | Prevention |
|---------|--------------|------------|
| Single validator down | Hardware/network failure | Redundant infrastructure |
| Multiple validators down | Coordinated failure, deployment issue | Staggered deployments |
| Leader can't propose | Network partition to leader | Improve network redundancy |
| Votes not reaching quorum | Network issues | Monitor inter-validator latency |
| Byzantine behavior | Compromised node, bug | Security audits, version consistency |

## Related Runbooks

- [Validator Down](validator-down.md)
- [Network Partition](network-partition.md)
- [Node Not Syncing](node-not-syncing.md)
