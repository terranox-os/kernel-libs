//! Intrusive red-black tree.
//!
//! A self-balancing BST where nodes are embedded in the containing
//! struct (like `IntrusiveList`). No heap allocation — the caller
//! manages node lifetime.
//!
//! Used for: scheduler run queues (ordered by deadline/priority),
//! VFS inode caches (ordered by inode number), timer wheels.
//!
//! # Safety
//!
//! Uses raw pointers because intrusive containers inherently require
//! aliasing. The caller must ensure nodes outlive the tree.

use core::cmp::Ordering;
use core::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
    Red = 0,
    Black = 1,
}

/// A red-black tree node embedded in the containing structure.
///
/// The `key` field is a `u64` used for ordering. The caller maps
/// their domain key to `u64` (e.g., deadline timestamp, inode number).
pub struct RbNode {
    pub key: u64,
    left: *mut RbNode,
    right: *mut RbNode,
    parent: *mut RbNode,
    color: Color,
}

impl RbNode {
    /// Create a new unlinked red node.
    pub const fn new(key: u64) -> Self {
        Self {
            key,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent: ptr::null_mut(),
            color: Color::Red,
        }
    }
}

/// An intrusive red-black tree.
pub struct RbTree {
    root: *mut RbNode,
    len: usize,
}

impl Default for RbTree {
    fn default() -> Self {
        Self::new()
    }
}

impl RbTree {
    /// Create an empty tree.
    pub const fn new() -> Self {
        Self {
            root: ptr::null_mut(),
            len: 0,
        }
    }

    /// Number of nodes in the tree.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    /// Insert a node into the tree.
    ///
    /// # Safety
    /// `node` must be a valid pointer to an initialized `RbNode` that
    /// is not already in any tree. The node must remain valid for the
    /// lifetime of the tree (or until removed).
    pub unsafe fn insert(&mut self, node: *mut RbNode) {
        unsafe {
            (*node).left = ptr::null_mut();
            (*node).right = ptr::null_mut();
            (*node).color = Color::Red;

            // Standard BST insert
            let mut parent: *mut RbNode = ptr::null_mut();
            let mut current = self.root;

            while !current.is_null() {
                parent = current;
                if (*node).key < (*current).key {
                    current = (*current).left;
                } else {
                    current = (*current).right;
                }
            }

            (*node).parent = parent;

            if parent.is_null() {
                self.root = node;
            } else if (*node).key < (*parent).key {
                (*parent).left = node;
            } else {
                (*parent).right = node;
            }

            self.len += 1;
            self.insert_fixup(node);
        }
    }

    /// Remove a node from the tree.
    ///
    /// # Safety
    /// `node` must be currently in this tree.
    pub unsafe fn remove(&mut self, node: *mut RbNode) {
        unsafe {
            let mut y = node;
            let mut y_orig_color = (*y).color;
            let x: *mut RbNode;
            let x_parent: *mut RbNode;

            if (*node).left.is_null() {
                x = (*node).right;
                x_parent = (*node).parent;
                self.transplant(node, (*node).right);
            } else if (*node).right.is_null() {
                x = (*node).left;
                x_parent = (*node).parent;
                self.transplant(node, (*node).left);
            } else {
                // Find in-order successor (minimum of right subtree)
                y = Self::subtree_min((*node).right);
                y_orig_color = (*y).color;
                x = (*y).right;

                if (*y).parent == node {
                    if !x.is_null() {
                        (*x).parent = y;
                    }
                    x_parent = y;
                } else {
                    x_parent = (*y).parent;
                    self.transplant(y, (*y).right);
                    (*y).right = (*node).right;
                    if !(*y).right.is_null() {
                        (*(*y).right).parent = y;
                    }
                }
                self.transplant(node, y);
                (*y).left = (*node).left;
                if !(*y).left.is_null() {
                    (*(*y).left).parent = y;
                }
                (*y).color = (*node).color;
            }

            self.len -= 1;

            // Clear removed node's links
            (*node).left = ptr::null_mut();
            (*node).right = ptr::null_mut();
            (*node).parent = ptr::null_mut();

            if y_orig_color == Color::Black {
                self.delete_fixup(x, x_parent);
            }
        }
    }

    /// Find a node by key. Returns null if not found.
    pub fn find(&self, key: u64) -> *mut RbNode {
        let mut current = self.root;
        while !current.is_null() {
            let cmp = key.cmp(unsafe { &(*current).key });
            match cmp {
                Ordering::Equal => return current,
                Ordering::Less => current = unsafe { (*current).left },
                Ordering::Greater => current = unsafe { (*current).right },
            }
        }
        ptr::null_mut()
    }

