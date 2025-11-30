//! Utility functions for common operations in CKB contracts

extern crate alloc;
use alloc::vec::Vec;

/// Decodes a hex string to bytes, handling both with and without "0x" prefix
///
/// # Arguments
/// * `hex_str` - A hex string that may or may not start with "0x"
///
/// # Returns
/// * `Ok(Vec<u8>)` - The decoded bytes
/// * `Err(())` - If the string contains invalid hex characters
///
/// # Examples
/// ```
/// use ckb_deterministic::utils::decode_hex;
///
/// assert_eq!(decode_hex("0x1234").unwrap(), vec![0x12, 0x34]);
/// assert_eq!(decode_hex("1234").unwrap(), vec![0x12, 0x34]);
/// assert_eq!(decode_hex("0xdeadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
/// ```
pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, ()> {
    // Skip "0x" or "0X" prefix if present
    let hex_str = if hex_str.len() >= 2 {
        let prefix = &hex_str[0..2];
        if prefix == "0x" || prefix == "0X" {
            &hex_str[2..]
        } else {
            hex_str
        }
    } else {
        hex_str
    };

    // Check if length is even
    if hex_str.len() % 2 != 0 {
        return Err(());
    }

    let mut result = Vec::with_capacity(hex_str.len() / 2);

    let bytes = hex_str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let high = decode_hex_digit(bytes[i])?;
        let low = decode_hex_digit(bytes[i + 1])?;
        result.push((high << 4) | low);
        i += 2;
    }

    Ok(result)
}

/// Decodes a single hex digit to its value
fn decode_hex_digit(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex_with_prefix() {
        assert_eq!(decode_hex("0x00").unwrap(), vec![0x00]);
        assert_eq!(decode_hex("0x1234").unwrap(), vec![0x12, 0x34]);
        assert_eq!(
            decode_hex("0xdeadbeef").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
        assert_eq!(decode_hex("0X1234").unwrap(), vec![0x12, 0x34]);
    }

    #[test]
    fn test_decode_hex_without_prefix() {
        assert_eq!(decode_hex("00").unwrap(), vec![0x00]);
        assert_eq!(decode_hex("1234").unwrap(), vec![0x12, 0x34]);
        assert_eq!(
            decode_hex("deadbeef").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn test_decode_hex_empty() {
        assert_eq!(decode_hex("").unwrap(), vec![]);
        assert_eq!(decode_hex("0x").unwrap(), vec![]);
    }

    #[test]
    fn test_decode_hex_invalid() {
        assert!(decode_hex("0x123").is_err()); // Odd length
        assert!(decode_hex("123").is_err()); // Odd length
        assert!(decode_hex("0xgg").is_err()); // Invalid hex
        assert!(decode_hex("zz").is_err()); // Invalid hex
    }

    #[test]
    fn test_decode_hex_case_insensitive() {
        assert_eq!(decode_hex("0xAbCdEf").unwrap(), vec![0xab, 0xcd, 0xef]);
        assert_eq!(decode_hex("ABCDEF").unwrap(), vec![0xab, 0xcd, 0xef]);
        assert_eq!(decode_hex("abcdef").unwrap(), vec![0xab, 0xcd, 0xef]);
    }
}
