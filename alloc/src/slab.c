/*
 * slab.c — Slab cache allocator.
 *
 * Classic slab design: each slab is a page of fixed-size objects.
 * A cache groups slabs into partial/full/empty lists for O(1)
 * allocation from the partial list.
 *
 * Freestanding. ACSL-annotated.
 */

#include "gen_alloc.h"

/* ── Internal helpers ────────────────────────────────────── */

static void slab_list_push(GenSlab **head, GenSlab *slab)
{
    slab->next = *head;
    *head = slab;
}

static void slab_list_remove(GenSlab **head, GenSlab *slab)
{
    GenSlab **pp = head;
    while (*pp) {
        if (*pp == slab) {
            *pp = slab->next;
            slab->next = (GenSlab *)0;
            return;
        }
        pp = &(*pp)->next;
    }
}

static GenSlab *slab_find_owner(GenSlab *head, const void *ptr,
                                uint32_t page_size)
{
    const uint8_t *p = (const uint8_t *)ptr;
    GenSlab *s = head;
    while (s) {
        if (p >= s->page && p < s->page + page_size) {
            return s;
        }
        s = s->next;
    }
    return (GenSlab *)0;
}

static void slab_init_freelist(GenSlab *slab, uint32_t obj_size,
                               uint16_t objs_per_slab)
{
    uint8_t *ptr = slab->page;

    for (uint16_t i = 0; i < objs_per_slab - 1; i++) {
        void **slot = (void **)ptr;
        *slot = ptr + obj_size;
        ptr += obj_size;
    }

    /* Last object points to NULL */
    void **last = (void **)ptr;
    *last = (void *)0;

    slab->free_head = slab->page;
    slab->total_objs = objs_per_slab;
    slab->free_objs = objs_per_slab;
    slab->next = (GenSlab *)0;
}

/* ── Public API ──────────────────────────────────────────── */

/*@
  requires \valid(cache);
  requires obj_size > 0;
  requires obj_align > 0;
  requires page_size > 0;
  requires page_size >= obj_size;
  assigns *cache;
  ensures cache->total_slabs == 0;
  ensures cache->total_free == 0;
  ensures cache->total_alloc == 0;
*/
void gen_slab_cache_init(GenSlabCache *cache,
                         uint32_t obj_size, uint32_t obj_align,
                         uint32_t page_size,
                         gen_slab_page_alloc_fn page_alloc,
                         gen_slab_page_free_fn  page_free,
                         void *page_ctx)
{
    /* Ensure obj_size >= sizeof(void*) for embedded free list */
    if (obj_size < sizeof(void *)) {
        obj_size = (uint32_t)sizeof(void *);
    }
    if (obj_align < sizeof(void *)) {
        obj_align = (uint32_t)sizeof(void *);
    }

    /* Round obj_size up to obj_align */
    obj_size = (obj_size + obj_align - 1) & ~(obj_align - 1);

    cache->partial    = (GenSlab *)0;
    cache->full       = (GenSlab *)0;
    cache->empty      = (GenSlab *)0;

    cache->obj_size     = obj_size;
    cache->obj_align    = obj_align;
    cache->page_size    = page_size;
    cache->objs_per_slab = (uint16_t)(page_size / obj_size);

    cache->total_slabs  = 0;
    cache->total_free   = 0;
    cache->total_alloc  = 0;

    cache->page_alloc = page_alloc;
    cache->page_free  = page_free;
    cache->page_ctx   = page_ctx;
}

/*@
  requires \valid(cache);
  requires \valid(slab);
  requires \valid((uint8_t *)page + (0 .. cache->page_size - 1));
  assigns *slab, cache->empty, cache->total_slabs, cache->total_free;
  ensures \result == 0 || \result < 0;
*/
GenResult gen_slab_cache_add_slab(GenSlabCache *cache,
                                  GenSlab *slab, void *page)
{
    if (!cache || !slab || !page || cache->objs_per_slab == 0) {
        return -1; /* GEN_ERR_INVALID_ARG */
    }

    slab->page = (uint8_t *)page;
    slab_init_freelist(slab, cache->obj_size, cache->objs_per_slab);

    slab_list_push(&cache->empty, slab);
    cache->total_slabs++;
    cache->total_free += cache->objs_per_slab;

    return 0; /* GEN_OK */
}

