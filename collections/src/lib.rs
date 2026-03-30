//! Freestanding no_std collections for kernel use.
//!
//! - `StaticVec<T, N>`: Fixed-capacity vector (no alloc)
//! - `RingBuf<T, N>`: Lock-free SPSC ring buffer
//! - `StaticHashMap<K, V, N>`: Array-backed hash map
//! - `IntrusiveList`: Linux-style embedded linked list
//! - `RbTree`: Intrusive red-black tree

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod intrusive_list;
pub mod rbtree;
pub mod ringbuf;
pub mod static_hashmap;
pub mod static_vec;

pub use rbtree::{RbInorderIter, RbNode, RbTree};
pub use ringbuf::RingBuf;
pub use static_hashmap::StaticHashMap;
pub use static_vec::StaticVec;
