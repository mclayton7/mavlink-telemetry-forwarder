# MAVLink Telemetry Sender

A robust Rust-based system for forwarding MAVLink telemetry from aircraft to ground stations and logging to tlog format.

## Architecture

- **Sender** (aircraft): Receives MAVLink from mavlink-router via UDP, forwards to ground station
- **Receiver** (ground): Receives MAVLink via UDP, writes to tlog files with timestamps

## Components

### Sender (`sender/`)
- Lightweight UDP forwarder
- Runs on aircraft (Raspberry Pi, etc.)
- Docker support for easy deployment
- Never crashes - logs errors and continues

### Receiver (`receiver/`)
- Receives telemetry and writes to tlog format
- Automatic file rotation based on size
- Validates MAVLink packets
- Compatible with Mission Planner and QGroundControl

### Common (`common/`)
- Shared utilities for tlog formatting
- MAVLink validation helpers

## Building

```bash
# Build all components
cargo build --release

# Build sender only
cargo build --release --bin mavlink-sender

# Build receiver only
cargo build --release --bin mavlink-receiver

# Run tests
cargo test
```

## Usage

### Sender (Aircraft)

```bash
# Run sender
./target/release/mavlink-sender --listen 0.0.0.0:14551 --forward 192.168.1.100:14550

# Or with environment variable
GROUND_ADDR=192.168.1.100:14550 ./target/release/mavlink-sender
```

### Receiver (Ground Station)

```bash
# Run receiver
./target/release/mavlink-receiver --listen 0.0.0.0:14550 --output ./logs --max-file-size 100
```

### Docker (Sender)

```bash
# Build image
docker build -f sender/Dockerfile -t mavlink-sender .

# Run with docker-compose
GROUND_ADDR=192.168.1.100:14550 docker-compose up -d

# Or run directly
docker run --network host -e GROUND_ADDR=192.168.1.100:14550 mavlink-sender
```

## Configuration

### mavlink-router Setup

Add sniffer endpoint to `/etc/mavlink-router/main.conf`:

```ini
[UdpEndpoint sniffer]
Address=127.0.0.1
Port=14551
```

### Environment Variables

- `GROUND_ADDR`: Ground station address for sender (e.g., `192.168.1.100:14550`)
- `RUST_LOG`: Log level (`error`, `warn`, `info`, `debug`, `trace`)

## Testing

### Unit Tests
```bash
cargo test
```

### Manual Testing

**Test sender forwarding:**
```bash
# Terminal 1: Start sender
cargo run --bin mavlink-sender -- --listen 127.0.0.1:14551 --forward 127.0.0.1:14550

# Terminal 2: Send test packet
echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14551

# Terminal 3: Verify forwarding
nc -ul 127.0.0.1 14550
```

**Test receiver logging:**
```bash
# Terminal 1: Start receiver
cargo run --bin mavlink-receiver -- --listen 127.0.0.1:14550 --output ./test-logs

# Terminal 2: Send test packet
echo -n -e '\xfd\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00' | nc -u 127.0.0.1 14550

# Verify tlog file
ls -lh test-logs/
hexdump -C test-logs/telemetry_*.tlog
```

## File Format

### tlog Format
Each packet in a tlog file consists of:
- 8 bytes: Unix timestamp in microseconds (little-endian u64)
- N bytes: MAVLink packet data

Compatible with:
- Mission Planner
- QGroundControl
- MAVProxy

## Performance

- **Packet rate**: 10-200 packets/sec typical
- **Bandwidth**: <1 Mbps
- **Disk usage**: ~360 KB/min at 100 packets/sec
- **Memory**: <10 MB
- **CPU**: <1%

## Production Deployment

### Aircraft (Sender)
- Use Docker with restart policy or systemd service
- Configure static IP or DDNS for ground station
- Monitor logs for connectivity issues

### Ground Station (Receiver)
- Ensure sufficient disk space
- Configure firewall to allow UDP on port 14550
- Consider log archival/compression for long-term storage

## License

MIT
