//! CRC-32 (IEEE 802.3 polynomial 0xEDB88320 reflected).

/// CRC-32 lookup table (256 entries, reflected polynomial).
#[cfg(feature = "crc32-table")]
const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0u32;
    while i < 256 {
        let mut crc = i;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i as usize] = crc;
        i += 1;
    }
    table
};

/// Compute CRC-32 over a byte slice (table-based, fast).
#[cfg(feature = "crc32-table")]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    crc ^ 0xFFFFFFFF
}

/// Compute CRC-32 over a byte slice (no table, smaller code size).
#[cfg(not(feature = "crc32-table"))]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFFFFFF
}

/// Incrementally update a running CRC-32 value.
pub fn crc32_update(crc: u32, data: &[u8]) -> u32 {
    let mut c = crc ^ 0xFFFFFFFF;
    for &byte in data {
        #[cfg(feature = "crc32-table")]
        {
            let idx = ((c ^ byte as u32) & 0xFF) as usize;
            c = (c >> 8) ^ CRC32_TABLE[idx];
        }
        #[cfg(not(feature = "crc32-table"))]
        {
            c ^= byte as u32;
            for _ in 0..8 {
                if c & 1 != 0 {
                    c = (c >> 1) ^ 0xEDB88320;
                } else {
                    c >>= 1;
                }
            }
        }
    }
    c ^ 0xFFFFFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(b""), 0x00000000);
    }

    #[test]
    fn test_crc32_check_value() {
        // The CRC-32 of "123456789" is 0xCBF43926 (standard check value)
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn test_crc32_single_byte() {
        assert_eq!(crc32(b"A"), 0xD3D99E8B);
    }

    #[test]
    fn test_crc32_incremental() {
        let full = crc32(b"hello world");
        let partial = crc32_update(crc32(b"hello "), b"world");
        assert_eq!(full, partial);
    }
}
