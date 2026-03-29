/*
 * gen_arch_riscv64.h — RISC-V 64-bit intrinsics: CSR access
 * (M-mode and S-mode), fence, sfence.vma, WFI.
 *
 * All functions are static inline.
 * Freestanding: requires only <stdint.h>.
 */

#ifndef GEN_ARCH_RISCV64_H
#define GEN_ARCH_RISCV64_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── CSR Access Macros ──────────────────────────────────── */

#define GEN_READ_CSR(name, csr)                                 \
    static inline uint64_t gen_read_##name(void)                \
    {                                                           \
        uint64_t val;                                           \
        __asm__ __volatile__("csrr %0, " #csr : "=r"(val));    \
        return val;                                             \
    }

#define GEN_WRITE_CSR(name, csr)                                \
    static inline void gen_write_##name(uint64_t val)           \
    {                                                           \
        __asm__ __volatile__("csrw " #csr ", %0" : : "r"(val));\
    }

#define GEN_SET_CSR(name, csr)                                  \
    static inline void gen_set_##name(uint64_t bits)            \
    {                                                           \
        __asm__ __volatile__("csrs " #csr ", %0" : : "r"(bits));\
    }

#define GEN_CLEAR_CSR(name, csr)                                \
    static inline void gen_clear_##name(uint64_t bits)          \
    {                                                           \
        __asm__ __volatile__("csrc " #csr ", %0" : : "r"(bits));\
    }

/* ── Machine-level CSRs ─────────────────────────────────── */

GEN_READ_CSR(mstatus, mstatus)
GEN_WRITE_CSR(mstatus, mstatus)
GEN_SET_CSR(mstatus, mstatus)
GEN_CLEAR_CSR(mstatus, mstatus)

GEN_READ_CSR(mtvec, mtvec)
GEN_WRITE_CSR(mtvec, mtvec)

GEN_READ_CSR(mepc, mepc)
GEN_WRITE_CSR(mepc, mepc)

GEN_READ_CSR(mcause, mcause)
GEN_READ_CSR(mtval, mtval)
GEN_READ_CSR(mip, mip)

GEN_READ_CSR(mie, mie)
GEN_WRITE_CSR(mie, mie)
GEN_SET_CSR(mie, mie)
GEN_CLEAR_CSR(mie, mie)

GEN_READ_CSR(mhartid, mhartid)

/* ── Supervisor-level CSRs ──────────────────────────────── */

GEN_READ_CSR(sstatus, sstatus)
GEN_WRITE_CSR(sstatus, sstatus)

GEN_READ_CSR(stvec, stvec)
GEN_WRITE_CSR(stvec, stvec)

GEN_READ_CSR(sepc, sepc)
GEN_WRITE_CSR(sepc, sepc)

GEN_READ_CSR(scause, scause)
GEN_READ_CSR(stval, stval)

GEN_READ_CSR(satp, satp)
GEN_WRITE_CSR(satp, satp)

GEN_READ_CSR(sie, sie)
GEN_WRITE_CSR(sie, sie)
GEN_SET_CSR(sie, sie)
GEN_CLEAR_CSR(sie, sie)

GEN_READ_CSR(sip, sip)

/* ── Time CSRs (read-only in U/S mode) ──────────────────── */

GEN_READ_CSR(time, time)
GEN_READ_CSR(cycle, cycle)

/* ── Fence / Barriers ───────────────────────────────────── */

static inline void gen_fence(void)
{
    __asm__ __volatile__("fence iorw, iorw" : : : "memory");
}

static inline void gen_fence_i(void)
{
    __asm__ __volatile__("fence.i" : : : "memory");
}

/* ── Wait / Power ───────────────────────────────────────── */

static inline void gen_wfi(void)
{
    __asm__ __volatile__("wfi" : : : "memory");
}

/* ── TLB ────────────────────────────────────────────────── */

static inline void gen_sfence_vma_all(void)
{
    __asm__ __volatile__("sfence.vma zero, zero" : : : "memory");
}

static inline void gen_sfence_vma(uint64_t vaddr)
{
    __asm__ __volatile__("sfence.vma %0, zero" : : "r"(vaddr) : "memory");
}

#ifdef __cplusplus
}
#endif

#endif /* GEN_ARCH_RISCV64_H */
