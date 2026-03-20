/*
 * alloc_test.c — Host-compiled tests for GenPmm and GenPool.
 */

#include "gen_alloc.h"
#include "gen_bitops.h"

static int failures = 0;

#define ASSERT(cond, msg)                                        \
    do {                                                         \
        if (!(cond)) {                                           \
            failures++;                                          \
        }                                                        \
    } while (0)

/* ── PMM tests ─────────────────────────────────────────────── */

static void test_pmm_init(void)
{
    uint32_t bitmap[4] = {0};
    GenPmm pmm;
    gen_pmm_init(&pmm, bitmap, 128, 0x100000, 4096);
    ASSERT(gen_pmm_free_count(&pmm) == 128, "pmm init free");
    ASSERT(gen_pmm_total_count(&pmm) == 128, "pmm init total");
}

static void test_pmm_alloc_free(void)
{
    uint32_t bitmap[4] = {0};
    GenPmm pmm;
    gen_pmm_init(&pmm, bitmap, 128, 0x100000, 4096);

    uint64_t page = gen_pmm_alloc_page(&pmm);
    ASSERT(page == 0x100000, "pmm first alloc");
    ASSERT(gen_pmm_free_count(&pmm) == 127, "pmm free after alloc");
    ASSERT(gen_pmm_is_allocated(&pmm, page), "pmm is_allocated");

    gen_pmm_free_page(&pmm, page);
    ASSERT(gen_pmm_free_count(&pmm) == 128, "pmm free after free");
    ASSERT(!gen_pmm_is_allocated(&pmm, page), "pmm not allocated after free");
}

static void test_pmm_exhaust(void)
{
    uint32_t bitmap[1] = {0};
    GenPmm pmm;
    gen_pmm_init(&pmm, bitmap, 4, 0x0, 4096);

    for (int i = 0; i < 4; i++) {
        ASSERT(gen_pmm_alloc_page(&pmm) != UINT64_MAX, "pmm alloc");
    }
    ASSERT(gen_pmm_alloc_page(&pmm) == UINT64_MAX, "pmm exhausted");
    ASSERT(gen_pmm_free_count(&pmm) == 0, "pmm all allocated");
}

static void test_pmm_contiguous(void)
{
    uint32_t bitmap[2] = {0};
    GenPmm pmm;
    gen_pmm_init(&pmm, bitmap, 64, 0x200000, 4096);

    uint64_t addr = gen_pmm_alloc_contiguous(&pmm, 8);
    ASSERT(addr == 0x200000, "pmm contiguous first");
    ASSERT(gen_pmm_free_count(&pmm) == 56, "pmm contiguous free");

    /* All 8 pages should be allocated */
    for (int i = 0; i < 8; i++) {
        ASSERT(gen_pmm_is_allocated(&pmm, 0x200000 + (uint64_t)i * 4096),
               "pmm contiguous page allocated");
    }
}

static void test_pmm_contiguous_fail(void)
{
    uint32_t bitmap[1] = {0};
    GenPmm pmm;
    gen_pmm_init(&pmm, bitmap, 8, 0x0, 4096);

    /* Allocate pages 0,2,4,6 to fragment */
    gen_pmm_alloc_page(&pmm); /* 0 */
    gen_pmm_alloc_page(&pmm); /* 1 */
    gen_pmm_free_page(&pmm, 0x0000);
    gen_pmm_free_page(&pmm, 0x2000);
    /* Now free: 0,3,4,5,6,7 and allocated: 1,2... wait, let me redo */

    /* Start fresh */
    gen_pmm_init(&pmm, bitmap, 8, 0x0, 4096);
    /* Reserve every other page */
    gen_pmm_mark_reserved(&pmm, 0x0000, 1); /* page 0 */
    gen_pmm_mark_reserved(&pmm, 0x2000, 1); /* page 2 */
    gen_pmm_mark_reserved(&pmm, 0x4000, 1); /* page 4 */
    gen_pmm_mark_reserved(&pmm, 0x6000, 1); /* page 6 */
    /* Free pages: 1,3,5,7 — no contiguous run of 2 */
    ASSERT(gen_pmm_alloc_contiguous(&pmm, 2) == UINT64_MAX,
           "pmm contiguous fragmented");
}

static void test_pmm_mark_reserved(void)
{
    uint32_t bitmap[1] = {0};
    GenPmm pmm;
    gen_pmm_init(&pmm, bitmap, 16, 0x0, 4096);

    gen_pmm_mark_reserved(&pmm, 0x0, 4);
    ASSERT(gen_pmm_free_count(&pmm) == 12, "pmm reserved free count");
    ASSERT(gen_pmm_is_allocated(&pmm, 0x0), "pmm reserved page 0");
    ASSERT(gen_pmm_is_allocated(&pmm, 0x3000), "pmm reserved page 3");
    ASSERT(!gen_pmm_is_allocated(&pmm, 0x4000), "pmm page 4 free");
}

/* ── Pool tests ────────────────────────────────────────────── */

static void test_pool_init(void)
{
    uint8_t region[256];
    GenPool pool;
    gen_pool_init(&pool, region, 32, 8);
    ASSERT(gen_pool_free_count(&pool) == 8, "pool init free");
    ASSERT(gen_pool_total_count(&pool) == 8, "pool init total");
}

static void test_pool_alloc_free(void)
{
    uint8_t region[256];
    GenPool pool;
    gen_pool_init(&pool, region, 32, 8);

    void *a = gen_pool_alloc(&pool);
    ASSERT(a != 0, "pool alloc non-null");
    ASSERT(gen_pool_free_count(&pool) == 7, "pool free after alloc");

    gen_pool_free(&pool, a);
    ASSERT(gen_pool_free_count(&pool) == 8, "pool free after free");
}

static void test_pool_exhaust(void)
{
    uint8_t region[128];
    GenPool pool;
    gen_pool_init(&pool, region, 16, 8);

    void *ptrs[8];
    for (int i = 0; i < 8; i++) {
        ptrs[i] = gen_pool_alloc(&pool);
        ASSERT(ptrs[i] != 0, "pool alloc");
    }
    ASSERT(gen_pool_alloc(&pool) == 0, "pool exhausted");
    ASSERT(gen_pool_free_count(&pool) == 0, "pool zero free");

    /* Free all and verify */
    for (int i = 0; i < 8; i++) {
        gen_pool_free(&pool, ptrs[i]);
    }
    ASSERT(gen_pool_free_count(&pool) == 8, "pool all freed");
}

static void test_pool_unique_blocks(void)
{
    uint8_t region[256];
    GenPool pool;
    gen_pool_init(&pool, region, 32, 8);

    void *ptrs[8];
    for (int i = 0; i < 8; i++) {
        ptrs[i] = gen_pool_alloc(&pool);
    }

    /* All pointers must be distinct */
    for (int i = 0; i < 8; i++) {
        for (int j = i + 1; j < 8; j++) {
            ASSERT(ptrs[i] != ptrs[j], "pool unique blocks");
        }
    }

    for (int i = 0; i < 8; i++) {
        gen_pool_free(&pool, ptrs[i]);
    }
}

/* ── Main ──────────────────────────────────────────────────── */

int main(void)
{
    test_pmm_init();
    test_pmm_alloc_free();
    test_pmm_exhaust();
    test_pmm_contiguous();
    test_pmm_contiguous_fail();
    test_pmm_mark_reserved();

    test_pool_init();
    test_pool_alloc_free();
    test_pool_exhaust();
    test_pool_unique_blocks();

    return failures;
}
