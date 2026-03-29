/*
 * hmac.c — HMAC-SHA256 (RFC 2104).
 *
 * Uses gen_sha256 internally. Freestanding.
 */

#include "gen_crypto.h"

#define BLOCK_SIZE 64
#define IPAD 0x36
#define OPAD 0x5c

void gen_hmac_sha256(const uint8_t *key, size_t key_len,
                     const uint8_t *data, size_t data_len,
                     uint8_t mac[32])
{
    uint8_t k_block[BLOCK_SIZE];
    size_t i;

    /* Zero the key block */
    for (i = 0; i < BLOCK_SIZE; i++)
        k_block[i] = 0;

    /* Normalize key: hash if > BLOCK_SIZE, else copy */
    if (key_len > BLOCK_SIZE) {
        gen_sha256_digest(key, key_len, k_block);
        /* k_block[32..63] already zeroed */
    } else {
        for (i = 0; i < key_len; i++)
            k_block[i] = key[i];
    }

    /* Inner: SHA256((K ^ ipad) || data) */
    uint8_t ipad_key[BLOCK_SIZE];
    for (i = 0; i < BLOCK_SIZE; i++)
        ipad_key[i] = k_block[i] ^ IPAD;

    GenSha256 inner;
    gen_sha256_init(&inner);
    gen_sha256_update(&inner, ipad_key, BLOCK_SIZE);
    gen_sha256_update(&inner, data, data_len);

    uint8_t inner_hash[32];
    gen_sha256_finalize(&inner, inner_hash);

    /* Outer: SHA256((K ^ opad) || inner_hash) */
    uint8_t opad_key[BLOCK_SIZE];
    for (i = 0; i < BLOCK_SIZE; i++)
        opad_key[i] = k_block[i] ^ OPAD;

    GenSha256 outer;
    gen_sha256_init(&outer);
    gen_sha256_update(&outer, opad_key, BLOCK_SIZE);
    gen_sha256_update(&outer, inner_hash, 32);
    gen_sha256_finalize(&outer, mac);
}
