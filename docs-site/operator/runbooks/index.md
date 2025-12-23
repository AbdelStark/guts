# Operational Runbooks

> Step-by-step procedures for handling common operational scenarios and incidents.

## Overview

These runbooks provide structured procedures for diagnosing and resolving issues with Guts nodes. Each runbook follows a consistent format:

1. **Symptoms** - How to identify the issue
2. **Detection** - Monitoring alerts that trigger
3. **Diagnosis** - Steps to understand the problem
4. **Resolution** - How to fix it
5. **Escalation** - When to get help
6. **Post-Incident** - Follow-up actions

## Runbook Index

### Node Health

| Runbook | Severity | Description |
|---------|----------|-------------|
| [Node Not Syncing](node-not-syncing.md) | P2 | Node can't sync with network |
| [High Memory](high-memory.md) | P3 | Memory usage exceeds threshold |
| [Disk Full](disk-full.md) | P1 | Storage space exhausted |
| [High CPU](high-cpu.md) | P3 | CPU usage exceeds threshold |

### Consensus

| Runbook | Severity | Description |
|---------|----------|-------------|
| [Consensus Stuck](consensus-stuck.md) | P1 | No blocks being produced |
| [Validator Down](validator-down.md) | P2 | Validator not participating |

### Networking

| Runbook | Severity | Description |
|---------|----------|-------------|
| [Network Partition](network-partition.md) | P1 | Split-brain scenario |
| [Low Peer Count](low-peers.md) | P3 | Insufficient peer connections |

### Data

| Runbook | Severity | Description |
|---------|----------|-------------|
| [Data Corruption](data-corruption.md) | P1 | Data integrity issues |
| [Backup Failed](backup-failed.md) | P2 | Backup job failure |

### Security

| Runbook | Severity | Description |
|---------|----------|-------------|
| [Key Rotation](key-rotation.md) | P3 | Scheduled key rotation |
| [Security Incident](security-incident.md) | P1 | Suspected compromise |

### Operations

| Runbook | Severity | Description |
|---------|----------|-------------|
| [Emergency Shutdown](emergency-shutdown.md) | P1 | Controlled emergency stop |
| [Upgrade Rollback](upgrade-rollback.md) | P2 | Rollback failed upgrade |

## Severity Levels

| Level | Response Time | Description |
|-------|---------------|-------------|
| **P1** | 15 min | Critical - Service completely down |
| **P2** | 30 min | High - Major functionality impaired |
| **P3** | 4 hours | Medium - Minor impact, workaround exists |
| **P4** | 24 hours | Low - Cosmetic or future concern |

## On-Call Procedures

### Initial Response

1. Acknowledge the alert
2. Assess severity based on impact
3. Open incident channel (if P1/P2)
4. Follow relevant runbook
5. Escalate if needed

### Communication

- P1: Notify stakeholders immediately
- P2: Update status page
- P3/P4: Log in issue tracker

### Handoff

When handing off to another responder:

1. Brief them on current state
2. Share diagnostic data collected
3. Document actions taken
4. Transfer alert ownership

## Diagnostic Tools

### Quick Health Check

```bash
# Full system check
guts-node status --json | jq

# API health
curl -s http://localhost:8080/health | jq

# Metrics snapshot
curl -s http://localhost:9090/metrics | grep -E "^guts_" | head -50
```

### Log Analysis

```bash
# Recent errors
journalctl -u guts-node --since "10 min ago" | grep -i error

# Full diagnostic bundle
guts-node diagnostics --output /tmp/diag-$(date +%Y%m%d-%H%M%S).tar.gz
```

### Network Diagnostics

```bash
# Check peer connections
curl -s http://localhost:8080/api/consensus/validators | jq

# P2P connectivity
ss -tlnp | grep guts
```

## Creating New Runbooks

Use this template for new runbooks:

```markdown
# Runbook: [Issue Name]

**Severity:** P1/P2/P3/P4
**Impact:** [Description of user/system impact]
**On-Call Action:** [Immediate action required]

## Symptoms

- [ ] Symptom 1
- [ ] Symptom 2

## Detection

**Alert Name:** `guts_[metric]_critical`

**Query:**
\`\`\`promql
[Prometheus query]
\`\`\`

## Diagnosis

### Step 1: [First diagnostic step]

\`\`\`bash
[Commands to run]
\`\`\`

Expected: [What you should see]
If issue present: [What indicates the problem]

## Resolution

### Option A: [First resolution path]

\`\`\`bash
[Step-by-step commands]
\`\`\`

## Escalation

If unresolved after [time]:
1. Collect diagnostics
2. Contact [team/person]
3. Include: [required information]

## Post-Incident

- [ ] Update monitoring
- [ ] Document learnings
- [ ] Create follow-up issues
```

## Related Documentation

- [Monitoring Guide](../operations/monitoring.md)
- [Troubleshooting](../troubleshooting/common-issues.md)
- [Architecture](../architecture.md)
