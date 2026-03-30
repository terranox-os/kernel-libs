//! Freestanding cryptographic primitives.
//!
//! - `crc32`: CRC-32 (IEEE 802.3) with optional lookup table
//! - `sha256`: SHA-256 (FIPS 180-4)
//! - `hmac`: HMAC-SHA256 (RFC 2104)

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod crc32;
pub mod sha256;
pub mod hmac;
