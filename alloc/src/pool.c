/*
 * pool.c — Fixed-block pool allocator.
 *
 * O(1) alloc/free using an embedded free list threaded through
 * the free blocks themselves. Each free block stores a pointer
 * to the next free block at its start.
 *
 * Freestanding. ACSL-annotated.
 */

#include "gen_alloc.h"

/*@
  requires \valid(pool);
  requires \valid((uint8_t *)region + (0 .. block_size * block_count - 1));
  requires block_size >= sizeof(void *);
  requires block_count > 0;
  assigns *pool;
*/
void gen_pool_init(GenPool *pool, void *region,
                   uint32_t block_size, uint32_t block_count)
{
    pool->region       = (uint8_t *)region;
    pool->block_size   = block_size;
    pool->total_blocks = block_count;
    pool->free_blocks  = block_count;

    /* Thread the free list through the blocks */
    uint8_t *ptr = pool->region;

    /*@
      loop invariant 0 <= i <= block_count;
      loop assigns i, ptr, *((void **)ptr);
      loop variant block_count - i;
    */
    for (uint32_t i = 0; i < block_count - 1; i++) {
        void **slot = (void **)ptr;
        *slot = ptr + block_size;
        ptr += block_size;
    }

    /* Last block points to NULL */
    void **last = (void **)ptr;
    *last = (void *)0;

    pool->free_head = pool->region;
}

/*@
  requires \valid(pool);
  assigns pool->free_head, pool->free_blocks;
  ensures \result == \null || pool->free_blocks == \old(pool->free_blocks) - 1;
*/
void *gen_pool_alloc(GenPool *pool)
{
    if (pool->free_head == (void *)0) {
        return (void *)0;
    }

    void *block = pool->free_head;
    pool->free_head = *(void **)block;
    pool->free_blocks--;

    return block;
}

/*@
  requires \valid(pool);
  requires \valid((uint8_t *)ptr);
  assigns pool->free_head, pool->free_blocks;
  ensures pool->free_blocks == \old(pool->free_blocks) + 1;
*/
void gen_pool_free(GenPool *pool, void *ptr)
{
    if (ptr == (void *)0) {
        return;
    }

    *(void **)ptr = pool->free_head;
    pool->free_head = ptr;
    pool->free_blocks++;
}

uint32_t gen_pool_free_count(const GenPool *pool)
{
    return pool->free_blocks;
}

uint32_t gen_pool_total_count(const GenPool *pool)
{
    return pool->total_blocks;
}
