/*
 * gen_arch_x86_64.h — x86_64 CPU intrinsics: control registers,
 * MSRs, port I/O, interrupts, TLB, TSC, CPUID.
 *
 * All functions are static inline and require ring 0 execution.
 * Freestanding: requires only <stdint.h>.
 */

#ifndef GEN_ARCH_X86_64_H
#define GEN_ARCH_X86_64_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Control Registers ──────────────────────────────────── */

static inline uint64_t gen_read_cr0(void)
{
    uint64_t val;
    __asm__ __volatile__("mov %0, cr0" : "=r"(val) : : "memory");
    return val;
}

static inline void gen_write_cr0(uint64_t val)
{
    __asm__ __volatile__("mov cr0, %0" : : "r"(val) : "memory");
}

static inline uint64_t gen_read_cr2(void)
{
    uint64_t val;
    __asm__ __volatile__("mov %0, cr2" : "=r"(val) : : "memory");
    return val;
}

static inline uint64_t gen_read_cr3(void)
{
    uint64_t val;
    __asm__ __volatile__("mov %0, cr3" : "=r"(val) : : "memory");
    return val;
}

static inline void gen_write_cr3(uint64_t val)
{
    __asm__ __volatile__("mov cr3, %0" : : "r"(val) : "memory");
}

static inline uint64_t gen_read_cr4(void)
{
    uint64_t val;
    __asm__ __volatile__("mov %0, cr4" : "=r"(val) : : "memory");
    return val;
}

static inline void gen_write_cr4(uint64_t val)
{
    __asm__ __volatile__("mov cr4, %0" : : "r"(val) : "memory");
}

/* ── Model-Specific Registers ───────────────────────────── */

static inline uint64_t gen_read_msr(uint32_t msr)
{
    uint32_t lo, hi;
    __asm__ __volatile__("rdmsr" : "=a"(lo), "=d"(hi) : "c"(msr));
    return ((uint64_t)hi << 32) | (uint64_t)lo;
}

static inline void gen_write_msr(uint32_t msr, uint64_t val)
{
    uint32_t lo = (uint32_t)val;
    uint32_t hi = (uint32_t)(val >> 32);
    __asm__ __volatile__("wrmsr" : : "c"(msr), "a"(lo), "d"(hi));
}

/* ── Port I/O ───────────────────────────────────────────── */

static inline uint8_t gen_inb(uint16_t port)
{
    uint8_t val;
    __asm__ __volatile__("inb %1, %0" : "=a"(val) : "Nd"(port));
    return val;
}

static inline void gen_outb(uint16_t port, uint8_t val)
{
    __asm__ __volatile__("outb %0, %1" : : "a"(val), "Nd"(port));
}

static inline uint16_t gen_inw(uint16_t port)
{
    uint16_t val;
    __asm__ __volatile__("inw %1, %0" : "=a"(val) : "Nd"(port));
    return val;
}

static inline void gen_outw(uint16_t port, uint16_t val)
{
    __asm__ __volatile__("outw %0, %1" : : "a"(val), "Nd"(port));
}

static inline uint32_t gen_inl(uint16_t port)
{
    uint32_t val;
    __asm__ __volatile__("inl %1, %0" : "=a"(val) : "Nd"(port));
    return val;
}

static inline void gen_outl(uint16_t port, uint32_t val)
{
    __asm__ __volatile__("outl %0, %1" : : "a"(val), "Nd"(port));
}

/* ── Interrupt Control ──────────────────────────────────── */

static inline void gen_cli(void)
{
    __asm__ __volatile__("cli" : : : "memory");
}

static inline void gen_sti(void)
{
    __asm__ __volatile__("sti" : : : "memory");
}

static inline void gen_hlt(void)
{
    __asm__ __volatile__("hlt" : : : "memory");
}

static inline uint64_t gen_read_rflags(void)
{
    uint64_t val;
    __asm__ __volatile__("pushfq\n\tpop %0" : "=r"(val) : : "memory");
    return val;
}

static inline int gen_interrupts_enabled(void)
{
    return (gen_read_rflags() & (1ULL << 9)) != 0;
}

/* ── TLB ────────────────────────────────────────────────── */

static inline void gen_invlpg(uint64_t addr)
{
    __asm__ __volatile__("invlpg (%0)" : : "r"(addr) : "memory");
}

/* ── Misc ───────────────────────────────────────────────── */

static inline uint64_t gen_rdtsc(void)
{
    uint32_t lo, hi;
    __asm__ __volatile__("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | (uint64_t)lo;
}

static inline uint64_t gen_rdtscp(uint32_t *aux)
{
    uint32_t lo, hi, a;
    __asm__ __volatile__("rdtscp" : "=a"(lo), "=d"(hi), "=c"(a));
    if (aux) *aux = a;
    return ((uint64_t)hi << 32) | (uint64_t)lo;
}

/*
 * CPUID instruction. Writes results to caller-provided pointers.
 * rbx is callee-saved and must be preserved.
 */
static inline void gen_cpuid(uint32_t leaf, uint32_t subleaf,
                             uint32_t *eax, uint32_t *ebx,
                             uint32_t *ecx, uint32_t *edx)
{
    uint32_t a, b, c, d;
    __asm__ __volatile__(
        "cpuid"
        : "=a"(a), "=b"(b), "=c"(c), "=d"(d)
        : "a"(leaf), "c"(subleaf)
    );
    if (eax) *eax = a;
    if (ebx) *ebx = b;
    if (ecx) *ecx = c;
    if (edx) *edx = d;
}

#ifdef __cplusplus
}
#endif

#endif /* GEN_ARCH_X86_64_H */