    /// Return the node with the minimum key, or null if empty.
    pub fn min(&self) -> *mut RbNode {
        if self.root.is_null() {
            return ptr::null_mut();
        }
        Self::subtree_min(self.root)
    }

    /// Return the node with the maximum key, or null if empty.
    pub fn max(&self) -> *mut RbNode {
        if self.root.is_null() {
            return ptr::null_mut();
        }
        Self::subtree_max(self.root)
    }

    // ── Internal helpers ────────────────────────────────────

    fn subtree_min(mut node: *mut RbNode) -> *mut RbNode {
        while !unsafe { (*node).left }.is_null() {
            node = unsafe { (*node).left };
        }
        node
    }

    fn subtree_max(mut node: *mut RbNode) -> *mut RbNode {
        while !unsafe { (*node).right }.is_null() {
            node = unsafe { (*node).right };
        }
        node
    }

    unsafe fn transplant(&mut self, u: *mut RbNode, v: *mut RbNode) {
        unsafe {
            if (*u).parent.is_null() {
                self.root = v;
            } else if u == (*(*u).parent).left {
                (*(*u).parent).left = v;
            } else {
                (*(*u).parent).right = v;
            }
            if !v.is_null() {
                (*v).parent = (*u).parent;
            }
        }
    }

    unsafe fn rotate_left(&mut self, x: *mut RbNode) {
        unsafe {
            let y = (*x).right;
            (*x).right = (*y).left;
            if !(*y).left.is_null() {
                (*(*y).left).parent = x;
            }
            (*y).parent = (*x).parent;
            if (*x).parent.is_null() {
                self.root = y;
            } else if x == (*(*x).parent).left {
                (*(*x).parent).left = y;
            } else {
                (*(*x).parent).right = y;
            }
            (*y).left = x;
            (*x).parent = y;
        }
    }

    unsafe fn rotate_right(&mut self, x: *mut RbNode) {
        unsafe {
            let y = (*x).left;
            (*x).left = (*y).right;
            if !(*y).right.is_null() {
                (*(*y).right).parent = x;
            }
            (*y).parent = (*x).parent;
            if (*x).parent.is_null() {
                self.root = y;
            } else if x == (*(*x).parent).right {
                (*(*x).parent).right = y;
            } else {
                (*(*x).parent).left = y;
            }
            (*y).right = x;
            (*x).parent = y;
        }
    }

    unsafe fn insert_fixup(&mut self, mut z: *mut RbNode) {
        unsafe {
            while !(*z).parent.is_null() && (*(*z).parent).color == Color::Red {
                let grandparent = (*(*z).parent).parent;
                if grandparent.is_null() {
                    break;
                }

                if (*z).parent == (*grandparent).left {
                    let uncle = (*grandparent).right;
                    if !uncle.is_null() && (*uncle).color == Color::Red {
                        // Case 1: uncle is red
                        (*(*z).parent).color = Color::Black;
                        (*uncle).color = Color::Black;
                        (*grandparent).color = Color::Red;
                        z = grandparent;
                    } else {
                        if z == (*(*z).parent).right {
                            // Case 2: z is right child
                            z = (*z).parent;
                            self.rotate_left(z);
                        }
                        // Case 3: z is left child
                        (*(*z).parent).color = Color::Black;
                        (*(*(*z).parent).parent).color = Color::Red;
                        self.rotate_right((*(*z).parent).parent);
                    }
                } else {
                    // Mirror: parent is right child of grandparent
                    let uncle = (*grandparent).left;
                    if !uncle.is_null() && (*uncle).color == Color::Red {
                        (*(*z).parent).color = Color::Black;
                        (*uncle).color = Color::Black;
                        (*grandparent).color = Color::Red;
                        z = grandparent;
                    } else {
                        if z == (*(*z).parent).left {
                            z = (*z).parent;
                            self.rotate_right(z);
                        }
                        (*(*z).parent).color = Color::Black;
                        (*(*(*z).parent).parent).color = Color::Red;
                        self.rotate_left((*(*z).parent).parent);
                    }
                }
            }
            (*self.root).color = Color::Black;
        }
    }

