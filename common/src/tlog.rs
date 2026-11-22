use chrono::Utc;
use mavlink_tlog::TlogWriter;

/// Get current Unix epoch timestamp in microseconds
pub fn get_timestamp_us() -> u64 {
    Utc::now()
        .timestamp_micros()
        .try_into()
        .expect("timestamp should be positive")
}

/// Format a MAVLink packet into tlog format: 8-byte timestamp (BE u64) + packet data
pub fn format_tlog_entry(packet: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut writer = TlogWriter::new(Vec::with_capacity(8 + packet.len()));
    writer.write_frame(get_timestamp_us(), packet)?;
    Ok(writer.into_inner())
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
        // HEARTBEAT, MAVLink 2
        let packet = vec![
            0xFD, 0x09, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x03, 0x51, 0x04, 0x03, 0xE7, 0x1E,
        ];
        let entry = format_tlog_entry(&packet).unwrap();

        // Should be 8 bytes timestamp + packet length
        assert_eq!(entry.len(), 8 + packet.len());

        // First 8 bytes are a BE u64 timestamp, as pymavlink, Mission Planner and QGC read it
        let timestamp_bytes: [u8; 8] = entry[..8].try_into().unwrap();
        let timestamp = u64::from_be_bytes(timestamp_bytes);
        assert!(timestamp > 1577836800_000000); // Reasonable timestamp
        assert!(timestamp < 4102444800_000000); // Before 2100, so not byte-swapped

        // Remaining bytes should match packet
        assert_eq!(&entry[8..], &packet[..]);

        // Reads back as one record
        assert_eq!(mavlink_tlog::TlogSlice::new(&entry).count(), 1);
    }
}
