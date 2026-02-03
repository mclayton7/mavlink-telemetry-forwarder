# Deployment Guide

## Quick Start

### Building

```bash
# Build all components
cargo build --release

# Binaries will be in:
# - target/release/mavlink-sender
# - target/release/mavlink-receiver
```

### Running on Aircraft (Sender)

**Option 1: Direct execution**
```bash
./target/release/mavlink-sender --listen 0.0.0.0:14551 --forward 192.168.1.100:14550
```

**Option 2: Docker**
```bash
# Build image
docker build -f sender/Dockerfile -t mavlink-sender .

# Run with docker-compose
GROUND_ADDR=192.168.1.100:14550 docker-compose up -d
```

**Option 3: systemd service**
```bash
# Copy binary
sudo cp target/release/mavlink-sender /usr/local/bin/

# Create service file
sudo tee /etc/systemd/system/mavlink-sender.service << 'EOF'
[Unit]
Description=MAVLink Telemetry Sender
After=network.target mavlink-router.service
Requires=mavlink-router.service

[Service]
Type=simple
User=pi
Environment=GROUND_ADDR=192.168.1.100:14550
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/mavlink-sender --listen 0.0.0.0:14551 --forward ${GROUND_ADDR}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable mavlink-sender
sudo systemctl start mavlink-sender

# Check status
sudo systemctl status mavlink-sender
```

### Running on Ground Station (Receiver)

**Terminal or background process**
```bash
./target/release/mavlink-receiver --listen 0.0.0.0:14550 --output ~/mavlink-logs --max-file-size 100
```

**systemd service**
```bash
# Copy binary
sudo cp target/release/mavlink-receiver /usr/local/bin/

# Create service file
sudo tee /etc/systemd/system/mavlink-receiver.service << 'EOF'
[Unit]
Description=MAVLink Telemetry Receiver
After=network.target

[Service]
Type=simple
User=groundstation
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/mavlink-receiver --listen 0.0.0.0:14550 --output /home/groundstation/mavlink-logs --max-file-size 100
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable mavlink-receiver
sudo systemctl start mavlink-receiver

# Check status
sudo systemctl status mavlink-receiver
```

## Configuration

### mavlink-router Configuration

On the aircraft, configure mavlink-router to send telemetry to the sender:

Edit `/etc/mavlink-router/main.conf`:

```ini
[General]
TcpServerPort=5760
ReportStats=false
MavlinkDialect=auto

# Serial connection to flight controller
[UartEndpoint alpha]
Device=/dev/ttyAMA0
Baud=921600

# Forward to sender application
[UdpEndpoint sender]
Mode=Normal
Address=127.0.0.1
Port=14551
```

Restart mavlink-router:
```bash
sudo systemctl restart mavlink-router
```

### Network Configuration

**Aircraft:**
- Ensure sender can reach ground station IP
- For cellular/LTE, configure static IP or DDNS
- Test connectivity: `ping <ground-station-ip>`

**Ground Station:**
- Open firewall for UDP port 14550:
  ```bash
  # Linux (ufw)
  sudo ufw allow 14550/udp

  # Linux (iptables)
  sudo iptables -A INPUT -p udp --dport 14550 -j ACCEPT

  # macOS
  # Add rule in System Preferences > Security > Firewall > Options
  ```

- For dynamic IP, use DDNS service (e.g., DuckDNS, No-IP)

### Environment Variables

Create `.env` file:
```bash
cp .env.example .env
# Edit .env with your ground station address
```

```
GROUND_ADDR=192.168.1.100:14550
RUST_LOG=info
```

## Docker Deployment

### Build

```bash
docker build -f sender/Dockerfile -t mavlink-sender .
```

Verify image size:
```bash
docker images mavlink-sender
# Should be ~5-10 MB
```

### Run with Docker Compose

```bash
# Create .env file with your settings
cat > .env << EOF
GROUND_ADDR=192.168.1.100:14550
RUST_LOG=info
EOF

# Start service
docker-compose up -d

# View logs
docker-compose logs -f

# Stop service
docker-compose down
```

### Run Manually

```bash
docker run -d \
  --name mavlink-sender \
  --network host \
  --restart unless-stopped \
  -e GROUND_ADDR=192.168.1.100:14550 \
  -e RUST_LOG=info \
  mavlink-sender
```

## Monitoring

### Logs

**Sender:**
```bash
# systemd
sudo journalctl -u mavlink-sender -f

# Docker
docker-compose logs -f mavlink-sender
```

**Receiver:**
```bash
# systemd
sudo journalctl -u mavlink-receiver -f

# Direct execution (already printed to terminal)
```

### Health Checks

**Check if sender is running:**
```bash
# systemd
systemctl status mavlink-sender

# Docker
docker ps | grep mavlink-sender

# Process
pgrep -f mavlink-sender
```

