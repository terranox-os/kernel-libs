/*
 * gen_alloc.h — Freestanding kernel memory allocators.
 *
 * - GenPmm: Bitmap-based physical memory manager
 * - GenPool: Fixed-block pool allocator (O(1), RTOS-suitable)
 *
 * Caller provides all backing storage. No global state, no heap.
 *
 * Freestanding: requires <stdint.h>, <stddef.h>, <stdbool.h>.
 */

#ifndef GEN_ALLOC_H
#define GEN_ALLOC_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

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

#ifdef __cplusplus
}
#endif

#endif /* GEN_ALLOC_H */
