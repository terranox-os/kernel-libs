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
#define GEN_SYS_OPEN           ((GenSyscallNr)0x0009)
#define GEN_SYS_CLOSE          ((GenSyscallNr)0x000A)
#define GEN_SYS_STAT           ((GenSyscallNr)0x000B)
#define GEN_SYS_FSTAT          ((GenSyscallNr)0x000C)
#define GEN_SYS_LSEEK          ((GenSyscallNr)0x000D)
#define GEN_SYS_BRK            ((GenSyscallNr)0x000E)
#define GEN_SYS_IOCTL          ((GenSyscallNr)0x000F)
#define GEN_SYS_DUP2           ((GenSyscallNr)0x0010)
#define GEN_SYS_PIPE           ((GenSyscallNr)0x0011)
#define GEN_SYS_FORK           ((GenSyscallNr)0x0012)
#define GEN_SYS_EXEC           ((GenSyscallNr)0x0013)
#define GEN_SYS_WAIT           ((GenSyscallNr)0x0014)
#define GEN_SYS_FCNTL          ((GenSyscallNr)0x0015)
#define GEN_SYS_POLL           ((GenSyscallNr)0x0016)

/* ── TerranoxOS syscalls (0x0100–0x01FF) ─────────────────── */
/*
 * Organized by subsystem in 16-slot blocks (0x10 per subsystem).
 * Subsystem index: (nr - 0x0100) >> 4
 *
 * Syscalls that overlap with the shared range (exit, read, write,
 * open, close, stat, fstat, lseek, yield, sleep, clock_gettime,
 * exec, wait, mmap, munmap, brk, ioctl, dup2, pipe, fork, fcntl,
 * poll, getpid) are NOT duplicated here — use the GEN_SYS_* shared
 * constants at 0x0000.
 */

/* Subsystem 0: Process management (0x0100–0x010F) */
#define GEN_SYS_TRX_PROCESS_CREATE     ((GenSyscallNr)0x0100)
/* 0x0101 reserved — process_exit uses shared GEN_SYS_EXIT (0x0000) */
/* 0x0102 reserved — process_wait uses shared GEN_SYS_WAIT (0x0014) */
#define GEN_SYS_TRX_PROCESS_KILL       ((GenSyscallNr)0x0103)
#define GEN_SYS_TRX_PROCESS_INFO       ((GenSyscallNr)0x0104)
#define GEN_SYS_TRX_PROCESS_CAP_GRANT  ((GenSyscallNr)0x0105)
#define GEN_SYS_TRX_PROCESS_CAP_REVOKE ((GenSyscallNr)0x0106)
#define GEN_SYS_TRX_PROCESS_CAP_QUERY  ((GenSyscallNr)0x0107)

/* Subsystem 1: Thread management (0x0110–0x011F) */
#define GEN_SYS_TRX_THREAD_CREATE       ((GenSyscallNr)0x0110)
#define GEN_SYS_TRX_THREAD_EXIT         ((GenSyscallNr)0x0111)
#define GEN_SYS_TRX_THREAD_JOIN         ((GenSyscallNr)0x0112)
#define GEN_SYS_TRX_THREAD_SET_AFFINITY ((GenSyscallNr)0x0114)
#define GEN_SYS_TRX_THREAD_GET_AFFINITY ((GenSyscallNr)0x0115)
#define GEN_SYS_TRX_THREAD_SET_NAME     ((GenSyscallNr)0x0116)
#define GEN_SYS_TRX_FUTEX_WAIT          ((GenSyscallNr)0x0117)
#define GEN_SYS_TRX_FUTEX_WAKE          ((GenSyscallNr)0x0118)

/* Subsystem 2: Memory management (0x0120–0x012F) */
#define GEN_SYS_TRX_MEM_PROTECT      ((GenSyscallNr)0x0122)
#define GEN_SYS_TRX_MEM_MAP          ((GenSyscallNr)0x0123)
#define GEN_SYS_TRX_MEM_UNMAP        ((GenSyscallNr)0x0124)
#define GEN_SYS_TRX_MEM_SHARE_CREATE ((GenSyscallNr)0x0125)
#define GEN_SYS_TRX_MEM_SHARE_MAP    ((GenSyscallNr)0x0126)
#define GEN_SYS_TRX_MEM_SHARE_UNMAP  ((GenSyscallNr)0x0127)
#define GEN_SYS_TRX_MEM_DMA_ALLOC   ((GenSyscallNr)0x0128)
#define GEN_SYS_TRX_MEM_DMA_FREE    ((GenSyscallNr)0x0129)

