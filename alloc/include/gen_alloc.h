/*
 * gen_alloc.h — Freestanding kernel memory allocators.
 *
 * - GenPmm: Bitmap-based physical memory manager
 * - GenPool: Fixed-block pool allocator (O(1), RTOS-suitable)
 * - GenSlabCache: Slab cache allocator for fixed-size objects
 *
 * Caller provides all backing storage. No global state, no heap.
 *
 * Freestanding: requires <stdint.h>, <stddef.h>, <stdbool.h>,
 * genesis_result.h.
 */

#ifndef GEN_ALLOC_H
#define GEN_ALLOC_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "genesis_result.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ── Bitmap Physical Memory Manager ──────────────────────── */

/*
 * GenPmm tracks physical page frames using a bitmap from the
 * bitops C API (gen_bit_set/clear/test, gen_bitmap_ffs/ffz).
 *
 * The caller provides the bitmap storage and the base address.
 * Pages are fixed-size (typically 4096 bytes).
 */
typedef struct GenPmm {
    uint32_t *bitmap;       /* Caller-provided bitmap array */
    uint32_t  total_pages;  /* Total number of tracked pages */
    uint32_t  free_pages;   /* Number of currently free pages */
    uint64_t  base_addr;    /* Physical address of page 0 */
    uint32_t  page_size;    /* Bytes per page (e.g., 4096) */
} GenPmm;

/*
 * Initialize a PMM instance.
 * bitmap must have at least (total_pages + 31) / 32 entries.
 * All pages start as free.
 */
void gen_pmm_init(GenPmm *pmm, uint32_t *bitmap,
                  uint32_t total_pages, uint64_t base_addr,
                  uint32_t page_size);

/*
 * Allocate a single page. Returns the physical address,
 * or UINT64_MAX if no pages are available.
 */
uint64_t gen_pmm_alloc_page(GenPmm *pmm);

/*
 * Free a single page by physical address.
 * The address must have been previously allocated.
 */
void gen_pmm_free_page(GenPmm *pmm, uint64_t addr);

/*
 * Allocate a contiguous range of count pages.
 * Returns the physical address of the first page,
 * or UINT64_MAX if not enough contiguous pages.
 */
uint64_t gen_pmm_alloc_contiguous(GenPmm *pmm, uint32_t count);

/*
 * Mark a range of pages as reserved (not available for allocation).
 * Used during early boot to exclude kernel/firmware regions.
 */
void gen_pmm_mark_reserved(GenPmm *pmm, uint64_t addr, uint32_t count);

/* Query functions */
uint32_t gen_pmm_free_count(const GenPmm *pmm);
uint32_t gen_pmm_total_count(const GenPmm *pmm);
bool     gen_pmm_is_allocated(const GenPmm *pmm, uint64_t addr);

/* ── Fixed-Block Pool Allocator ──────────────────────────── */

/*
 * GenPool: O(1) alloc/free pool of fixed-size blocks.
 * Uses a free-list threaded through the blocks themselves.
 * Suitable for RTOS / interrupt-safe contexts.
 *
 * The caller provides a contiguous memory region; GenPool
 * partitions it into blocks of block_size bytes.
 */
typedef struct GenPool {
    void    *free_head;     /* Head of embedded free list */
    uint8_t *region;        /* Start of the memory region */
    uint32_t block_size;    /* Size of each block (>= sizeof(void*)) */
    uint32_t total_blocks;  /* Total blocks in the pool */
    uint32_t free_blocks;   /* Currently available blocks */
} GenPool;

/*
 * Initialize a pool allocator.
 * region must be at least block_size * block_count bytes.
 * block_size must be >= sizeof(void*) and properly aligned.
 */
void gen_pool_init(GenPool *pool, void *region,
                   uint32_t block_size, uint32_t block_count);

/* Allocate a single block. Returns NULL if exhausted. */
void *gen_pool_alloc(GenPool *pool);

/* Free a block back to the pool. */
void gen_pool_free(GenPool *pool, void *ptr);

/* Query functions */
uint32_t gen_pool_free_count(const GenPool *pool);
uint32_t gen_pool_total_count(const GenPool *pool);

