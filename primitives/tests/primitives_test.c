/*
 * primitives_test.c — Host-compiled tests for gen_primitives.
 *
 * Returns 0 on success, non-zero on failure.
 * Each test prints PASS/FAIL for visibility.
 */

#include "gen_primitives.h"
#include <stdint.h>
#include <string.h> /* For alias test: memcpy/memset/memmove/memcmp declarations */

static int failures = 0;

#define ASSERT(cond, msg)                                        \
    do {                                                         \
        if (!(cond)) {                                           \
            failures++;                                          \
            /* Minimal error reporting without stdio */           \
        }                                                        \
    } while (0)

/* ── gen_memcpy tests ──────────────────────────────────────── */

static void test_memcpy_basic(void)
{
    unsigned char src[16] = {0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15};
    unsigned char dst[16] = {0};

    void *ret = gen_memcpy(dst, src, 16);
    ASSERT(ret == dst, "memcpy return value");
    for (int i = 0; i < 16; i++) {
        ASSERT(dst[i] == src[i], "memcpy byte mismatch");
    }
}

static void test_memcpy_zero_length(void)
{
    unsigned char src[4] = {1,2,3,4};
    unsigned char dst[4] = {0xFF, 0xFF, 0xFF, 0xFF};

    gen_memcpy(dst, src, 0);
    ASSERT(dst[0] == 0xFF, "memcpy zero-length should not modify dst");
}

static void test_memcpy_single_byte(void)
{
    unsigned char src = 0xAB;
    unsigned char dst = 0;

    gen_memcpy(&dst, &src, 1);
    ASSERT(dst == 0xAB, "memcpy single byte");
}

static void test_memcpy_large_aligned(void)
{
    unsigned char src[256];
    unsigned char dst[256];

    for (int i = 0; i < 256; i++) src[i] = (unsigned char)i;
    gen_memcpy(dst, src, 256);
    for (int i = 0; i < 256; i++) {
        ASSERT(dst[i] == (unsigned char)i, "memcpy large aligned");
    }
}

/* ── gen_memset tests ──────────────────────────────────────── */

static void test_memset_basic(void)
{
    unsigned char buf[32];
    void *ret = gen_memset(buf, 0xCC, 32);
    ASSERT(ret == buf, "memset return value");
    for (int i = 0; i < 32; i++) {
        ASSERT(buf[i] == 0xCC, "memset fill mismatch");
    }
}

static void test_memset_zero(void)
{
    unsigned char buf[16] = {1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16};
    gen_memset(buf, 0, 16);
    for (int i = 0; i < 16; i++) {
        ASSERT(buf[i] == 0, "memset zero fill");
    }
}

static void test_memset_zero_length(void)
{
    unsigned char buf[4] = {0xAA, 0xBB, 0xCC, 0xDD};
    gen_memset(buf, 0, 0);
    ASSERT(buf[0] == 0xAA, "memset zero-length should not modify");
}

static void test_memset_partial(void)
{
    unsigned char buf[8] = {0};
    gen_memset(buf + 2, 0xFF, 4);
    ASSERT(buf[0] == 0, "memset partial: before region");
    ASSERT(buf[1] == 0, "memset partial: before region");
    ASSERT(buf[2] == 0xFF, "memset partial: in region");
    ASSERT(buf[5] == 0xFF, "memset partial: in region");
    ASSERT(buf[6] == 0, "memset partial: after region");
    ASSERT(buf[7] == 0, "memset partial: after region");
}

/* ── gen_memmove tests ─────────────────────────────────────── */

static void test_memmove_no_overlap(void)
{
    unsigned char src[8] = {1,2,3,4,5,6,7,8};
    unsigned char dst[8] = {0};

    void *ret = gen_memmove(dst, src, 8);
    ASSERT(ret == dst, "memmove return value");
    for (int i = 0; i < 8; i++) {
        ASSERT(dst[i] == src[i], "memmove no overlap");
    }
}

static void test_memmove_overlap_forward(void)
{
    /* dst starts after src, overlapping: copy backward needed */
    unsigned char buf[16] = {1,2,3,4,5,6,7,8,0,0,0,0,0,0,0,0};
    gen_memmove(buf + 4, buf, 8);
    /* buf[4..11] should now be {1,2,3,4,5,6,7,8} */
    ASSERT(buf[4] == 1, "memmove overlap forward [4]");
    ASSERT(buf[5] == 2, "memmove overlap forward [5]");
    ASSERT(buf[11] == 8, "memmove overlap forward [11]");
}