/* Subsystem 3: IPC channels (0x0130–0x013F) */
#define GEN_SYS_TRX_CHANNEL_CREATE   ((GenSyscallNr)0x0130)
#define GEN_SYS_TRX_CHANNEL_SEND     ((GenSyscallNr)0x0131)
#define GEN_SYS_TRX_CHANNEL_RECV     ((GenSyscallNr)0x0132)
#define GEN_SYS_TRX_CHANNEL_CLOSE    ((GenSyscallNr)0x0133)
#define GEN_SYS_TRX_CHANNEL_POLL     ((GenSyscallNr)0x0134)
#define GEN_SYS_TRX_SIGNAL_CREATE    ((GenSyscallNr)0x0135)
#define GEN_SYS_TRX_SIGNAL_RAISE     ((GenSyscallNr)0x0136)
#define GEN_SYS_TRX_SIGNAL_WAIT      ((GenSyscallNr)0x0137)
#define GEN_SYS_TRX_SIGNAL_CLEAR     ((GenSyscallNr)0x0138)
#define GEN_SYS_TRX_EVENT_WAIT_MANY  ((GenSyscallNr)0x0139)

/* Subsystem 4: File system extensions (0x0140–0x014F) */
#define GEN_SYS_TRX_FS_MKDIR  ((GenSyscallNr)0x0147)
#define GEN_SYS_TRX_FS_UNLINK ((GenSyscallNr)0x0148)
#define GEN_SYS_TRX_FS_RENAME ((GenSyscallNr)0x0149)

/* Subsystem 5: Display / compositor (0x0150–0x015F) */
#define GEN_SYS_TRX_DISPLAY_ENUMERATE  ((GenSyscallNr)0x0150)
#define GEN_SYS_TRX_DISPLAY_SET_MODE   ((GenSyscallNr)0x0151)
#define GEN_SYS_TRX_COMPOSITOR_CREATE  ((GenSyscallNr)0x0152)
#define GEN_SYS_TRX_COMPOSITOR_PRESENT ((GenSyscallNr)0x0153)
#define GEN_SYS_TRX_SURFACE_CREATE     ((GenSyscallNr)0x0154)
#define GEN_SYS_TRX_SURFACE_DESTROY    ((GenSyscallNr)0x0155)
#define GEN_SYS_TRX_SURFACE_RESIZE     ((GenSyscallNr)0x0156)
#define GEN_SYS_TRX_BUFFER_CREATE      ((GenSyscallNr)0x0157)
#define GEN_SYS_TRX_BUFFER_MAP         ((GenSyscallNr)0x0158)
#define GEN_SYS_TRX_BUFFER_UNMAP       ((GenSyscallNr)0x0159)

/* Subsystem 6: Input devices (0x0160–0x016F) */
#define GEN_SYS_TRX_INPUT_ENUMERATE   ((GenSyscallNr)0x0160)
#define GEN_SYS_TRX_INPUT_OPEN        ((GenSyscallNr)0x0161)
#define GEN_SYS_TRX_INPUT_CLOSE       ((GenSyscallNr)0x0162)
#define GEN_SYS_TRX_INPUT_READ_EVENTS ((GenSyscallNr)0x0163)
#define GEN_SYS_TRX_INPUT_GRAB        ((GenSyscallNr)0x0164)
#define GEN_SYS_TRX_INPUT_UNGRAB      ((GenSyscallNr)0x0165)
#define GEN_SYS_TRX_INPUT_SET_KEYMAP  ((GenSyscallNr)0x0166)
#define GEN_SYS_TRX_TOUCH_READ_EVENTS ((GenSyscallNr)0x0167)
#define GEN_SYS_TRX_INPUT_SET_ACCEL   ((GenSyscallNr)0x0168)

