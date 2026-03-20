//! Freestanding kernel memory allocators.
//!
//! Rust side provides the bump (linear) allocator.
//! C side provides the bitmap PMM and fixed-block pool allocator
//! (see `gen_alloc.h`, `bitmap_pmm.c`, `pool.c`).

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod bump;
pub use bump::BumpAllocator;