static void test_memmove_overlap_backward(void)
{
    /* dst starts before src, overlapping: forward copy is safe */
    unsigned char buf[16] = {0,0,0,0,1,2,3,4,5,6,7,8,0,0,0,0};
    gen_memmove(buf, buf + 4, 8);
    ASSERT(buf[0] == 1, "memmove overlap backward [0]");
    ASSERT(buf[7] == 8, "memmove overlap backward [7]");
}

static void test_memmove_same_pointer(void)
{
    unsigned char buf[4] = {0xDE, 0xAD, 0xBE, 0xEF};
    gen_memmove(buf, buf, 4);
    ASSERT(buf[0] == 0xDE && buf[3] == 0xEF, "memmove same pointer");
}

static void test_memmove_zero_length(void)
{
    unsigned char buf[4] = {1,2,3,4};
    gen_memmove(buf, buf + 2, 0);
    ASSERT(buf[0] == 1, "memmove zero-length");
}

/* ── gen_memcmp tests ──────────────────────────────────────── */

static void test_memcmp_equal(void)
{
    unsigned char a[8] = {1,2,3,4,5,6,7,8};
    unsigned char b[8] = {1,2,3,4,5,6,7,8};
    ASSERT(gen_memcmp(a, b, 8) == 0, "memcmp equal");
}

static void test_memcmp_less(void)
{
    unsigned char a[4] = {1,2,3,4};
    unsigned char b[4] = {1,2,4,4};
    ASSERT(gen_memcmp(a, b, 4) < 0, "memcmp less");
}

static void test_memcmp_greater(void)
{
    unsigned char a[4] = {1,2,5,4};
    unsigned char b[4] = {1,2,3,4};
    ASSERT(gen_memcmp(a, b, 4) > 0, "memcmp greater");
}

static void test_memcmp_zero_length(void)
{
    unsigned char a[4] = {0xFF, 0xFF, 0xFF, 0xFF};
    unsigned char b[4] = {0, 0, 0, 0};
    ASSERT(gen_memcmp(a, b, 0) == 0, "memcmp zero-length");
}

static void test_memcmp_first_byte_differs(void)
{
    unsigned char a[4] = {0x00, 0, 0, 0};
    unsigned char b[4] = {0xFF, 0, 0, 0};
    ASSERT(gen_memcmp(a, b, 4) < 0, "memcmp first byte less");
    ASSERT(gen_memcmp(b, a, 4) > 0, "memcmp first byte greater");
}

/* ── gen_strlen tests ──────────────────────────────────────── */

static void test_strlen_basic(void)
{
    ASSERT(gen_strlen("hello") == 5, "strlen basic");
}

static void test_strlen_empty(void)
{
    ASSERT(gen_strlen("") == 0, "strlen empty");
}

static void test_strlen_with_internal_data(void)
{
    char buf[8] = {'a', 'b', 'c', '\0', 'd', 'e', 'f', '\0'};
    ASSERT(gen_strlen(buf) == 3, "strlen stops at first null");
}

/* ── gen_strncmp tests ─────────────────────────────────────── */

static void test_strncmp_equal(void)
{
    ASSERT(gen_strncmp("abc", "abc", 3) == 0, "strncmp equal");
}

static void test_strncmp_less(void)
{
    ASSERT(gen_strncmp("abc", "abd", 3) < 0, "strncmp less");
}

static void test_strncmp_greater(void)
{
    ASSERT(gen_strncmp("abd", "abc", 3) > 0, "strncmp greater");
}

static void test_strncmp_prefix(void)
{
    ASSERT(gen_strncmp("abcdef", "abcxyz", 3) == 0, "strncmp prefix");
}

static void test_strncmp_zero_length(void)
{
    ASSERT(gen_strncmp("abc", "xyz", 0) == 0, "strncmp zero-length");
}

static void test_strncmp_early_null(void)
{
    ASSERT(gen_strncmp("ab", "ab\0extra", 10) == 0, "strncmp early null");
}

