# Upgrade Procedures

> Zero-downtime upgrade procedures for Guts nodes.

## Pre-Upgrade Checklist

Before any upgrade:

- [ ] Review [release notes](https://github.com/guts-network/guts/releases) for breaking changes
- [ ] Check version compatibility matrix
- [ ] Create backup of current installation
- [ ] Test upgrade in staging environment
- [ ] Schedule maintenance window (for major upgrades)
- [ ] Notify stakeholders
- [ ] Verify rollback procedure

## Version Compatibility

| Upgrade Path | Data Migration | Downtime Required |
|--------------|----------------|-------------------|
| 1.0.x → 1.0.y | None | No |
| 1.x.0 → 1.y.0 | Automatic | No |
| 1.x.x → 2.0.0 | Manual | Yes |

## Upgrade Methods

### Method 1: Rolling Upgrade (Kubernetes)

For zero-downtime upgrades in Kubernetes:

```bash
# 1. Update Helm values or image tag
helm upgrade guts-node guts/guts-node \
  --namespace guts \
  --set image.tag=v1.1.0

# 2. Monitor rollout
kubectl rollout status statefulset/guts-node -n guts

# 3. Verify pods are healthy
kubectl get pods -n guts -l app.kubernetes.io/name=guts-node

# 4. Check logs for errors
kubectl logs -n guts -l app.kubernetes.io/name=guts-node --tail=50
```

### Method 2: Blue-Green Deployment

Deploy new version alongside old, then switch traffic:

```bash
# 1. Deploy new version
docker run -d \
  --name guts-node-new \
  -p 8081:8080 \
  -v guts-data:/data \
  ghcr.io/guts-network/guts-node:v1.1.0

# 2. Wait for sync
curl http://localhost:8081/health/ready
guts-node status --node http://localhost:8081 --wait-sync

# 3. Verify new version works
curl http://localhost:8081/api/repos

# 4. Switch traffic (update load balancer/proxy)
# ... update nginx/haproxy/etc ...

# 5. Stop old version
docker stop guts-node-old
docker rm guts-node-old

# 6. Rename new container
docker rename guts-node-new guts-node
```

### Method 3: In-Place Upgrade (Single Node)

For single-node deployments with brief downtime:

```bash
# 1. Create backup
/usr/local/bin/guts-backup.sh

# 2. Download new version
VERSION="v1.1.0"
curl -sSL "https://github.com/guts-network/guts/releases/download/${VERSION}/guts-node-linux-amd64" \
  -o /tmp/guts-node-new

# 3. Verify binary
/tmp/guts-node-new --version

# 4. Stop service
sudo systemctl stop guts-node

# 5. Backup current binary
sudo cp /usr/local/bin/guts-node /usr/local/bin/guts-node.bak

# 6. Install new binary
sudo install -m 755 /tmp/guts-node-new /usr/local/bin/guts-node

# 7. Start service
sudo systemctl start guts-node

# 8. Verify
sudo systemctl status guts-node
curl http://localhost:8080/health/ready
guts-node --version
```

### Method 4: Canary Deployment

Gradually roll out to subset of nodes:

```bash
# 1. Upgrade 1 node (canary)
kubectl set image statefulset/guts-node \
  guts-node=ghcr.io/guts-network/guts-node:v1.1.0 \
  -n guts

# 2. Pause rollout after 1 pod
kubectl rollout pause statefulset/guts-node -n guts

# 3. Monitor canary for 1 hour
# Check metrics, logs, errors

# 4. If healthy, continue rollout
kubectl rollout resume statefulset/guts-node -n guts

# 5. If issues, rollback
kubectl rollout undo statefulset/guts-node -n guts
```

## Docker Upgrade

```bash
# 1. Pull new image
docker pull ghcr.io/guts-network/guts-node:v1.1.0

# 2. Stop current container
docker stop guts-node

# 3. Remove container (keeps volumes)
docker rm guts-node

# 4. Start with new image
docker run -d \
  --name guts-node \
  --restart unless-stopped \
  -p 8080:8080 \
  -p 9000:9000 \
  -v guts-data:/data \
  ghcr.io/guts-network/guts-node:v1.1.0

# 5. Verify
docker logs guts-node | head -20
curl http://localhost:8080/health/ready
```

## Docker Compose Upgrade

```bash
# 1. Update image tag in docker-compose.yml
# image: ghcr.io/guts-network/guts-node:v1.1.0

# 2. Pull new image
docker compose pull guts-node

# 3. Recreate container
docker compose up -d guts-node

# 4. Verify
docker compose logs guts-node | head -20
```

## Systemd Upgrade

```bash
# 1. Download new binary
sudo curl -sSL "https://github.com/guts-network/guts/releases/download/v1.1.0/guts-node-linux-amd64" \
  -o /tmp/guts-node-new
sudo chmod +x /tmp/guts-node-new

# 2. Verify version
/tmp/guts-node-new --version

# 3. Stop service
sudo systemctl stop guts-node

# 4. Replace binary
sudo mv /usr/local/bin/guts-node /usr/local/bin/guts-node.bak
sudo mv /tmp/guts-node-new /usr/local/bin/guts-node

# 5. Reload systemd (if service file changed)
sudo systemctl daemon-reload

# 6. Start service
sudo systemctl start guts-node

# 7. Verify
sudo systemctl status guts-node
sudo journalctl -u guts-node --since "1 minute ago"
```

## Data Migration

### Automatic Migration

Most upgrades include automatic data migration:

```bash
# During startup, migration runs automatically
guts-node --config /etc/guts/config.yaml

# Logs will show:
# INFO Checking for pending migrations...
# INFO Running migration: v1.0.0 → v1.1.0
# INFO Migration completed successfully
```

### Manual Migration

For major version upgrades:

```bash
# 1. Stop service
sudo systemctl stop guts-node

# 2. Backup data
guts-node backup create --output /backup/pre-migration.tar.gz

# 3. Run migration tool
guts-node migrate --from 1.0.0 --to 2.0.0 --data-dir /var/lib/guts

# 4. Verify migration
guts-node verify --data-dir /var/lib/guts --full

# 5. Start service
sudo systemctl start guts-node
```

## Rollback Procedures

### Quick Rollback (Binary)

```bash
# 1. Stop service
sudo systemctl stop guts-node

# 2. Restore previous binary
sudo mv /usr/local/bin/guts-node.bak /usr/local/bin/guts-node

# 3. Start service
sudo systemctl start guts-node
```

### Full Rollback (With Data)

```bash
# 1. Stop service
sudo systemctl stop guts-node

# 2. Restore previous binary
sudo mv /usr/local/bin/guts-node.bak /usr/local/bin/guts-node

# 3. Restore data from backup
guts-node backup restore /backup/pre-upgrade.tar.gz

# 4. Start service
sudo systemctl start guts-node
```

### Kubernetes Rollback

```bash
# Rollback to previous revision
kubectl rollout undo statefulset/guts-node -n guts

# Rollback to specific revision
kubectl rollout undo statefulset/guts-node -n guts --to-revision=2

# Check rollout history
kubectl rollout history statefulset/guts-node -n guts
```

## Validator Upgrade Coordination

For validator networks, coordinate upgrades:

### Upgrade Order

1. Upgrade non-leader validators first
2. Wait for sync confirmation
3. Upgrade remaining validators
4. Never upgrade more than f validators simultaneously

### Upgrade Script

```bash
#!/bin/bash
# validator-upgrade.sh

VALIDATORS=("validator1" "validator2" "validator3" "validator4")
VERSION="v1.1.0"

for validator in "${VALIDATORS[@]}"; do
    echo "Upgrading $validator..."

    # Check if leader
    if guts-node consensus status --node "http://$validator:8080" | grep -q "role: leader"; then
        echo "Skipping leader $validator, will upgrade last"
        continue
    fi

    # Upgrade
    ssh "$validator" "
        sudo systemctl stop guts-node
        sudo curl -sSL https://github.com/guts-network/guts/releases/download/$VERSION/guts-node-linux-amd64 -o /usr/local/bin/guts-node
        sudo chmod +x /usr/local/bin/guts-node
        sudo systemctl start guts-node
    "

    # Wait for sync
    echo "Waiting for $validator to sync..."
    until guts-node status --node "http://$validator:8080" | grep -q "synced: true"; do
        sleep 5
    done

    echo "$validator upgraded successfully"
    sleep 30  # Stabilization period
done

echo "All validators upgraded"
```

## Post-Upgrade Verification

### Health Checks

```bash
# Check service status
systemctl status guts-node

# Check version
guts-node --version

# Check API health
curl http://localhost:8080/health/ready | jq

# Check consensus (validators)
curl http://localhost:8080/api/consensus/status | jq

# Check peer connectivity
curl http://localhost:8080/api/consensus/validators | jq
```

### Functional Tests

```bash
# Create test repository
curl -X POST http://localhost:8080/api/repos \
  -H "Content-Type: application/json" \
  -d '{"name": "upgrade-test", "owner": "test"}'

# Verify Git operations
git clone http://localhost:8080/git/test/upgrade-test /tmp/upgrade-test
cd /tmp/upgrade-test
echo "test" > README.md
git add . && git commit -m "Test commit"
git push origin main

# Cleanup
rm -rf /tmp/upgrade-test
curl -X DELETE http://localhost:8080/api/repos/test/upgrade-test
```

### Metrics Verification

```bash
# Check key metrics
curl -s http://localhost:9090/metrics | grep -E "^guts_" | head -20

# Compare with baseline
# - Request latency should be similar
# - Error rates should not increase
# - Memory/CPU usage should be stable
```

## Troubleshooting Upgrades

### Service Won't Start After Upgrade

```bash
# Check logs
sudo journalctl -u guts-node --since "5 minutes ago"

# Common issues:
# - Config format changed: Update config file
# - Missing migrations: Run migration manually
# - Permission issues: Check file ownership
```

### Data Corruption After Upgrade

```bash
# 1. Stop service
sudo systemctl stop guts-node

# 2. Rollback to previous version
sudo mv /usr/local/bin/guts-node.bak /usr/local/bin/guts-node

# 3. Restore from backup
guts-node backup restore /backup/pre-upgrade.tar.gz

# 4. Start service
sudo systemctl start guts-node

# 5. Report issue
# Include: version numbers, logs, error messages
```

### Consensus Issues After Upgrade

```bash
# Check if validators are on same version
for node in validator{1..4}; do
    echo "$node: $(ssh $node 'guts-node --version')"
done

# Check block heights
for node in validator{1..4}; do
    echo "$node: $(curl -s http://$node:8080/api/consensus/status | jq .block_height)"
done

# If heights diverge, may need coordinated rollback
```

## Related Documentation

- [Backup & Recovery](backup.md)
- [Monitoring Guide](monitoring.md)
- [Runbooks](../runbooks/)
