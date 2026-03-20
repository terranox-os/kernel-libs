/*
 * bitmap.c — Bitmap operations on uint32_t arrays.
 *
 * Freestanding. Uses __builtin_ctz() for ffs (BSF on x86, CTZ on ARM)
 * and __builtin_popcount() for popcount (POPCNT on x86).
 *
 * ACSL-annotated for Frama-C WP verification.
 */

#include "gen_bitops.h"

/* ── Single-bit operations ───────────────────────────────── */

/*@
  requires \valid(bitmap + (bit / 32));
  assigns bitmap[bit / 32];
  ensures (bitmap[bit / 32] & ((uint32_t)1 << (bit % 32))) != 0;
*/
void gen_bit_set(uint32_t *bitmap, uint32_t bit)
{
    bitmap[bit / 32] |= (uint32_t)1 << (bit % 32);
}

/*@
  requires \valid(bitmap + (bit / 32));
  assigns bitmap[bit / 32];
  ensures (bitmap[bit / 32] & ((uint32_t)1 << (bit % 32))) == 0;
*/
void gen_bit_clear(uint32_t *bitmap, uint32_t bit)
{
    bitmap[bit / 32] &= ~((uint32_t)1 << (bit % 32));
}

/*@
  requires \valid_read(bitmap + (bit / 32));
  assigns \nothing;
*/
bool gen_bit_test(const uint32_t *bitmap, uint32_t bit)
{
    return (bitmap[bit / 32] & ((uint32_t)1 << (bit % 32))) != 0;
}

/*@
  requires \valid(bitmap + (bit / 32));
  assigns bitmap[bit / 32];
*/
void gen_bit_toggle(uint32_t *bitmap, uint32_t bit)
{
    bitmap[bit / 32] ^= (uint32_t)1 << (bit % 32);
}

/* ── Bitmap scanning ─────────────────────────────────────── */

/*@
  requires nbits == 0 ||
           \valid_read(bitmap + (0 .. (nbits - 1) / 32));
  assigns \nothing;
  ensures \result == UINT32_MAX ||
          (\result < nbits &&
           (bitmap[\result / 32] & ((uint32_t)1 << (\result % 32))) != 0);
*/
uint32_t gen_bitmap_ffs(const uint32_t *bitmap, uint32_t nbits)
{
    uint32_t nwords = (nbits + 31) / 32;

    /*@
      loop invariant 0 <= i <= nwords;
      loop assigns i;
      loop variant nwords - i;
    */
    for (uint32_t i = 0; i < nwords; i++) {
        uint32_t word = bitmap[i];
        if (word != 0) {
            uint32_t bit = i * 32 + (uint32_t)__builtin_ctz(word);
            if (bit < nbits) {
                return bit;
            }
            return UINT32_MAX;
        }
    }

    return UINT32_MAX;
}

/*@
  requires nbits == 0 ||
           \valid_read(bitmap + (0 .. (nbits - 1) / 32));
  assigns \nothing;
  ensures \result == UINT32_MAX ||
          (\result < nbits &&
           (bitmap[\result / 32] & ((uint32_t)1 << (\result % 32))) == 0);
*/
uint32_t gen_bitmap_ffz(const uint32_t *bitmap, uint32_t nbits)
{
    uint32_t nwords = (nbits + 31) / 32;

    /*@
      loop invariant 0 <= i <= nwords;
      loop assigns i;
      loop variant nwords - i;
    */
    for (uint32_t i = 0; i < nwords; i++) {
        uint32_t word = ~bitmap[i];
        if (word != 0) {
            uint32_t bit = i * 32 + (uint32_t)__builtin_ctz(word);
            if (bit < nbits) {
                return bit;
            }
            return UINT32_MAX;
        }
    }

    return UINT32_MAX;
}

/* ── Bitmap statistics ───────────────────────────────────── */

/*@
  requires nbits == 0 ||
           \valid_read(bitmap + (0 .. (nbits - 1) / 32));
  assigns \nothing;
  ensures \result <= nbits;
*/
uint32_t gen_bitmap_popcount(const uint32_t *bitmap, uint32_t nbits)
{
    uint32_t count = 0;
    uint32_t full_words = nbits / 32;
    uint32_t remaining = nbits % 32;

    /*@
      loop invariant 0 <= i <= full_words;
      loop assigns i, count;
      loop variant full_words - i;
    */
    for (uint32_t i = 0; i < full_words; i++) {
        count += (uint32_t)__builtin_popcount(bitmap[i]);
    }

    if (remaining > 0) {
        uint32_t mask = ((uint32_t)1 << remaining) - 1;
        count += (uint32_t)__builtin_popcount(bitmap[full_words] & mask);
    }

    return count;
}

/* ── Range operations ────────────────────────────────────── */

/*@
  requires count == 0 ||
           \valid(bitmap + (start / 32 .. (start + count - 1) / 32));
  assigns bitmap[start / 32 .. (start + count - 1) / 32];
*/
void gen_bitmap_set_range(uint32_t *bitmap, uint32_t start, uint32_t count)
{
    /*@
      loop invariant 0 <= count;
      loop assigns count, start, bitmap[start / 32 .. (start + count - 1) / 32];
      loop variant count;
    */
    while (count > 0) {
        uint32_t word_idx = start / 32;
        uint32_t bit_off = start % 32;
        uint32_t bits_in_word = 32 - bit_off;

        if (bits_in_word > count) {
            bits_in_word = count;
        }

        uint32_t mask;
        if (bits_in_word == 32) {
            mask = UINT32_MAX;
        } else {
            mask = ((uint32_t)1 << bits_in_word) - 1;
        }
        bitmap[word_idx] |= mask << bit_off;

        start += bits_in_word;
        count -= bits_in_word;
    }
}

/*@
  requires count == 0 ||
           \valid(bitmap + (start / 32 .. (start + count - 1) / 32));
  assigns bitmap[start / 32 .. (start + count - 1) / 32];
*/
void gen_bitmap_clear_range(uint32_t *bitmap, uint32_t start, uint32_t count)
{
    /*@
      loop invariant 0 <= count;
      loop assigns count, start, bitmap[start / 32 .. (start + count - 1) / 32];
      loop variant count;
    */
    while (count > 0) {
        uint32_t word_idx = start / 32;
        uint32_t bit_off = start % 32;
        uint32_t bits_in_word = 32 - bit_off;

        if (bits_in_word > count) {
            bits_in_word = count;
        }

        uint32_t mask;
        if (bits_in_word == 32) {
            mask = UINT32_MAX;
        } else {
            mask = ((uint32_t)1 << bits_in_word) - 1;
        }
        bitmap[word_idx] &= ~(mask << bit_off);

        start += bits_in_word;
        count -= bits_in_word;
    }
}

/*@
  requires nbits == 0 ||
           \valid(bitmap + (0 .. (nbits - 1) / 32));
  assigns bitmap[0 .. (nbits - 1) / 32];
  ensures \forall integer k; 0 <= k < (nbits + 31) / 32 ==> bitmap[k] == 0;
*/
void gen_bitmap_clear_all(uint32_t *bitmap, uint32_t nbits)
{
    uint32_t nwords = (nbits + 31) / 32;

    /*@
      loop invariant 0 <= i <= nwords;
      loop assigns i, bitmap[0 .. nwords - 1];
      loop variant nwords - i;
    */
    for (uint32_t i = 0; i < nwords; i++) {
        bitmap[i] = 0;
    }
}
