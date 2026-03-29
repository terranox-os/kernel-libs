/*
 * gen_arch_aarch64.h — AArch64 CPU intrinsics: system registers,
 * barriers, TLB/cache maintenance, interrupt control, WFI/WFE.
 *
 * All functions are static inline and require EL1 or higher.
 * Freestanding: requires only <stdint.h>.
 */

#ifndef GEN_ARCH_AARCH64_H
#define GEN_ARCH_AARCH64_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── System Register Macros ─────────────────────────────── */

#define GEN_READ_SYSREG(name, reg)                              \
    static inline uint64_t gen_read_##name(void)                \
    {                                                           \
        uint64_t val;                                           \
        __asm__ __volatile__("mrs %0, " #reg : "=r"(val));     \
        return val;                                             \
    }

#define GEN_WRITE_SYSREG(name, reg)                             \
    static inline void gen_write_##name(uint64_t val)           \
    {                                                           \
        __asm__ __volatile__("msr " #reg ", %0" : : "r"(val)); \
    }

GEN_READ_SYSREG(sctlr_el1, sctlr_el1)
GEN_WRITE_SYSREG(sctlr_el1, sctlr_el1)

GEN_READ_SYSREG(ttbr0_el1, ttbr0_el1)
GEN_WRITE_SYSREG(ttbr0_el1, ttbr0_el1)

GEN_READ_SYSREG(ttbr1_el1, ttbr1_el1)
GEN_WRITE_SYSREG(ttbr1_el1, ttbr1_el1)

GEN_READ_SYSREG(tcr_el1, tcr_el1)
GEN_WRITE_SYSREG(tcr_el1, tcr_el1)

GEN_READ_SYSREG(mair_el1, mair_el1)
GEN_WRITE_SYSREG(mair_el1, mair_el1)

GEN_READ_SYSREG(vbar_el1, vbar_el1)
GEN_WRITE_SYSREG(vbar_el1, vbar_el1)

GEN_READ_SYSREG(esr_el1, esr_el1)
GEN_READ_SYSREG(far_el1, far_el1)
GEN_READ_SYSREG(elr_el1, elr_el1)
GEN_READ_SYSREG(spsr_el1, spsr_el1)

GEN_READ_SYSREG(mpidr_el1, mpidr_el1)
GEN_READ_SYSREG(cntfrq_el0, cntfrq_el0)
GEN_READ_SYSREG(cntvct_el0, cntvct_el0)
GEN_READ_SYSREG(cntpct_el0, cntpct_el0)

GEN_READ_SYSREG(daif, daif)
GEN_WRITE_SYSREG(daif, daif)

/* ── Barriers ───────────────────────────────────────────── */

static inline void gen_dmb_sy(void)
{
    __asm__ __volatile__("dmb sy" : : : "memory");
}

static inline void gen_dmb_ish(void)
{
    __asm__ __volatile__("dmb ish" : : : "memory");
}

static inline void gen_dsb_sy(void)
{
    __asm__ __volatile__("dsb sy" : : : "memory");
}

static inline void gen_dsb_ish(void)
{
    __asm__ __volatile__("dsb ish" : : : "memory");
}

static inline void gen_isb(void)
{
    __asm__ __volatile__("isb" : : : "memory");
}

/* ── TLB Maintenance ────────────────────────────────────── */

static inline void gen_tlbi_vmalle1is(void)
{
    __asm__ __volatile__("tlbi vmalle1is" : : : "memory");
    __asm__ __volatile__("dsb ish" : : : "memory");
    __asm__ __volatile__("isb" : : : "memory");
}

/* ── Cache Maintenance ──────────────────────────────────── */

static inline void gen_dc_civac(uint64_t addr)
{
    __asm__ __volatile__("dc civac, %0" : : "r"(addr) : "memory");
}

static inline void gen_ic_iallu(void)
{
    __asm__ __volatile__("ic iallu" : : : "memory");
}

/* ── Wait / Interrupt Control ───────────────────────────── */

static inline void gen_wfi(void)
{
    __asm__ __volatile__("wfi" : : : "memory");
}

static inline void gen_wfe(void)
{
    __asm__ __volatile__("wfe" : : : "memory");
}

static inline void gen_sev(void)
{
    __asm__ __volatile__("sev" : : : "memory");
}

static inline void gen_disable_irq(void)
{
    __asm__ __volatile__("msr daifset, #2" : : : "memory");
}

static inline void gen_enable_irq(void)
{
    __asm__ __volatile__("msr daifclr, #2" : : : "memory");
}

static inline void gen_disable_all_exceptions(void)
{
    __asm__ __volatile__("msr daifset, #15" : : : "memory");
}

static inline void gen_enable_all_exceptions(void)
{
    __asm__ __volatile__("msr daifclr, #15" : : : "memory");
}

#ifdef __cplusplus
}
#endif

#endif /* GEN_ARCH_AARCH64_H */
