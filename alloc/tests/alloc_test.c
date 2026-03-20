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

/* ── Slab cache test helpers ────────────────────────────────── */

static uint8_t __attribute__((aligned(4096))) slab_pages[4][4096];
static int slab_page_idx = 0;
static int slab_pages_freed = 0;

static void *test_page_alloc(void *ctx, uint32_t page_size)
{
    (void)ctx; (void)page_size;
    if (slab_page_idx >= 4) return (void *)0;
    return &slab_pages[slab_page_idx++];
}

static void test_page_free(void *ctx, void *page, uint32_t page_size)
{
    (void)ctx; (void)page; (void)page_size;
    slab_pages_freed++;
}

static void slab_test_reset(void)
{
    slab_page_idx = 0;
    slab_pages_freed = 0;
}

/* ── Slab cache tests ──────────────────────────────────────── */

static void test_slab_cache_init(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 64, 8, 4096, (void *)0, (void *)0, (void *)0);
    ASSERT(cache.obj_size == 64, "slab init obj_size");
    ASSERT(cache.objs_per_slab == 64, "slab init objs_per_slab");
    ASSERT(gen_slab_cache_free_count(&cache) == 0, "slab init free");
    ASSERT(gen_slab_cache_slab_count(&cache) == 0, "slab init slabs");
}

static void test_slab_cache_init_small_obj(void)
{
    GenSlabCache cache;
    /* obj_size < sizeof(void*) should be rounded up */
    gen_slab_cache_init(&cache, 1, 1, 4096, (void *)0, (void *)0, (void *)0);
    ASSERT(cache.obj_size >= sizeof(void *), "slab min obj size");
    ASSERT(cache.objs_per_slab > 0, "slab objs_per_slab > 0");
}

static void test_slab_add_slab(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 64, 8, 4096, (void *)0, (void *)0, (void *)0);

    uint8_t __attribute__((aligned(64))) page[4096];
    GenSlab slab;
    GenResult r = gen_slab_cache_add_slab(&cache, &slab, page);
    ASSERT(r == GEN_OK, "slab add ok");
    ASSERT(gen_slab_cache_slab_count(&cache) == 1, "slab count 1");
    ASSERT(gen_slab_cache_free_count(&cache) == 64, "slab free 64");
}

static void test_slab_alloc_single(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 64, 8, 4096, (void *)0, (void *)0, (void *)0);

    uint8_t __attribute__((aligned(64))) page[4096];
    GenSlab slab;
    gen_slab_cache_add_slab(&cache, &slab, page);

    void *obj = gen_slab_cache_alloc(&cache);
    ASSERT(obj != (void *)0, "slab alloc non-null");
    ASSERT(gen_slab_cache_free_count(&cache) == 63, "slab free 63");
    ASSERT(gen_slab_cache_allocated_count(&cache) == 1, "slab alloc 1");
}

static void test_slab_alloc_free(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 128, 8, 4096, (void *)0, (void *)0, (void *)0);

    uint8_t __attribute__((aligned(128))) page[4096];
    GenSlab slab;
    gen_slab_cache_add_slab(&cache, &slab, page);

    void *obj = gen_slab_cache_alloc(&cache);
    gen_slab_cache_free(&cache, obj);
    ASSERT(gen_slab_cache_free_count(&cache) == 32, "slab free restored");
    ASSERT(gen_slab_cache_allocated_count(&cache) == 0, "slab alloc 0");
}

static void test_slab_exhaust_single(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 256, 8, 4096, (void *)0, (void *)0, (void *)0);

    uint8_t __attribute__((aligned(256))) page[4096];
    GenSlab slab;
    gen_slab_cache_add_slab(&cache, &slab, page);

    uint32_t n = cache.objs_per_slab; /* 4096/256 = 16 */
    for (uint32_t i = 0; i < n; i++) {
        ASSERT(gen_slab_cache_alloc(&cache) != (void *)0, "slab alloc loop");
    }
    ASSERT(gen_slab_cache_alloc(&cache) == (void *)0, "slab exhausted");
    ASSERT(gen_slab_cache_free_count(&cache) == 0, "slab free 0");
}

static void test_slab_multi_slab(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 512, 8, 4096, (void *)0, (void *)0, (void *)0);

    uint8_t __attribute__((aligned(512))) page1[4096];
    uint8_t __attribute__((aligned(512))) page2[4096];
    GenSlab slab1, slab2;

    gen_slab_cache_add_slab(&cache, &slab1, page1);
    gen_slab_cache_add_slab(&cache, &slab2, page2);

    uint32_t total = gen_slab_cache_free_count(&cache);
    ASSERT(total == 16, "slab multi total"); /* 2 * (4096/512) = 16 */

    for (uint32_t i = 0; i < total; i++) {
        ASSERT(gen_slab_cache_alloc(&cache) != (void *)0, "slab multi alloc");
    }
    ASSERT(gen_slab_cache_alloc(&cache) == (void *)0, "slab multi exhausted");
}

