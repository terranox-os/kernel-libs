//! Freestanding no_std collections for kernel use.
//!
//! - `StaticVec<T, N>`: Fixed-capacity vector (no alloc)
//! - `RingBuf<T, N>`: Lock-free SPSC ring buffer
//! - `StaticHashMap<K, V, N>`: Array-backed hash map
//! - `IntrusiveList`: Linux-style embedded linked list
//! - `RbTree`: Intrusive red-black tree

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod static_vec;
pub mod ringbuf;
pub mod static_hashmap;
pub mod intrusive_list;
pub mod rbtree;

pub use static_vec::StaticVec;
pub use ringbuf::RingBuf;
pub use static_hashmap::StaticHashMap;
pub use rbtree::{RbTree, RbNode, RbInorderIter};
