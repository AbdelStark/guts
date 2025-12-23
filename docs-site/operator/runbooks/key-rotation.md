# Runbook: Key Rotation

**Severity:** P3
**Impact:** Planned maintenance, brief service interruption
**On-Call Action:** Scheduled maintenance

## Overview

Key rotation is a security best practice that should be performed:

- Periodically (e.g., annually)
- After personnel changes
- After security incidents
- When keys may have been compromised

## Types of Keys

| Key Type | Location | Rotation Impact |
|----------|----------|-----------------|
| Node Private Key | `/etc/guts/node.key` | Changes node identity |
| TLS Certificates | `/etc/guts/tls/` | Requires restart |
| API Tokens | Database | Can revoke/regenerate |

## Pre-Rotation Checklist

- [ ] Schedule maintenance window
- [ ] Notify stakeholders
- [ ] Backup current keys
- [ ] Prepare new keys
- [ ] Test rotation in staging

## Procedure: Node Private Key Rotation

### For Full Nodes (Non-Validator)

Node identity can change without network impact:

```bash
# 1. Generate new key
guts-node keygen > /tmp/new-node.key

# 2. Backup old key
cp /etc/guts/node.key /etc/guts/node.key.bak-$(date +%Y%m%d)

# 3. Stop node
sudo systemctl stop guts-node

# 4. Install new key
sudo cp /tmp/new-node.key /etc/guts/node.key
sudo chmod 600 /etc/guts/node.key
sudo chown guts:guts /etc/guts/node.key

# 5. Start node
sudo systemctl start guts-node

# 6. Verify new identity
curl -s http://localhost:8080/api/consensus/status | jq '.node_id'

# 7. Securely delete temp file
shred -u /tmp/new-node.key
```

### For Validators

Validator key rotation requires coordination:

#### Step 1: Prepare New Key

```bash
# Generate new validator key
guts-node keygen > /tmp/new-validator.key

# Extract public key
NEW_PUBKEY=$(head -1 /tmp/new-validator.key)
echo "New public key: $NEW_PUBKEY"
```

#### Step 2: Update Genesis/Validator Set

Coordinate with other validators to update the validator set:

```bash
# Option A: If governance supports it
guts-node governance propose-validator-change \
  --old-key $(head -1 /etc/guts/node.key) \
  --new-key $NEW_PUBKEY

# Option B: If requires genesis update
# All validators must agree and update genesis.json
```

#### Step 3: Wait for Approval

```bash
# Check proposal status
guts-node governance proposal-status --proposal-id <id>
```

#### Step 4: Rotate When Approved

```bash
# 1. Backup old key
cp /etc/guts/node.key /etc/guts/node.key.bak-$(date +%Y%m%d)

# 2. Stop validator (coordinate timing with other validators)
sudo systemctl stop guts-node

# 3. Install new key
sudo cp /tmp/new-validator.key /etc/guts/node.key
sudo chmod 600 /etc/guts/node.key
sudo chown guts:guts /etc/guts/node.key

# 4. Start validator
sudo systemctl start guts-node

# 5. Verify participating in consensus
curl -s http://localhost:8080/api/consensus/status | jq
```

## Procedure: TLS Certificate Rotation

### Using Let's Encrypt (Automatic)

If using certbot, certificates renew automatically:

```bash
# Check renewal status
sudo certbot certificates

# Test renewal
sudo certbot renew --dry-run

# Force renewal if needed
sudo certbot renew --force-renewal

# Restart to pick up new certs
sudo systemctl restart guts-node
```

### Manual Certificate Rotation

```bash
# 1. Obtain new certificate
# (From your CA or generate self-signed for testing)

# 2. Backup old certs
cp /etc/guts/tls/cert.pem /etc/guts/tls/cert.pem.bak
cp /etc/guts/tls/key.pem /etc/guts/tls/key.pem.bak

# 3. Install new certs
cp new-cert.pem /etc/guts/tls/cert.pem
cp new-key.pem /etc/guts/tls/key.pem
chmod 600 /etc/guts/tls/*.pem

# 4. Verify certificate
openssl x509 -in /etc/guts/tls/cert.pem -text -noout | grep -E "Not After|Subject:"

# 5. Restart node
sudo systemctl restart guts-node

# 6. Verify HTTPS works
curl -v https://localhost:8443/health
```

## Procedure: API Token Rotation

### Revoke Compromised Token

```bash
# List user's tokens
curl -X GET http://localhost:8080/api/user/tokens \
  -H "Authorization: Bearer $ADMIN_TOKEN" | jq

# Revoke specific token
curl -X DELETE http://localhost:8080/api/user/tokens/<token-id> \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

### Generate New Token

```bash
# Create new token
curl -X POST http://localhost:8080/api/user/tokens \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"note": "Rotated token", "scopes": ["repo:read", "repo:write"]}' | jq
```

### Rotate All User Tokens

For security incidents affecting all tokens:

```bash
# This invalidates ALL tokens
guts-node admin revoke-all-tokens --confirm

# Users must re-authenticate
```

## Automation

### Scheduled Key Rotation Script

```bash
#!/bin/bash
# /usr/local/bin/guts-rotate-keys.sh

set -euo pipefail

LOG="/var/log/guts-key-rotation.log"
BACKUP_DIR="/var/backups/guts-keys"

log() {
    echo "$(date -Iseconds) $*" | tee -a "$LOG"
}

# Create backup directory
mkdir -p "$BACKUP_DIR"

log "Starting key rotation..."

# Backup current key
cp /etc/guts/node.key "$BACKUP_DIR/node.key.$(date +%Y%m%d)"

# Generate new key
guts-node keygen > /tmp/new-node.key

# Stop, rotate, start
log "Stopping node..."
systemctl stop guts-node

log "Installing new key..."
cp /tmp/new-node.key /etc/guts/node.key
chmod 600 /etc/guts/node.key
chown guts:guts /etc/guts/node.key

log "Starting node..."
systemctl start guts-node

# Cleanup
shred -u /tmp/new-node.key

# Verify
sleep 10
if curl -s http://localhost:8080/health/ready | grep -q '"status":"up"'; then
    log "Key rotation completed successfully"
else
    log "ERROR: Node not healthy after rotation"
    exit 1
fi

# Cleanup old backups (keep 5)
ls -t "$BACKUP_DIR"/node.key.* | tail -n +6 | xargs -r rm
```

### Schedule with Cron

```bash
# Annual key rotation
0 2 1 1 * root /usr/local/bin/guts-rotate-keys.sh
```

## Verification

After any key rotation:

```bash
# Check node identity
curl -s http://localhost:8080/api/consensus/status | jq '.node_id'

# Verify TLS certificate
echo | openssl s_client -connect localhost:8443 2>/dev/null | openssl x509 -noout -dates

# Test API authentication
curl -s http://localhost:8080/api/user -H "Authorization: Bearer $NEW_TOKEN" | jq
```

## Rollback

If rotation fails:

```bash
# Stop node
sudo systemctl stop guts-node

# Restore old key
sudo cp /etc/guts/node.key.bak-* /etc/guts/node.key
sudo chmod 600 /etc/guts/node.key

# Start node
sudo systemctl start guts-node
```

## Related Runbooks

- [Security Incident](security-incident.md)
- [Emergency Shutdown](emergency-shutdown.md)