static void test_slab_free_state_transitions(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 1024, 8, 4096, (void *)0, (void *)0, (void *)0);

    uint8_t __attribute__((aligned(1024))) page[4096];
    GenSlab slab;
    gen_slab_cache_add_slab(&cache, &slab, page);

    /* 4096/1024 = 4 objects */
    void *objs[4];
    for (int i = 0; i < 4; i++) {
        objs[i] = gen_slab_cache_alloc(&cache);
    }
    /* slab should be full */
    ASSERT(gen_slab_cache_free_count(&cache) == 0, "slab full");

    /* Free one — moves full → partial */
    gen_slab_cache_free(&cache, objs[0]);
    ASSERT(gen_slab_cache_free_count(&cache) == 1, "slab partial");

    /* Free remaining — moves partial → empty */
    for (int i = 1; i < 4; i++) {
        gen_slab_cache_free(&cache, objs[i]);
    }
    ASSERT(gen_slab_cache_free_count(&cache) == 4, "slab empty again");
}

static void test_slab_unique_objects(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 64, 8, 4096, (void *)0, (void *)0, (void *)0);

    uint8_t __attribute__((aligned(64))) page[4096];
    GenSlab slab;
    gen_slab_cache_add_slab(&cache, &slab, page);

    void *ptrs[64];
    for (int i = 0; i < 64; i++) {
        ptrs[i] = gen_slab_cache_alloc(&cache);
        ASSERT(ptrs[i] != (void *)0, "slab unique alloc");
        /* Check within page bounds */
        ASSERT((uint8_t *)ptrs[i] >= page, "slab in bounds lo");
        ASSERT((uint8_t *)ptrs[i] < page + 4096, "slab in bounds hi");
    }

    /* All distinct */
    for (int i = 0; i < 64; i++) {
        for (int j = i + 1; j < 64; j++) {
            ASSERT(ptrs[i] != ptrs[j], "slab unique ptrs");
        }
    }

    for (int i = 0; i < 64; i++) {
        gen_slab_cache_free(&cache, ptrs[i]);
    }
}

static void test_slab_alloc_grow(void)
{
    slab_test_reset();
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 64, 8, 4096,
                        test_page_alloc, test_page_free, (void *)0);

    /* No slabs yet — alloc should fail */
    ASSERT(gen_slab_cache_alloc(&cache) == (void *)0, "slab grow empty");

    /* alloc_grow with a slab buffer should auto-add a slab */
    GenSlab slab_buf;
    void *obj = gen_slab_cache_alloc_grow(&cache, &slab_buf);
    ASSERT(obj != (void *)0, "slab grow alloc");
    ASSERT(gen_slab_cache_slab_count(&cache) == 1, "slab grow count");
    ASSERT(slab_page_idx == 1, "slab grow page used");

    gen_slab_cache_free(&cache, obj);
}

static void test_slab_shrink(void)
{
    slab_test_reset();
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 64, 8, 4096,
                        test_page_alloc, test_page_free, (void *)0);

    GenSlab slabs[2];
    gen_slab_cache_alloc_grow(&cache, &slabs[0]);
    gen_slab_cache_alloc_grow(&cache, &slabs[1]); /* slabs[0] is full, grows new */

    /* Free everything to make slabs empty */
    /* slab 0 has objs_per_slab-0 free (it was alloc_grow, which alloced 1 from it) */
    /* Actually let's just free all allocated */
    while (gen_slab_cache_allocated_count(&cache) > 0) {
        /* We don't have the pointers easily; let's do a simpler test */
        break;
    }

    /* Simpler: manually add slabs, don't alloc, then shrink */
    slab_test_reset();
    GenSlabCache cache2;
    gen_slab_cache_init(&cache2, 64, 8, 4096,
                        test_page_alloc, test_page_free, (void *)0);

    GenSlab s1, s2;
    void *p1 = test_page_alloc((void *)0, 4096);
    void *p2 = test_page_alloc((void *)0, 4096);
    gen_slab_cache_add_slab(&cache2, &s1, p1);
    gen_slab_cache_add_slab(&cache2, &s2, p2);

    ASSERT(gen_slab_cache_slab_count(&cache2) == 2, "shrink 2 slabs");
    uint32_t freed = gen_slab_cache_shrink(&cache2);
    ASSERT(freed == 2, "shrink freed 2");
    ASSERT(slab_pages_freed == 2, "shrink pages freed");
    ASSERT(gen_slab_cache_slab_count(&cache2) == 0, "shrink 0 slabs");
}

static void test_slab_free_null(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 64, 8, 4096, (void *)0, (void *)0, (void *)0);
    gen_slab_cache_free(&cache, (void *)0); /* Should be no-op */
    ASSERT(gen_slab_cache_free_count(&cache) == 0, "free null no-op");
}

static void test_slab_alignment(void)
{
    GenSlabCache cache;
    gen_slab_cache_init(&cache, 32, 32, 4096, (void *)0, (void *)0, (void *)0);
    ASSERT(cache.obj_size == 32, "slab aligned obj_size");

    uint8_t __attribute__((aligned(4096))) page[4096];
    GenSlab slab;
    gen_slab_cache_add_slab(&cache, &slab, page);

    for (int i = 0; i < 8; i++) {
        void *obj = gen_slab_cache_alloc(&cache);
        ASSERT(obj != (void *)0, "slab align alloc");
        ASSERT(((uintptr_t)obj % 32) == 0, "slab 32-byte aligned");
        gen_slab_cache_free(&cache, obj);
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

    test_slab_cache_init();
    test_slab_cache_init_small_obj();
    test_slab_add_slab();
    test_slab_alloc_single();
    test_slab_alloc_free();
    test_slab_exhaust_single();
    test_slab_multi_slab();
    test_slab_free_state_transitions();
    test_slab_unique_objects();
    test_slab_alloc_grow();
    test_slab_shrink();
    test_slab_free_null();
    test_slab_alignment();

    return failures;
}
