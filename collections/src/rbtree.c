/*
 * rbtree.c --- Intrusive red-black tree (Cormen et al., CLRS).
 *
 * Mirrors the Rust RbTree implementation. Uses NULL as the sentinel
 * (NIL) node. Colors: RED = 0, BLACK = 1.
 *
 * Freestanding: no libc, no heap.
 */

#include "gen_collections.h"

#define RED   0
#define BLACK 1

/* ── Static helpers ─────────────────────────────────────────── */

static void rotate_left(GenRbTree *tree, GenRbNode *x)
{
    GenRbNode *y = x->right;

    x->right = y->left;
    if (y->left != NULL)
        y->left->parent = x;

    y->parent = x->parent;

    if (x->parent == NULL)
        tree->root = y;
    else if (x == x->parent->left)
        x->parent->left = y;
    else
        x->parent->right = y;

    y->left = x;
    x->parent = y;
}

static void rotate_right(GenRbTree *tree, GenRbNode *x)
{
    GenRbNode *y = x->left;

    x->left = y->right;
    if (y->right != NULL)
        y->right->parent = x;

    y->parent = x->parent;

    if (x->parent == NULL)
        tree->root = y;
    else if (x == x->parent->right)
        x->parent->right = y;
    else
        x->parent->left = y;

    y->right = x;
    x->parent = y;
}

/*
 * Replace subtree rooted at u with subtree rooted at v.
 * v may be NULL.
 */
static void transplant(GenRbTree *tree, GenRbNode *u, GenRbNode *v)
{
    if (u->parent == NULL)
        tree->root = v;
    else if (u == u->parent->left)
        u->parent->left = v;
    else
        u->parent->right = v;

    if (v != NULL)
        v->parent = u->parent;
}

static GenRbNode *subtree_min(GenRbNode *node)
{
    while (node->left != NULL)
        node = node->left;
    return node;
}

static GenRbNode *subtree_max(GenRbNode *node)
{
    while (node->right != NULL)
        node = node->right;
    return node;
}

/* ── Insert fixup (CLRS) ───────────────────────────────────── */

static void insert_fixup(GenRbTree *tree, GenRbNode *z)
{
    while (z->parent != NULL && z->parent->color == RED) {
        GenRbNode *grandparent = z->parent->parent;
        if (grandparent == NULL)
            break;

        if (z->parent == grandparent->left) {
            GenRbNode *uncle = grandparent->right;

            if (uncle != NULL && uncle->color == RED) {
                /* Case 1: uncle is red */
                z->parent->color = BLACK;
                uncle->color = BLACK;
                grandparent->color = RED;
                z = grandparent;
            } else {
                if (z == z->parent->right) {
                    /* Case 2: z is right child --- rotate to case 3 */
                    z = z->parent;
                    rotate_left(tree, z);
                }
                /* Case 3: z is left child */
                z->parent->color = BLACK;
                z->parent->parent->color = RED;
                rotate_right(tree, z->parent->parent);
            }
        } else {
            /* Mirror: parent is right child of grandparent */
            GenRbNode *uncle = grandparent->left;

            if (uncle != NULL && uncle->color == RED) {
                z->parent->color = BLACK;
                uncle->color = BLACK;
                grandparent->color = RED;
                z = grandparent;
            } else {
                if (z == z->parent->left) {
                    z = z->parent;
                    rotate_right(tree, z);
                }
                z->parent->color = BLACK;
                z->parent->parent->color = RED;
                rotate_left(tree, z->parent->parent);
            }
        }
    }
    tree->root->color = BLACK;
}

/* ── Delete fixup (CLRS) ───────────────────────────────────── */

