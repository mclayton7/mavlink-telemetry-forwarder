use chrono::Utc;

/// Get current Unix epoch timestamp in microseconds
pub fn get_timestamp_us() -> u64 {
    Utc::now().timestamp_micros() as u64
}

/// Format a MAVLink packet into tlog format: 8-byte timestamp (LE u64) + packet data
pub fn format_tlog_entry(packet: &[u8]) -> Vec<u8> {
    let timestamp = get_timestamp_us();
    let mut entry = Vec::with_capacity(8 + packet.len());
    entry.extend_from_slice(&timestamp.to_le_bytes());
    entry.extend_from_slice(packet);
    entry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_format() {
        let timestamp = get_timestamp_us();
        // Timestamp should be a reasonable value (after 2020)
        assert!(timestamp > 1577836800_000000); // Jan 1, 2020 in microseconds
    }

    #[test]
    fn test_tlog_entry_format() {
        let packet = vec![0xFD, 0x09, 0x00, 0x00, 0x00];
        let entry = format_tlog_entry(&packet);

        // Should be 8 bytes timestamp + packet length
        assert_eq!(entry.len(), 8 + packet.len());

        // First 8 bytes should be LE u64 timestamp
        let timestamp_bytes: [u8; 8] = entry[..8].try_into().unwrap();
        let timestamp = u64::from_le_bytes(timestamp_bytes);
        assert!(timestamp > 1577836800_000000); // Reasonable timestamp

        // Remaining bytes should match packet
        assert_eq!(&entry[8..], &packet[..]);
    }
}
