//! Atomic bitmap operations on `&[AtomicU32]`.
//!
//! Used by SMP-safe scheduler bitmaps and concurrent capability
//! checks. For single-threaded / interrupt-disabled contexts,
//! use the non-atomic `kernel-bitops` crate instead.

use core::sync::atomic::{AtomicU32, Ordering};

/// Atomically set bit `bit` in `bitmap`.
#[inline]
pub fn atomic_bit_set(bitmap: &[AtomicU32], bit: u32) {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    bitmap[word].fetch_or(mask, Ordering::AcqRel);
}

/// Atomically clear bit `bit` in `bitmap`.
#[inline]
pub fn atomic_bit_clear(bitmap: &[AtomicU32], bit: u32) {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    bitmap[word].fetch_and(!mask, Ordering::AcqRel);
}

/// Atomically test whether bit `bit` is set.
#[inline]
pub fn atomic_bit_test(bitmap: &[AtomicU32], bit: u32) -> bool {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    (bitmap[word].load(Ordering::Acquire) & mask) != 0
}

/// Atomically toggle bit `bit`. Returns the previous value.
#[inline]
pub fn atomic_bit_toggle(bitmap: &[AtomicU32], bit: u32) -> bool {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    let prev = bitmap[word].fetch_xor(mask, Ordering::AcqRel);
    (prev & mask) != 0
}

/// Atomically set bit `bit`, returning true if it was already set.
#[inline]
pub fn atomic_bit_test_and_set(bitmap: &[AtomicU32], bit: u32) -> bool {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    let prev = bitmap[word].fetch_or(mask, Ordering::AcqRel);
    (prev & mask) != 0
}

/// Atomically clear bit `bit`, returning true if it was set.
#[inline]
pub fn atomic_bit_test_and_clear(bitmap: &[AtomicU32], bit: u32) -> bool {
    let word = bit as usize / 32;
    let mask = 1u32 << (bit % 32);
    let prev = bitmap[word].fetch_and(!mask, Ordering::AcqRel);
    (prev & mask) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bitmap<const N: usize>() -> [AtomicU32; N] {
        core::array::from_fn(|_| AtomicU32::new(0))
    }

    #[test]
    fn test_set_clear_test() {
        let bm = make_bitmap::<4>();
        assert!(!atomic_bit_test(&bm, 5));

        atomic_bit_set(&bm, 5);
        assert!(atomic_bit_test(&bm, 5));

        atomic_bit_clear(&bm, 5);
        assert!(!atomic_bit_test(&bm, 5));
    }

    #[test]
    fn test_across_words() {
        let bm = make_bitmap::<4>();
        atomic_bit_set(&bm, 31);
        atomic_bit_set(&bm, 32);
        atomic_bit_set(&bm, 95);

        assert!(atomic_bit_test(&bm, 31));
        assert!(atomic_bit_test(&bm, 32));
        assert!(atomic_bit_test(&bm, 95));
        assert!(!atomic_bit_test(&bm, 30));
        assert!(!atomic_bit_test(&bm, 33));
    }

    #[test]
    fn test_toggle() {
        let bm = make_bitmap::<2>();

        let was_set = atomic_bit_toggle(&bm, 10);
        assert!(!was_set);
        assert!(atomic_bit_test(&bm, 10));

        let was_set = atomic_bit_toggle(&bm, 10);
        assert!(was_set);
        assert!(!atomic_bit_test(&bm, 10));
    }

    #[test]
    fn test_test_and_set() {
        let bm = make_bitmap::<2>();

        let was_set = atomic_bit_test_and_set(&bm, 7);
        assert!(!was_set);
        assert!(atomic_bit_test(&bm, 7));

        let was_set = atomic_bit_test_and_set(&bm, 7);
        assert!(was_set);
    }

    #[test]
    fn test_test_and_clear() {
        let bm = make_bitmap::<2>();
        atomic_bit_set(&bm, 3);

        let was_set = atomic_bit_test_and_clear(&bm, 3);
        assert!(was_set);
        assert!(!atomic_bit_test(&bm, 3));

        let was_set = atomic_bit_test_and_clear(&bm, 3);
        assert!(!was_set);
    }
}
