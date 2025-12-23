# Systemd Service Configuration

> Detailed guide for configuring and managing Guts nodes with systemd.

## Service File

### Basic Service

```ini
# /etc/systemd/system/guts-node.service
[Unit]
Description=Guts Node - Decentralized Code Collaboration
Documentation=https://docs.guts.network
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=guts
Group=guts
ExecStart=/usr/local/bin/guts-node --config /etc/guts/config.yaml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Production-Hardened Service

```ini
# /etc/systemd/system/guts-node.service
[Unit]
Description=Guts Node - Decentralized Code Collaboration
Documentation=https://docs.guts.network
After=network-online.target local-fs.target
Wants=network-online.target
StartLimitIntervalSec=500
StartLimitBurst=5

[Service]
Type=simple
User=guts
Group=guts

# Execution
ExecStart=/usr/local/bin/guts-node --config /etc/guts/config.yaml
ExecReload=/bin/kill -HUP $MAINPID
ExecStop=/bin/kill -TERM $MAINPID

# Restart behavior
Restart=on-failure
RestartSec=5
TimeoutStartSec=60
TimeoutStopSec=60

# Security hardening
NoNewPrivileges=yes
PrivateTmp=yes
PrivateDevices=yes
ProtectSystem=strict
ProtectHome=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectKernelLogs=yes
ProtectControlGroups=yes
RestrictSUIDSGID=yes
RestrictNamespaces=yes
RestrictRealtime=yes
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX
LockPersonality=yes
MemoryDenyWriteExecute=yes
SystemCallArchitectures=native
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources

# Capabilities
CapabilityBoundingSet=
AmbientCapabilities=

# Allow write to data directory
ReadWritePaths=/var/lib/guts
ReadOnlyPaths=/etc/guts

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=guts-node

# Resource limits
LimitNOFILE=65535
LimitNPROC=32768
LimitCORE=0
MemoryMax=32G
CPUQuota=800%
TasksMax=4096

# Watchdog
WatchdogSec=60

[Install]
WantedBy=multi-user.target
```

## Service Management

### Basic Commands

```bash
# Enable service (start on boot)
sudo systemctl enable guts-node

# Start service
sudo systemctl start guts-node

# Stop service
sudo systemctl stop guts-node

# Restart service
sudo systemctl restart guts-node

# Reload configuration (if supported)
sudo systemctl reload guts-node

# Check status
sudo systemctl status guts-node

# Disable service
sudo systemctl disable guts-node
```

### Viewing Logs

```bash
# Follow logs in real-time
sudo journalctl -u guts-node -f

# View recent logs
sudo journalctl -u guts-node --since "1 hour ago"

# View logs from boot
sudo journalctl -u guts-node -b

# View errors only
sudo journalctl -u guts-node -p err

# Output as JSON
sudo journalctl -u guts-node -o json-pretty

# Limit output lines
sudo journalctl -u guts-node -n 100
```

### Checking Service Health

```bash
# Detailed status
systemctl show guts-node

# Show specific properties
systemctl show guts-node --property=ActiveState,SubState,MainPID

# Check if enabled
systemctl is-enabled guts-node

# Check if active
systemctl is-active guts-node

# List failed units
systemctl --failed
```

## Multiple Instances

Run multiple Guts nodes on the same machine using template units:

### Template Unit

```ini
# /etc/systemd/system/guts-node@.service
[Unit]
Description=Guts Node %i
Documentation=https://docs.guts.network
After=network-online.target

[Service]
Type=simple
User=guts
Group=guts
ExecStart=/usr/local/bin/guts-node --config /etc/guts/node-%i.yaml
Restart=always
RestartSec=5

# Instance-specific data directory
Environment=GUTS_DATA_DIR=/var/lib/guts/node-%i

# Security
NoNewPrivileges=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/guts/node-%i

[Install]
WantedBy=multi-user.target
```

### Managing Instances

```bash
# Create instance configurations
sudo cp /etc/guts/config.yaml /etc/guts/node-1.yaml
sudo cp /etc/guts/config.yaml /etc/guts/node-2.yaml

# Edit configurations (different ports)
sudo sed -i 's/8080/8081/' /etc/guts/node-1.yaml
sudo sed -i 's/8080/8082/' /etc/guts/node-2.yaml

# Create data directories
sudo mkdir -p /var/lib/guts/node-{1,2}
sudo chown guts:guts /var/lib/guts/node-{1,2}

# Enable and start instances
sudo systemctl enable guts-node@1 guts-node@2
sudo systemctl start guts-node@1 guts-node@2

# Check status
sudo systemctl status 'guts-node@*'
```

## Socket Activation

For faster startup and better resource management:

### Socket Unit

```ini
# /etc/systemd/system/guts-node.socket
[Unit]
Description=Guts Node Socket

[Socket]
ListenStream=8080
ListenDatagram=9000

# Accept connections
Accept=no

# Backlog
Backlog=4096

[Install]
WantedBy=sockets.target
```

### Socket-Activated Service

```ini
# /etc/systemd/system/guts-node.service
[Unit]
Description=Guts Node
Requires=guts-node.socket
After=guts-node.socket

[Service]
Type=simple
User=guts
ExecStart=/usr/local/bin/guts-node --config /etc/guts/config.yaml
StandardInput=socket
StandardOutput=journal

