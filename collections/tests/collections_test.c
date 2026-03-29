/*
 * collections_test.c --- Host-compiled tests for GenList and GenRbTree.
 */

#include "gen_collections.h"
#include <stdio.h>

static int failures = 0;

#define ASSERT(cond, msg)                                          \
    do {                                                           \
        if (!(cond)) {                                             \
            fprintf(stderr, "FAIL: %s (%s:%d)\n",                  \
                    msg, __FILE__, __LINE__);                       \
            failures++;                                            \
        }                                                          \
    } while (0)

/* ══════════════════════════════════════════════════════════════
 * List tests
 * ══════════════════════════════════════════════════════════════ */

static void test_list_empty(void)
{
    GenList list;
    gen_list_init(&list);
    ASSERT(gen_list_is_empty(&list), "new list is empty");
}

static void test_list_push_front(void)
{
    GenList list;
    gen_list_init(&list);

    GenListNode a, b;
    gen_list_push_front(&list, &a);
    ASSERT(!gen_list_is_empty(&list), "list not empty after push_front");

    gen_list_push_front(&list, &b);

    /* b is at front, a is at back */
    GenListNode *front = gen_list_pop_front(&list);
    ASSERT(front == &b, "push_front: b is front");

    front = gen_list_pop_front(&list);
    ASSERT(front == &a, "push_front: a is second");

    ASSERT(gen_list_is_empty(&list), "list empty after popping all");
}

static void test_list_push_back(void)
{
    GenList list;
    gen_list_init(&list);

    GenListNode a, b;
    gen_list_push_back(&list, &a);
    gen_list_push_back(&list, &b);

    /* a is at front, b is at back */
    GenListNode *front = gen_list_pop_front(&list);
    ASSERT(front == &a, "push_back: a is front");

    front = gen_list_pop_front(&list);
    ASSERT(front == &b, "push_back: b is second");
}

static void test_list_pop_front(void)
{
    GenList list;
    gen_list_init(&list);

    GenListNode a, b, c;
    gen_list_push_back(&list, &a);
    gen_list_push_back(&list, &b);
    gen_list_push_back(&list, &c);

    ASSERT(gen_list_pop_front(&list) == &a, "pop_front: a");
    ASSERT(gen_list_pop_front(&list) == &b, "pop_front: b");
    ASSERT(gen_list_pop_front(&list) == &c, "pop_front: c");
    ASSERT(gen_list_is_empty(&list), "pop_front: empty");
}

static void test_list_pop_back(void)
{
    GenList list;
    gen_list_init(&list);

    GenListNode a, b, c;
    gen_list_push_back(&list, &a);
    gen_list_push_back(&list, &b);
    gen_list_push_back(&list, &c);

    ASSERT(gen_list_pop_back(&list) == &c, "pop_back: c");
    ASSERT(gen_list_pop_back(&list) == &b, "pop_back: b");
    ASSERT(gen_list_pop_back(&list) == &a, "pop_back: a");
    ASSERT(gen_list_is_empty(&list), "pop_back: empty");
}

static void test_list_remove_middle(void)
{
    GenList list;
    gen_list_init(&list);

    GenListNode a, b, c;
    gen_list_push_back(&list, &a);
    gen_list_push_back(&list, &b);
    gen_list_push_back(&list, &c);

    gen_list_remove(&b);
    /* b should be unlinked (points to itself) */
    ASSERT(b.next == &b, "remove: b unlinked next");
    ASSERT(b.prev == &b, "remove: b unlinked prev");

    GenListNode *first = gen_list_pop_front(&list);
    ASSERT(first == &a, "remove middle: a is front");

    GenListNode *second = gen_list_pop_front(&list);
    ASSERT(second == &c, "remove middle: c is second");

    ASSERT(gen_list_is_empty(&list), "remove middle: empty");
}

static void test_list_pop_empty(void)
{
    GenList list;
    gen_list_init(&list);

    ASSERT(gen_list_pop_front(&list) == NULL, "pop_front empty returns NULL");
    ASSERT(gen_list_pop_back(&list) == NULL, "pop_back empty returns NULL");
}

/* ══════════════════════════════════════════════════════════════
 * Red-black tree tests
 * ══════════════════════════════════════════════════════════════ */

