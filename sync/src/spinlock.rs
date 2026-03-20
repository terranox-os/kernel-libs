//! Ticket spinlock with RAII guard.
//!
//! A fair spinlock that serves waiters in FIFO order via a
//! ticket/turn mechanism. The `SpinLockGuard` implements `Drop`
//! to release the lock automatically.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};

/// A ticket-based spinlock protecting data of type `T`.
///
/// Fair: waiters are served in the order they arrive.
pub struct SpinLock<T> {
    next_ticket: AtomicU32,
    now_serving: AtomicU32,
    data: UnsafeCell<T>,
}

// Safety: SpinLock provides mutual exclusion, so it is safe to
// send and share across threads when T: Send.
unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Create a new unlocked `SpinLock` wrapping `value`.
    pub const fn new(value: T) -> Self {
        Self {
            next_ticket: AtomicU32::new(0),
            now_serving: AtomicU32::new(0),
            data: UnsafeCell::new(value),
        }
    }

    /// Acquire the lock, spinning until it is available.
    /// Returns a guard that releases the lock on drop.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        let ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);
        while self.now_serving.load(Ordering::Acquire) != ticket {
            core::hint::spin_loop();
        }
        SpinLockGuard { lock: self }
    }

    /// Try to acquire the lock without spinning.
    /// Returns `None` if the lock is already held.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        let current = self.now_serving.load(Ordering::Relaxed);
        match self.next_ticket.compare_exchange(
            current,
            current.wrapping_add(1),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Some(SpinLockGuard { lock: self }),
            Err(_) => None,
        }
    }

    /// Returns true if the lock is currently held.
    pub fn is_locked(&self) -> bool {
        let ticket = self.next_ticket.load(Ordering::Relaxed);
        let serving = self.now_serving.load(Ordering::Relaxed);
        ticket != serving
    }
}

/// RAII guard that releases the spinlock when dropped.
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // Safety: the guard guarantees exclusive access
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: the guard guarantees exclusive access
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.now_serving.fetch_add(1, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_unlock() {
        let lock = SpinLock::new(42u32);
        assert!(!lock.is_locked());

        {
            let guard = lock.lock();
            assert!(lock.is_locked());
            assert_eq!(*guard, 42);
        }

        assert!(!lock.is_locked());
    }

    #[test]
    fn test_try_lock() {
        let lock = SpinLock::new(0u32);
        let guard = lock.try_lock();
        assert!(guard.is_some());

        // Lock is held, try_lock should fail
        assert!(lock.try_lock().is_none());

        drop(guard);
        assert!(lock.try_lock().is_some());
    }

    #[test]
    fn test_mutate_through_guard() {
        let lock = SpinLock::new(0u32);
        {
            let mut guard = lock.lock();
            *guard = 99;
        }
        {
            let guard = lock.lock();
            assert_eq!(*guard, 99);
        }
    }

    #[test]
    fn test_const_new() {
        static LOCK: SpinLock<u32> = SpinLock::new(0);
        let guard = LOCK.lock();
        assert_eq!(*guard, 0);
    }
}
