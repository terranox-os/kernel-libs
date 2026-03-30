//! AArch64 intrinsics: system registers, barriers, cache, WFI.

use core::arch::asm;

// ── System Registers ────────────────────────────────────────

/// Read a system register via MRS.
macro_rules! read_sysreg {
    ($name:ident, $reg:literal) => {
        /// # Safety
        /// Caller must be executing in ring 0 (kernel mode). Invalid values
        /// may cause a triple-fault or undefined CPU behavior.
        #[inline]
        pub unsafe fn $name() -> u64 {
            let val: u64;
            unsafe { asm!(concat!("mrs {}, ", $reg), out(reg) val, options(nomem, nostack, preserves_flags)) };
            val
        }
    };
}

/// Write a system register via MSR.
macro_rules! write_sysreg {
    ($name:ident, $reg:literal) => {
        /// # Safety
        /// Caller must be executing in ring 0 (kernel mode). Invalid values
        /// may cause a triple-fault or undefined CPU behavior.
        #[inline]
        pub unsafe fn $name(val: u64) {
            unsafe { asm!(concat!("msr ", $reg, ", {}"), in(reg) val, options(nomem, nostack, preserves_flags)) };
        }
    };
}

read_sysreg!(read_sctlr_el1, "sctlr_el1");
write_sysreg!(write_sctlr_el1, "sctlr_el1");

read_sysreg!(read_ttbr0_el1, "ttbr0_el1");
write_sysreg!(write_ttbr0_el1, "ttbr0_el1");

read_sysreg!(read_ttbr1_el1, "ttbr1_el1");
write_sysreg!(write_ttbr1_el1, "ttbr1_el1");

read_sysreg!(read_tcr_el1, "tcr_el1");
write_sysreg!(write_tcr_el1, "tcr_el1");

read_sysreg!(read_mair_el1, "mair_el1");
write_sysreg!(write_mair_el1, "mair_el1");

read_sysreg!(read_vbar_el1, "vbar_el1");
write_sysreg!(write_vbar_el1, "vbar_el1");

read_sysreg!(read_esr_el1, "esr_el1");
read_sysreg!(read_far_el1, "far_el1");
read_sysreg!(read_elr_el1, "elr_el1");
read_sysreg!(read_spsr_el1, "spsr_el1");

read_sysreg!(read_mpidr_el1, "mpidr_el1");
read_sysreg!(read_cntfrq_el0, "cntfrq_el0");
read_sysreg!(read_cntvct_el0, "cntvct_el0");
read_sysreg!(read_cntpct_el0, "cntpct_el0");

read_sysreg!(read_daif, "daif");
write_sysreg!(write_daif, "daif");

// ── Barriers ────────────────────────────────────────────────

/// Data Memory Barrier (full system).
///
/// # Safety
/// Must be called from privileged mode. Barriers affect memory ordering
/// visible to all cores.
#[inline]
pub unsafe fn dmb_sy() {
    unsafe { asm!("dmb sy", options(nostack, preserves_flags)) };
}

/// Data Memory Barrier (inner shareable).
///
/// # Safety
/// Must be called from privileged mode. Barriers affect memory ordering
/// visible to all cores.
#[inline]
pub unsafe fn dmb_ish() {
    unsafe { asm!("dmb ish", options(nostack, preserves_flags)) };
}

/// Data Synchronization Barrier (full system).
///
/// # Safety
/// Must be called from privileged mode. Barriers affect memory ordering
/// visible to all cores.
#[inline]
pub unsafe fn dsb_sy() {
    unsafe { asm!("dsb sy", options(nostack, preserves_flags)) };
}

/// Data Synchronization Barrier (inner shareable).
///
/// # Safety
/// Must be called from privileged mode. Barriers affect memory ordering
/// visible to all cores.
#[inline]
pub unsafe fn dsb_ish() {
    unsafe { asm!("dsb ish", options(nostack, preserves_flags)) };
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

// ── TLB Maintenance ─────────────────────────────────────────

/// Invalidate all TLB entries (inner shareable).
///
/// # Safety
/// Caller must be in kernel mode. Incorrect TLB/cache invalidation
/// can cause stale mappings or data corruption.
#[inline]
pub unsafe fn tlbi_vmalle1is() {
    unsafe {
        asm!("tlbi vmalle1is", options(nomem, nostack, preserves_flags));
        asm!("dsb ish", options(nostack, preserves_flags));
        asm!("isb", options(nostack, preserves_flags));
    };
}

// ── Cache Maintenance ───────────────────────────────────────

/// Clean and invalidate data cache line by virtual address.
///
/// # Safety
/// Caller must be in kernel mode. Incorrect TLB/cache invalidation
/// can cause stale mappings or data corruption.
#[inline]
pub unsafe fn dc_civac(addr: u64) {
    unsafe { asm!("dc civac, {}", in(reg) addr, options(nostack, preserves_flags)) };
}

/// Invalidate instruction cache (all).
///
/// # Safety
/// Caller must be in kernel mode. Incorrect TLB/cache invalidation
/// can cause stale mappings or data corruption.
#[inline]
pub unsafe fn ic_iallu() {
    unsafe { asm!("ic iallu", options(nomem, nostack, preserves_flags)) };
}

// ── Wait / Interrupt Control ────────────────────────────────

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

/// Send event to all PEs.
///
/// # Safety
/// Halts the CPU until an interrupt/event occurs. Must only be called
/// when interrupts are enabled or an event is expected.
#[inline]
pub unsafe fn sev() {
    unsafe { asm!("sev", options(nomem, nostack, preserves_flags)) };
}

/// Disable IRQs (set DAIF.I).
///
/// # Safety
/// Caller must be executing in ring 0. Disabling interrupts without
/// re-enabling them will eventually hang the system.
#[inline]
pub unsafe fn disable_irq() {
    unsafe { asm!("msr daifset, #2", options(nomem, nostack, preserves_flags)) };
}

/// Enable IRQs (clear DAIF.I).
///
/// # Safety
/// Caller must be executing in ring 0. Disabling interrupts without
/// re-enabling them will eventually hang the system.
#[inline]
pub unsafe fn enable_irq() {
    unsafe { asm!("msr daifclr, #2", options(nomem, nostack, preserves_flags)) };
}

/// Disable all exceptions (IRQ, FIQ, SError, Debug).
///
/// # Safety
/// Caller must be executing in ring 0. Disabling interrupts without
/// re-enabling them will eventually hang the system.
#[inline]
pub unsafe fn disable_all_exceptions() {
    unsafe { asm!("msr daifset, #15", options(nomem, nostack, preserves_flags)) };
}

/// Enable all exceptions.
///
/// # Safety
/// Caller must be executing in ring 0. Disabling interrupts without
/// re-enabling them will eventually hang the system.
#[inline]
pub unsafe fn enable_all_exceptions() {
    unsafe { asm!("msr daifclr, #15", options(nomem, nostack, preserves_flags)) };
}
