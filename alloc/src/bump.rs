//! Bump (linear) allocator.
//!
//! Useful for early kernel init (before PMM is ready) and per-frame
//! scratch allocation in GenesisOS-RT sensor processing.
//!
//! Cannot free individual allocations — only reset the entire region.

/// A simple bump allocator over a fixed memory region.
///
/// Allocations advance a pointer forward. The only way to reclaim
/// memory is to reset the entire allocator.
pub struct BumpAllocator {
    base: usize,
    end: usize,
    next: usize,
}

impl BumpAllocator {
    /// Create a new bump allocator over `[base, base + size)`.
    pub const fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            end: base + size,
            next: base,
        }
    }

    /// Allocate `size` bytes with the given alignment.
    /// Returns the start address, or `None` if exhausted.
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<usize> {
        let aligned = (self.next + align - 1) & !(align - 1);
        let new_next = aligned.checked_add(size)?;
        if new_next > self.end {
            return None;
        }
        self.next = new_next;
        Some(aligned)
    }

    /// Reset the allocator, freeing all allocations.
    pub fn reset(&mut self) {
        self.next = self.base;
    }

    /// Number of bytes currently allocated.
    pub const fn used(&self) -> usize {
        self.next - self.base
    }

    /// Number of bytes remaining.
    pub const fn remaining(&self) -> usize {
        self.end - self.next
    }

    /// Total capacity in bytes.
    pub const fn capacity(&self) -> usize {
        self.end - self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_alloc() {
        let mut bump = BumpAllocator::new(0x1000, 4096);
        let a = bump.alloc(100, 1).unwrap();
        assert_eq!(a, 0x1000);
        assert_eq!(bump.used(), 100);
    }

    #[test]
    fn test_aligned_alloc() {
        let mut bump = BumpAllocator::new(0x1000, 4096);
        bump.alloc(1, 1).unwrap(); // consume 1 byte
        let a = bump.alloc(64, 16).unwrap();
        assert_eq!(a % 16, 0);
        assert!(a >= 0x1001);
    }

    #[test]
    fn test_exhaustion() {
        let mut bump = BumpAllocator::new(0x1000, 64);
        assert!(bump.alloc(64, 1).is_some());
        assert!(bump.alloc(1, 1).is_none());
    }

    #[test]
    fn test_reset() {
        let mut bump = BumpAllocator::new(0x1000, 128);
        bump.alloc(128, 1).unwrap();
        assert_eq!(bump.remaining(), 0);

        bump.reset();
        assert_eq!(bump.used(), 0);
        assert_eq!(bump.remaining(), 128);
        assert!(bump.alloc(128, 1).is_some());
    }

    #[test]
    fn test_multiple_allocs() {
        let mut bump = BumpAllocator::new(0x1000, 1024);
        let a = bump.alloc(256, 8).unwrap();
        let b = bump.alloc(256, 8).unwrap();
        let c = bump.alloc(256, 8).unwrap();
        assert!(a < b);
        assert!(b < c);
        assert_eq!(bump.used(), 768);
    }

    #[test]
    fn test_zero_size() {
        let mut bump = BumpAllocator::new(0x1000, 64);
        let a = bump.alloc(0, 1).unwrap();
        assert_eq!(a, 0x1000);
        assert_eq!(bump.used(), 0);
    }

    #[test]
    fn test_const_new() {
        const BUMP: BumpAllocator = BumpAllocator::new(0x2000, 512);
        assert_eq!(BUMP.capacity(), 512);
        assert_eq!(BUMP.used(), 0);
    }
}
