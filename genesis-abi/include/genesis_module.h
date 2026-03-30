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
typedef void      (*GenModuleFiniFinFn)(void);

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
    GenModuleFiniFinFn fini;
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