    unsafe fn delete_fixup(&mut self, mut x: *mut RbNode, mut x_parent: *mut RbNode) {
        unsafe {
            while x != self.root && (x.is_null() || (*x).color == Color::Black) {
                if x_parent.is_null() {
                    break;
                }
                if x == (*x_parent).left {
                    let mut w = (*x_parent).right;
                    if !w.is_null() && (*w).color == Color::Red {
                        (*w).color = Color::Black;
                        (*x_parent).color = Color::Red;
                        self.rotate_left(x_parent);
                        w = (*x_parent).right;
                    }
                    if w.is_null() {
                        x = x_parent;
                        x_parent = (*x).parent;
                        continue;
                    }
                    let wl_black = (*w).left.is_null() || (*(*w).left).color == Color::Black;
                    let wr_black = (*w).right.is_null() || (*(*w).right).color == Color::Black;
                    if wl_black && wr_black {
                        (*w).color = Color::Red;
                        x = x_parent;
                        x_parent = (*x).parent;
                    } else {
                        if wr_black {
                            if !(*w).left.is_null() {
                                (*(*w).left).color = Color::Black;
                            }
                            (*w).color = Color::Red;
                            self.rotate_right(w);
                            w = (*x_parent).right;
                        }
                        if !w.is_null() {
                            (*w).color = (*x_parent).color;
                        }
                        (*x_parent).color = Color::Black;
                        if !w.is_null() && !(*w).right.is_null() {
                            (*(*w).right).color = Color::Black;
                        }
                        self.rotate_left(x_parent);
                        x = self.root;
                        x_parent = ptr::null_mut();
                    }
                } else {
                    // Mirror
                    let mut w = (*x_parent).left;
                    if !w.is_null() && (*w).color == Color::Red {
                        (*w).color = Color::Black;
                        (*x_parent).color = Color::Red;
                        self.rotate_right(x_parent);
                        w = (*x_parent).left;
                    }
                    if w.is_null() {
                        x = x_parent;
                        x_parent = (*x).parent;
                        continue;
                    }
                    let wr_black = (*w).right.is_null() || (*(*w).right).color == Color::Black;
                    let wl_black = (*w).left.is_null() || (*(*w).left).color == Color::Black;
                    if wr_black && wl_black {
                        (*w).color = Color::Red;
                        x = x_parent;
                        x_parent = (*x).parent;
                    } else {
                        if wl_black {
                            if !(*w).right.is_null() {
                                (*(*w).right).color = Color::Black;
                            }
                            (*w).color = Color::Red;
                            self.rotate_left(w);
                            w = (*x_parent).left;
                        }
                        if !w.is_null() {
                            (*w).color = (*x_parent).color;
                        }
                        (*x_parent).color = Color::Black;
                        if !w.is_null() && !(*w).left.is_null() {
                            (*(*w).left).color = Color::Black;
                        }
                        self.rotate_right(x_parent);
                        x = self.root;
                        x_parent = ptr::null_mut();
                    }
                }
            }
            if !x.is_null() {
                (*x).color = Color::Black;
            }
        }
    }
}

// ────────────────────────────────────────────────────────────
// In-order iterator
// ────────────────────────────────────────────────────────────

/// In-order iterator over tree nodes (ascending key order).
pub struct RbInorderIter {
    stack: [*mut RbNode; 64], // max depth ~2*log2(n), 64 handles billions of nodes
    sp: usize,
}

impl RbInorderIter {
    pub fn new(tree: &RbTree) -> Self {
        let mut iter = Self {
            stack: [ptr::null_mut(); 64],
            sp: 0,
        };
        iter.push_left_chain(tree.root);
        iter
    }

    fn push_left_chain(&mut self, mut node: *mut RbNode) {
        while !node.is_null() && self.sp < 64 {
            self.stack[self.sp] = node;
            self.sp += 1;
            node = unsafe { (*node).left };
        }
    }
}

impl Iterator for RbInorderIter {
    type Item = *mut RbNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sp == 0 {
            return None;
        }
        self.sp -= 1;
        let node = self.stack[self.sp];
        self.push_left_chain(unsafe { (*node).right });
        Some(node)
    }
}

