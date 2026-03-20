//! Fixed-capacity vector backed by an inline array.

use core::mem::MaybeUninit;

/// A vector with fixed capacity `N`, stored entirely inline.
/// No heap allocation. Panics on push beyond capacity.
pub struct StaticVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Create an empty `StaticVec`.
    pub const fn new() -> Self {
        Self {
            // Safety: MaybeUninit does not require initialization
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    /// Push an element. Panics if full.
    pub fn push(&mut self, value: T) {
        assert!(self.len < N, "StaticVec: push on full vector");
        self.data[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    /// Try to push an element. Returns `Err(value)` if full.
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }
        self.data[self.len] = MaybeUninit::new(value);
        self.len += 1;
        Ok(())
    }

    /// Pop the last element. Returns `None` if empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // Safety: element at self.len was initialized by push
        Some(unsafe { self.data[self.len].assume_init_read() })
    }

    /// Get a reference to element at `index`.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        // Safety: elements 0..len are initialized
        Some(unsafe { self.data[index].assume_init_ref() })
    }

    /// Get a mutable reference to element at `index`.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        // Safety: elements 0..len are initialized
        Some(unsafe { self.data[index].assume_init_mut() })
    }

    /// Access by index. Panics if out of bounds.
    pub fn index(&self, index: usize) -> &T {
        assert!(index < self.len, "StaticVec: index out of bounds");
        // Safety: elements 0..len are initialized
        unsafe { self.data[index].assume_init_ref() }
    }

    /// Returns a slice of the initialized elements.
    pub fn as_slice(&self) -> &[T] {
        // Safety: elements 0..len are initialized and contiguous
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        // Drop all initialized elements
        for i in 0..self.len {
            // Safety: elements 0..len are initialized
            unsafe { self.data[i].assume_init_drop() };
        }
        self.len = 0;
    }
}

impl<T, const N: usize> Drop for StaticVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let v: StaticVec<u32, 8> = StaticVec::new();
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
        assert_eq!(v.capacity(), 8);
    }

    #[test]
    fn test_push_pop() {
        let mut v: StaticVec<u32, 4> = StaticVec::new();
        v.push(10);
        v.push(20);
        v.push(30);
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Some(30));
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.pop(), Some(10));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn test_get() {
        let mut v: StaticVec<u32, 4> = StaticVec::new();
        v.push(100);
        v.push(200);
        assert_eq!(v.get(0), Some(&100));
        assert_eq!(v.get(1), Some(&200));
        assert_eq!(v.get(2), None);
    }

    #[test]
    fn test_try_push_full() {
        let mut v: StaticVec<u32, 2> = StaticVec::new();
        assert!(v.try_push(1).is_ok());
        assert!(v.try_push(2).is_ok());
        assert_eq!(v.try_push(3), Err(3));
    }

    #[test]
    fn test_as_slice() {
        let mut v: StaticVec<u32, 4> = StaticVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_clear() {
        let mut v: StaticVec<u32, 4> = StaticVec::new();
        v.push(1);
        v.push(2);
        v.clear();
        assert!(v.is_empty());
        v.push(3);
        assert_eq!(v.len(), 1);
    }

    #[test]
    #[should_panic]
    fn test_push_overflow_panics() {
        let mut v: StaticVec<u32, 1> = StaticVec::new();
        v.push(1);
        v.push(2); // should panic
    }
}
