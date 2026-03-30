//! Freestanding kernel formatter using `core::fmt::Write`.
//!
//! Provides `KernelWriter`, a zero-allocation formatter that sends
//! each byte through a caller-provided closure. This is the Rust
//! equivalent of the C `gen_kprintf` engine.
//!
//! Each OS provides its own backend:
//! - TerranoxOS: encrypted audit log
//! - GenesisOS-RT: UART serial output
//! - HermeticaOS: module-subscribable ring buffer

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

use core::fmt::{self, Write};

/// A `core::fmt::Write` implementation that sends each byte through
/// a caller-provided closure.
///
/// # Example
///
/// ```ignore
/// use kernel_kfmt::KernelWriter;
/// use core::fmt::Write;
///
/// let mut buf = [0u8; 64];
/// let mut pos = 0;
/// {
///     let mut writer = KernelWriter::new(|b| {
///         buf[pos] = b;
///         pos += 1;
///     });
///     write!(writer, "hello {}", 42).unwrap();
/// }
/// ```
pub struct KernelWriter<F: FnMut(u8)> {
    output: F,
}

impl<F: FnMut(u8)> KernelWriter<F> {
    /// Create a new `KernelWriter` with the given byte-output closure.
    #[inline]
    pub fn new(output: F) -> Self {
        Self { output }
    }

    /// Consume the writer and return the inner closure.
    #[inline]
    pub fn into_inner(self) -> F {
        self.output
    }
}

impl<F: FnMut(u8)> Write for KernelWriter<F> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            (self.output)(b);
        }
        Ok(())
    }
}

/// A `KernelWriter` that also counts bytes written.
pub struct CountingWriter<F: FnMut(u8)> {
    output: F,
    count: usize,
}

impl<F: FnMut(u8)> CountingWriter<F> {
    /// Create a new `CountingWriter`.
    #[inline]
    pub fn new(output: F) -> Self {
        Self { output, count: 0 }
    }

    /// Returns the number of bytes written so far.
    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Consume the writer and return (closure, byte_count).
    #[inline]
    pub fn into_inner(self) -> (F, usize) {
        (self.output, self.count)
    }
}

impl<F: FnMut(u8)> Write for CountingWriter<F> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            (self.output)(b);
            self.count += 1;
        }
        Ok(())
    }
}

/// Write formatted output through a byte-output closure.
/// Returns the number of bytes written on success.
///
/// This is a convenience wrapper around `CountingWriter` + `write!`.
#[macro_export]
macro_rules! kwrite {
    ($output:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        let mut writer = $crate::CountingWriter::new($output);
        match write!(writer, $($arg)*) {
            Ok(()) => Ok(writer.count()),
            Err(e) => Err(e),
        }
    }};
}

// ────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────

#[cfg(test)]
extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use core::fmt::Write;

    /// Helper: write into a Vec<u8> via KernelWriter.
    macro_rules! fmt_vec {
        ($($arg:tt)*) => {{
            let mut buf = Vec::new();
            let mut w = KernelWriter::new(|b| buf.push(b));
            write!(w, $($arg)*).unwrap();
            drop(w);
            buf
        }};
    }

    #[test]
    fn test_basic_string() {
        assert_eq!(fmt_vec!("hello"), b"hello");
    }

    #[test]
    fn test_empty_string() {
        assert!(fmt_vec!("").is_empty());
    }

    #[test]
    fn test_integer_formatting() {
        assert_eq!(fmt_vec!("{}", 42), b"42");
    }

    #[test]
    fn test_negative_integer() {
        assert_eq!(fmt_vec!("{}", -123), b"-123");
    }

    #[test]
    fn test_hex_formatting() {
        assert_eq!(fmt_vec!("{:#x}", 0xDEADu32), b"0xdead");
    }

    #[test]
    fn test_hex_upper() {
        assert_eq!(fmt_vec!("{:#X}", 0xBEEFu32), b"0xBEEF");
    }

    #[test]
    fn test_zero_padded_hex() {
        assert_eq!(fmt_vec!("{:08x}", 0xABu32), b"000000ab");
    }

    #[test]
    fn test_mixed_format() {
        assert_eq!(
            fmt_vec!("v={} addr={:#x}", 10, 0x1000u32),
            b"v=10 addr=0x1000"
        );
    }

    #[test]
    fn test_string_interpolation() {
        assert_eq!(fmt_vec!("hello {}", "world"), b"hello world");
    }

    #[test]
    fn test_counting_writer() {
        let mut buf = Vec::new();
        let mut cw = CountingWriter::new(|b| buf.push(b));
        write!(cw, "abc{}", 42).unwrap();
        assert_eq!(cw.count(), 5);
        assert_eq!(&buf, b"abc42");
    }

    #[test]
    fn test_into_inner() {
        let mut buf = Vec::new();
        {
            let mut w = KernelWriter::new(|b| buf.push(b));
            write!(w, "test").unwrap();
        }
        assert_eq!(&buf, b"test");
    }

    #[test]
    fn test_kwrite_macro() {
        let mut buf = Vec::new();
        let result = kwrite!(|b| buf.push(b), "x={}", 99);
        assert_eq!(result.unwrap(), 4);
        assert_eq!(&buf, b"x=99");
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(fmt_vec!("a\nb\tc"), b"a\nb\tc");
    }

    #[test]
    fn test_zero() {
        assert_eq!(fmt_vec!("{}", 0), b"0");
    }

    #[test]
    fn test_bool() {
        assert_eq!(fmt_vec!("{}", true), b"true");
    }
}