/* ── Slab Cache Allocator ────────────────────────────────── */

/*
 * Page allocator callbacks. The slab cache uses these to
 * request/return backing pages, decoupling it from GenPmm.
 */
typedef void *(*gen_slab_page_alloc_fn)(void *ctx, uint32_t page_size);
typedef void  (*gen_slab_page_free_fn)(void *ctx, void *page, uint32_t page_size);

/*
 * GenSlab: A single slab — one page of fixed-size objects.
 *
 * Metadata is stored out-of-band (not in the page itself).
 * Free objects use an embedded free list (same as GenPool).
 */
typedef struct GenSlab {
    struct GenSlab *next;        /* Linked list pointer for cache lists */
    void           *free_head;   /* Head of embedded free list in page */
    uint8_t        *page;        /* Start of the backing page */
    uint16_t        total_objs;  /* Total objects in this slab */
    uint16_t        free_objs;   /* Currently free objects */
} GenSlab;

/*
 * GenSlabCache: Groups all slabs for one object size.
 *
 * Three lists partition slabs by state:
 *   partial — has some free objects (preferred for allocation)
 *   full    — no free objects
 *   empty   — all objects free (can be returned to page allocator)
 *
 * Not thread-safe. Caller must serialize access.
 */
typedef struct GenSlabCache {
    GenSlab     *partial;
    GenSlab     *full;
    GenSlab     *empty;

    uint32_t     obj_size;       /* Effective object size (aligned) */
    uint32_t     obj_align;      /* Object alignment */
    uint32_t     page_size;      /* Backing page size */
    uint16_t     objs_per_slab;  /* page_size / obj_size */

    uint32_t     total_slabs;
    uint32_t     total_free;     /* Free objects across all slabs */
    uint32_t     total_alloc;    /* Allocated objects */

    gen_slab_page_alloc_fn  page_alloc;
    gen_slab_page_free_fn   page_free;
    void                   *page_ctx;
} GenSlabCache;

/*
 * Initialize a slab cache for objects of a given size.
 *
 * obj_size is rounded up to max(obj_size, sizeof(void*)) and
 * aligned to obj_align. page_alloc/page_free may be NULL if
 * the caller will only use gen_slab_cache_add_slab().
 */
void gen_slab_cache_init(GenSlabCache *cache,
                         uint32_t obj_size, uint32_t obj_align,
                         uint32_t page_size,
                         gen_slab_page_alloc_fn page_alloc,
                         gen_slab_page_free_fn  page_free,
                         void *page_ctx);

/*
 * Add a slab to the cache manually. The caller provides
 * both the GenSlab metadata struct and the backing page.
 * page must be at least cache->page_size bytes.
 */
GenResult gen_slab_cache_add_slab(GenSlabCache *cache,
                                  GenSlab *slab, void *page);

/* Allocate a single object. Returns NULL if exhausted. */
void *gen_slab_cache_alloc(GenSlabCache *cache);

/*
 * Allocate with automatic growth. If no slabs are available
 * and page_alloc is set, uses slab_buf + page_alloc to grow.
 * slab_buf may be NULL (behaves like gen_slab_cache_alloc).
 */
void *gen_slab_cache_alloc_grow(GenSlabCache *cache, GenSlab *slab_buf);

/* Free an object back to its owning slab. */
void gen_slab_cache_free(GenSlabCache *cache, void *ptr);

/* Return all empty slabs to the page allocator. Returns count freed. */
uint32_t gen_slab_cache_shrink(GenSlabCache *cache);

/* Destroy: free all empty slabs. Partial/full slabs are NOT freed. */
void gen_slab_cache_destroy(GenSlabCache *cache);

/* Query functions */
uint32_t gen_slab_cache_free_count(const GenSlabCache *cache);
uint32_t gen_slab_cache_allocated_count(const GenSlabCache *cache);
uint32_t gen_slab_cache_slab_count(const GenSlabCache *cache);

#ifdef __cplusplus
}
#endif

#endif /* GEN_ALLOC_H */
