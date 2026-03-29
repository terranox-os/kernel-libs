/*
 * gen_arch_arm_cm.h — ARM Cortex-M intrinsics: PRIMASK, BASEPRI,
 * stack pointers, barriers, PendSV, SysTick.
 *
 * Targets ARMv7-M and ARMv8-M (Cortex-M3/M4/M7/M33/M55).
 * All functions are static inline.
 * Freestanding: requires only <stdint.h>.
 */

#ifndef GEN_ARCH_ARM_CM_H
#define GEN_ARCH_ARM_CM_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Interrupt Control ──────────────────────────────────── */

static inline void gen_disable_interrupts(void)
{
    __asm__ __volatile__("cpsid i" : : : "memory");
}

static inline void gen_enable_interrupts(void)
{
    __asm__ __volatile__("cpsie i" : : : "memory");
}

static inline uint32_t gen_read_primask(void)
{
    uint32_t val;
    __asm__ __volatile__("mrs %0, PRIMASK" : "=r"(val));
    return val;
}

static inline uint32_t gen_read_basepri(void)
{
    uint32_t val;
    __asm__ __volatile__("mrs %0, BASEPRI" : "=r"(val));
    return val;
}

static inline void gen_write_basepri(uint32_t val)
{
    __asm__ __volatile__("msr BASEPRI, %0" : : "r"(val) : "memory");
}

static inline uint32_t gen_read_faultmask(void)
{
    uint32_t val;
    __asm__ __volatile__("mrs %0, FAULTMASK" : "=r"(val));
    return val;
}

static inline uint32_t gen_read_control(void)
{
    uint32_t val;
    __asm__ __volatile__("mrs %0, CONTROL" : "=r"(val));
    return val;
}

static inline void gen_write_control(uint32_t val)
{
    __asm__ __volatile__("msr CONTROL, %0" : : "r"(val) : "memory");
    __asm__ __volatile__("isb" : : : "memory");
}

/* ── Stack Pointers ─────────────────────────────────────── */

static inline uint32_t gen_read_msp(void)
{
    uint32_t val;
    __asm__ __volatile__("mrs %0, MSP" : "=r"(val));
    return val;
}

static inline void gen_write_msp(uint32_t val)
{
    __asm__ __volatile__("msr MSP, %0" : : "r"(val));
}

static inline uint32_t gen_read_psp(void)
{
    uint32_t val;
    __asm__ __volatile__("mrs %0, PSP" : "=r"(val));
    return val;
}

static inline void gen_write_psp(uint32_t val)
{
    __asm__ __volatile__("msr PSP, %0" : : "r"(val));
}

/* ── Power / Wait ───────────────────────────────────────── */

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

/* ── Barriers ───────────────────────────────────────────── */

static inline void gen_dmb(void)
{
    __asm__ __volatile__("dmb sy" : : : "memory");
}

static inline void gen_dsb(void)
{
    __asm__ __volatile__("dsb sy" : : : "memory");
}

static inline void gen_isb(void)
{
    __asm__ __volatile__("isb" : : : "memory");
}

/* ── Software Interrupts ────────────────────────────────── */

#define GEN_ICSR_ADDR    ((volatile uint32_t *)0xE000ED04U)
#define GEN_PENDSVSET    (1U << 28)

static inline void gen_trigger_pendsv(void)
{
    *GEN_ICSR_ADDR = GEN_PENDSVSET;
}

#define GEN_SYST_RVR    ((volatile uint32_t *)0xE000E014U)
#define GEN_SYST_CVR    ((volatile uint32_t *)0xE000E018U)
#define GEN_SYST_CSR    ((volatile uint32_t *)0xE000E010U)

static inline void gen_systick_enable(uint32_t reload)
{
    *GEN_SYST_RVR = reload & 0x00FFFFFFU;
    *GEN_SYST_CVR = 0;
    *GEN_SYST_CSR = 0x07U; /* enable + tickint + clksource */
}

#ifdef __cplusplus
}
#endif

#endif /* GEN_ARCH_ARM_CM_H */
