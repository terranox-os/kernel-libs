/*
 * gen_crypto.h — Freestanding cryptographic primitives.
 *
 * CRC-32 (IEEE 802.3), SHA-256 (FIPS 180-4), HMAC-SHA256 (RFC 2104).
 * All implementations are constant-time where practical and use no
 * dynamic allocation.
 *
 * Freestanding: requires only <stdint.h> and <stddef.h>.
 */

#ifndef GEN_CRYPTO_H
#define GEN_CRYPTO_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── CRC-32 (IEEE 802.3) ───────────────────────────────── */

/*
 * Compute CRC-32 over a byte buffer.
 * Uses reflected polynomial 0xEDB88320.
 * When GEN_CRC32_NO_TABLE is defined, uses bit-by-bit fallback
 * (smaller code, ~8x slower).
 */
uint32_t gen_crc32(const uint8_t *data, size_t len);

/*
 * Incrementally update a running CRC-32.
 *
 * Usage:
 *   uint32_t crc = gen_crc32(chunk1, len1);
 *   crc = gen_crc32_update(crc, chunk2, len2);
 */
uint32_t gen_crc32_update(uint32_t crc, const uint8_t *data, size_t len);

/* ── SHA-256 (FIPS 180-4) ──────────────────────────────── */

#define GEN_SHA256_DIGEST_SIZE  32
#define GEN_SHA256_BLOCK_SIZE   64

typedef struct GenSha256 {
    uint32_t state[8];
    uint8_t  buf[GEN_SHA256_BLOCK_SIZE];
    size_t   buf_len;
    uint64_t total_len;
} GenSha256;

void gen_sha256_init(GenSha256 *ctx);
void gen_sha256_update(GenSha256 *ctx, const uint8_t *data, size_t len);
void gen_sha256_finalize(GenSha256 *ctx, uint8_t digest[GEN_SHA256_DIGEST_SIZE]);

/* One-shot convenience */
void gen_sha256_digest(const uint8_t *data, size_t len,
                       uint8_t digest[GEN_SHA256_DIGEST_SIZE]);

/* ── HMAC-SHA256 (RFC 2104) ─────────────────────────────── */

void gen_hmac_sha256(const uint8_t *key, size_t key_len,
                     const uint8_t *data, size_t data_len,
                     uint8_t mac[GEN_SHA256_DIGEST_SIZE]);

#ifdef __cplusplus
}
#endif

#endif /* GEN_CRYPTO_H */
