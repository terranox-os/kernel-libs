//! Freestanding bitmap operations on `&[u32]` slices.
//!
//! Provides the same semantics as the C API in `gen_bitops.h` with
//! Rust bounds checking. Each `u32` element holds 32 bits, LSB-first.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

/// Set bit `bit` in `bitmap`. Panics if out of bounds.
#[inline]
pub fn bit_set(bitmap: &mut [u32], bit: u32) {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    bitmap[word] |= mask;
}

/// Clear bit `bit` in `bitmap`. Panics if out of bounds.
#[inline]
pub fn bit_clear(bitmap: &mut [u32], bit: u32) {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    bitmap[word] &= !mask;
}

/// Test whether bit `bit` is set. Panics if out of bounds.
#[inline]
pub fn bit_test(bitmap: &[u32], bit: u32) -> bool {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    (bitmap[word] & mask) != 0
}

/// Toggle bit `bit` in `bitmap`. Panics if out of bounds.
#[inline]
pub fn bit_toggle(bitmap: &mut [u32], bit: u32) {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    bitmap[word] ^= mask;
}

/// Find first set bit in the first `nbits` of `bitmap`.
/// Returns `None` if no bit is set.
pub fn bitmap_ffs(bitmap: &[u32], nbits: u32) -> Option<u32> {
    let nwords = (nbits as usize).div_ceil(32);

    for (i, &word) in bitmap[..nwords].iter().enumerate() {
        if word != 0 {
            let bit = (i as u32) * 32 + word.trailing_zeros();
            if bit < nbits {
                return Some(bit);
            }
            return None;
        }
    }

    None
}

/// Find first zero bit in the first `nbits` of `bitmap`.
/// Returns `None` if all bits are set.
pub fn bitmap_ffz(bitmap: &[u32], nbits: u32) -> Option<u32> {
    let nwords = (nbits as usize).div_ceil(32);

    for (i, &raw) in bitmap[..nwords].iter().enumerate() {
        let word = !raw;
        if word != 0 {
            let bit = (i as u32) * 32 + word.trailing_zeros();
            if bit < nbits {
                return Some(bit);
            }
            return None;
        }
    }

    None
}

/// Count the number of set bits in the first `nbits` of `bitmap`.
pub fn bitmap_popcount(bitmap: &[u32], nbits: u32) -> u32 {
    let full_words = nbits as usize / 32;
    let remaining = nbits % 32;
    let mut count = 0u32;

    for word in &bitmap[..full_words] {
        count += word.count_ones();
    }

    if remaining > 0 {
        let mask = (1u32 << remaining) - 1;
        count += (bitmap[full_words] & mask).count_ones();
    }

    count
}

/// Set a range of `count` bits starting at `start`.
pub fn bitmap_set_range(bitmap: &mut [u32], start: u32, count: u32) {
    let mut pos = start;
    let mut remaining = count;

    while remaining > 0 {
        let word_idx = pos as usize / 32;
        let bit_off = pos % 32;
        let bits_in_word = core::cmp::min(32 - bit_off, remaining);

        let mask = if bits_in_word == 32 {
            u32::MAX
        } else {
            (1u32 << bits_in_word) - 1
        };
        bitmap[word_idx] |= mask << bit_off;

        pos += bits_in_word;
        remaining -= bits_in_word;
    }
}

/// Clear a range of `count` bits starting at `start`.
pub fn bitmap_clear_range(bitmap: &mut [u32], start: u32, count: u32) {
    let mut pos = start;
    let mut remaining = count;

    while remaining > 0 {
        let word_idx = pos as usize / 32;
        let bit_off = pos % 32;
        let bits_in_word = core::cmp::min(32 - bit_off, remaining);

        let mask = if bits_in_word == 32 {
            u32::MAX
        } else {
            (1u32 << bits_in_word) - 1
        };
        bitmap[word_idx] &= !(mask << bit_off);

        pos += bits_in_word;
        remaining -= bits_in_word;
    }
}

/// Clear all bits in `bitmap` (first `nbits` bits).
pub fn bitmap_clear_all(bitmap: &mut [u32], nbits: u32) {
    let nwords = (nbits as usize).div_ceil(32);
    bitmap[..nwords].fill(0);
}

