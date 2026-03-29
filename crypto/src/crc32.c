/*
 * crc32.c — CRC-32 (IEEE 802.3, reflected polynomial 0xEDB88320).
 *
 * Define GEN_CRC32_NO_TABLE to use bit-by-bit fallback (saves 1 KB).
 * Freestanding.
 */

#include "gen_crypto.h"

#ifndef GEN_CRC32_NO_TABLE

/* ── Table-based (fast) ─────────────────────────────────── */

/*
 * Build the CRC-32 lookup table at program initialization.
 * Alternatively, a pre-computed table could be used as a static const.
 */
static uint32_t crc32_table[256];
static int crc32_table_ready = 0;

static void crc32_build_table(void)
{
    if (crc32_table_ready) return;
    for (uint32_t i = 0; i < 256; i++) {
        uint32_t crc = i;
        for (int j = 0; j < 8; j++) {
            if (crc & 1)
                crc = (crc >> 1) ^ 0xEDB88320U;
            else
                crc >>= 1;
        }
        crc32_table[i] = crc;
    }
    crc32_table_ready = 1;
}

static uint32_t crc32_compute(uint32_t crc, const uint8_t *data, size_t len)
{
    crc32_build_table();
    for (size_t i = 0; i < len; i++) {
        uint32_t idx = (crc ^ data[i]) & 0xFFU;
        crc = (crc >> 8) ^ crc32_table[idx];
    }
    return crc;
}

#else /* GEN_CRC32_NO_TABLE */

/* ── Bit-by-bit (small) ─────────────────────────────────── */

static uint32_t crc32_compute(uint32_t crc, const uint8_t *data, size_t len)
{
    for (size_t i = 0; i < len; i++) {
        crc ^= data[i];
        for (int j = 0; j < 8; j++) {
            if (crc & 1)
                crc = (crc >> 1) ^ 0xEDB88320U;
            else
                crc >>= 1;
        }
    }
    return crc;
}

#endif /* GEN_CRC32_NO_TABLE */

uint32_t gen_crc32(const uint8_t *data, size_t len)
{
    return crc32_compute(0xFFFFFFFFU, data, len) ^ 0xFFFFFFFFU;
}

uint32_t gen_crc32_update(uint32_t crc, const uint8_t *data, size_t len)
{
    return crc32_compute(crc ^ 0xFFFFFFFFU, data, len) ^ 0xFFFFFFFFU;
}
