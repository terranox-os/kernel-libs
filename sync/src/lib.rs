//! Freestanding synchronization primitives.
//!
//! - `SpinLock<T>`: Fair ticket spinlock with RAII guard
//! - `Once<T>`: One-time initialization
//! - `atomic_bitops`: Atomic bitmap set/clear/test for SMP contexts

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod spinlock;
pub mod once;
pub mod atomic_bitops;

pub use spinlock::{SpinLock, SpinLockGuard};
pub use once::Once;