/// Iterator over set bits in a bitmap.
pub struct BitIter<'a> {
    bitmap: &'a [u32],
    nbits: u32,
    current_word: u32,
    word_idx: u32,
}

impl<'a> BitIter<'a> {
    /// Create an iterator over set bits in the first `nbits` of `bitmap`.
    pub fn new(bitmap: &'a [u32], nbits: u32) -> Self {
        let current_word = if bitmap.is_empty() || nbits == 0 {
            0
        } else {
            bitmap[0]
        };
        Self {
            bitmap,
            nbits,
            current_word,
            word_idx: 0,
        }
    }
}

impl Iterator for BitIter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        loop {
            if self.current_word != 0 {
                let tz = self.current_word.trailing_zeros();
                let bit = self.word_idx * 32 + tz;
                if bit >= self.nbits {
                    return None;
                }
                // Clear the lowest set bit
                self.current_word &= self.current_word - 1;
                return Some(bit);
            }

            self.word_idx += 1;
            let nwords = (self.nbits as usize).div_ceil(32);
            if self.word_idx as usize >= nwords {
                return None;
            }
            self.current_word = self.bitmap[self.word_idx as usize];
        }
    }
}

/// Convenience function: iterate over set bits.
pub fn bitmap_iter_set(bitmap: &[u32], nbits: u32) -> BitIter<'_> {
    BitIter::new(bitmap, nbits)
}

// ────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────

