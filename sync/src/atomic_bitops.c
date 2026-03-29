/*
 * atomic_bitops.c -- Atomic bitmap operations using C11 atomics.
 *
 * Operates on arrays of _Atomic(uint32_t). Each function computes
 * the word index (bit / 32) and bit mask (1u << (bit % 32)), then
 * uses the appropriate atomic RMW with AcqRel ordering.
 */

#include "gen_sync.h"

void gen_atomic_bit_set(_Atomic(uint32_t) *bitmap, uint32_t bit)
{
    uint32_t word = bit / 32;
    uint32_t mask = 1u << (bit % 32);
    atomic_fetch_or_explicit(&bitmap[word], mask, memory_order_acq_rel);
}

void gen_atomic_bit_clear(_Atomic(uint32_t) *bitmap, uint32_t bit)
{
    uint32_t word = bit / 32;
    uint32_t mask = 1u << (bit % 32);
    atomic_fetch_and_explicit(&bitmap[word], ~mask, memory_order_acq_rel);
}

bool gen_atomic_bit_test(const _Atomic(uint32_t) *bitmap, uint32_t bit)
{
    uint32_t word = bit / 32;
    uint32_t mask = 1u << (bit % 32);
    return (atomic_load_explicit(&bitmap[word], memory_order_acquire) & mask)
           != 0;
}

bool gen_atomic_bit_toggle(_Atomic(uint32_t) *bitmap, uint32_t bit)
{
    uint32_t word = bit / 32;
    uint32_t mask = 1u << (bit % 32);
    uint32_t old = atomic_fetch_xor_explicit(&bitmap[word], mask,
                                             memory_order_acq_rel);
    return (old & mask) != 0;
}

bool gen_atomic_bit_test_and_set(_Atomic(uint32_t) *bitmap, uint32_t bit)
{
    uint32_t word = bit / 32;
    uint32_t mask = 1u << (bit % 32);
    uint32_t old = atomic_fetch_or_explicit(&bitmap[word], mask,
                                            memory_order_acq_rel);
    return (old & mask) != 0;
}

bool gen_atomic_bit_test_and_clear(_Atomic(uint32_t) *bitmap, uint32_t bit)
{
    uint32_t word = bit / 32;
    uint32_t mask = 1u << (bit % 32);
    uint32_t old = atomic_fetch_and_explicit(&bitmap[word], ~mask,
                                             memory_order_acq_rel);
    return (old & mask) != 0;
}
