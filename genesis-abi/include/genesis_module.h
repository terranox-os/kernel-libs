/*
 * genesis_module.h — Module ABI types for HermeticaOS.
 *
 * Defines GenCapability, GenKernelAPI, and GenModuleDescriptor.
 * TerranoxOS and GenesisOS-RT include this header but may ignore
 * module-specific types unless they adopt module loading.
 *
 * Freestanding: requires only <stdint.h>, <stddef.h>, genesis_result.h.
 */

#ifndef GENESIS_MODULE_H
#define GENESIS_MODULE_H

#include <stdint.h>
#include <stddef.h>
#include "genesis_result.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ── ABI version ─────────────────────────────────────────── */

#define GEN_MODULE_ABI_VERSION_MAJOR 0
#define GEN_MODULE_ABI_VERSION_MINOR 1

/* ── Capabilities ────────────────────────────────────────── */

/*
 * GenCapability: Bitfield of permissions a module can request.
 * The kernel grants a subset based on policy. Modules operate
 * only within their granted capability set.
 */
typedef uint64_t GenCapability;

#define GEN_CAP_NONE           ((GenCapability)0)
#define GEN_CAP_MEM_READ       ((GenCapability)(1ULL <<  0))
#define GEN_CAP_MEM_WRITE      ((GenCapability)(1ULL <<  1))
#define GEN_CAP_MEM_EXEC       ((GenCapability)(1ULL <<  2))
#define GEN_CAP_IO_PORT        ((GenCapability)(1ULL <<  3))
#define GEN_CAP_IRQ_REGISTER   ((GenCapability)(1ULL <<  4))
#define GEN_CAP_DMA            ((GenCapability)(1ULL <<  5))
#define GEN_CAP_TIMER          ((GenCapability)(1ULL <<  6))
#define GEN_CAP_NET            ((GenCapability)(1ULL <<  7))
#define GEN_CAP_BLOCK          ((GenCapability)(1ULL <<  8))
#define GEN_CAP_FS             ((GenCapability)(1ULL <<  9))
#define GEN_CAP_PROCESS_CREATE ((GenCapability)(1ULL << 10))
#define GEN_CAP_PROCESS_SIGNAL ((GenCapability)(1ULL << 11))
#define GEN_CAP_IPC            ((GenCapability)(1ULL << 12))
#define GEN_CAP_AUDIT          ((GenCapability)(1ULL << 13))
#define GEN_CAP_MODULE_LOAD    ((GenCapability)(1ULL << 14))
#define GEN_CAP_CRYPTO         ((GenCapability)(1ULL << 15))
/* Bits 16–63 reserved */

#define GEN_CAP_ALL            ((GenCapability)0xFFFFULL)

static inline int gen_cap_contains(GenCapability set, GenCapability cap)
{
    return (set & cap) == cap;
}

/* ── TrxCapSet: 128-bit hierarchical capabilities ───────── */

/*
 * TrxCapSet: 128-bit domain-partitioned capability bitmask.
 * 12 domains, 40 leaf capabilities. Hierarchy resolved at compile time.
 *
 * lo word (bits 0-63):
 *   0-3   process: create, signal, inspect, manage
 *   4-7   memory:  alloc, map, share, dma
 *   8-10  thread:  create, join, affinity
 *   11-13 ipc:     channel, signal, event
 *   14-17 fs:      read, write, create, delete
 *   18-20 io:      port, irq, mmio
 *   21-24 display: compositor, surface, buffer, mode
 *   25-27 input:   keyboard, pointer, touch
 *
 * hi word (bits 0-63):
 *   0-2   gpu:     render, compute, alloc
 *   3-5   net:     socket, bind, raw
 *   6-8   time:    read, sleep, timer
 *   9-11  system:  reboot, module, audit
 */
typedef struct TrxCapSet {
    uint64_t lo;
    uint64_t hi;
} TrxCapSet;

/* Domain parent constants (lo word) */
#define TRX_CAP_PROCESS  ((TrxCapSet){ .lo = 0xFULL,         .hi = 0 })
#define TRX_CAP_MEMORY   ((TrxCapSet){ .lo = 0xF0ULL,        .hi = 0 })
#define TRX_CAP_THREAD   ((TrxCapSet){ .lo = 0x700ULL,       .hi = 0 })
#define TRX_CAP_IPC      ((TrxCapSet){ .lo = 0x3800ULL,      .hi = 0 })
#define TRX_CAP_FS       ((TrxCapSet){ .lo = 0x3C000ULL,     .hi = 0 })
#define TRX_CAP_IO       ((TrxCapSet){ .lo = 0x1C0000ULL,    .hi = 0 })
#define TRX_CAP_DISPLAY  ((TrxCapSet){ .lo = 0x1E00000ULL,   .hi = 0 })
#define TRX_CAP_INPUT    ((TrxCapSet){ .lo = 0xE000000ULL,   .hi = 0 })

/* Domain parent constants (hi word) */
#define TRX_CAP_GPU      ((TrxCapSet){ .lo = 0, .hi = 0x7ULL })
#define TRX_CAP_NET      ((TrxCapSet){ .lo = 0, .hi = 0x38ULL })
#define TRX_CAP_TIME     ((TrxCapSet){ .lo = 0, .hi = 0x1C0ULL })
#define TRX_CAP_SYSTEM   ((TrxCapSet){ .lo = 0, .hi = 0xE00ULL })

