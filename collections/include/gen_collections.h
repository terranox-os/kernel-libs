/*
 * gen_collections.h --- Intrusive doubly-linked list and red-black tree.
 *
 * C API for intrusive containers. The Rust crate provides equivalent
 * IntrusiveList and RbTree with the same semantics.
 *
 * Freestanding: requires only <stdint.h> and <stddef.h>.
 */

#ifndef GEN_COLLECTIONS_H
#define GEN_COLLECTIONS_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Intrusive Doubly-Linked List ───────────────────────────── */

/*
 * GenListNode: embedded in the containing structure.
 * A standalone (unlinked) node points to itself.
 */
typedef struct GenListNode {
    struct GenListNode *next;
    struct GenListNode *prev;
} GenListNode;

/*
 * GenList: circular doubly-linked list with a sentinel head node.
 * An empty list has head.next == head.prev == &head.
 */
typedef struct GenList {
    GenListNode head;
} GenList;

/*
 * Static initializer macro.
 * Usage: GenList my_list = GEN_LIST_INIT(my_list);
 */
#define GEN_LIST_INIT(name) { { &(name).head, &(name).head } }

/* Initialize a list at runtime. */
void gen_list_init(GenList *list);

/* Returns nonzero if the list has no elements. */
int gen_list_is_empty(const GenList *list);

/* Insert node at the front (after head). */
void gen_list_push_front(GenList *list, GenListNode *node);

/* Insert node at the back (before head). */
void gen_list_push_back(GenList *list, GenListNode *node);

/* Remove and return the front node, or NULL if empty. */
GenListNode *gen_list_pop_front(GenList *list);

/* Remove and return the back node, or NULL if empty. */
GenListNode *gen_list_pop_back(GenList *list);

/*
 * Remove a node from whatever list it is in.
 * After removal the node points to itself (unlinked).
 */
void gen_list_remove(GenListNode *node);

/* ── Intrusive Red-Black Tree ───────────────────────────────── */

/*
 * GenRbNode: embedded in the containing structure.
 * key is a uint64_t used for ordering.
 * Color: 0 = red, 1 = black.
 */
typedef struct GenRbNode {
    uint64_t          key;
    struct GenRbNode *left;
    struct GenRbNode *right;
    struct GenRbNode *parent;
    uint8_t           color;
} GenRbNode;

/*
 * GenRbTree: root pointer + element count.
 * An empty tree has root == NULL, len == 0.
 */
typedef struct GenRbTree {
    GenRbNode *root;
    size_t     len;
} GenRbTree;

/* Initialize a tree (empty). */
void gen_rb_tree_init(GenRbTree *tree);

/* Number of nodes in the tree. */
size_t gen_rb_tree_len(const GenRbTree *tree);

/* Returns nonzero if the tree is empty. */
int gen_rb_tree_is_empty(const GenRbTree *tree);

/*
 * Insert a node. The caller must set node->key before calling.
 * The node must not already be in any tree.
 */
void gen_rb_tree_insert(GenRbTree *tree, GenRbNode *node);

/*
 * Remove a node that is currently in this tree.
 * After removal the node's pointers are cleared.
 */
void gen_rb_tree_remove(GenRbTree *tree, GenRbNode *node);

/* Find a node by key. Returns NULL if not found. */
GenRbNode *gen_rb_tree_find(const GenRbTree *tree, uint64_t key);

/* Return the node with the minimum key, or NULL if empty. */
GenRbNode *gen_rb_tree_min(const GenRbTree *tree);

/* Return the node with the maximum key, or NULL if empty. */
GenRbNode *gen_rb_tree_max(const GenRbTree *tree);

/*
 * Visitor callback for in-order traversal.
 * Return 0 to continue, nonzero to stop early.
 */
typedef int (*gen_rb_visit_fn)(GenRbNode *node, void *ctx);

/*
 * In-order traversal (ascending key order).
 * Calls visitor for each node. Stops if visitor returns nonzero.
 * Uses an iterative stack-based approach (stack depth 64).
 */
void gen_rb_tree_inorder(const GenRbTree *tree, gen_rb_visit_fn visitor,
                         void *ctx);

#ifdef __cplusplus
}
#endif

#endif /* GEN_COLLECTIONS_H */
