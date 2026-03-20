/*
 * bitops_test.c — Host-compiled tests for gen_bitops C API.
 *
 * Returns 0 on success, non-zero on failure count.
 */

#include "gen_bitops.h"

static int failures = 0;

#define ASSERT(cond, msg)                                        \
    do {                                                         \
        if (!(cond)) {                                           \
            failures++;                                          \
        }                                                        \
    } while (0)

/* ── Single-bit operations ─────────────────────────────────── */

static void test_bit_set_clear_test(void)
{
    uint32_t bm[4] = {0};
    ASSERT(!gen_bit_test(bm, 0), "initial zero");

    gen_bit_set(bm, 0);
    ASSERT(gen_bit_test(bm, 0), "set bit 0");

    gen_bit_clear(bm, 0);
    ASSERT(!gen_bit_test(bm, 0), "clear bit 0");
}

static void test_bit_across_words(void)
{
    uint32_t bm[4] = {0};

    gen_bit_set(bm, 31);
    gen_bit_set(bm, 32);
    gen_bit_set(bm, 95);

    ASSERT(gen_bit_test(bm, 31), "bit 31");
    ASSERT(gen_bit_test(bm, 32), "bit 32");
    ASSERT(gen_bit_test(bm, 95), "bit 95");
    ASSERT(!gen_bit_test(bm, 30), "bit 30 unset");
    ASSERT(!gen_bit_test(bm, 33), "bit 33 unset");
}

static void test_bit_toggle(void)
{
    uint32_t bm[2] = {0};

    gen_bit_toggle(bm, 5);
    ASSERT(gen_bit_test(bm, 5), "toggle on");

    gen_bit_toggle(bm, 5);
    ASSERT(!gen_bit_test(bm, 5), "toggle off");
}

/* ── FFS / FFZ ─────────────────────────────────────────────── */

static void test_ffs_empty(void)
{
    uint32_t bm[4] = {0};
    ASSERT(gen_bitmap_ffs(bm, 128) == UINT32_MAX, "ffs empty");
}

static void test_ffs_first_bit(void)
{
    uint32_t bm[4] = {1, 0, 0, 0};
    ASSERT(gen_bitmap_ffs(bm, 128) == 0, "ffs first");
}

static void test_ffs_middle(void)
{
    uint32_t bm[4] = {0};
    gen_bit_set(bm, 67);
    ASSERT(gen_bitmap_ffs(bm, 128) == 67, "ffs middle");
}

static void test_ffs_respects_nbits(void)
{
    uint32_t bm[4] = {0};
    gen_bit_set(bm, 100);
    ASSERT(gen_bitmap_ffs(bm, 64) == UINT32_MAX, "ffs beyond nbits");
    ASSERT(gen_bitmap_ffs(bm, 128) == 100, "ffs within nbits");
}

static void test_ffz_full(void)
{
    uint32_t bm[4] = {UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX};
    ASSERT(gen_bitmap_ffz(bm, 128) == UINT32_MAX, "ffz full");
}

static void test_ffz_first_zero(void)
{
    uint32_t bm[4] = {0xFFFFFFFE, UINT32_MAX, UINT32_MAX, UINT32_MAX};
    ASSERT(gen_bitmap_ffz(bm, 128) == 0, "ffz first");
}

static void test_ffz_middle(void)
{
    uint32_t bm[4] = {UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX};
    gen_bit_clear(bm, 50);
    ASSERT(gen_bitmap_ffz(bm, 128) == 50, "ffz middle");
}

/* ── Popcount ──────────────────────────────────────────────── */

static void test_popcount_empty(void)
{
    uint32_t bm[4] = {0};
    ASSERT(gen_bitmap_popcount(bm, 128) == 0, "popcount empty");
}

static void test_popcount_full(void)
{
    uint32_t bm[4] = {UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX};
    ASSERT(gen_bitmap_popcount(bm, 128) == 128, "popcount full");
}

