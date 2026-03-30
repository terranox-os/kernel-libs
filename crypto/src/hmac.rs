//! HMAC-SHA256 (RFC 2104).

use crate::sha256::{sha256_digest, Sha256};

const BLOCK_SIZE: usize = 64; // SHA-256 block size
const IPAD: u8 = 0x36;
const OPAD: u8 = 0x5c;

/// Compute HMAC-SHA256 of `data` with the given `key`.
///
/// If the key is longer than 64 bytes, it is hashed first.
/// If shorter, it is zero-padded to 64 bytes.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    // Normalize key to exactly BLOCK_SIZE bytes
    let mut k_block = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let hashed = sha256_digest(key);
        k_block[..32].copy_from_slice(&hashed);
    } else {
        k_block[..key.len()].copy_from_slice(key);
    }

    // Inner: SHA256((K ^ ipad) || data)
    let mut ipad_key = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad_key[i] = k_block[i] ^ IPAD;
    }

    let mut inner = Sha256::new();
    inner.update(&ipad_key);
    inner.update(data);
    let inner_hash = inner.finalize();

    // Outer: SHA256((K ^ opad) || inner_hash)
    let mut opad_key = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        opad_key[i] = k_block[i] ^ OPAD;
    }

    let mut outer = Sha256::new();
    outer.update(&opad_key);
    outer.update(&inner_hash);
    outer.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 4231 test vectors

    #[test]
    fn test_hmac_rfc4231_case1() {
        // Key = 0x0b repeated 20 times
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let mac = hmac_sha256(&key, data);
        assert_eq!(
            mac,
            [
                0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
                0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
                0x2e, 0x32, 0xcf, 0xf7,
            ]
        );
    }

    #[test]
    fn test_hmac_rfc4231_case2() {
        // Key = "Jefe"
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let mac = hmac_sha256(key, data);
        assert_eq!(
            mac,
            [
                0x5b, 0xdc, 0xc1, 0x46, 0xbf, 0x60, 0x75, 0x4e, 0x6a, 0x04, 0x24, 0x26, 0x08, 0x95,
                0x75, 0xc7, 0x5a, 0x00, 0x3f, 0x08, 0x9d, 0x27, 0x39, 0x83, 0x9d, 0xec, 0x58, 0xb9,
                0x64, 0xec, 0x38, 0x43,
            ]
        );
    }

    #[test]
    fn test_hmac_long_key() {
        // Key longer than block size (64 bytes) — should be hashed first
        let key = [0xAAu8; 131];
        let data = b"Test Using Larger Than Block-Size Key - Hash Key First";
        let mac = hmac_sha256(&key, data);
        assert_eq!(
            mac,
            [
                0x60, 0xe4, 0x31, 0x59, 0x1e, 0xe0, 0xb6, 0x7f, 0x0d, 0x8a, 0x26, 0xaa, 0xcb, 0xf5,
                0xb7, 0x7f, 0x8e, 0x0b, 0xc6, 0x21, 0x37, 0x28, 0xc5, 0x14, 0x05, 0x46, 0x04, 0x0f,
                0x0e, 0xe3, 0x7f, 0x54,
            ]
        );
    }
}
