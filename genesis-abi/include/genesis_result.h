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

/* ── RT errors (-80 to -95) — GenesisOS-RT ───────────────── */

#define GEN_ERR_DEADLINE_MISS  ((GenResult)-80)
#define GEN_ERR_PRIORITY_INV   ((GenResult)-81)
#define GEN_ERR_STACK_OVERFLOW ((GenResult)-82)
/* -83 to -95 reserved for future RT errors */

/* ── Syscall errors (-96 to -111) — TerranoxOS, HermeticaOS  */

#define GEN_ERR_BAD_SYSCALL          ((GenResult)-96)
#define GEN_ERR_BAD_HANDLE           ((GenResult)-97)
#define GEN_ERR_SYSCALL_INTERRUPTED  ((GenResult)-98)
#define GEN_ERR_HANDLE_LIMIT         ((GenResult)-99)
/* -100 to -111 reserved for future syscall errors */

/* ── TerranoxOS I/O extension errors (-35 to -47) ───────── */

#define GEN_ERR_CHANNEL_CLOSED   ((GenResult)-35)
#define GEN_ERR_DISPLAY_OFFLINE  ((GenResult)-36)
#define GEN_ERR_GPU_ERROR        ((GenResult)-37)

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
    case GEN_ERR_DEADLINE_MISS:       return "DEADLINE_MISS";
    case GEN_ERR_PRIORITY_INV:        return "PRIORITY_INV";
    case GEN_ERR_STACK_OVERFLOW:      return "STACK_OVERFLOW";
    case GEN_ERR_BAD_SYSCALL:         return "BAD_SYSCALL";
    case GEN_ERR_BAD_HANDLE:          return "BAD_HANDLE";
    case GEN_ERR_SYSCALL_INTERRUPTED: return "SYSCALL_INTERRUPTED";
    case GEN_ERR_HANDLE_LIMIT:        return "HANDLE_LIMIT";
    case GEN_ERR_CHANNEL_CLOSED:      return "CHANNEL_CLOSED";
    case GEN_ERR_DISPLAY_OFFLINE:     return "DISPLAY_OFFLINE";
    case GEN_ERR_GPU_ERROR:           return "GPU_ERROR";
    default:                          return "UNKNOWN";
    }
}

/* ── POSIX errno mapping (TerranoxOS syscall boundary) ──── */

/*
 * POSIX errno values used for translation at the syscall interface.
 * Kernel-internal errors use the gap-based GenResult scheme above;
 * these constants are only used by gen_result_to_errno() / gen_result_from_errno().
 */
#define GEN_POSIX_EPERM      1
#define GEN_POSIX_ENOENT     2
#define GEN_POSIX_ESRCH      3
#define GEN_POSIX_EBADF      9
#define GEN_POSIX_EAGAIN    11
#define GEN_POSIX_ENOMEM    12
#define GEN_POSIX_EACCES    13
#define GEN_POSIX_EFAULT    14
#define GEN_POSIX_EBUSY     16
#define GEN_POSIX_EEXIST    17
#define GEN_POSIX_EINVAL    22
#define GEN_POSIX_EPIPE     32
#define GEN_POSIX_ENOSYS    38
#define GEN_POSIX_ETIMEDOUT 110