static void delete_fixup(GenRbTree *tree, GenRbNode *x, GenRbNode *x_parent)
{
    while (x != tree->root && (x == NULL || x->color == BLACK)) {
        if (x_parent == NULL)
            break;

        if (x == x_parent->left) {
            GenRbNode *w = x_parent->right;

            if (w != NULL && w->color == RED) {
                /* Case 1: sibling is red */
                w->color = BLACK;
                x_parent->color = RED;
                rotate_left(tree, x_parent);
                w = x_parent->right;
            }

            if (w == NULL) {
                x = x_parent;
                x_parent = x->parent;
                continue;
            }

            int wl_black = (w->left == NULL || w->left->color == BLACK);
            int wr_black = (w->right == NULL || w->right->color == BLACK);

            if (wl_black && wr_black) {
                /* Case 2: both of sibling's children are black */
                w->color = RED;
                x = x_parent;
                x_parent = x->parent;
            } else {
                if (wr_black) {
                    /* Case 3: sibling's right child is black */
                    if (w->left != NULL)
                        w->left->color = BLACK;
                    w->color = RED;
                    rotate_right(tree, w);
                    w = x_parent->right;
                }
                /* Case 4: sibling's right child is red */
                if (w != NULL)
                    w->color = x_parent->color;
                x_parent->color = BLACK;
                if (w != NULL && w->right != NULL)
                    w->right->color = BLACK;
                rotate_left(tree, x_parent);
                x = tree->root;
                x_parent = NULL;
            }
        } else {
            /* Mirror: x is right child */
            GenRbNode *w = x_parent->left;

            if (w != NULL && w->color == RED) {
                w->color = BLACK;
                x_parent->color = RED;
                rotate_right(tree, x_parent);
                w = x_parent->left;
            }

            if (w == NULL) {
                x = x_parent;
                x_parent = x->parent;
                continue;
            }

            int wr_black = (w->right == NULL || w->right->color == BLACK);
            int wl_black = (w->left == NULL || w->left->color == BLACK);

            if (wr_black && wl_black) {
                w->color = RED;
                x = x_parent;
                x_parent = x->parent;
            } else {
                if (wl_black) {
                    if (w->right != NULL)
                        w->right->color = BLACK;
                    w->color = RED;
                    rotate_left(tree, w);
                    w = x_parent->left;
                }
                if (w != NULL)
                    w->color = x_parent->color;
                x_parent->color = BLACK;
                if (w != NULL && w->left != NULL)
                    w->left->color = BLACK;
                rotate_right(tree, x_parent);
                x = tree->root;
                x_parent = NULL;
            }
        }
    }
    if (x != NULL)
        x->color = BLACK;
}

/* ── Public API ─────────────────────────────────────────────── */

void gen_rb_tree_init(GenRbTree *tree)
{
    tree->root = NULL;
    tree->len = 0;
}

size_t gen_rb_tree_len(const GenRbTree *tree)
{
    return tree->len;
}

int gen_rb_tree_is_empty(const GenRbTree *tree)
{
    return tree->root == NULL;
}

void gen_rb_tree_insert(GenRbTree *tree, GenRbNode *node)
{
    node->left = NULL;
    node->right = NULL;
    node->color = RED;

    /* Standard BST insert */
    GenRbNode *parent = NULL;
    GenRbNode *current = tree->root;

    while (current != NULL) {
        parent = current;
        if (node->key < current->key)
            current = current->left;
        else
            current = current->right;
    }

    node->parent = parent;

    if (parent == NULL)
        tree->root = node;
    else if (node->key < parent->key)
        parent->left = node;
    else
        parent->right = node;

    tree->len++;
    insert_fixup(tree, node);
}

void gen_rb_tree_remove(GenRbTree *tree, GenRbNode *node)
{
    GenRbNode *y = node;
    uint8_t y_orig_color = y->color;
    GenRbNode *x;
    GenRbNode *x_parent;

    if (node->left == NULL) {
        x = node->right;
        x_parent = node->parent;
        transplant(tree, node, node->right);
    } else if (node->right == NULL) {
        x = node->left;
        x_parent = node->parent;
        transplant(tree, node, node->left);
    } else {
        /* Two children: find in-order successor */
        y = subtree_min(node->right);
        y_orig_color = y->color;
        x = y->right;

        if (y->parent == node) {
            if (x != NULL)
                x->parent = y;
            x_parent = y;
        } else {
            x_parent = y->parent;
            transplant(tree, y, y->right);
            y->right = node->right;
            if (y->right != NULL)
                y->right->parent = y;
        }
        transplant(tree, node, y);
        y->left = node->left;
        if (y->left != NULL)
            y->left->parent = y;
        y->color = node->color;
    }

    tree->len--;

    /* Clear removed node's links */
    node->left = NULL;
    node->right = NULL;
    node->parent = NULL;

    if (y_orig_color == BLACK)
        delete_fixup(tree, x, x_parent);
}

GenRbNode *gen_rb_tree_find(const GenRbTree *tree, uint64_t key)
{
    GenRbNode *current = tree->root;

    while (current != NULL) {
        if (key == current->key)
            return current;
        else if (key < current->key)
            current = current->left;
        else
            current = current->right;
    }
    return NULL;
}

GenRbNode *gen_rb_tree_min(const GenRbTree *tree)
{
    if (tree->root == NULL)
        return NULL;
    return subtree_min(tree->root);
}

GenRbNode *gen_rb_tree_max(const GenRbTree *tree)
{
    if (tree->root == NULL)
        return NULL;
    return subtree_max(tree->root);
}

void gen_rb_tree_inorder(const GenRbTree *tree, gen_rb_visit_fn visitor,
                         void *ctx)
{
    if (tree->root == NULL || visitor == NULL)
        return;

    /* Iterative in-order traversal with explicit stack. */
    GenRbNode *stack[64];
    int sp = 0;
    GenRbNode *current = tree->root;

    for (;;) {
        /* Push left chain */
        while (current != NULL && sp < 64) {
            stack[sp++] = current;
            current = current->left;
        }

        if (sp == 0)
            break;

        current = stack[--sp];

        if (visitor(current, ctx) != 0)
            return;

        current = current->right;
    }
}
