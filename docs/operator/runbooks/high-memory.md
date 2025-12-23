# Runbook: High Memory Usage

**Severity:** P3
**Impact:** Degraded performance, potential OOM kills
**On-Call Action:** Investigate within 4 hours

## Symptoms

- [ ] Memory usage > 90% of configured limit
- [ ] Node becoming slow or unresponsive
- [ ] OOM killer messages in system logs
- [ ] Increased request latency
- [ ] `guts_process_resident_memory_bytes` exceeding threshold

## Detection

**Alert Name:** `GutsHighMemoryUsage`

**Query:**
```promql
guts_process_resident_memory_bytes / guts_config_max_memory > 0.9
```

Or system-level:
```promql
(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes > 0.9
```

**Dashboard:** Node Exporter → Memory Usage

## Diagnosis

### Step 1: Check Current Memory Usage

```bash
# Process memory
ps aux --sort=-%mem | head -10

# Detailed guts-node memory
cat /proc/$(pgrep guts-node)/status | grep -E "VmRSS|VmHWM|VmSize"

# System overview
free -h
```

### Step 2: Check Memory Trends

```bash
# Memory over time (requires sar)
sar -r 1 10

# Or via metrics
curl -s http://localhost:9090/metrics | grep guts_process_resident_memory_bytes
```

### Step 3: Identify Memory Consumers

```bash
# Check cache size
curl -s http://localhost:8080/api/debug/cache | jq

# Check connection count (each holds memory)
curl -s http://localhost:8080/api/debug/connections | jq

# Check pending transactions
curl -s http://localhost:8080/api/consensus/mempool | jq
```

### Step 4: Check for Memory Leaks

```bash
# Monitor over time
while true; do
  echo "$(date): $(cat /proc/$(pgrep guts-node)/status | grep VmRSS)"
  sleep 60
done
```

If memory continuously grows without plateau, likely a leak.

### Step 5: Check Recent Changes

```bash
# Recent deployments
git log --oneline -10

# Config changes
diff /etc/guts/config.yaml /etc/guts/config.yaml.bak
```

## Resolution

### Option A: Restart Node (Quick Fix)

If immediate relief needed:

```bash
# Graceful restart
sudo systemctl restart guts-node

# Monitor memory after restart
watch -n 5 'free -h | grep Mem'
```

### Option B: Reduce Cache Size

```bash
# Update configuration
cat >> /etc/guts/config.yaml << 'EOF'
storage:
  cache:
    max_size: 134217728  # Reduce to 128MB from 256MB
EOF

# Restart to apply
sudo systemctl restart guts-node
```

### Option C: Limit Concurrent Connections

```bash
# Update configuration
cat >> /etc/guts/config.yaml << 'EOF'
api:
  max_connections: 1000  # Reduce from unlimited
p2p:
  max_peers: 25  # Reduce from 50
EOF

# Restart to apply
sudo systemctl restart guts-node
```

### Option D: Tune RocksDB Memory

```bash
# Update configuration
cat >> /etc/guts/config.yaml << 'EOF'
storage:
  rocksdb:
    block_cache_size: 268435456  # 256MB
    write_buffer_size: 33554432  # 32MB
    max_write_buffer_number: 2
EOF

# Restart to apply
sudo systemctl restart guts-node
```

### Option E: Clear Memory Caches

```bash
# Drop system caches (temporary relief)
sync; echo 3 > /proc/sys/vm/drop_caches

# Force guts cache clear
guts-node cache clear
```

### Option F: Add Memory (If Undersized)

For systemd-managed nodes:

```bash
# Update service limits
sudo systemctl edit guts-node

# Add:
[Service]
MemoryMax=64G

# Reload and restart
sudo systemctl daemon-reload
sudo systemctl restart guts-node
```

For Kubernetes:

```bash
# Update resource limits
kubectl patch statefulset guts-node -n guts --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/containers/0/resources/limits/memory", "value": "64Gi"}]'
```

## Investigation: Memory Leak

If memory continuously grows:

### Step 1: Enable Memory Profiling

```bash
# If built with profiling
guts-node --heap-profile /tmp/heap.prof

# After running for a while
go tool pprof /tmp/heap.prof
```

### Step 2: Collect Heap Dump

```bash
# Send signal to dump heap
kill -USR1 $(pgrep guts-node)

# Heap dump saved to /var/lib/guts/heap-*.prof
```

### Step 3: Analyze

```bash
# Top memory consumers
go tool pprof -top /var/lib/guts/heap-*.prof

# Generate flamegraph
go tool pprof -http=:8081 /var/lib/guts/heap-*.prof
```

## Prevention

### Set Memory Limits

Always set memory limits in production:

```ini
# systemd
[Service]
MemoryMax=32G
MemoryHigh=28G  # Soft limit, triggers reclaim
```

```yaml
# Kubernetes
resources:
  limits:
    memory: 32Gi
  requests:
    memory: 8Gi
```

### Configure OOM Handling

```bash
# Adjust OOM score (lower = less likely to be killed)
echo -500 > /proc/$(pgrep guts-node)/oom_score_adj
```

### Monitor Memory Trends

Set up alerting for gradual growth:

```yaml
groups:
  - name: memory
    rules:
      - alert: GutsMemoryGrowth
        expr: |
          predict_linear(guts_process_resident_memory_bytes[1h], 3600 * 4)
          > guts_config_max_memory * 0.9
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "Memory predicted to exceed limit in 4 hours"
```

## Escalation

If memory issues persist after optimization:

1. **Collect diagnostics:**
   ```bash
   guts-node diagnostics --include-heap --output /tmp/mem-diag.tar.gz
   ```

2. **File issue:**
   - Include: Memory profile, configuration, workload description
   - Tag: `memory-leak` if growth is unbounded

## Post-Incident

- [ ] Verify memory usage stabilized
- [ ] Document optimal configuration
- [ ] Update resource allocations if undersized
- [ ] Set up trend-based alerting
- [ ] Review capacity planning

## Related Runbooks

- [High CPU](high-cpu.md)
- [Disk Full](disk-full.md)
- [Node Not Syncing](node-not-syncing.md)