static void test_rb_empty(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);
    ASSERT(gen_rb_tree_is_empty(&tree), "new tree is empty");
    ASSERT(gen_rb_tree_len(&tree) == 0, "new tree len == 0");
    ASSERT(gen_rb_tree_min(&tree) == NULL, "empty tree min == NULL");
    ASSERT(gen_rb_tree_max(&tree) == NULL, "empty tree max == NULL");
    ASSERT(gen_rb_tree_find(&tree, 42) == NULL, "empty tree find == NULL");
}

static void test_rb_insert_single(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode n;
    n.key = 42;
    gen_rb_tree_insert(&tree, &n);

    ASSERT(gen_rb_tree_len(&tree) == 1, "insert single len == 1");
    ASSERT(!gen_rb_tree_is_empty(&tree), "insert single not empty");
    ASSERT(gen_rb_tree_min(&tree) != NULL, "insert single min != NULL");
    ASSERT(gen_rb_tree_min(&tree)->key == 42, "insert single min key == 42");
    ASSERT(gen_rb_tree_max(&tree)->key == 42, "insert single max key == 42");
}

static void test_rb_insert_ascending(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode nodes[10];
    for (int i = 0; i < 10; i++) {
        nodes[i].key = (uint64_t)i;
        gen_rb_tree_insert(&tree, &nodes[i]);
    }

    ASSERT(gen_rb_tree_len(&tree) == 10, "ascending len == 10");
    ASSERT(gen_rb_tree_min(&tree)->key == 0, "ascending min == 0");
    ASSERT(gen_rb_tree_max(&tree)->key == 9, "ascending max == 9");
}

static void test_rb_insert_descending(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode nodes[10];
    for (int i = 0; i < 10; i++) {
        nodes[i].key = (uint64_t)(9 - i);
        gen_rb_tree_insert(&tree, &nodes[i]);
    }

    ASSERT(gen_rb_tree_len(&tree) == 10, "descending len == 10");
    ASSERT(gen_rb_tree_min(&tree)->key == 0, "descending min == 0");
    ASSERT(gen_rb_tree_max(&tree)->key == 9, "descending max == 9");
}

static void test_rb_find_hit_miss(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode nodes[5];
    for (int i = 0; i < 5; i++) {
        nodes[i].key = (uint64_t)(i * 10);
        gen_rb_tree_insert(&tree, &nodes[i]);
    }

    GenRbNode *found = gen_rb_tree_find(&tree, 20);
    ASSERT(found != NULL, "find hit != NULL");
    ASSERT(found->key == 20, "find hit key == 20");

    ASSERT(gen_rb_tree_find(&tree, 15) == NULL, "find miss == NULL");
    ASSERT(gen_rb_tree_find(&tree, 50) == NULL, "find miss high == NULL");
}

static void test_rb_remove_leaf(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode nodes[3];
    nodes[0].key = 10;
    nodes[1].key = 20;
    nodes[2].key = 30;
    gen_rb_tree_insert(&tree, &nodes[0]);
    gen_rb_tree_insert(&tree, &nodes[1]);
    gen_rb_tree_insert(&tree, &nodes[2]);

    ASSERT(gen_rb_tree_len(&tree) == 3, "before remove len == 3");

    gen_rb_tree_remove(&tree, &nodes[2]);
    ASSERT(gen_rb_tree_len(&tree) == 2, "after remove leaf len == 2");
    ASSERT(gen_rb_tree_find(&tree, 30) == NULL, "removed 30 not found");
    ASSERT(gen_rb_tree_find(&tree, 10) != NULL, "10 still present");
    ASSERT(gen_rb_tree_find(&tree, 20) != NULL, "20 still present");
}

static void test_rb_remove_root(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode nodes[3];
    nodes[0].key = 10;
    nodes[1].key = 5;
    nodes[2].key = 15;
    gen_rb_tree_insert(&tree, &nodes[0]);
    gen_rb_tree_insert(&tree, &nodes[1]);
    gen_rb_tree_insert(&tree, &nodes[2]);

    /* Find and remove the root */
    uint64_t root_key = tree.root->key;
    GenRbNode *root_ptr = gen_rb_tree_find(&tree, root_key);
    ASSERT(root_ptr != NULL, "root found");
    gen_rb_tree_remove(&tree, root_ptr);

    ASSERT(gen_rb_tree_len(&tree) == 2, "after remove root len == 2");
    ASSERT(gen_rb_tree_find(&tree, root_key) == NULL, "root removed");
}

