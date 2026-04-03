/*
 * genesis_trx_types.h — TerranoxOS syscall interface data structures.
 *
 * These types are used as arguments and return values for TerranoxOS
 * syscalls defined in genesis_syscall.h. They are TerranoxOS-specific
 * and not required by GenesisOS-RT or HermeticaOS.
 *
 * Freestanding: requires only <stdint.h>, <stddef.h>.
 */

#ifndef GENESIS_TRX_TYPES_H
#define GENESIS_TRX_TYPES_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Capability token (128-bit handle) ──────────────────── */

/*
 * GenTrxCapToken: A transferable capability handle used at the
 * syscall interface. The kernel assigns `id` from the capability
 * DAG; `rights` is a bitmask of permitted operations.
 */
typedef struct GenTrxCapToken {
    uint64_t id;
    uint64_t rights;
} GenTrxCapToken;

/*
 * GenTrxCapTokenSet: Header for a variable-length array of
 * capability tokens. The `count` field indicates how many
 * GenTrxCapToken entries follow immediately in memory.
 *
 * Usage: allocate sizeof(GenTrxCapTokenSet) + count * sizeof(GenTrxCapToken).
 */
typedef struct GenTrxCapTokenSet {
    uint32_t count;
    uint32_t _pad0;
    /* GenTrxCapToken tokens[] follows in memory */
} GenTrxCapTokenSet;

/* ── Process info ───────────────────────────────────────── */

typedef struct GenTrxProcessInfo {
    int64_t  pid;
    int32_t  state;          /* RUNNING=0, SLEEPING=1, STOPPED=2, ZOMBIE=3 */
    uint32_t thread_count;
    uint64_t memory_bytes;
    uint64_t cpu_time_ns;
    uint32_t cap_count;
    uint32_t _pad0;
} GenTrxProcessInfo;

/* ── Display info ───────────────────────────────────────── */

typedef struct GenTrxDisplayInfo {
    uint32_t display_id;
    uint32_t width_px;
    uint32_t height_px;
    uint32_t refresh_mhz;    /* millihertz: 60000 = 60 Hz */
    uint32_t connector;      /* DRM connector type */
    char     name[32];
    uint32_t _pad0;
} GenTrxDisplayInfo;

/* ── Input event (libinput-compatible layout) ───────────── */
/*
 * ABI SYNC WARNING: This struct must match TrxInputEvent in
 * terranox-os/kernel/include/terranox/evdev.h (24 bytes).
 * Both are Linux struct input_event compatible.
 * Any change here requires updating the kernel copy and adding
 * a _Static_assert(sizeof(GenTrxInputEvent) == sizeof(TrxInputEvent))
 * in the kernel's compile_test. See issue #16.
 */

typedef struct GenTrxInputEvent {
    uint64_t timestamp_ns;
    uint32_t type;           /* EV_KEY, EV_REL, EV_ABS */
    uint32_t code;           /* KEY_A, REL_X, ABS_MT_POSITION_X */
    int32_t  value;
    uint32_t device_id;
} GenTrxInputEvent;

/* ── Touch event (multi-touch) ──────────────────────────── */

typedef struct GenTrxTouchEvent {
    uint64_t timestamp_ns;
    uint32_t slot;           /* finger tracking ID */
    uint32_t type;           /* TOUCH_DOWN=0, TOUCH_MOVE=1, TOUCH_UP=2 */
    int32_t  x;              /* surface-relative */
    int32_t  y;
    int32_t  pressure;       /* 0–65535 */
    uint32_t _pad0;
} GenTrxTouchEvent;

/* ── Wait item (for trx_event_wait_many) ────────────────── */

typedef struct GenTrxWaitItem {
    int64_t  handle;         /* channel, signal, or timer handle */
    uint32_t events;         /* WAIT_READABLE, WAIT_WRITABLE, WAIT_SIGNAL */
    uint32_t observed;       /* filled by kernel with triggered events */
} GenTrxWaitItem;

/* ── Timespec (POSIX-compatible) ────────────────────────── */

typedef struct GenTrxTimespec {
    int64_t tv_sec;
    int64_t tv_nsec;
} GenTrxTimespec;

/* ── GPU info ───────────────────────────────────────────── */

typedef struct GenTrxGpuInfo {
    uint32_t vendor_id;
    uint32_t device_id;
    uint64_t vram_bytes;
    uint32_t max_texture_size;
    uint32_t supported_formats; /* bitmask of pixel formats */
    char     driver_name[64];
} GenTrxGpuInfo;

/* ── Audit entry ────────────────────────────────────────── */

typedef struct GenTrxAuditEntry {
    uint64_t        timestamp_ns;
    int64_t         pid;
    int64_t         tid;
    GenTrxCapToken  capability;
    uint32_t        syscall_nr;
    uint32_t        result;  /* GRANTED=0, DENIED=1 */
} GenTrxAuditEntry;

#ifdef __cplusplus
}
#endif

#endif /* GENESIS_TRX_TYPES_H */
