/*
 * genesis_syscall.h — Syscall number definitions.
 *
 * Four namespaced ranges (256 entries each):
 *   0x0000–0x00FF  Shared (all three kernels)
 *   0x0100–0x01FF  TerranoxOS (security)
 *   0x0200–0x02FF  GenesisOS-RT (real-time)
 *   0x0300–0x03FF  HermeticaOS (module hot-swap)
 *
 * Freestanding: requires only <stdint.h>.
 */

#ifndef GENESIS_SYSCALL_H
#define GENESIS_SYSCALL_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t GenSyscallNr;

/* ── Range bases ─────────────────────────────────────────── */

#define GEN_SYSCALL_SHARED_BASE    ((GenSyscallNr)0x0000)
#define GEN_SYSCALL_TERRANOX_BASE  ((GenSyscallNr)0x0100)
#define GEN_SYSCALL_GENESISRT_BASE ((GenSyscallNr)0x0200)
#define GEN_SYSCALL_HERMETICA_BASE ((GenSyscallNr)0x0300)

/* ── Range limits (exclusive) ────────────────────────────── */

#define GEN_SYSCALL_SHARED_LIMIT    ((GenSyscallNr)0x0100)
#define GEN_SYSCALL_TERRANOX_LIMIT  ((GenSyscallNr)0x0200)
#define GEN_SYSCALL_GENESISRT_LIMIT ((GenSyscallNr)0x0300)
#define GEN_SYSCALL_HERMETICA_LIMIT ((GenSyscallNr)0x0400)

/* ── Shared syscalls (0x0000–0x00FF) ─────────────────────── */

#define GEN_SYS_EXIT           ((GenSyscallNr)0x0000)
#define GEN_SYS_WRITE          ((GenSyscallNr)0x0001)
#define GEN_SYS_READ           ((GenSyscallNr)0x0002)
#define GEN_SYS_MMAP           ((GenSyscallNr)0x0003)
#define GEN_SYS_MUNMAP         ((GenSyscallNr)0x0004)
#define GEN_SYS_YIELD          ((GenSyscallNr)0x0005)
#define GEN_SYS_GETPID         ((GenSyscallNr)0x0006)
#define GEN_SYS_SLEEP          ((GenSyscallNr)0x0007)
#define GEN_SYS_CLOCK_GETTIME  ((GenSyscallNr)0x0008)

/* ── TerranoxOS syscalls (0x0100–0x01FF) ─────────────────── */

#define GEN_SYS_CAP_GRANT      ((GenSyscallNr)0x0100)
#define GEN_SYS_CAP_REVOKE     ((GenSyscallNr)0x0101)
#define GEN_SYS_CAP_CHECK      ((GenSyscallNr)0x0102)
#define GEN_SYS_SIGIL_SIGN     ((GenSyscallNr)0x0103)
#define GEN_SYS_SIGIL_VERIFY   ((GenSyscallNr)0x0104)
#define GEN_SYS_AUDIT_LOG      ((GenSyscallNr)0x0105)
#define GEN_SYS_SANDBOX_CREATE ((GenSyscallNr)0x0106)
#define GEN_SYS_SANDBOX_ENTER  ((GenSyscallNr)0x0107)

/* ── GenesisOS-RT syscalls (0x0200–0x02FF) ───────────────── */

#define GEN_SYS_RT_TASK_CREATE       ((GenSyscallNr)0x0200)
#define GEN_SYS_RT_TASK_SET_PRIO     ((GenSyscallNr)0x0201)
#define GEN_SYS_RT_TASK_SET_DEADLINE ((GenSyscallNr)0x0202)
#define GEN_SYS_RT_TIMER_CREATE      ((GenSyscallNr)0x0203)
#define GEN_SYS_RT_TIMER_ARM         ((GenSyscallNr)0x0204)
#define GEN_SYS_RT_SENSOR_READ       ((GenSyscallNr)0x0205)
#define GEN_SYS_RT_ACTUATOR_WRITE    ((GenSyscallNr)0x0206)

/* ── HermeticaOS syscalls (0x0300–0x03FF) ────────────────── */

#define GEN_SYS_MOD_LOAD        ((GenSyscallNr)0x0300)
#define GEN_SYS_MOD_UNLOAD      ((GenSyscallNr)0x0301)
#define GEN_SYS_MOD_QUERY       ((GenSyscallNr)0x0302)
#define GEN_SYS_MOD_HOT_SWAP    ((GenSyscallNr)0x0303)
#define GEN_SYS_MOD_IPC_SEND    ((GenSyscallNr)0x0304)
#define GEN_SYS_MOD_IPC_RECV    ((GenSyscallNr)0x0305)
#define GEN_SYS_MOD_CAP_REQUEST ((GenSyscallNr)0x0306)

/* ── Range validation helpers ────────────────────────────── */

static inline int gen_syscall_is_shared(GenSyscallNr nr)
{
    return nr < GEN_SYSCALL_SHARED_LIMIT;
}

static inline int gen_syscall_is_terranox(GenSyscallNr nr)
{
    return nr >= GEN_SYSCALL_TERRANOX_BASE && nr < GEN_SYSCALL_TERRANOX_LIMIT;
}

static inline int gen_syscall_is_genesisrt(GenSyscallNr nr)
{
    return nr >= GEN_SYSCALL_GENESISRT_BASE && nr < GEN_SYSCALL_GENESISRT_LIMIT;
}

static inline int gen_syscall_is_hermetica(GenSyscallNr nr)
{
    return nr >= GEN_SYSCALL_HERMETICA_BASE && nr < GEN_SYSCALL_HERMETICA_LIMIT;
}

#ifdef __cplusplus
}
#endif

#endif /* GENESIS_SYSCALL_H */
