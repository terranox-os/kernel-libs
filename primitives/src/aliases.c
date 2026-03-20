/*
 * aliases.c — Compiler-required symbol definitions.
 *
 * Provides memcpy, memset, memmove, memcmp as thin wrappers
 * around the gen_* implementations. This file MUST be compiled
 * and linked into exactly one translation unit in the final
 * kernel image.
 *
 * Note: __attribute__((alias)) requires the target symbol in the
 * same translation unit. Since gen_* are defined in separate .c
 * files, we use simple forwarding functions instead. The compiler
 * will inline/tail-call these at any optimization level.
 */

#include "gen_primitives.h"

void *memcpy(void *dst, const void *src, size_t n)
{
    return gen_memcpy(dst, src, n);
}

void *memset(void *dst, int c, size_t n)
{
    return gen_memset(dst, c, n);
}

void *memmove(void *dst, const void *src, size_t n)
{
    return gen_memmove(dst, src, n);
}

int memcmp(const void *s1, const void *s2, size_t n)
{
    return gen_memcmp(s1, s2, n);
}
