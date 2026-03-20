//! Intrusive doubly-linked list.
//!
//! Linux-style embedded list head. The list node is embedded in the
//! containing struct. No heap allocation — the caller manages the
//! lifetime of nodes.
//!
//! # Safety
//!
//! This module uses raw pointers because intrusive lists inherently
//! require aliasing. The caller is responsible for ensuring nodes
//! are not freed while linked.

use core::ptr;

/// A list node embedded in the containing structure.
///
/// Both `next` and `prev` point to other `ListNode`s. A standalone
/// (unlinked) node points to itself.
pub struct ListNode {
    pub next: *mut ListNode,
    pub prev: *mut ListNode,
}

impl ListNode {
    /// Create a new unlinked node (points to itself).
    pub const fn new() -> Self {
        Self {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }

    /// Initialize a node to point to itself (sentinel state).
    ///
    /// # Safety
    /// `node` must be a valid pointer.
    pub unsafe fn init(node: *mut ListNode) {
        unsafe {
            (*node).next = node;
            (*node).prev = node;
        }
    }

    /// Returns true if the node is not linked into any list.
    ///
    /// # Safety
    /// `node` must be a valid pointer.
    pub unsafe fn is_unlinked(node: *const ListNode) -> bool {
        unsafe { (*node).next == node as *mut ListNode || (*node).next.is_null() }
    }
}

/// An intrusive doubly-linked list with a sentinel head node.
pub struct IntrusiveList {
    head: ListNode,
}

impl IntrusiveList {
    /// Create a new empty list.
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(),
        }
    }

    /// Initialize the list (must be called before use if created with `new`).
    pub fn init(&mut self) {
        self.head.next = &mut self.head as *mut ListNode;
        self.head.prev = &mut self.head as *mut ListNode;
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        let head_ptr = &self.head as *const ListNode;
        self.head.next == head_ptr as *mut ListNode
    }

    /// Insert `node` at the front of the list (after head).
    ///
    /// # Safety
    /// `node` must be a valid pointer and must not already be in a list.
    pub unsafe fn push_front(&mut self, node: *mut ListNode) {
        unsafe {
            let head = &mut self.head as *mut ListNode;
            (*node).next = (*head).next;
            (*node).prev = head;
            (*(*head).next).prev = node;
            (*head).next = node;
        }
    }

    /// Insert `node` at the back of the list (before head).
    ///
    /// # Safety
    /// `node` must be a valid pointer and must not already be in a list.
    pub unsafe fn push_back(&mut self, node: *mut ListNode) {
        unsafe {
            let head = &mut self.head as *mut ListNode;
            (*node).next = head;
            (*node).prev = (*head).prev;
            (*(*head).prev).next = node;
            (*head).prev = node;
        }
    }

    /// Remove and return the front node. Returns null if empty.
    pub unsafe fn pop_front(&mut self) -> *mut ListNode {
        if self.is_empty() {
            return ptr::null_mut();
        }
        let node = self.head.next;
        unsafe { Self::remove(node) };
        node
    }

    /// Remove and return the back node. Returns null if empty.
    pub unsafe fn pop_back(&mut self) -> *mut ListNode {
        if self.is_empty() {
            return ptr::null_mut();
        }
        let node = self.head.prev;
        unsafe { Self::remove(node) };
        node
    }

    /// Remove a node from whatever list it is in.
    ///
    /// # Safety
    /// `node` must be linked into a list.
    pub unsafe fn remove(node: *mut ListNode) {
        unsafe {
            (*(*node).prev).next = (*node).next;
            (*(*node).next).prev = (*node).prev;
            (*node).next = node;
            (*node).prev = node;
        }
    }

    /// Returns a pointer to the front node, or null if empty.
    pub fn front(&self) -> *mut ListNode {
        if self.is_empty() {
            return ptr::null_mut();
        }
        self.head.next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_list() {
        let mut list = IntrusiveList::new();
        list.init();
        assert!(list.is_empty());
        assert!(list.front().is_null());
    }

    #[test]
    fn test_push_pop_front() {
        let mut list = IntrusiveList::new();
        list.init();

        let mut a = ListNode::new();
        let mut b = ListNode::new();
        unsafe {
            ListNode::init(&mut a);
            ListNode::init(&mut b);

            list.push_front(&mut a);
            assert!(!list.is_empty());

            list.push_front(&mut b);

            // b is at front, a is at back
            let front = list.pop_front();
            assert_eq!(front, &mut b as *mut ListNode);

            let front = list.pop_front();
            assert_eq!(front, &mut a as *mut ListNode);

            assert!(list.is_empty());
        }
    }

    #[test]
    fn test_push_back() {
        let mut list = IntrusiveList::new();
        list.init();

        let mut a = ListNode::new();
        let mut b = ListNode::new();
        unsafe {
            ListNode::init(&mut a);
            ListNode::init(&mut b);

            list.push_back(&mut a);
            list.push_back(&mut b);

            // a is at front, b is at back
            let front = list.pop_front();
            assert_eq!(front, &mut a as *mut ListNode);
        }
    }

    #[test]
    fn test_remove() {
        let mut list = IntrusiveList::new();
        list.init();

        let mut a = ListNode::new();
        let mut b = ListNode::new();
        let mut c = ListNode::new();
        unsafe {
            ListNode::init(&mut a);
            ListNode::init(&mut b);
            ListNode::init(&mut c);

            list.push_back(&mut a);
            list.push_back(&mut b);
            list.push_back(&mut c);

            // Remove middle element
            IntrusiveList::remove(&mut b);
            assert!(ListNode::is_unlinked(&b));

            let first = list.pop_front();
            assert_eq!(first, &mut a as *mut ListNode);

            let second = list.pop_front();
            assert_eq!(second, &mut c as *mut ListNode);

            assert!(list.is_empty());
        }
    }

    #[test]
    fn test_pop_empty() {
        let mut list = IntrusiveList::new();
        list.init();
        unsafe {
            assert!(list.pop_front().is_null());
            assert!(list.pop_back().is_null());
        }
    }
}
