/*
 * gen_primitives.h — Freestanding memory and string primitives.
 *
 * All functions are namespaced with gen_ prefix. Compiler-required
 * symbols (memcpy, memset, memmove, memcmp) are provided as thin
 * wrappers in aliases.c — link that file exactly once.
 *
 * Freestanding: requires only <stddef.h>.
 */

#ifndef GEN_PRIMITIVES_H
#define GEN_PRIMITIVES_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Memory operations ───────────────────────────────────── */

void  *gen_memcpy(void *dst, const void *src, size_t n);
void  *gen_memset(void *dst, int c, size_t n);
void  *gen_memmove(void *dst, const void *src, size_t n);
int    gen_memcmp(const void *s1, const void *s2, size_t n);

/* ── String operations ───────────────────────────────────── */

size_t gen_strlen(const char *s);
int    gen_strncmp(const char *s1, const char *s2, size_t n);
size_t gen_strnlen(const char *s, size_t maxlen);

/* ── Security ────────────────────────────────────────────── */

/*
 * gen_secure_zero: Guaranteed-not-optimized-away memory zeroing.
 * Uses volatile writes to prevent the compiler from eliding the
 * operation when the buffer is not read afterward.
 */
void   gen_secure_zero(volatile void *dst, size_t n);


#ifdef __cplusplus
}
#endif

#endif /* GEN_PRIMITIVES_H */