static void test_popcount_partial_word(void)
{
    uint32_t bm[2] = {UINT32_MAX, UINT32_MAX};
    ASSERT(gen_bitmap_popcount(bm, 48) == 48, "popcount partial");
}

static void test_popcount_scattered(void)
{
    uint32_t bm[4] = {0};
    gen_bit_set(bm, 0);
    gen_bit_set(bm, 31);
    gen_bit_set(bm, 64);
    ASSERT(gen_bitmap_popcount(bm, 128) == 3, "popcount scattered");
}

/* ── Range operations ──────────────────────────────────────── */

static void test_set_range_within_word(void)
{
    uint32_t bm[2] = {0};
    gen_bitmap_set_range(bm, 4, 8);
    for (uint32_t i = 0; i < 64; i++) {
        bool expected = (i >= 4 && i < 12);
        ASSERT(gen_bit_test(bm, i) == expected, "set_range within word");
    }
}

static void test_set_range_across_words(void)
{
    uint32_t bm[4] = {0};
    gen_bitmap_set_range(bm, 28, 10);
    for (uint32_t i = 0; i < 128; i++) {
        bool expected = (i >= 28 && i < 38);
        ASSERT(gen_bit_test(bm, i) == expected, "set_range across words");
    }
}

static void test_clear_range(void)
{
    uint32_t bm[4] = {UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX};
    gen_bitmap_clear_range(bm, 10, 20);
    for (uint32_t i = 0; i < 128; i++) {
        bool expected = !(i >= 10 && i < 30);
        ASSERT(gen_bit_test(bm, i) == expected, "clear_range");
    }
}

static void test_clear_all(void)
{
    uint32_t bm[4] = {UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX};
    gen_bitmap_clear_all(bm, 128);
    ASSERT(gen_bitmap_popcount(bm, 128) == 0, "clear_all");
}

static void test_set_range_zero_count(void)
{
    uint32_t bm[2] = {0};
    gen_bitmap_set_range(bm, 5, 0);
    ASSERT(gen_bitmap_popcount(bm, 64) == 0, "set_range zero count");
}

/* ── Edge cases ────────────────────────────────────────────── */

static void test_nbits_zero(void)
{
    uint32_t bm[1] = {UINT32_MAX};
    ASSERT(gen_bitmap_ffs(bm, 0) == UINT32_MAX, "ffs nbits=0");
    ASSERT(gen_bitmap_ffz(bm, 0) == UINT32_MAX, "ffz nbits=0");
    ASSERT(gen_bitmap_popcount(bm, 0) == 0, "popcount nbits=0");
}

static void test_nbits_not_word_aligned(void)
{
    uint32_t bm[1] = {0};
    gen_bit_set(bm, 5);
    gen_bit_set(bm, 20);

    ASSERT(gen_bitmap_popcount(bm, 10) == 1, "popcount nbits=10");
    ASSERT(gen_bitmap_popcount(bm, 21) == 2, "popcount nbits=21");
    ASSERT(gen_bitmap_ffs(bm, 10) == 5, "ffs nbits=10");
    ASSERT(gen_bitmap_ffs(bm, 5) == UINT32_MAX, "ffs nbits=5");
}

/* ── Main ──────────────────────────────────────────────────── */

int main(void)
{
    test_bit_set_clear_test();
    test_bit_across_words();
    test_bit_toggle();

    test_ffs_empty();
    test_ffs_first_bit();
    test_ffs_middle();
    test_ffs_respects_nbits();
    test_ffz_full();
    test_ffz_first_zero();
    test_ffz_middle();

    test_popcount_empty();
    test_popcount_full();
    test_popcount_partial_word();
    test_popcount_scattered();

    test_set_range_within_word();
    test_set_range_across_words();
    test_clear_range();
    test_clear_all();
    test_set_range_zero_count();

    test_nbits_zero();
    test_nbits_not_word_aligned();

    return failures;
}
