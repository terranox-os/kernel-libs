/*
 * genesis_result.h — Universal kernel result type and error codes.
 *
 * Shared by TerranoxOS, GenesisOS-RT, and HermeticaOS.
 * This header is the ABI source of truth; the Rust crate mirrors it.
 *
 * Freestanding: requires only <stdint.h> and <stddef.h>.
 */

#ifndef GENESIS_RESULT_H
#define GENESIS_RESULT_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * GenResult: Universal kernel result type.
 *
 * Encoding: 0 = success, negative = error.
 * Signed 32-bit integer rather than enum to allow extension
 * without ABI breaks.
 */
typedef int32_t GenResult;

/* ── Success ─────────────────────────────────────────────── */

#define GEN_OK ((GenResult)0)

/* ── General errors (-1 to -15) ──────────────────────────── */

#define GEN_ERR_INVALID_ARG      ((GenResult) -1)
#define GEN_ERR_OUT_OF_MEMORY    ((GenResult) -2)
#define GEN_ERR_NOT_FOUND        ((GenResult) -3)
#define GEN_ERR_ALREADY_EXISTS   ((GenResult) -4)
#define GEN_ERR_BUFFER_TOO_SMALL ((GenResult) -5)
#define GEN_ERR_NOT_SUPPORTED    ((GenResult) -6)
#define GEN_ERR_BUSY             ((GenResult) -7)
#define GEN_ERR_TIMEOUT          ((GenResult) -8)
#define GEN_ERR_INTERRUPTED      ((GenResult) -9)
#define GEN_ERR_OVERFLOW         ((GenResult)-10)

/* ── Permission / security errors (-16 to -31) ──────────── */

#define GEN_ERR_PERMISSION_DENIED  ((GenResult)-16)
#define GEN_ERR_ACCESS_VIOLATION   ((GenResult)-17)
#define GEN_ERR_INVALID_CAPABILITY ((GenResult)-18)

/* ── I/O and hardware errors (-32 to -47) ────────────────── */

#define GEN_ERR_IO             ((GenResult)-32)
#define GEN_ERR_DEVICE_OFFLINE ((GenResult)-33)
#define GEN_ERR_BAD_ADDRESS    ((GenResult)-34)

/* ── Format / parse errors (-48 to -63) ──────────────────── */

#define GEN_ERR_INVALID_FORMAT     ((GenResult)-48)
#define GEN_ERR_CHECKSUM_MISMATCH  ((GenResult)-49)
#define GEN_ERR_VERSION_MISMATCH   ((GenResult)-50)

/* ── Module errors (-64 to -79) — HermeticaOS ───────────── */

#define GEN_ERR_MODULE_LOAD_FAILED  ((GenResult)-64)
#define GEN_ERR_MODULE_INIT_FAILED  ((GenResult)-65)
#define GEN_ERR_MODULE_NOT_FOUND    ((GenResult)-66)
#define GEN_ERR_MODULE_INCOMPATIBLE ((GenResult)-67)

/* ── Convenience helpers ─────────────────────────────────── */

static inline int gen_result_is_ok(GenResult r)    { return r == GEN_OK; }
static inline int gen_result_is_error(GenResult r)  { return r < 0; }

static inline const char *gen_result_name(GenResult r)
{
    switch (r) {
    case GEN_OK:                      return "OK";
    case GEN_ERR_INVALID_ARG:         return "INVALID_ARG";
    case GEN_ERR_OUT_OF_MEMORY:       return "OUT_OF_MEMORY";
    case GEN_ERR_NOT_FOUND:           return "NOT_FOUND";
    case GEN_ERR_ALREADY_EXISTS:      return "ALREADY_EXISTS";
    case GEN_ERR_BUFFER_TOO_SMALL:    return "BUFFER_TOO_SMALL";
    case GEN_ERR_NOT_SUPPORTED:       return "NOT_SUPPORTED";
    case GEN_ERR_BUSY:                return "BUSY";
    case GEN_ERR_TIMEOUT:             return "TIMEOUT";
    case GEN_ERR_INTERRUPTED:         return "INTERRUPTED";
    case GEN_ERR_OVERFLOW:            return "OVERFLOW";
    case GEN_ERR_PERMISSION_DENIED:   return "PERMISSION_DENIED";
    case GEN_ERR_ACCESS_VIOLATION:    return "ACCESS_VIOLATION";
    case GEN_ERR_INVALID_CAPABILITY:  return "INVALID_CAPABILITY";
    case GEN_ERR_IO:                  return "IO";
    case GEN_ERR_DEVICE_OFFLINE:      return "DEVICE_OFFLINE";
    case GEN_ERR_BAD_ADDRESS:         return "BAD_ADDRESS";
    case GEN_ERR_INVALID_FORMAT:      return "INVALID_FORMAT";
    case GEN_ERR_CHECKSUM_MISMATCH:   return "CHECKSUM_MISMATCH";
    case GEN_ERR_VERSION_MISMATCH:    return "VERSION_MISMATCH";
    case GEN_ERR_MODULE_LOAD_FAILED:  return "MODULE_LOAD_FAILED";
    case GEN_ERR_MODULE_INIT_FAILED:  return "MODULE_INIT_FAILED";
    case GEN_ERR_MODULE_NOT_FOUND:    return "MODULE_NOT_FOUND";
    case GEN_ERR_MODULE_INCOMPATIBLE: return "MODULE_INCOMPATIBLE";
    default:                          return "UNKNOWN";
    }
}

/*
 * GEN_STRUCT_HAS_FIELD(ptr, field)
 *
 * Compile-time check: evaluates true if the struct type of *ptr
 * is large enough to contain 'field'. Enables forward-compatible
 * ABI checks when structures grow across versions.
 *
 * Uses __typeof__ (GCC/Clang extension; all three target kernels
 * compile with GCC or Clang).
 */
#define GEN_STRUCT_HAS_FIELD(ptr, field) \
    (sizeof(*(ptr)) >= offsetof(__typeof__(*(ptr)), field) + sizeof((ptr)->field))

#ifdef __cplusplus
}
#endif

#endif /* GENESIS_RESULT_H */