/*@ requires \true;
    assigns  \nothing;
    ensures  r == GEN_OK ==> \result == 0;
    ensures  r != GEN_OK ==> \result > 0;
*/
static inline int gen_result_to_errno(GenResult r)
{
    switch (r) {
    case GEN_OK:                      return 0;
    case GEN_ERR_INVALID_ARG:         return GEN_POSIX_EINVAL;
    case GEN_ERR_OUT_OF_MEMORY:       return GEN_POSIX_ENOMEM;
    case GEN_ERR_NOT_FOUND:           return GEN_POSIX_ENOENT;
    case GEN_ERR_ALREADY_EXISTS:      return GEN_POSIX_EEXIST;
    case GEN_ERR_BUFFER_TOO_SMALL:    return GEN_POSIX_EINVAL;
    case GEN_ERR_NOT_SUPPORTED:       return GEN_POSIX_ENOSYS;
    case GEN_ERR_BUSY:                return GEN_POSIX_EBUSY;
    case GEN_ERR_TIMEOUT:             return GEN_POSIX_ETIMEDOUT;
    case GEN_ERR_INTERRUPTED:         return GEN_POSIX_EAGAIN;
    case GEN_ERR_OVERFLOW:            return GEN_POSIX_EINVAL;
    case GEN_ERR_PERMISSION_DENIED:   return GEN_POSIX_EPERM;
    case GEN_ERR_ACCESS_VIOLATION:    return GEN_POSIX_EACCES;
    case GEN_ERR_INVALID_CAPABILITY:  return GEN_POSIX_EPERM;
    case GEN_ERR_IO:                  return GEN_POSIX_EPIPE;
    case GEN_ERR_BAD_ADDRESS:         return GEN_POSIX_EFAULT;
    case GEN_ERR_BAD_SYSCALL:         return GEN_POSIX_ENOSYS;
    case GEN_ERR_BAD_HANDLE:          return GEN_POSIX_EBADF;
    case GEN_ERR_SYSCALL_INTERRUPTED: return GEN_POSIX_EAGAIN;
    case GEN_ERR_DEVICE_OFFLINE:       return GEN_POSIX_ENOENT;
    case GEN_ERR_CHANNEL_CLOSED:      return GEN_POSIX_EPIPE;
    case GEN_ERR_DISPLAY_OFFLINE:     return GEN_POSIX_ENOENT;
    case GEN_ERR_GPU_ERROR:           return GEN_POSIX_EPIPE;
    case GEN_ERR_INVALID_FORMAT:      return GEN_POSIX_EINVAL;
    case GEN_ERR_CHECKSUM_MISMATCH:   return GEN_POSIX_EINVAL;
    case GEN_ERR_VERSION_MISMATCH:    return GEN_POSIX_EINVAL;
    case GEN_ERR_MODULE_LOAD_FAILED:  return GEN_POSIX_ENOENT;
    case GEN_ERR_MODULE_INIT_FAILED:  return GEN_POSIX_EPERM;
    case GEN_ERR_MODULE_NOT_FOUND:    return GEN_POSIX_ENOENT;
    case GEN_ERR_MODULE_INCOMPATIBLE: return GEN_POSIX_EINVAL;
    case GEN_ERR_DEADLINE_MISS:       return GEN_POSIX_ETIMEDOUT;
    case GEN_ERR_PRIORITY_INV:        return GEN_POSIX_EAGAIN;
    case GEN_ERR_STACK_OVERFLOW:      return GEN_POSIX_ENOMEM;
    case GEN_ERR_HANDLE_LIMIT:        return GEN_POSIX_ENOMEM;
    default:                          return GEN_POSIX_EINVAL;
    }
}

/*@ requires \true;
    assigns  \nothing;
    ensures  e == 0 ==> \result == GEN_OK;
*/
static inline GenResult gen_result_from_errno(int e)
{
    switch (e) {
    case 0:                  return GEN_OK;
    case GEN_POSIX_EPERM:    return GEN_ERR_PERMISSION_DENIED;
    case GEN_POSIX_ENOENT:   return GEN_ERR_NOT_FOUND;
    case GEN_POSIX_ESRCH:    return GEN_ERR_NOT_FOUND;
    case GEN_POSIX_EBADF:    return GEN_ERR_BAD_HANDLE;
    case GEN_POSIX_EAGAIN:   return GEN_ERR_INTERRUPTED;
    case GEN_POSIX_ENOMEM:   return GEN_ERR_OUT_OF_MEMORY;
    case GEN_POSIX_EACCES:   return GEN_ERR_ACCESS_VIOLATION;
    case GEN_POSIX_EFAULT:   return GEN_ERR_BAD_ADDRESS;
    case GEN_POSIX_EBUSY:    return GEN_ERR_BUSY;
    case GEN_POSIX_EEXIST:   return GEN_ERR_ALREADY_EXISTS;
    case GEN_POSIX_EINVAL:   return GEN_ERR_INVALID_ARG;
    case GEN_POSIX_EPIPE:    return GEN_ERR_CHANNEL_CLOSED;
    case GEN_POSIX_ENOSYS:   return GEN_ERR_BAD_SYSCALL;
    case GEN_POSIX_ETIMEDOUT:return GEN_ERR_TIMEOUT;
    default:                 return GEN_ERR_NOT_SUPPORTED;
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
