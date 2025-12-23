# Runbook: Emergency Shutdown

**Severity:** P1
**Impact:** Node completely stopped, service unavailable
**On-Call Action:** Use only when necessary for safety

## When to Use Emergency Shutdown

Use this procedure when:

- [ ] Security incident requires immediate isolation
- [ ] Data corruption is actively spreading
- [ ] Node is causing network-wide issues
- [ ] Regulatory or legal requirement
- [ ] Hardware failure imminent (smoke, overheating)

**⚠️ WARNING:** This will make the node unavailable. For validators, this may impact consensus if quorum is affected.

## Pre-Shutdown Checklist

### For Validators

Before shutting down a validator, verify quorum will be maintained:

```bash
# Check current validator count
curl -s http://localhost:8080/api/consensus/validators | jq '.validators | length'

# Check how many are currently active
for v in $(curl -s http://localhost:8080/api/consensus/validators | jq -r '.validators[].addr'); do
  if curl -s --connect-timeout 2 "$v/health" >/dev/null 2>&1; then
    echo "$v: UP"
  else
    echo "$v: DOWN"
  fi
done

# Formula: Need > 2n/3 validators
# 4 validators: Need 3+ (can lose 1)
# 7 validators: Need 5+ (can lose 2)
```

**If shutdown will break quorum:** Coordinate with other operators first.

### For Full Nodes

Check if there are other nodes available:

```bash
# Ensure load balancer has healthy alternatives
curl http://load-balancer/health
```

## Emergency Shutdown Procedure

### Step 1: Notify Stakeholders

```bash
# Update status page
# Send alert to operations channel
# Log the shutdown reason
echo "$(date): Emergency shutdown initiated - Reason: [REASON]" >> /var/log/guts-emergency.log
```

### Step 2: Graceful Shutdown (Preferred)

Try graceful shutdown first (allows in-flight requests to complete):

```bash
# Graceful stop (30 second timeout)
sudo systemctl stop guts-node

# Verify stopped
systemctl is-active guts-node
```

### Step 3: Forceful Shutdown (If Graceful Fails)

If node doesn't stop within 30 seconds:

```bash
# Force stop
sudo systemctl kill guts-node

# Or direct kill
sudo kill -9 $(pgrep guts-node)

# Verify
pgrep guts-node || echo "Process stopped"
```

### Step 4: Isolate from Network

Prevent any network communication:

```bash
# Block P2P ports
sudo iptables -A INPUT -p tcp --dport 9000 -j DROP
sudo iptables -A INPUT -p udp --dport 9000 -j DROP
sudo iptables -A OUTPUT -p tcp --dport 9000 -j DROP

# Block API port (if needed)
sudo iptables -A INPUT -p tcp --dport 8080 -j DROP
```

### Step 5: Preserve Evidence (If Security Incident)

```bash
# Create forensic snapshot
mkdir -p /tmp/forensics

# Copy logs
cp -r /var/log/guts* /tmp/forensics/
journalctl -u guts-node > /tmp/forensics/journal.log

# Copy data directory (if possible)
tar -czf /tmp/forensics/data-$(date +%Y%m%d-%H%M%S).tar.gz -C /var/lib/guts .

# Network connections at time of shutdown
ss -tlnp > /tmp/forensics/connections.txt
netstat -an > /tmp/forensics/netstat.txt

# Running processes
ps auxf > /tmp/forensics/processes.txt

# Secure the evidence
chmod 600 /tmp/forensics/*
```

### Step 6: Disable Auto-Restart

Prevent systemd from restarting the node:

```bash
# Disable the service
sudo systemctl disable guts-node

# Or mask it completely
sudo systemctl mask guts-node
```

## Post-Shutdown Actions

### Document the Incident

```markdown
## Emergency Shutdown Report

**Date/Time:** [timestamp]
**Node:** [node identifier]
**Operator:** [your name]

### Reason for Shutdown
[Describe why emergency shutdown was necessary]

### Impact
- Duration: [start time] to [end time]
- Users affected: [description]
- Data impact: [any data loss or corruption]

### Actions Taken
1. [First action]
2. [Second action]
...

### Evidence Preserved
- Logs: /tmp/forensics/
- Data snapshot: [location]

### Next Steps
- [ ] Investigation required
- [ ] Security review
- [ ] Recovery planning
```

### Notify Relevant Teams

| Reason | Notify |
|--------|--------|
| Security incident | Security team, management |
| Data corruption | Engineering, data team |
| Hardware failure | Infrastructure, vendor |
| Consensus issues | Other validators |

## Recovery After Emergency Shutdown

### Step 1: Assess Situation

Before bringing node back online:

```bash
# Check if issue is resolved
# This depends on the original reason for shutdown

# For security: Wait for security team clearance
# For hardware: Replace/repair hardware
# For corruption: Verify data integrity
```

### Step 2: Remove Network Isolation

```bash
# Remove iptables rules
sudo iptables -D INPUT -p tcp --dport 9000 -j DROP
sudo iptables -D INPUT -p udp --dport 9000 -j DROP
sudo iptables -D OUTPUT -p tcp --dport 9000 -j DROP
sudo iptables -D INPUT -p tcp --dport 8080 -j DROP
```

### Step 3: Re-enable Service

```bash
# Unmask and enable
sudo systemctl unmask guts-node
sudo systemctl enable guts-node
```

### Step 4: Start Node

```bash
# Start with extra logging
sudo systemctl start guts-node

# Monitor startup
sudo journalctl -u guts-node -f
```

### Step 5: Verify Recovery

```bash
# Check health
curl http://localhost:8080/health/ready

# Check sync status
curl http://localhost:8080/api/consensus/status | jq

# Verify peer connectivity
curl http://localhost:8080/api/consensus/validators | jq
```

## Scenarios

### Security Incident

1. **Immediate:** Shut down and isolate
2. **Preserve:** Collect all evidence
3. **Analyze:** Work with security team
4. **Remediate:** Patch vulnerabilities
5. **Recover:** Fresh install if compromised

### Data Corruption

1. **Immediate:** Stop to prevent spread
2. **Assess:** Determine extent of corruption
3. **Recover:** Restore from backup or resync
4. **Verify:** Full data integrity check

### Hardware Failure

1. **Immediate:** Graceful shutdown if possible
2. **Replace:** Fix/replace hardware
3. **Recover:** Boot and verify
4. **Validate:** Run hardware diagnostics

### Network Attack

1. **Immediate:** Block all traffic
2. **Analyze:** Identify attack vector
3. **Mitigate:** Apply firewall rules
4. **Resume:** Gradual traffic restoration

## Related Runbooks

- [Security Incident](security-incident.md)
- [Data Corruption](data-corruption.md)
- [Consensus Stuck](consensus-stuck.md)
- [Network Partition](network-partition.md)