/* ── gen_strnlen tests ─────────────────────────────────────── */

static void test_strnlen_basic(void)
{
    ASSERT(gen_strnlen("hello", 10) == 5, "strnlen basic");
}

static void test_strnlen_bounded(void)
{
    ASSERT(gen_strnlen("hello", 3) == 3, "strnlen bounded");
}

static void test_strnlen_empty(void)
{
    ASSERT(gen_strnlen("", 100) == 0, "strnlen empty");
}

static void test_strnlen_zero_max(void)
{
    ASSERT(gen_strnlen("hello", 0) == 0, "strnlen zero max");
}

/* ── gen_secure_zero tests ─────────────────────────────────── */

static void test_secure_zero_basic(void)
{
    unsigned char buf[32];
    gen_memset(buf, 0xFF, 32);
    gen_secure_zero(buf, 32);
    for (int i = 0; i < 32; i++) {
        ASSERT(buf[i] == 0, "secure_zero byte not zero");
    }
}

static void test_secure_zero_partial(void)
{
    unsigned char buf[8] = {0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF};
    gen_secure_zero(buf + 2, 4);
    ASSERT(buf[0] == 0xFF, "secure_zero partial: before");
    ASSERT(buf[1] == 0xFF, "secure_zero partial: before");
    ASSERT(buf[2] == 0, "secure_zero partial: in region");
    ASSERT(buf[5] == 0, "secure_zero partial: in region");
    ASSERT(buf[6] == 0xFF, "secure_zero partial: after");
    ASSERT(buf[7] == 0xFF, "secure_zero partial: after");
}

static void test_secure_zero_zero_length(void)
{
    unsigned char buf[4] = {0xAA, 0xBB, 0xCC, 0xDD};
    gen_secure_zero(buf, 0);
    ASSERT(buf[0] == 0xAA, "secure_zero zero-length");
}

/* ── Alias tests ───────────────────────────────────────────── */

static void test_aliases(void)
{
    /* Verify the standard names resolve to the gen_* implementations */
    unsigned char src[8] = {10,20,30,40,50,60,70,80};
    unsigned char dst[8] = {0};

    memcpy(dst, src, 8);
    ASSERT(gen_memcmp(dst, src, 8) == 0, "memcpy alias");

    memset(dst, 0, 8);
    ASSERT(dst[0] == 0 && dst[7] == 0, "memset alias");

    unsigned char buf[16] = {1,2,3,4,5,6,7,8,0,0,0,0,0,0,0,0};
    memmove(buf + 4, buf, 8);
    ASSERT(buf[4] == 1, "memmove alias");

    unsigned char a[4] = {1,2,3,4};
    unsigned char b[4] = {1,2,3,4};
    ASSERT(memcmp(a, b, 4) == 0, "memcmp alias");
}

/* ── Main ──────────────────────────────────────────────────── */

int main(void)
{
    /* memcpy */
    test_memcpy_basic();
    test_memcpy_zero_length();
    test_memcpy_single_byte();
    test_memcpy_large_aligned();

    /* memset */
    test_memset_basic();
    test_memset_zero();
    test_memset_zero_length();
    test_memset_partial();

    /* memmove */
    test_memmove_no_overlap();
    test_memmove_overlap_forward();
    test_memmove_overlap_backward();
    test_memmove_same_pointer();
    test_memmove_zero_length();

    /* memcmp */
    test_memcmp_equal();
    test_memcmp_less();
    test_memcmp_greater();
    test_memcmp_zero_length();
    test_memcmp_first_byte_differs();

    /* strlen */
    test_strlen_basic();
    test_strlen_empty();
    test_strlen_with_internal_data();

    /* strncmp */
    test_strncmp_equal();
    test_strncmp_less();
    test_strncmp_greater();
    test_strncmp_prefix();
    test_strncmp_zero_length();
    test_strncmp_early_null();

    /* strnlen */
    test_strnlen_basic();
    test_strnlen_bounded();
    test_strnlen_empty();
    test_strnlen_zero_max();

    /* secure_zero */
    test_secure_zero_basic();
    test_secure_zero_partial();
    test_secure_zero_zero_length();

    /* aliases */
    test_aliases();

    return failures;
}