[Install]
WantedBy=multi-user.target
```

## Resource Management with cgroups

### CPU Limits

```ini
[Service]
# Limit to 4 CPU cores (400%)
CPUQuota=400%

# Set CPU weight (relative priority)
CPUWeight=100

# Set CPU affinity
CPUAffinity=0-3
```

### Memory Limits

```ini
[Service]
# Hard memory limit
MemoryMax=32G

# Soft memory limit (triggers reclaim)
MemoryHigh=28G

# Swap limit
MemorySwapMax=4G
```

### I/O Limits

```ini
[Service]
# I/O weight (1-10000, default 100)
IOWeight=500

# Read bandwidth limit
IOReadBandwidthMax=/dev/sda 100M

# Write bandwidth limit
IOWriteBandwidthMax=/dev/sda 50M
```

### Combined Resource Slice

```ini
# /etc/systemd/system/guts.slice
[Unit]
Description=Guts Services Slice
Before=slices.target

[Slice]
CPUQuota=800%
MemoryMax=64G
IOWeight=500
```

```ini
# In guts-node.service
[Service]
Slice=guts.slice
```

## Environment Files

### Using Environment File

```ini
[Service]
EnvironmentFile=/etc/guts/environment
ExecStart=/usr/local/bin/guts-node
```

```bash
# /etc/guts/environment
GUTS_API_ADDR=0.0.0.0:8080
GUTS_P2P_ADDR=0.0.0.0:9000
GUTS_LOG_LEVEL=info
GUTS_DATA_DIR=/var/lib/guts
```

### Secure Credentials

```ini
[Service]
# Load credentials from secure location
LoadCredential=node.key:/etc/guts/credentials/node.key
Environment=GUTS_PRIVATE_KEY_FILE=%d/node.key
```

## Watchdog Integration

### Enable Watchdog

```ini
[Service]
WatchdogSec=60
NotifyAccess=main

# Action on watchdog timeout
WatchdogSignal=SIGKILL
TimeoutAbortSec=90
```

### Application Support

The application must periodically notify systemd:

```rust
// In guts-node Rust code
use systemd::daemon;

fn main_loop() {
    loop {
        // Do work...

        // Notify systemd we're alive
        daemon::notify(false, &[daemon::Notification::WatchdogTick]).ok();
    }
}
```

## Integration with Other Services

### Dependency on Database

```ini
[Unit]
After=postgresql.service
Requires=postgresql.service
```

### Start After Network

```ini
[Unit]
After=network-online.target
Wants=network-online.target

# Wait for specific port
ExecStartPre=/bin/bash -c 'until nc -z localhost 5432; do sleep 1; done'
```

### Ordering Multiple Services

```ini
# guts-consensus.service
[Unit]
Before=guts-node.service

# guts-node.service
[Unit]
After=guts-consensus.service
BindsTo=guts-consensus.service
```

## Maintenance Operations

### Scheduled Maintenance

```ini
# /etc/systemd/system/guts-maintenance.service
[Unit]
Description=Guts Maintenance Tasks

[Service]
Type=oneshot
User=guts
ExecStart=/usr/local/bin/guts-maintenance.sh
```

```ini
# /etc/systemd/system/guts-maintenance.timer
[Unit]
Description=Guts Weekly Maintenance

[Timer]
OnCalendar=Sun 02:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

### Backup Timer

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

## Debugging

### Analyze Boot Time

```bash
# Show service startup time
systemd-analyze blame | grep guts

# Show critical chain
systemd-analyze critical-chain guts-node.service
```

### Debug Mode

```bash
# Start manually for debugging
sudo -u guts /usr/local/bin/guts-node --config /etc/guts/config.yaml

# Run with strace
sudo strace -f -p $(pgrep guts-node)

# Check cgroup limits
systemctl show guts-node | grep -E 'Memory|CPU'
cat /sys/fs/cgroup/system.slice/guts-node.service/memory.current
```

### Common Issues

```bash
# Service fails to start
sudo journalctl -u guts-node --no-pager | tail -50

# Permission denied
namei -l /var/lib/guts
getfacl /var/lib/guts

# Socket issues
ss -tlnp | grep 8080

# Reload after editing service file
sudo systemctl daemon-reload
```

## Security Verification

### Verify Hardening

```bash
# Check security settings
systemd-analyze security guts-node

# Expected output shows security score
# Lower number = more secure
```

### Audit Service Capabilities

```bash
# Check what capabilities the service has
grep Cap /proc/$(pgrep guts-node)/status

# Check security context
ps -eZ | grep guts-node
```

## Reference

### Service States

| State | Description |
|-------|-------------|
| active (running) | Service is running |
| active (exited) | Service ran and exited |
| inactive (dead) | Service is stopped |
| failed | Service failed to start |
| activating | Service is starting |
| deactivating | Service is stopping |

### Exit Codes

| Code | Description | Restart Behavior |
|------|-------------|------------------|
| 0 | Success | Restart=on-failure: No |
| 1-255 | Failure | Restart=on-failure: Yes |
| SIGTERM | Normal stop | Restart=always: Yes |
| SIGKILL | Force stop | Restart=always: Yes |

## Next Steps

- [Configure networking](../configuration/networking.md)
- [Set up monitoring](../operations/monitoring.md)
- [Configure backups](../operations/backup.md)