// ────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = RbTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert!(tree.min().is_null());
        assert!(tree.max().is_null());
    }

    #[test]
    fn test_insert_single() {
        let mut tree = RbTree::new();
        let mut n = RbNode::new(42);
        unsafe { tree.insert(&mut n) };
        assert_eq!(tree.len(), 1);
        assert!(!tree.min().is_null());
        assert_eq!(unsafe { (*tree.min()).key }, 42);
    }

    // Helper: get raw pointer to array element without &mut reborrow
    // that would invalidate previous raw pointers in the tree.
    unsafe fn node_ptr(nodes: *mut RbNode, i: usize) -> *mut RbNode {
        unsafe { nodes.add(i) }
    }

    #[test]
    fn test_insert_ascending() {
        let mut tree = RbTree::new();
        let mut nodes: [RbNode; 10] = core::array::from_fn(|i| RbNode::new(i as u64));
        let base = nodes.as_mut_ptr();
        for i in 0..10 {
            unsafe { tree.insert(node_ptr(base, i)) };
        }
        assert_eq!(tree.len(), 10);
        assert_eq!(unsafe { (*tree.min()).key }, 0);
        assert_eq!(unsafe { (*tree.max()).key }, 9);
    }

    #[test]
    fn test_insert_descending() {
        let mut tree = RbTree::new();
        let mut nodes: [RbNode; 10] = core::array::from_fn(|i| RbNode::new(9 - i as u64));
        let base = nodes.as_mut_ptr();
        for i in 0..10 {
            unsafe { tree.insert(node_ptr(base, i)) };
        }
        assert_eq!(tree.len(), 10);
        assert_eq!(unsafe { (*tree.min()).key }, 0);
        assert_eq!(unsafe { (*tree.max()).key }, 9);
    }

    #[test]
    fn test_find() {
        let mut tree = RbTree::new();
        let mut nodes: [RbNode; 5] = core::array::from_fn(|i| RbNode::new((i * 10) as u64));
        let base = nodes.as_mut_ptr();
        for i in 0..5 {
            unsafe { tree.insert(node_ptr(base, i)) };
        }

        assert!(!tree.find(20).is_null());
        assert_eq!(unsafe { (*tree.find(20)).key }, 20);
        assert!(tree.find(15).is_null());
    }

    #[test]
    fn test_remove_leaf() {
        let mut tree = RbTree::new();
        let mut nodes = [RbNode::new(10), RbNode::new(20), RbNode::new(30)];
        let base = nodes.as_mut_ptr();
        unsafe {
            tree.insert(node_ptr(base, 0));
            tree.insert(node_ptr(base, 1));
            tree.insert(node_ptr(base, 2));
        }
        assert_eq!(tree.len(), 3);

        unsafe { tree.remove(node_ptr(base, 2)) };
        assert_eq!(tree.len(), 2);
        assert!(tree.find(30).is_null());
        assert!(!tree.find(10).is_null());
        assert!(!tree.find(20).is_null());
    }

    #[test]
    fn test_remove_root() {
        let mut tree = RbTree::new();
        let mut nodes = [RbNode::new(10), RbNode::new(5), RbNode::new(15)];
        let base = nodes.as_mut_ptr();
        unsafe {
            tree.insert(node_ptr(base, 0));
            tree.insert(node_ptr(base, 1));
            tree.insert(node_ptr(base, 2));
        }

        let root_key = unsafe { (*tree.root).key };
        let root_ptr = tree.find(root_key);
        unsafe { tree.remove(root_ptr) };
        assert_eq!(tree.len(), 2);
        assert!(tree.find(root_key).is_null());
    }

    #[test]
    fn test_remove_all() {
        let mut tree = RbTree::new();
        let mut nodes: [RbNode; 8] = core::array::from_fn(|i| RbNode::new(i as u64));
        let base = nodes.as_mut_ptr();
        for i in 0..8 {
            unsafe { tree.insert(node_ptr(base, i)) };
        }

        for i in 0..8 {
            unsafe { tree.remove(node_ptr(base, i)) };
        }
        assert!(tree.is_empty());
    }

    #[test]
    fn test_inorder_iter() {
        let mut tree = RbTree::new();
        let keys = [50u64, 20, 80, 10, 30, 60, 90, 5, 15, 25];
        let mut nodes: [RbNode; 10] = core::array::from_fn(|i| RbNode::new(keys[i]));
        let base = nodes.as_mut_ptr();
        for i in 0..10 {
            unsafe { tree.insert(node_ptr(base, i)) };
        }

        let iter = RbInorderIter::new(&tree);
        let collected: Vec<u64> = iter.map(|n| unsafe { (*n).key }).collect();

        let mut sorted = keys;
        sorted.sort();
        assert_eq!(collected, sorted.to_vec());
    }

    #[test]
    fn test_insert_remove_interleaved() {
        let mut tree = RbTree::new();
        let mut nodes: [RbNode; 20] = core::array::from_fn(|i| RbNode::new(i as u64));
        let base = nodes.as_mut_ptr();

        for i in 0..20 {
            unsafe { tree.insert(node_ptr(base, i)) };
        }
        assert_eq!(tree.len(), 20);

        for i in (0..20).step_by(2) {
            unsafe { tree.remove(node_ptr(base, i)) };
        }
        assert_eq!(tree.len(), 10);

        for i in 0..20 {
            if i % 2 == 0 {
                assert!(tree.find(i as u64).is_null());
            } else {
                assert!(!tree.find(i as u64).is_null());
            }
        }
    }

    /// Verify red-black tree invariants:
    /// 1. min() and max() are valid (non-null when tree is non-empty)
    /// 2. Inorder traversal yields keys in sorted (ascending) order
    /// 3. len() matches the number of nodes visited by the iterator
    fn verify_rb_properties(tree: &RbTree) {
        if tree.is_empty() {
            assert!(tree.min().is_null());
            assert!(tree.max().is_null());
            assert_eq!(tree.len(), 0);
            return;
        }

        // Property 1: min and max are valid
        let min_ptr = tree.min();
        let max_ptr = tree.max();
        assert!(
            !min_ptr.is_null(),
            "min must be non-null for non-empty tree"
        );
        assert!(
            !max_ptr.is_null(),
            "max must be non-null for non-empty tree"
        );

        // Property 2 & 3: inorder traversal is sorted and count matches len
        let iter = RbInorderIter::new(tree);
        let mut count = 0usize;
        let mut prev_key: Option<u64> = None;
        for node in iter {
            let key = unsafe { (*node).key };
            if let Some(prev) = prev_key {
                assert!(key >= prev, "inorder violation: {} followed {}", key, prev);
            }
            prev_key = Some(key);
            count += 1;
        }
        assert_eq!(
            count,
            tree.len(),
            "iterator count ({}) does not match len ({})",
            count,
            tree.len()
        );

        // Verify min and max are consistent with traversal
        let min_key = unsafe { (*min_ptr).key };
        let max_key = unsafe { (*max_ptr).key };
        // Re-traverse to get first and last
        let mut iter2 = RbInorderIter::new(tree);
        let first_key = unsafe { (*iter2.next().unwrap()).key };
        let mut last_key = first_key;
        for node in iter2 {
            last_key = unsafe { (*node).key };
        }
        assert_eq!(min_key, first_key, "min() key must equal first inorder key");
        assert_eq!(max_key, last_key, "max() key must equal last inorder key");
    }

    #[test]
    fn rbtree_property_verification() {
        let mut tree = RbTree::new();

        // 20 keys in a pseudo-random order (not sequential, not sorted)
        let keys: [u64; 20] = [
            42, 7, 91, 3, 55, 28, 73, 15, 64, 1, 88, 36, 50, 19, 82, 10, 67, 44, 23, 99,
        ];

        let mut nodes: [RbNode; 20] = core::array::from_fn(|i| RbNode::new(keys[i]));
        let base = nodes.as_mut_ptr();

        // Insert all 20 nodes, verify properties after each insert
        for i in 0..20 {
            unsafe { tree.insert(node_ptr(base, i)) };
            verify_rb_properties(&tree);
            assert_eq!(tree.len(), i + 1);
        }

        // Remove them one by one, verify after each remove
        // Remove in a different order than insertion
        let remove_order: [usize; 20] = [
            5, 0, 19, 12, 3, 17, 8, 1, 14, 6, 11, 9, 16, 2, 18, 4, 13, 7, 15, 10,
        ];

        for (step, &idx) in remove_order.iter().enumerate() {
            unsafe { tree.remove(node_ptr(base, idx)) };
            verify_rb_properties(&tree);
            assert_eq!(tree.len(), 20 - step - 1);
        }

        assert!(tree.is_empty());
    }

    #[test]
    fn test_large_tree() {
        let mut tree = RbTree::new();
        let mut nodes: [RbNode; 100] = core::array::from_fn(|i| RbNode::new(i as u64));
        let base = nodes.as_mut_ptr();
        for i in 0..100 {
            unsafe { tree.insert(node_ptr(base, i)) };
        }
        assert_eq!(tree.len(), 100);
        assert_eq!(unsafe { (*tree.min()).key }, 0);
        assert_eq!(unsafe { (*tree.max()).key }, 99);

        let iter = RbInorderIter::new(&tree);
        let mut prev = None;
        let mut count = 0;
        for node in iter {
            let key = unsafe { (*node).key };
            if let Some(p) = prev {
                assert!(key > p, "in-order violation");
            }
            prev = Some(key);
            count += 1;
        }
        assert_eq!(count, 100);
    }
}

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
use alloc::vec::Vec;
