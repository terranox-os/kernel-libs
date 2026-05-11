//! Lock-free single-producer single-consumer (SPSC) ring buffer.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

/// A fixed-capacity SPSC ring buffer.
///
/// One thread pushes, another pops. No locks required.
/// Capacity is `N - 1` to distinguish full from empty.
pub struct RingBuf<T, const N: usize> {
    data: UnsafeCell<[MaybeUninit<T>; N]>,
    head: AtomicUsize, // read index (consumer)
    tail: AtomicUsize, // write index (producer)
}

// Safety: SPSC design. One thread owns push (tail), another owns pop (head).
unsafe impl<T: Send, const N: usize> Send for RingBuf<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for RingBuf<T, N> {}

impl<T, const N: usize> Default for RingBuf<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> RingBuf<T, N> {
    /// Create a new empty ring buffer. Usable capacity is `N - 1`.
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Usable capacity (N - 1).
    pub const fn capacity(&self) -> usize {
        N - 1
    }

    /// Number of elements currently in the buffer.
    pub fn len(&self) -> usize {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        tail.wrapping_sub(head) % N
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire)
    }

    pub fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        (tail + 1) % N == head
    }

    /// Push an element (producer side). Returns `Err(value)` if full.
    pub fn push(&self, value: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % N;

        if next_tail == self.head.load(Ordering::Acquire) {
            return Err(value);
        }

        // Safety: we own the tail slot; no concurrent writer.
        // UnsafeCell allows mutable access through a shared reference.
        unsafe {
            let data = &mut *self.data.get();
            data[tail] = MaybeUninit::new(value);
        }

        self.tail.store(next_tail, Ordering::Release);
        Ok(())
    }

    /// Pop an element (consumer side). Returns `None` if empty.
    pub fn pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        if head == self.tail.load(Ordering::Acquire) {
            return None;
        }

        // Safety: we own the head slot; producer won't touch it
        let value = unsafe {
            let data = &*self.data.get();
            data[head].assume_init_read()
        };

        self.head.store((head + 1) % N, Ordering::Release);
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let rb: RingBuf<u32, 8> = RingBuf::new();
        assert!(rb.is_empty());
        assert!(!rb.is_full());
        assert_eq!(rb.capacity(), 7);
    }

    #[test]
    fn test_push_pop() {
        let rb: RingBuf<u32, 4> = RingBuf::new();
        assert!(rb.push(1).is_ok());
        assert!(rb.push(2).is_ok());
        assert!(rb.push(3).is_ok());
        assert!(rb.is_full());
        assert!(rb.push(4).is_err());

        assert_eq!(rb.pop(), Some(1));
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn test_wraparound() {
        let rb: RingBuf<u32, 4> = RingBuf::new();
        // Fill and drain multiple times to exercise wraparound
        for round in 0..5 {
            let base = round * 3;
            assert!(rb.push(base + 1).is_ok());
            assert!(rb.push(base + 2).is_ok());
            assert!(rb.push(base + 3).is_ok());
            assert_eq!(rb.pop(), Some(base + 1));
            assert_eq!(rb.pop(), Some(base + 2));
            assert_eq!(rb.pop(), Some(base + 3));
        }
    }

    #[test]
    fn test_len() {
        let rb: RingBuf<u32, 8> = RingBuf::new();
        assert_eq!(rb.len(), 0);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        assert_eq!(rb.len(), 2);
        rb.pop();
        assert_eq!(rb.len(), 1);
    }
}