**Check if receiving data:**
```bash
# On ground station, watch tlog directory
watch -n 1 'ls -lh ~/mavlink-logs/*.tlog | tail -5'

# Monitor file growth
watch -n 1 'stat ~/mavlink-logs/telemetry_*.tlog | grep Size'
```

**Network connectivity test:**
```bash
# From aircraft, test UDP to ground
echo "test" | nc -u <ground-ip> 14550

# From ground, listen for packets
nc -ul 14550
```

## Troubleshooting

### Sender not forwarding packets

1. Check mavlink-router is sending to port 14551:
   ```bash
   sudo tcpdump -i lo port 14551
   ```

2. Check sender logs for errors:
   ```bash
   sudo journalctl -u mavlink-sender -n 50
   ```

3. Verify network connectivity to ground station:
   ```bash
   ping <ground-ip>
   traceroute <ground-ip>
   ```

### Receiver not writing tlog files

1. Check receiver is listening:
   ```bash
   sudo ss -ulnp | grep 14550
   ```

2. Check for incoming packets:
   ```bash
   sudo tcpdump -i any port 14550
   ```

3. Check disk space:
   ```bash
   df -h ~/mavlink-logs
   ```

4. Check permissions:
   ```bash
   ls -ld ~/mavlink-logs
   touch ~/mavlink-logs/test.txt
   ```

### Firewall issues

**Test UDP connectivity:**
```bash
# From aircraft
echo "test" | nc -u <ground-ip> 14550

# On ground station
nc -ul 14550
```

**Check firewall rules:**
```bash
# Linux
sudo iptables -L -n | grep 14550
sudo ufw status

# Check if packets are being dropped
sudo iptables -L -v -n
```

### High CPU or memory usage

Check resource usage:
```bash
top -p $(pgrep mavlink-sender)
top -p $(pgrep mavlink-receiver)
```

Expected values:
- CPU: <1% per process
- Memory: <10 MB per process

If higher, check for:
- Network flooding (too many packets)
- Disk I/O issues (slow storage)

## Maintenance

### Log rotation and cleanup

**Automatic rotation:**
Receiver automatically rotates files at configured size (default 100 MB).

**Manual cleanup:**
```bash
# Compress old logs
find ~/mavlink-logs -name "*.tlog" -mtime +7 -exec gzip {} \;

# Archive to backup
tar -czf mavlink-logs-$(date +%Y%m%d).tar.gz ~/mavlink-logs/*.tlog.gz
mv mavlink-logs-*.tar.gz ~/backups/

# Delete old compressed logs
find ~/mavlink-logs -name "*.tlog.gz" -mtime +30 -delete
```

**Set up automatic cleanup with cron:**
```bash
crontab -e

# Add this line to compress logs older than 7 days daily at 2 AM
0 2 * * * find ~/mavlink-logs -name "*.tlog" -mtime +7 -exec gzip {} \;

# Delete compressed logs older than 30 days
0 3 * * * find ~/mavlink-logs -name "*.tlog.gz" -mtime +30 -delete
```

### Updates

```bash
# Pull latest code
git pull

# Rebuild
cargo build --release

# Restart services
sudo systemctl restart mavlink-sender
sudo systemctl restart mavlink-receiver

# Or for Docker
docker-compose down
docker-compose build
docker-compose up -d
```

## Security Considerations

1. **Network security:**
   - Use VPN for public internet deployment
   - Consider encryption layer (WireGuard, etc.)
   - Restrict firewall rules to known IPs if possible

2. **File permissions:**
   - Run receiver as non-root user
   - Ensure log directory has appropriate permissions

3. **Resource limits:**
   - Consider setting systemd resource limits:
     ```ini
     [Service]
     MemoryMax=50M
     CPUQuota=10%
     ```

## Performance Tuning

### For high packet rates (>100 packets/sec)

**Increase flush interval** in `receiver/src/main.rs`:
```rust
const FLUSH_INTERVAL_PACKETS: usize = 500; // Instead of 100
```

**Use faster storage** for tlog files (SSD instead of SD card)

**Increase UDP buffer size** (in code or via sysctl):
```bash
sudo sysctl -w net.core.rmem_max=8388608
sudo sysctl -w net.core.rmem_default=8388608
```

## Backup Strategy

### Critical flights

Before important flights:
1. Ensure sufficient disk space (at least 10 GB free)
2. Verify receiver is running and writing files
3. Test end-to-end connectivity
4. Have backup recording method (local tlog on aircraft)

### Post-flight

1. Copy tlog files to safe location immediately:
   ```bash
   rsync -av ~/mavlink-logs/ ~/backups/flight-$(date +%Y%m%d)/
   ```

2. Verify file integrity:
   ```bash
   # Check tlog format
   hexdump -C telemetry_*.tlog | head -10

   # Try opening in Mission Planner
   ```

3. Upload to cloud storage (optional):
   ```bash
   rclone copy ~/mavlink-logs/ remote:mavlink-backups/
   ```
