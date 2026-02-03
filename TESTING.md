# Testing Guide

This document provides detailed testing instructions for the MAVLink Telemetry Sender system.

## Unit Tests

Run all unit tests:
```bash
cargo test
```

Run tests with verbose output:
```bash
cargo test -- --nocapture
```

## Integration Tests

### 1. Test Sender Forwarding

**Terminal 1: Start sender**
```bash
cargo run --bin mavlink-sender -- --listen 127.0.0.1:14551 --forward 127.0.0.1:14550
```

**Terminal 2: Start netcat listener (simulates receiver)**
```bash
nc -ul 127.0.0.1 14550
```

**Terminal 3: Send test MAVLink packet**
```bash
# MAVLink 2.0 HEARTBEAT packet (simplified)
echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14551
```

**Expected result:**
- Terminal 2 should display the received packet
- Terminal 1 should show log messages about receiving and forwarding

### 2. Test Receiver Logging

**Terminal 1: Start receiver**
```bash
cargo run --bin mavlink-receiver -- --listen 127.0.0.1:14550 --output ./test-logs --max-file-size 1
```

**Terminal 2: Send test MAVLink packets**
```bash
# Send a valid MAVLink 2.0 packet
echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14550

# Send an invalid packet (should be rejected)
echo -n -e '\x00\x01\x02\x03\x04' | nc -u 127.0.0.1 14550

# Send a MAVLink 1.0 packet (should be accepted)
echo -n -e '\xfe\x09\x01\x02\x03\x04\x05\x06\x07\x08\x09' | nc -u 127.0.0.1 14550
```

**Terminal 3: Verify tlog file**
```bash
# Check file was created
ls -lh test-logs/

# View file contents in hex
hexdump -C test-logs/telemetry_*.tlog

# Expected format:
# [8 bytes: timestamp in LE u64]
# [N bytes: MAVLink packet starting with 0xFD or 0xFE]
```

**Expected result:**
- `test-logs/` directory contains `telemetry_YYYYMMDD_HHMMSS.tlog`
- File contains 8-byte timestamp + MAVLink packet
- Invalid packets are rejected (log warning shown)
- Valid packets are written to file

### 3. Test End-to-End

**Terminal 1: Start receiver**
```bash
cargo run --bin mavlink-receiver -- --listen 127.0.0.1:14550 --output ./test-logs
```

**Terminal 2: Start sender**
```bash
cargo run --bin mavlink-sender -- --listen 127.0.0.1:14551 --forward 127.0.0.1:14550
```

**Terminal 3: Send test packets**
```bash
# Send multiple packets
for i in {1..10}; do
    echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14551
    sleep 0.1
done
```

**Expected result:**
- Sender receives packets and forwards them
- Receiver writes all packets to tlog file
- File size increases with each packet (19 bytes per packet: 8 timestamp + 11 data)

### 4. Test File Rotation

**Terminal 1: Start receiver with small max file size**
```bash
cargo run --bin mavlink-receiver -- --listen 127.0.0.1:14550 --output ./test-logs --max-file-size 1
```

**Terminal 2: Send many packets**
```bash
# Send 1000 packets to trigger rotation
for i in {1..1000}; do
    echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14550
done
```

**Expected result:**
- Multiple tlog files created in `test-logs/`
- Each file is approximately 1 MB
- Files named with timestamps: `telemetry_YYYYMMDD_HHMMSS.tlog`

### 5. Test with mavproxy

If you have mavproxy installed:

**Terminal 1: Start receiver**
```bash
cargo run --bin mavlink-receiver -- --listen 127.0.0.1:14550 --output ./test-logs
```

**Terminal 2: Start sender**
```bash
cargo run --bin mavlink-sender -- --listen 127.0.0.1:14551 --forward 127.0.0.1:14550
```

**Terminal 3: Run mavproxy in simulation mode**
```bash
# Connect mavproxy to sender port
mavproxy.py --master=udp:127.0.0.1:14551
```

**Expected result:**
- Real MAVLink messages forwarded through sender to receiver
- Tlog file contains valid MAVLink telemetry
- Can be opened in Mission Planner or QGroundControl

## Docker Testing

### Build Docker Image

```bash
docker build -f sender/Dockerfile -t mavlink-sender .
```

**Expected result:**
- Build completes successfully
- Image size should be small (~5-10 MB)

Check image size:
```bash
docker images mavlink-sender
```

### Test Docker Container

**Terminal 1: Start receiver (host)**
```bash
cargo run --bin mavlink-receiver -- --listen 0.0.0.0:14550 --output ./test-logs
```

**Terminal 2: Run sender in Docker**
```bash
docker run --network host -e GROUND_ADDR=127.0.0.1:14550 mavlink-sender
```

**Terminal 3: Send test packets**
```bash
echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14551
```

**Expected result:**
- Docker container receives and forwards packets
- Receiver on host writes to tlog file

### Test Docker Compose

```bash
# Create .env file
cat > .env << EOF
GROUND_ADDR=127.0.0.1:14550
RUST_LOG=info
EOF

# Start with docker-compose
docker-compose up
```

## Validating tlog Files

### Check tlog Format

```bash
# View first 100 bytes in hex
hexdump -C test-logs/telemetry_*.tlog | head -20

# Expected format for each packet:
# Bytes 0-7:   Timestamp (LE u64, should be reasonable Unix microseconds)
# Byte 8:      MAVLink magic (0xFD or 0xFE)
# Remaining:   MAVLink payload
```

### Open in Mission Planner (Windows)

1. Open Mission Planner
2. Go to Flight Data tab
3. Click "Tlog > Ktlog" or use "Load Log" button
4. Select the .tlog file
5. Verify messages are displayed correctly

### Open in QGroundControl

1. Open QGroundControl
2. Go to Analyze Tools > Log Download
3. Load the .tlog file
4. Verify telemetry data is readable

## Performance Testing

### Measure Packet Rate

```bash
# Terminal 1: Start receiver
cargo run --release --bin mavlink-receiver -- --listen 127.0.0.1:14550 --output ./test-logs

# Terminal 2: Send packets at high rate
while true; do
    echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14550
done

# Monitor file growth
watch -n 1 'ls -lh test-logs/*.tlog'
```

### Check Resource Usage

```bash
# Start applications
cargo run --release --bin mavlink-sender -- --listen 127.0.0.1:14551 --forward 127.0.0.1:14550 &
SENDER_PID=$!

cargo run --release --bin mavlink-receiver -- --listen 127.0.0.1:14550 --output ./test-logs &
RECEIVER_PID=$!

# Monitor CPU and memory
top -pid $SENDER_PID -pid $RECEIVER_PID
```

**Expected results:**
- CPU usage: <1% per process
- Memory: <10 MB per process
- Can handle 200+ packets/sec

## Cleanup

```bash
# Stop all processes
pkill -f mavlink-sender
pkill -f mavlink-receiver

# Remove test logs
rm -rf test-logs/

# Remove Docker containers
docker-compose down
```
