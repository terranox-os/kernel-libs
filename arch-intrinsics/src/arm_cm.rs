//! ARM Cortex-M intrinsics: PRIMASK, NVIC helpers, SysTick, WFI.
//!
//! Targets ARMv7-M and ARMv8-M (Cortex-M3/M4/M7/M33/M55).

use core::arch::asm;

// ── Interrupt Control ───────────────────────────────────────

/// Disable interrupts by setting PRIMASK.
///
/// # Safety
/// Caller must be executing in ring 0. Disabling interrupts without
/// re-enabling them will eventually hang the system.
#[inline]
pub unsafe fn disable_interrupts() {
    unsafe { asm!("cpsid i", options(nomem, nostack, preserves_flags)) };
}

/// Enable interrupts by clearing PRIMASK.
///
/// # Safety
/// Caller must be executing in ring 0. Disabling interrupts without
/// re-enabling them will eventually hang the system.
#[inline]
pub unsafe fn enable_interrupts() {
    unsafe { asm!("cpsie i", options(nomem, nostack, preserves_flags)) };
}

/// Read PRIMASK (0 = interrupts enabled, 1 = disabled).
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn read_primask() -> u32 {
    let val: u32;
    unsafe { asm!("mrs {}, PRIMASK", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Read BASEPRI register.
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn read_basepri() -> u32 {
    let val: u32;
    unsafe { asm!("mrs {}, BASEPRI", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Write BASEPRI register to mask interrupts below a priority.
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn write_basepri(val: u32) {
    unsafe { asm!("msr BASEPRI, {}", in(reg) val, options(nomem, nostack, preserves_flags)) };
}

/// Read FAULTMASK.
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn read_faultmask() -> u32 {
    let val: u32;
    unsafe { asm!("mrs {}, FAULTMASK", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Read CONTROL register.
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn read_control() -> u32 {
    let val: u32;
    unsafe { asm!("mrs {}, CONTROL", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Write CONTROL register.
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn write_control(val: u32) {
    unsafe {
        asm!("msr CONTROL, {}", in(reg) val, options(nomem, nostack, preserves_flags));
        asm!("isb", options(nomem, nostack, preserves_flags));
    };
}

// ── Stack Pointers ──────────────────────────────────────────

/// Read the Main Stack Pointer (MSP).
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn read_msp() -> u32 {
    let val: u32;
    unsafe { asm!("mrs {}, MSP", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Write the Main Stack Pointer (MSP).
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn write_msp(val: u32) {
    unsafe { asm!("msr MSP, {}", in(reg) val, options(nomem, nostack)) };
}

/// Read the Process Stack Pointer (PSP).
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn read_psp() -> u32 {
    let val: u32;
    unsafe { asm!("mrs {}, PSP", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Write the Process Stack Pointer (PSP).
///
/// # Safety
/// Caller must be executing in ring 0 (kernel mode). Invalid values
/// may cause a triple-fault or undefined CPU behavior.
#[inline]
pub unsafe fn write_psp(val: u32) {
    unsafe { asm!("msr PSP, {}", in(reg) val, options(nomem, nostack)) };
}

// ── Power / Wait ────────────────────────────────────────────

/// Wait for interrupt (low-power idle).
///
/// # Safety
/// Halts the CPU until an interrupt/event occurs. Must only be called
/// when interrupts are enabled or an event is expected.
#[inline]
pub unsafe fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) };
}

/// Wait for event.
///
/// # Safety
/// Halts the CPU until an interrupt/event occurs. Must only be called
/// when interrupts are enabled or an event is expected.
#[inline]
pub unsafe fn wfe() {
    unsafe { asm!("wfe", options(nomem, nostack, preserves_flags)) };
}

/// Send event.
///
/// # Safety
/// Halts the CPU until an interrupt/event occurs. Must only be called
/// when interrupts are enabled or an event is expected.
#[inline]
pub unsafe fn sev() {
    unsafe { asm!("sev", options(nomem, nostack, preserves_flags)) };
}

// ── Barriers ────────────────────────────────────────────────

/// Data Memory Barrier.
///
/// # Safety
/// Must be called from privileged mode. Barriers affect memory ordering
/// visible to all cores.
#[inline]
pub unsafe fn dmb() {
    unsafe { asm!("dmb sy", options(nostack, preserves_flags)) };
}

/// Data Synchronization Barrier.
///
/// # Safety
/// Must be called from privileged mode. Barriers affect memory ordering
/// visible to all cores.
#[inline]
pub unsafe fn dsb() {
    unsafe { asm!("dsb sy", options(nostack, preserves_flags)) };
}

/// Instruction Synchronization Barrier.
///
/// # Safety
/// Must be called from privileged mode. Barriers affect memory ordering
/// visible to all cores.
#[inline]
pub unsafe fn isb() {
    unsafe { asm!("isb", options(nostack, preserves_flags)) };
}

// ── Software Interrupts ─────────────────────────────────────

/// Trigger PendSV exception (used for context switching).
///
/// Writes to the ICSR register at 0xE000ED04.
///
/// # Safety
/// Writes to memory-mapped hardware registers. Must only be called
/// from handler mode or with appropriate privilege.
#[inline]
pub unsafe fn trigger_pendsv() {
    const ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
    const PENDSVSET: u32 = 1 << 28;
    unsafe { core::ptr::write_volatile(ICSR, PENDSVSET) };
}

/// Trigger SysTick reconfiguration via SYST_CSR.
///
/// SysTick Control and Status Register at 0xE000E010.
///
/// # Safety
/// Writes to memory-mapped hardware registers. Must only be called
/// from handler mode or with appropriate privilege.
#[inline]
pub unsafe fn systick_enable(reload: u32) {
    const SYST_RVR: *mut u32 = 0xE000_E014 as *mut u32;
    const SYST_CVR: *mut u32 = 0xE000_E018 as *mut u32;
    const SYST_CSR: *mut u32 = 0xE000_E010 as *mut u32;
    unsafe {
        core::ptr::write_volatile(SYST_RVR, reload & 0x00FF_FFFF);
        core::ptr::write_volatile(SYST_CVR, 0);
        // Enable SysTick with processor clock and interrupt
        core::ptr::write_volatile(SYST_CSR, 0x07);
    }
}
