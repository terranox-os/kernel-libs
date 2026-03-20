/*
 * gen_bitops.h — Bitmap operations on uint32_t arrays.
 *
 * C API for bit manipulation on caller-provided bitmaps.
 * The Rust crate provides equivalent operations on &[u32] slices
 * with bounds checking.
 *
 * Freestanding: requires only <stdint.h> and <stdbool.h>.
 */

#ifndef GEN_BITOPS_H
#define GEN_BITOPS_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Single-bit operations ───────────────────────────────── */

void gen_bit_set(uint32_t *bitmap, uint32_t bit);
void gen_bit_clear(uint32_t *bitmap, uint32_t bit);
bool gen_bit_test(const uint32_t *bitmap, uint32_t bit);
void gen_bit_toggle(uint32_t *bitmap, uint32_t bit);

/* ── Bitmap scanning ─────────────────────────────────────── */

/*
 * Find first set / first zero bit in a bitmap of nbits bits.
 * Returns the bit index, or UINT32_MAX if not found.
 */
uint32_t gen_bitmap_ffs(const uint32_t *bitmap, uint32_t nbits);
uint32_t gen_bitmap_ffz(const uint32_t *bitmap, uint32_t nbits);

/* ── Bitmap statistics ───────────────────────────────────── */

uint32_t gen_bitmap_popcount(const uint32_t *bitmap, uint32_t nbits);

/* ── Range operations ────────────────────────────────────── */

void gen_bitmap_set_range(uint32_t *bitmap, uint32_t start, uint32_t count);
void gen_bitmap_clear_range(uint32_t *bitmap, uint32_t start, uint32_t count);
void gen_bitmap_clear_all(uint32_t *bitmap, uint32_t nbits);

#ifdef __cplusplus
}
#endif

#endif /* GEN_BITOPS_H */
