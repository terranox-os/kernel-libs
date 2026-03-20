//! One-time initialization primitive.
//!
//! `Once<T>` holds a value that is initialized exactly once. Subsequent
//! calls to `call_once` are no-ops. The value can then be read without
//! synchronization overhead.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

const UNINIT: u8 = 0;
const RUNNING: u8 = 1;
const COMPLETE: u8 = 2;

/// A one-time initialization container.
///
/// The value is not available until `call_once` has been invoked.
/// After initialization, `get` returns the value without atomic
/// operations on the fast path.
pub struct Once<T> {
    state: AtomicU8,
    data: UnsafeCell<Option<T>>,
}

// Safety: Once provides synchronized one-time init; after that,
// data is effectively immutable.
unsafe impl<T: Send + Sync> Send for Once<T> {}
unsafe impl<T: Send + Sync> Sync for Once<T> {}

impl<T> Once<T> {
    /// Create an uninitialized `Once`.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(UNINIT),
            data: UnsafeCell::new(None),
        }
    }

    /// Initialize the value if it hasn't been set yet.
    ///
    /// If two threads race, exactly one will run `f`. The other
    /// spins until initialization is complete.
    pub fn call_once(&self, f: impl FnOnce() -> T) {
        if self.state.load(Ordering::Acquire) == COMPLETE {
            return;
        }

        match self.state.compare_exchange(
            UNINIT,
            RUNNING,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                // We won the race — initialize
                // Safety: we are the only thread in RUNNING state
                unsafe {
                    *self.data.get() = Some(f());
                }
                self.state.store(COMPLETE, Ordering::Release);
            }
            Err(_) => {
                // Another thread is initializing — spin
                while self.state.load(Ordering::Acquire) != COMPLETE {
                    core::hint::spin_loop();
                }
            }
        }
    }

    /// Returns the value if initialized, or `None`.
    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == COMPLETE {
            // Safety: data is immutable after COMPLETE
            unsafe { (*self.data.get()).as_ref() }
        } else {
            None
        }
    }

    /// Returns true if initialization is complete.
    pub fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_once_basic() {
        let once: Once<u32> = Once::new();
        assert!(!once.is_initialized());
        assert!(once.get().is_none());

        once.call_once(|| 42);
        assert!(once.is_initialized());
        assert_eq!(once.get(), Some(&42));
    }

    #[test]
    fn test_once_idempotent() {
        let once: Once<u32> = Once::new();
        once.call_once(|| 1);
        once.call_once(|| 2); // Should be ignored
        assert_eq!(once.get(), Some(&1));
    }

    #[test]
    fn test_once_const_new() {
        static ONCE: Once<u32> = Once::new();
        assert!(!ONCE.is_initialized());
        ONCE.call_once(|| 99);
        assert_eq!(ONCE.get(), Some(&99));
    }
}