static void test_rb_remove_all(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode nodes[8];
    for (int i = 0; i < 8; i++) {
        nodes[i].key = (uint64_t)i;
        gen_rb_tree_insert(&tree, &nodes[i]);
    }
    ASSERT(gen_rb_tree_len(&tree) == 8, "insert 8 len == 8");

    for (int i = 0; i < 8; i++) {
        gen_rb_tree_remove(&tree, &nodes[i]);
    }
    ASSERT(gen_rb_tree_is_empty(&tree), "remove all: empty");
    ASSERT(gen_rb_tree_len(&tree) == 0, "remove all: len == 0");
}

/* Visitor context for collecting keys during in-order traversal. */
typedef struct {
    uint64_t *keys;
    int       count;
    int       capacity;
} CollectCtx;

static int collect_visitor(GenRbNode *node, void *ctx)
{
    CollectCtx *c = (CollectCtx *)ctx;
    if (c->count < c->capacity) {
        c->keys[c->count] = node->key;
        c->count++;
    }
    return 0;
}

static void test_rb_inorder_sorted(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    /* Insert in shuffled order */
    uint64_t insert_keys[10] = {50, 20, 80, 10, 30, 60, 90, 5, 15, 25};
    GenRbNode nodes[10];
    for (int i = 0; i < 10; i++) {
        nodes[i].key = insert_keys[i];
        gen_rb_tree_insert(&tree, &nodes[i]);
    }

    uint64_t collected[10];
    CollectCtx ctx = { collected, 0, 10 };
    gen_rb_tree_inorder(&tree, collect_visitor, &ctx);

    ASSERT(ctx.count == 10, "inorder visited all 10 nodes");

    /* Verify sorted ascending order */
    for (int i = 1; i < ctx.count; i++) {
        ASSERT(collected[i] > collected[i - 1], "inorder sorted");
    }
}

static void test_rb_large_tree(void)
{
    GenRbTree tree;
    gen_rb_tree_init(&tree);

    GenRbNode nodes[100];
    for (int i = 0; i < 100; i++) {
        nodes[i].key = (uint64_t)i;
        gen_rb_tree_insert(&tree, &nodes[i]);
    }

    ASSERT(gen_rb_tree_len(&tree) == 100, "large tree len == 100");
    ASSERT(gen_rb_tree_min(&tree)->key == 0, "large tree min == 0");
    ASSERT(gen_rb_tree_max(&tree)->key == 99, "large tree max == 99");

    /* Verify in-order traversal produces sorted output */
    uint64_t collected[100];
    CollectCtx ctx = { collected, 0, 100 };
    gen_rb_tree_inorder(&tree, collect_visitor, &ctx);

    ASSERT(ctx.count == 100, "large tree inorder count == 100");
    for (int i = 1; i < ctx.count; i++) {
        ASSERT(collected[i] > collected[i - 1], "large tree inorder sorted");
    }

    /* Find every key */
    for (int i = 0; i < 100; i++) {
        GenRbNode *found = gen_rb_tree_find(&tree, (uint64_t)i);
        ASSERT(found != NULL, "large tree find hit");
        ASSERT(found->key == (uint64_t)i, "large tree find key match");
    }

    /* Remove all and verify empty */
    for (int i = 0; i < 100; i++) {
        gen_rb_tree_remove(&tree, &nodes[i]);
    }
    ASSERT(gen_rb_tree_is_empty(&tree), "large tree remove all: empty");
}

/* ══════════════════════════════════════════════════════════════
 * Main
 * ══════════════════════════════════════════════════════════════ */

int main(void)
{
    /* List tests */
    test_list_empty();
    test_list_push_front();
    test_list_push_back();
    test_list_pop_front();
    test_list_pop_back();
    test_list_remove_middle();
    test_list_pop_empty();

    /* RbTree tests */
    test_rb_empty();
    test_rb_insert_single();
    test_rb_insert_ascending();
    test_rb_insert_descending();
    test_rb_find_hit_miss();
    test_rb_remove_leaf();
    test_rb_remove_root();
    test_rb_remove_all();
    test_rb_inorder_sorted();
    test_rb_large_tree();

    if (failures == 0)
        fprintf(stderr, "collections_test: all tests passed\n");
    else
        fprintf(stderr, "collections_test: %d failures\n", failures);

    return failures;
}
