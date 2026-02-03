/// Check if a packet looks like valid MAVLink by checking the magic byte
/// MAVLink 1.0: 0xFE, MAVLink 2.0: 0xFD
pub fn is_valid_mavlink(packet: &[u8]) -> bool {
    if packet.is_empty() {
        return false;
    }
    matches!(packet[0], 0xFD | 0xFE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mavlink_v2() {
        let packet = vec![0xFD, 0x09, 0x00, 0x00, 0x00];
        assert!(is_valid_mavlink(&packet));
    }

    #[test]
    fn test_valid_mavlink_v1() {
        let packet = vec![0xFE, 0x09, 0x01, 0x02, 0x03];
        assert!(is_valid_mavlink(&packet));
    }

    #[test]
    fn test_invalid_mavlink() {
        let packet = vec![0x00, 0x01, 0x02];
        assert!(!is_valid_mavlink(&packet));
    }

    #[test]
    fn test_empty_packet() {
        let packet = vec![];
        assert!(!is_valid_mavlink(&packet));
    }
}