#[cfg(test)]
extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    // -- Single-bit operations --

    #[test]
    fn test_bit_set_clear_test() {
        let mut bm = [0u32; 4];
        assert!(!bit_test(&bm, 0));

        bit_set(&mut bm, 0);
        assert!(bit_test(&bm, 0));

        bit_clear(&mut bm, 0);
        assert!(!bit_test(&bm, 0));
    }

    #[test]
    fn test_bit_across_words() {
        let mut bm = [0u32; 4];

        bit_set(&mut bm, 31);
        bit_set(&mut bm, 32);
        bit_set(&mut bm, 95);

        assert!(bit_test(&bm, 31));
        assert!(bit_test(&bm, 32));
        assert!(bit_test(&bm, 95));
        assert!(!bit_test(&bm, 30));
        assert!(!bit_test(&bm, 33));
        assert!(!bit_test(&bm, 96));
    }

    #[test]
    fn test_bit_toggle() {
        let mut bm = [0u32; 2];

        bit_toggle(&mut bm, 5);
        assert!(bit_test(&bm, 5));

        bit_toggle(&mut bm, 5);
        assert!(!bit_test(&bm, 5));
    }

    // -- FFS / FFZ --

    #[test]
    fn test_ffs_empty() {
        let bm = [0u32; 4];
        assert_eq!(bitmap_ffs(&bm, 128), None);
    }

    #[test]
    fn test_ffs_first_bit() {
        let bm = [1u32, 0, 0, 0];
        assert_eq!(bitmap_ffs(&bm, 128), Some(0));
    }

    #[test]
    fn test_ffs_middle() {
        let mut bm = [0u32; 4];
        bit_set(&mut bm, 67);
        assert_eq!(bitmap_ffs(&bm, 128), Some(67));
    }

    #[test]
    fn test_ffs_respects_nbits() {
        let mut bm = [0u32; 4];
        bit_set(&mut bm, 100);
        assert_eq!(bitmap_ffs(&bm, 64), None);
        assert_eq!(bitmap_ffs(&bm, 128), Some(100));
    }

    #[test]
    fn test_ffz_full() {
        let bm = [u32::MAX; 4];
        assert_eq!(bitmap_ffz(&bm, 128), None);
    }

    #[test]
    fn test_ffz_first_zero() {
        let bm = [0xFFFFFFFEu32, u32::MAX, u32::MAX, u32::MAX];
        assert_eq!(bitmap_ffz(&bm, 128), Some(0));
    }

    #[test]
    fn test_ffz_middle() {
        let mut bm = [u32::MAX; 4];
        bit_clear(&mut bm, 50);
        assert_eq!(bitmap_ffz(&bm, 128), Some(50));
    }

    // -- Popcount --

    #[test]
    fn test_popcount_empty() {
        let bm = [0u32; 4];
        assert_eq!(bitmap_popcount(&bm, 128), 0);
    }

    #[test]
    fn test_popcount_full() {
        let bm = [u32::MAX; 4];
        assert_eq!(bitmap_popcount(&bm, 128), 128);
    }

    #[test]
    fn test_popcount_partial_word() {
        let bm = [u32::MAX; 2];
        assert_eq!(bitmap_popcount(&bm, 48), 48);
    }

    #[test]
    fn test_popcount_scattered() {
        let mut bm = [0u32; 4];
        bit_set(&mut bm, 0);
        bit_set(&mut bm, 31);
        bit_set(&mut bm, 64);
        assert_eq!(bitmap_popcount(&bm, 128), 3);
    }

    // -- Range operations --

    #[test]
    fn test_set_range_within_word() {
        let mut bm = [0u32; 2];
        bitmap_set_range(&mut bm, 4, 8);
        for i in 0..64 {
            assert_eq!(bit_test(&bm, i), (4..12).contains(&i), "bit {i}");
        }
    }

    #[test]
    fn test_set_range_across_words() {
        let mut bm = [0u32; 4];
        bitmap_set_range(&mut bm, 28, 10);
        for i in 0..128 {
            assert_eq!(bit_test(&bm, i), (28..38).contains(&i), "bit {i}");
        }
    }

    #[test]
    fn test_clear_range() {
        let mut bm = [u32::MAX; 4];
        bitmap_clear_range(&mut bm, 10, 20);
        for i in 0..128 {
            assert_eq!(bit_test(&bm, i), !(10..30).contains(&i), "bit {i}");
        }
    }

    #[test]
    fn test_clear_all() {
        let mut bm = [u32::MAX; 4];
        bitmap_clear_all(&mut bm, 128);
        assert_eq!(bitmap_popcount(&bm, 128), 0);
    }

    #[test]
    fn test_set_range_zero_count() {
        let mut bm = [0u32; 2];
        bitmap_set_range(&mut bm, 5, 0);
        assert_eq!(bitmap_popcount(&bm, 64), 0);
    }

    // -- BitIter --

    #[test]
    fn test_iter_empty() {
        let bm = [0u32; 2];
        let bits: Vec<u32> = bitmap_iter_set(&bm, 64).collect();
        assert!(bits.is_empty());
    }

    #[test]
    fn test_iter_all_set() {
        let bm = [u32::MAX; 1];
        let bits: Vec<u32> = bitmap_iter_set(&bm, 32).collect();
        assert_eq!(bits.len(), 32);
        for (i, &b) in bits.iter().enumerate() {
            assert_eq!(b, i as u32);
        }
    }

    #[test]
    fn test_iter_scattered() {
        let mut bm = [0u32; 4];
        bit_set(&mut bm, 3);
        bit_set(&mut bm, 31);
        bit_set(&mut bm, 32);
        bit_set(&mut bm, 100);

        let bits: Vec<u32> = bitmap_iter_set(&bm, 128).collect();
        assert_eq!(bits, vec![3, 31, 32, 100]);
    }

    #[test]
    fn test_iter_respects_nbits() {
        let mut bm = [0u32; 4];
        bit_set(&mut bm, 10);
        bit_set(&mut bm, 100);

        let bits: Vec<u32> = bitmap_iter_set(&bm, 64).collect();
        assert_eq!(bits, vec![10]);
    }

    // -- Edge cases --

    #[test]
    fn test_nbits_zero() {
        let bm = [u32::MAX; 1];
        assert_eq!(bitmap_ffs(&bm, 0), None);
        assert_eq!(bitmap_ffz(&bm, 0), None);
        assert_eq!(bitmap_popcount(&bm, 0), 0);
    }

    #[test]
    fn test_nbits_not_word_aligned() {
        let mut bm = [0u32; 1];
        bit_set(&mut bm, 5);
        bit_set(&mut bm, 20);

        assert_eq!(bitmap_popcount(&bm, 10), 1);
        assert_eq!(bitmap_popcount(&bm, 21), 2);
        assert_eq!(bitmap_ffs(&bm, 10), Some(5));
        assert_eq!(bitmap_ffs(&bm, 5), None);
    }
}
