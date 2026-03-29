/*
 * crypto_test.c — Host-compiled tests for gen_crypto C API.
 *
 * Same test vectors as the Rust tests:
 *   CRC-32:  IEEE 802.3 standard check value
 *   SHA-256: FIPS 180-4 test vectors
 *   HMAC:    RFC 4231 test vectors
 *
 * Returns 0 on success, non-zero failure count.
 */

#include "gen_crypto.h"
#include <string.h>
#include <stdio.h>

static int failures = 0;

#define ASSERT(cond, msg)                      \
    do {                                       \
        if (!(cond)) {                         \
            fprintf(stderr, "FAIL: %s\n", msg);\
            failures++;                        \
        }                                      \
    } while (0)

#define ASSERT_EQ_U32(a, b, msg)                                       \
    do {                                                               \
        uint32_t _a = (a), _b = (b);                                   \
        if (_a != _b) {                                                \
            fprintf(stderr, "FAIL: %s (got 0x%08X, expected 0x%08X)\n",\
                    msg, _a, _b);                                      \
            failures++;                                                \
        }                                                              \
    } while (0)

static int memcmp_digest(const uint8_t *a, const uint8_t *b, size_t n)
{
    for (size_t i = 0; i < n; i++)
        if (a[i] != b[i]) return 1;
    return 0;
}

/* ── CRC-32 tests ───────────────────────────────────────── */

static void test_crc32_empty(void)
{
    ASSERT_EQ_U32(gen_crc32((const uint8_t *)"", 0), 0x00000000U, "crc32 empty");
}

static void test_crc32_check_value(void)
{
    ASSERT_EQ_U32(gen_crc32((const uint8_t *)"123456789", 9), 0xCBF43926U,
                  "crc32 check value");
}

static void test_crc32_single_byte(void)
{
    ASSERT_EQ_U32(gen_crc32((const uint8_t *)"A", 1), 0xD3D99E8BU,
                  "crc32 single byte");
}

static void test_crc32_incremental(void)
{
    uint32_t full = gen_crc32((const uint8_t *)"hello world", 11);
    uint32_t partial = gen_crc32_update(
        gen_crc32((const uint8_t *)"hello ", 6),
        (const uint8_t *)"world", 5);
    ASSERT_EQ_U32(full, partial, "crc32 incremental");
}

/* ── SHA-256 tests ──────────────────────────────────────── */

static void test_sha256_empty(void)
{
    uint8_t digest[32];
    gen_sha256_digest((const uint8_t *)"", 0, digest);
    static const uint8_t expected[32] = {
        0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
        0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
        0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
        0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
    };
    ASSERT(memcmp_digest(digest, expected, 32) == 0, "sha256 empty");
}

static void test_sha256_abc(void)
{
    uint8_t digest[32];
    gen_sha256_digest((const uint8_t *)"abc", 3, digest);
    static const uint8_t expected[32] = {
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
        0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
        0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
        0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    };
    ASSERT(memcmp_digest(digest, expected, 32) == 0, "sha256 abc");
}

static void test_sha256_two_block(void)
{
    uint8_t digest[32];
    const char *msg = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    gen_sha256_digest((const uint8_t *)msg, strlen(msg), digest);
    static const uint8_t expected[32] = {
        0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8,
        0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39,
        0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67,
        0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1,
    };
    ASSERT(memcmp_digest(digest, expected, 32) == 0, "sha256 two-block");
}

static void test_sha256_incremental(void)
{
    GenSha256 ctx;
    gen_sha256_init(&ctx);
    gen_sha256_update(&ctx, (const uint8_t *)"abc", 3);
    gen_sha256_update(&ctx, (const uint8_t *)"dbcdecdefdefg", 13);
    gen_sha256_update(&ctx, (const uint8_t *)"efghfghighijhijkijkljklmklmnlmnomnopnopq", 40);
    uint8_t digest[32];
    gen_sha256_finalize(&ctx, digest);
    static const uint8_t expected[32] = {
        0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8,
        0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39,
        0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67,
        0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1,
    };
    ASSERT(memcmp_digest(digest, expected, 32) == 0, "sha256 incremental");
}

/* ── HMAC-SHA256 tests ──────────────────────────────────── */

static void test_hmac_rfc4231_case1(void)
{
    uint8_t key[20];
    for (int i = 0; i < 20; i++) key[i] = 0x0b;
    uint8_t mac[32];
    gen_hmac_sha256(key, 20, (const uint8_t *)"Hi There", 8, mac);
    static const uint8_t expected[32] = {
        0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53,
        0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b, 0xf1, 0x2b,
        0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7,
        0x26, 0xe9, 0x37, 0x6c, 0x2e, 0x32, 0xcf, 0xf7,
    };
    ASSERT(memcmp_digest(mac, expected, 32) == 0, "hmac rfc4231 case1");
}

static void test_hmac_rfc4231_case2(void)
{
    uint8_t mac[32];
    gen_hmac_sha256((const uint8_t *)"Jefe", 4,
                    (const uint8_t *)"what do ya want for nothing?", 28, mac);
    static const uint8_t expected[32] = {
        0x5b, 0xdc, 0xc1, 0x46, 0xbf, 0x60, 0x75, 0x4e,
        0x6a, 0x04, 0x24, 0x26, 0x08, 0x95, 0x75, 0xc7,
        0x5a, 0x00, 0x3f, 0x08, 0x9d, 0x27, 0x39, 0x83,
        0x9d, 0xec, 0x58, 0xb9, 0x64, 0xec, 0x38, 0x43,
    };
    ASSERT(memcmp_digest(mac, expected, 32) == 0, "hmac rfc4231 case2");
}

static void test_hmac_long_key(void)
{
    uint8_t key[131];
    for (int i = 0; i < 131; i++) key[i] = 0xAA;
    const char *data = "Test Using Larger Than Block-Size Key - Hash Key First";
    uint8_t mac[32];
    gen_hmac_sha256(key, 131, (const uint8_t *)data, strlen(data), mac);
    static const uint8_t expected[32] = {
        0x60, 0xe4, 0x31, 0x59, 0x1e, 0xe0, 0xb6, 0x7f,
        0x0d, 0x8a, 0x26, 0xaa, 0xcb, 0xf5, 0xb7, 0x7f,
        0x8e, 0x0b, 0xc6, 0x21, 0x37, 0x28, 0xc5, 0x14,
        0x05, 0x46, 0x04, 0x0f, 0x0e, 0xe3, 0x7f, 0x54,
    };
    ASSERT(memcmp_digest(mac, expected, 32) == 0, "hmac long key");
}

/* ── Main ───────────────────────────────────────────────── */

int main(void)
{
    /* CRC-32 */
    test_crc32_empty();
    test_crc32_check_value();
    test_crc32_single_byte();
    test_crc32_incremental();

    /* SHA-256 */
    test_sha256_empty();
    test_sha256_abc();
    test_sha256_two_block();
    test_sha256_incremental();

    /* HMAC-SHA256 */
    test_hmac_rfc4231_case1();
    test_hmac_rfc4231_case2();
    test_hmac_long_key();

    return failures;
}
