/// Check if a packet looks like valid MAVLink by checking the magic byte
/// and enforcing minimum header lengths.
/// MAVLink 1.0 (0xFE): minimum 8 bytes
/// MAVLink 2.0 (0xFD): minimum 10 bytes
pub fn is_valid_mavlink(packet: &[u8]) -> bool {
    match packet.first() {
        Some(0xFD) => packet.len() >= 10,
        Some(0xFE) => packet.len() >= 8,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mavlink_v2() {
        let packet = vec![0xFD, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(is_valid_mavlink(&packet));
    }

    #[test]
    fn test_valid_mavlink_v1() {
        let packet = vec![0xFE, 0x09, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        assert!(is_valid_mavlink(&packet));
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
