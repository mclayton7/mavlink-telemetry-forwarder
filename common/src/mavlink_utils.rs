use mavlink_tlog::{CrcPolicy, FrameRef};

/// Check that a packet is exactly one MAVLink 1 or 2 frame with a valid
/// checksum (CRC_EXTRA for known message ids; unknown ids pass on structure).
pub fn is_valid_mavlink(packet: &[u8]) -> bool {
    FrameRef::parse(packet, CrcPolicy::Lenient).is_ok_and(|f| f.len() == packet.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mavlink_v2() {
        // HEARTBEAT
        let packet = vec![
            0xFD, 0x09, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x03, 0x51, 0x04, 0x03, 0xE7, 0x1E,
        ];
        assert!(is_valid_mavlink(&packet));
    }

    #[test]
    fn test_valid_mavlink_v1() {
        // HEARTBEAT
        let packet = vec![
            0xFE, 0x09, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x03, 0x51, 0x04,
            0x03, 0x7D, 0xDD,
        ];
        assert!(is_valid_mavlink(&packet));
    }

    #[test]
    fn test_corrupted_checksum() {
        // HEARTBEAT with one payload byte flipped by line noise
        let packet = vec![
            0xFD, 0x09, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x03, 0x50, 0x04, 0x03, 0xE7, 0x1E,
        ];
        assert!(!is_valid_mavlink(&packet));
    }

    #[test]
    fn test_truncated_frame() {
        // HEARTBEAT header promises 9 payload bytes; datagram was cut short
        let packet = vec![
            0xFD, 0x09, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert!(!is_valid_mavlink(&packet));
    }

    #[test]
    fn test_invalid_magic_byte() {
        let packet = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        assert!(!is_valid_mavlink(&packet));
    }

    #[test]
    fn test_empty_packet() {
        let packet: Vec<u8> = vec![];
        assert!(!is_valid_mavlink(&packet));
    }

    #[test]
    fn test_v2_too_short() {
        let packet = vec![0xFD, 0x09, 0x00, 0x00, 0x00];
        assert!(!is_valid_mavlink(&packet));
    }

    #[test]
    fn test_v1_too_short() {
        let packet = vec![0xFE, 0x09, 0x01];
        assert!(!is_valid_mavlink(&packet));
    }
}