/* Subsystem 7: GPU / DRM (0x0170–0x017F) */
#define GEN_SYS_TRX_GPU_OPEN          ((GenSyscallNr)0x0170)
#define GEN_SYS_TRX_GPU_CLOSE         ((GenSyscallNr)0x0171)
#define GEN_SYS_TRX_GPU_ALLOC_BO      ((GenSyscallNr)0x0172)
#define GEN_SYS_TRX_GPU_FREE_BO       ((GenSyscallNr)0x0173)
#define GEN_SYS_TRX_GPU_MAP_BO        ((GenSyscallNr)0x0174)
#define GEN_SYS_TRX_GPU_SUBMIT        ((GenSyscallNr)0x0175)
#define GEN_SYS_TRX_GPU_WAIT_FENCE    ((GenSyscallNr)0x0176)
#define GEN_SYS_TRX_GPU_EXPORT_DMABUF ((GenSyscallNr)0x0177)
#define GEN_SYS_TRX_GPU_IMPORT_DMABUF ((GenSyscallNr)0x0178)
#define GEN_SYS_TRX_GPU_GET_INFO      ((GenSyscallNr)0x0179)

/* Subsystem 8: Networking (0x0180–0x018F) */
#define GEN_SYS_TRX_NET_SOCKET  ((GenSyscallNr)0x0180)
#define GEN_SYS_TRX_NET_BIND    ((GenSyscallNr)0x0181)
#define GEN_SYS_TRX_NET_LISTEN  ((GenSyscallNr)0x0182)
#define GEN_SYS_TRX_NET_ACCEPT  ((GenSyscallNr)0x0183)
#define GEN_SYS_TRX_NET_CONNECT ((GenSyscallNr)0x0184)
#define GEN_SYS_TRX_NET_SENDMSG ((GenSyscallNr)0x0185)
#define GEN_SYS_TRX_NET_RECVMSG ((GenSyscallNr)0x0186)

/* Subsystem 9: Time / timers (0x0190–0x019F) */
#define GEN_SYS_TRX_TIMER_CREATE ((GenSyscallNr)0x0192)
#define GEN_SYS_TRX_TIMER_SET    ((GenSyscallNr)0x0193)

/* Subsystem 10: System / audit (0x01A0–0x01AF) */
#define GEN_SYS_TRX_SYSTEM_REBOOT   ((GenSyscallNr)0x01A0)
#define GEN_SYS_TRX_MODULE_LOAD     ((GenSyscallNr)0x01A1)
#define GEN_SYS_TRX_MODULE_UNLOAD   ((GenSyscallNr)0x01A2)
#define GEN_SYS_TRX_AUDIT_READ      ((GenSyscallNr)0x01A3)
#define GEN_SYS_TRX_AUDIT_SET_POLICY ((GenSyscallNr)0x01A4)
#define GEN_SYS_TRX_AUDIT_WRITE     ((GenSyscallNr)0x01A5)

/* Subsystem 11: Sigil / sandbox — legacy (0x01B0–0x01BF) */
#define GEN_SYS_TRX_SIGIL_SIGN      ((GenSyscallNr)0x01B0)
#define GEN_SYS_TRX_SIGIL_VERIFY    ((GenSyscallNr)0x01B1)
#define GEN_SYS_TRX_SANDBOX_CREATE  ((GenSyscallNr)0x01B2)
#define GEN_SYS_TRX_SANDBOX_ENTER   ((GenSyscallNr)0x01B3)

/* Deprecated aliases — removed in v0.2.0. Use GEN_SYS_TRX_* names. */
#define GEN_SYS_CAP_GRANT      GEN_SYS_TRX_PROCESS_CAP_GRANT
#define GEN_SYS_CAP_REVOKE     GEN_SYS_TRX_PROCESS_CAP_REVOKE
#define GEN_SYS_CAP_CHECK      GEN_SYS_TRX_PROCESS_CAP_QUERY
#define GEN_SYS_SIGIL_SIGN     GEN_SYS_TRX_SIGIL_SIGN
#define GEN_SYS_SIGIL_VERIFY   GEN_SYS_TRX_SIGIL_VERIFY
#define GEN_SYS_AUDIT_LOG      GEN_SYS_TRX_AUDIT_WRITE  /* was AUDIT_READ — semantic fix: LOG is a write op */
#define GEN_SYS_SANDBOX_CREATE GEN_SYS_TRX_SANDBOX_CREATE
#define GEN_SYS_SANDBOX_ENTER  GEN_SYS_TRX_SANDBOX_ENTER

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

/*
 * Returns the TerranoxOS subsystem index (0-15) for a TerranoxOS syscall.
 * Returns -1 if the syscall is not in the TerranoxOS range.
 */
static inline int gen_syscall_trx_subsystem(GenSyscallNr nr)
{
    if (!gen_syscall_is_terranox(nr)) return -1;
    return (int)((nr - GEN_SYSCALL_TERRANOX_BASE) >> 4);
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