#define TRX_CAP_NONE     ((TrxCapSet){ .lo = 0, .hi = 0 })
#define TRX_CAP_ROOT     ((TrxCapSet){ .lo = 0xFFFFFFFULL, .hi = 0xFFFULL })

static inline int trx_cap_contains(TrxCapSet set, TrxCapSet cap)
{
    return (set.lo & cap.lo) == cap.lo && (set.hi & cap.hi) == cap.hi;
}

static inline TrxCapSet trx_cap_union(TrxCapSet a, TrxCapSet b)
{
    return (TrxCapSet){ .lo = a.lo | b.lo, .hi = a.hi | b.hi };
}

static inline TrxCapSet trx_cap_intersection(TrxCapSet a, TrxCapSet b)
{
    return (TrxCapSet){ .lo = a.lo & b.lo, .hi = a.hi & b.hi };
}

static inline TrxCapSet trx_cap_difference(TrxCapSet a, TrxCapSet b)
{
    return (TrxCapSet){ .lo = a.lo & ~b.lo, .hi = a.hi & ~b.hi };
}

/* ── Kernel API table ────────────────────────────────────── */

/*
 * GenKernelAPI: Function pointer table provided by the host kernel
 * to loaded modules. Each pointer may be NULL if the kernel does
 * not support that operation. Modules must check before calling.
 */
typedef struct GenKernelAPI {
    /* Memory management */
    void *(*alloc_pages)(size_t count);
    void  (*free_pages)(void *addr, size_t count);

    /* Logging */
    GenResult (*log)(uint8_t level, const char *msg, size_t len);

    /* IPC */
    GenResult (*ipc_send)(uint32_t target_module_id,
                          const void *data, size_t len);
    GenResult (*ipc_recv)(uint32_t *source_module_id,
                          void *buf, size_t buf_len, size_t *out_len);

    /* Interrupt registration */
    GenResult (*irq_register)(uint32_t irq_num,
                              void (*handler)(uint32_t irq_num, void *ctx),
                              void *ctx);
    GenResult (*irq_unregister)(uint32_t irq_num);

    /* Timer */
    GenResult (*timer_create)(uint64_t interval_ns,
                              void (*callback)(void *ctx), void *ctx,
                              uint32_t *out_timer_id);
    GenResult (*timer_cancel)(uint32_t timer_id);

    /* Capability queries */
    GenResult (*query_capability)(GenCapability cap, int *out_granted);
} GenKernelAPI;

/* ── Module descriptor ───────────────────────────────────── */

#define GEN_MODULE_NAME_MAX 64
#define GEN_MODULE_MAGIC    0x47454E4DU  /* "GENM" in little-endian ASCII */
#define GEN_MODULE_SECTION  ".gen_module"

typedef GenResult (*GenModuleInitFn)(const GenKernelAPI *api);
typedef void      (*GenModuleFiniFn)(void);

/*
 * GenModuleDescriptor: Placed in a well-known ELF section by modules.
 * The kernel reads this to determine module identity, capabilities,
 * and entry points.
 *
 * Fields ordered for natural alignment on both 32-bit and 64-bit.
 */
typedef struct GenModuleDescriptor {
    uint32_t          magic;
    uint16_t          abi_version_major;
    uint16_t          abi_version_minor;

    char              name[GEN_MODULE_NAME_MAX];
    uint32_t          module_version_major;
    uint32_t          module_version_minor;
    uint32_t          module_version_patch;
    uint32_t          _pad0;

    GenCapability     required_caps;
    GenCapability     optional_caps;

    GenModuleInitFn   init;
    GenModuleFiniFn fini;
} GenModuleDescriptor;

/*
 * GEN_DECLARE_MODULE: Convenience macro for module authors.
 *
 * Places a GenModuleDescriptor in the ".gen_module" ELF section.
 *
 * Usage:
 *   static GenResult my_init(const GenKernelAPI *api) { ... }
 *   static void my_fini(void) { ... }
 *   GEN_DECLARE_MODULE("my_driver", 1, 0, 0,
 *                      GEN_CAP_IRQ_REGISTER | GEN_CAP_MEM_READ,
 *                      GEN_CAP_DMA,
 *                      my_init, my_fini);
 */
#define GEN_DECLARE_MODULE(mod_name, vmaj, vmin, vpatch,         \
                           req_caps, opt_caps, init_fn, fini_fn) \
    __attribute__((section(GEN_MODULE_SECTION), used))           \
    static const GenModuleDescriptor _gen_module_desc = {        \
        .magic              = GEN_MODULE_MAGIC,                  \
        .abi_version_major  = GEN_MODULE_ABI_VERSION_MAJOR,      \
        .abi_version_minor  = GEN_MODULE_ABI_VERSION_MINOR,      \
        .name               = mod_name,                          \
        .module_version_major = (vmaj),                          \
        .module_version_minor = (vmin),                          \
        .module_version_patch = (vpatch),                        \
        ._pad0              = 0,                                 \
        .required_caps      = (req_caps),                        \
        .optional_caps      = (opt_caps),                        \
        .init               = (init_fn),                         \
        .fini               = (fini_fn),                         \
    }

#ifdef __cplusplus
}
#endif

#endif /* GENESIS_MODULE_H */
