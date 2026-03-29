/*
 * sha256.c — SHA-256 (FIPS 180-4).
 *
 * Streaming hash with init/update/finalize API.
 * Freestanding: no libc, no dynamic allocation.
 */

#include "gen_crypto.h"

/* ── Constants ──────────────────────────────────────────── */

static const uint32_t K[64] = {
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
};

static const uint32_t H_INIT[8] = {
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
};

/* ── Helpers ────────────────────────────────────────────── */

static inline uint32_t rotr(uint32_t x, unsigned n)
{
    return (x >> n) | (x << (32 - n));
}

static inline uint32_t ch(uint32_t x, uint32_t y, uint32_t z)
{
    return (x & y) ^ (~x & z);
}

static inline uint32_t maj(uint32_t x, uint32_t y, uint32_t z)
{
    return (x & y) ^ (x & z) ^ (y & z);
}

static inline uint32_t sigma0(uint32_t x)
{
    return rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22);
}

static inline uint32_t sigma1(uint32_t x)
{
    return rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25);
}

static inline uint32_t lsigma0(uint32_t x)
{
    return rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3);
}

static inline uint32_t lsigma1(uint32_t x)
{
    return rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10);
}

static inline uint32_t read_be32(const uint8_t *p)
{
    return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] << 8)  |  (uint32_t)p[3];
}

static inline void write_be32(uint8_t *p, uint32_t v)
{
    p[0] = (uint8_t)(v >> 24);
    p[1] = (uint8_t)(v >> 16);
    p[2] = (uint8_t)(v >> 8);
    p[3] = (uint8_t)(v);
}

static inline void write_be64(uint8_t *p, uint64_t v)
{
    p[0] = (uint8_t)(v >> 56);
    p[1] = (uint8_t)(v >> 48);
    p[2] = (uint8_t)(v >> 40);
    p[3] = (uint8_t)(v >> 32);
    p[4] = (uint8_t)(v >> 24);
    p[5] = (uint8_t)(v >> 16);
    p[6] = (uint8_t)(v >> 8);
    p[7] = (uint8_t)(v);
}

/* ── Block processing ───────────────────────────────────── */

static void sha256_process_block(GenSha256 *ctx, const uint8_t block[64])
{
    uint32_t w[64];

    for (int i = 0; i < 16; i++)
        w[i] = read_be32(&block[i * 4]);

    for (int i = 16; i < 64; i++)
        w[i] = lsigma1(w[i - 2]) + w[i - 7] +
               lsigma0(w[i - 15]) + w[i - 16];

    uint32_t a = ctx->state[0], b = ctx->state[1];
    uint32_t c = ctx->state[2], d = ctx->state[3];
    uint32_t e = ctx->state[4], f = ctx->state[5];
    uint32_t g = ctx->state[6], h = ctx->state[7];

    for (int i = 0; i < 64; i++) {
        uint32_t t1 = h + sigma1(e) + ch(e, f, g) + K[i] + w[i];
        uint32_t t2 = sigma0(a) + maj(a, b, c);
        h = g; g = f; f = e; e = d + t1;
        d = c; c = b; b = a; a = t1 + t2;
    }

    ctx->state[0] += a; ctx->state[1] += b;
    ctx->state[2] += c; ctx->state[3] += d;
    ctx->state[4] += e; ctx->state[5] += f;
    ctx->state[6] += g; ctx->state[7] += h;
}

/* ── Public API ─────────────────────────────────────────── */

void gen_sha256_init(GenSha256 *ctx)
{
    for (int i = 0; i < 8; i++)
        ctx->state[i] = H_INIT[i];
    ctx->buf_len = 0;
    ctx->total_len = 0;
}

void gen_sha256_update(GenSha256 *ctx, const uint8_t *data, size_t len)
{
    size_t offset = 0;
    ctx->total_len += len;

    /* Fill partial buffer */
    if (ctx->buf_len > 0) {
        size_t need = 64 - ctx->buf_len;
        size_t take = len < need ? len : need;
        for (size_t i = 0; i < take; i++)
            ctx->buf[ctx->buf_len + i] = data[i];
        ctx->buf_len += take;
        offset += take;

        if (ctx->buf_len == 64) {
            sha256_process_block(ctx, ctx->buf);
            ctx->buf_len = 0;
        }
    }

    /* Process full blocks */
    while (offset + 64 <= len) {
        sha256_process_block(ctx, &data[offset]);
        offset += 64;
    }

    /* Buffer remaining */
    size_t remaining = len - offset;
    if (remaining > 0) {
        for (size_t i = 0; i < remaining; i++)
            ctx->buf[i] = data[offset + i];
        ctx->buf_len = remaining;
    }
}

void gen_sha256_finalize(GenSha256 *ctx, uint8_t digest[32])
{
    uint64_t bit_len = ctx->total_len * 8;

    /* Append 0x80 */
    ctx->buf[ctx->buf_len] = 0x80;
    ctx->buf_len++;

    /* If not enough room for length, pad and process */
    if (ctx->buf_len > 56) {
        for (size_t i = ctx->buf_len; i < 64; i++)
            ctx->buf[i] = 0;
        sha256_process_block(ctx, ctx->buf);
        ctx->buf_len = 0;
    }

    /* Pad to 56 bytes */
    for (size_t i = ctx->buf_len; i < 56; i++)
        ctx->buf[i] = 0;

    /* Append 64-bit big-endian length */
    write_be64(&ctx->buf[56], bit_len);
    sha256_process_block(ctx, ctx->buf);

    /* Write digest */
    for (int i = 0; i < 8; i++)
        write_be32(&digest[i * 4], ctx->state[i]);
}

void gen_sha256_digest(const uint8_t *data, size_t len, uint8_t digest[32])
{
    GenSha256 ctx;
    gen_sha256_init(&ctx);
    gen_sha256_update(&ctx, data, len);
    gen_sha256_finalize(&ctx, digest);
}
