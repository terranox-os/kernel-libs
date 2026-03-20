/*
 * gen_kfmt.h — Callback-based kernel printf engine.
 *
 * Output goes through a caller-provided function pointer, making
 * this library backend-agnostic:
 *   TerranoxOS  → encrypted audit log
 *   GenesisOS-RT → UART serial
 *   HermeticaOS  → module-subscribable ring buffer
 *
 * Callback receives uint8_t (not char) for unambiguous byte semantics
 * across platforms where char signedness varies.
 *
 * Freestanding: requires only <stdint.h>, <stdarg.h>, <stddef.h>.
 *
 * Supported format specifiers:
 *   %d, %i    signed decimal (int32_t)
 *   %u        unsigned decimal (uint32_t)
 *   %x, %X    hexadecimal (uint32_t), lower/upper case
 *   %p        pointer (void *), printed as 0x... hex
 *   %s        null-terminated string
 *   %c        single character
 *   %%        literal '%'
 *
 * Width and zero-padding supported: %08x, %4d, etc.
 * No floating point. No %n. No dynamic width (*).
 */

#ifndef GEN_KFMT_H
#define GEN_KFMT_H

#include <stdint.h>
#include <stdarg.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void (*gen_output_fn)(uint8_t byte, void *ctx);

int32_t gen_kprintf(gen_output_fn out, void *ctx, const char *fmt, ...);
int32_t gen_kvprintf(gen_output_fn out, void *ctx, const char *fmt, va_list args);

#ifdef __cplusplus
}
#endif

#endif /* GEN_KFMT_H */