/*@
  requires \valid(cache);
  assigns cache->partial, cache->full, cache->empty,
          cache->total_free, cache->total_alloc;
  ensures \result == \null ||
          cache->total_free == \old(cache->total_free) - 1;
*/
void *gen_slab_cache_alloc(GenSlabCache *cache)
{
    GenSlab *slab;

    if (cache->partial) {
        slab = cache->partial;
    } else if (cache->empty) {
        /* Promote an empty slab to partial */
        slab = cache->empty;
        slab_list_remove(&cache->empty, slab);
        slab_list_push(&cache->partial, slab);
    } else {
        return (void *)0;
    }

    /* Pop from free list */
    void *obj = slab->free_head;
    slab->free_head = *(void **)obj;
    slab->free_objs--;
    cache->total_free--;
    cache->total_alloc++;

    /* If slab is now full, move to full list */
    if (slab->free_objs == 0) {
        slab_list_remove(&cache->partial, slab);
        slab_list_push(&cache->full, slab);
    }

    return obj;
}

void *gen_slab_cache_alloc_grow(GenSlabCache *cache, GenSlab *slab_buf)
{
    void *obj = gen_slab_cache_alloc(cache);
    if (obj) {
        return obj;
    }

    /* Try to grow if page_alloc is available and slab_buf provided */
    if (!cache->page_alloc || !slab_buf) {
        return (void *)0;
    }

    void *page = cache->page_alloc(cache->page_ctx, cache->page_size);
    if (!page) {
        return (void *)0;
    }

    if (gen_slab_cache_add_slab(cache, slab_buf, page) != 0) {
        cache->page_free(cache->page_ctx, page, cache->page_size);
        return (void *)0;
    }

    return gen_slab_cache_alloc(cache);
}

/*@
  requires \valid(cache);
  requires ptr != \null ==> \valid((uint8_t *)ptr);
  assigns cache->partial, cache->full, cache->empty,
          cache->total_free, cache->total_alloc;
*/
void gen_slab_cache_free(GenSlabCache *cache, void *ptr)
{
    if (!ptr) {
        return;
    }

    /* Find the owning slab across all three lists */
    GenSlab *slab = slab_find_owner(cache->partial, ptr, cache->page_size);
    GenSlab **src_list = &cache->partial;

    if (!slab) {
        slab = slab_find_owner(cache->full, ptr, cache->page_size);
        src_list = &cache->full;
    }
    if (!slab) {
        slab = slab_find_owner(cache->empty, ptr, cache->page_size);
        src_list = &cache->empty;
    }
    if (!slab) {
        return; /* ptr not owned by this cache */
    }

    int was_full = (slab->free_objs == 0);

    /* Push onto slab's free list */
    *(void **)ptr = slab->free_head;
    slab->free_head = ptr;
    slab->free_objs++;
    cache->total_free++;
    cache->total_alloc--;

    if (was_full) {
        /* full → partial */
        slab_list_remove(src_list, slab);
        slab_list_push(&cache->partial, slab);
    } else if (slab->free_objs == slab->total_objs) {
        /* partial → empty */
        slab_list_remove(src_list, slab);
        slab_list_push(&cache->empty, slab);
    }
}

uint32_t gen_slab_cache_shrink(GenSlabCache *cache)
{
    if (!cache->page_free) {
        return 0;
    }

    uint32_t count = 0;
    GenSlab *slab = cache->empty;

    while (slab) {
        GenSlab *next = slab->next;
        cache->total_free -= slab->total_objs;
        cache->total_slabs--;
        cache->page_free(cache->page_ctx, slab->page, cache->page_size);
        count++;
        slab = next;
    }

    cache->empty = (GenSlab *)0;
    return count;
}

void gen_slab_cache_destroy(GenSlabCache *cache)
{
    gen_slab_cache_shrink(cache);
}

uint32_t gen_slab_cache_free_count(const GenSlabCache *cache)
{
    return cache->total_free;
}

uint32_t gen_slab_cache_allocated_count(const GenSlabCache *cache)
{
    return cache->total_alloc;
}

uint32_t gen_slab_cache_slab_count(const GenSlabCache *cache)
{
    return cache->total_slabs;
}
