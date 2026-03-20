/*
 * bitmap_pmm.c — Bitmap-based physical memory manager.
 *
 * Uses the bitops C API (gen_bit_set/clear/test, gen_bitmap_ffz)
 * for bitmap manipulation. This is a pure C→C dependency.
 *
 * Freestanding. ACSL-annotated.
 */

#include "gen_alloc.h"
#include "gen_bitops.h"

/*@
  requires \valid(pmm);
  requires total_pages == 0 ||
           \valid(bitmap + (0 .. (total_pages - 1) / 32));
  requires page_size > 0;
  assigns *pmm;
  ensures pmm->free_pages == total_pages;
*/
void gen_pmm_init(GenPmm *pmm, uint32_t *bitmap,
                  uint32_t total_pages, uint64_t base_addr,
                  uint32_t page_size)
{
    pmm->bitmap      = bitmap;
    pmm->total_pages = total_pages;
    pmm->free_pages  = total_pages;
    pmm->base_addr   = base_addr;
    pmm->page_size   = page_size;

    /* Clear all bits: 0 = free, 1 = allocated/reserved */
    gen_bitmap_clear_all(bitmap, total_pages);
}

/*@
  requires \valid(pmm);
  assigns pmm->free_pages, pmm->bitmap[0 .. (pmm->total_pages - 1) / 32];
*/
uint64_t gen_pmm_alloc_page(GenPmm *pmm)
{
    if (pmm->free_pages == 0) {
        return UINT64_MAX;
    }

    uint32_t idx = gen_bitmap_ffz(pmm->bitmap, pmm->total_pages);
    if (idx == UINT32_MAX) {
        return UINT64_MAX;
    }

    gen_bit_set(pmm->bitmap, idx);
    pmm->free_pages--;

    return pmm->base_addr + (uint64_t)idx * pmm->page_size;
}

/*@
  requires \valid(pmm);
  assigns pmm->free_pages, pmm->bitmap[0 .. (pmm->total_pages - 1) / 32];
*/
void gen_pmm_free_page(GenPmm *pmm, uint64_t addr)
{
    if (addr < pmm->base_addr) {
        return;
    }

    uint64_t offset = addr - pmm->base_addr;
    uint32_t idx = (uint32_t)(offset / pmm->page_size);

    if (idx >= pmm->total_pages) {
        return;
    }

    if (gen_bit_test(pmm->bitmap, idx)) {
        gen_bit_clear(pmm->bitmap, idx);
        pmm->free_pages++;
    }
}

/*@
  requires \valid(pmm);
  requires count > 0;
  assigns pmm->free_pages, pmm->bitmap[0 .. (pmm->total_pages - 1) / 32];
*/
uint64_t gen_pmm_alloc_contiguous(GenPmm *pmm, uint32_t count)
{
    if (count == 0 || pmm->free_pages < count) {
        return UINT64_MAX;
    }

    /* Linear scan for a contiguous run of free pages */
    uint32_t run_start = 0;
    uint32_t run_len = 0;

    /*@
      loop invariant 0 <= i <= pmm->total_pages;
      loop assigns i, run_start, run_len;
      loop variant pmm->total_pages - i;
    */
    for (uint32_t i = 0; i < pmm->total_pages; i++) {
        if (!gen_bit_test(pmm->bitmap, i)) {
            if (run_len == 0) {
                run_start = i;
            }
            run_len++;
            if (run_len >= count) {
                /* Found a run — mark all as allocated */
                gen_bitmap_set_range(pmm->bitmap, run_start, count);
                pmm->free_pages -= count;
                return pmm->base_addr + (uint64_t)run_start * pmm->page_size;
            }
        } else {
            run_len = 0;
        }
    }

    return UINT64_MAX;
}

/*@
  requires \valid(pmm);
  assigns pmm->free_pages, pmm->bitmap[0 .. (pmm->total_pages - 1) / 32];
*/
void gen_pmm_mark_reserved(GenPmm *pmm, uint64_t addr, uint32_t count)
{
    if (addr < pmm->base_addr) {
        return;
    }

    uint32_t start = (uint32_t)((addr - pmm->base_addr) / pmm->page_size);

    for (uint32_t i = 0; i < count && (start + i) < pmm->total_pages; i++) {
        if (!gen_bit_test(pmm->bitmap, start + i)) {
            gen_bit_set(pmm->bitmap, start + i);
            pmm->free_pages--;
        }
    }
}

uint32_t gen_pmm_free_count(const GenPmm *pmm)
{
    return pmm->free_pages;
}

uint32_t gen_pmm_total_count(const GenPmm *pmm)
{
    return pmm->total_pages;
}

bool gen_pmm_is_allocated(const GenPmm *pmm, uint64_t addr)
{
    if (addr < pmm->base_addr) {
        return false;
    }
    uint32_t idx = (uint32_t)((addr - pmm->base_addr) / pmm->page_size);
    if (idx >= pmm->total_pages) {
        return false;
    }
    return gen_bit_test(pmm->bitmap, idx);
}
