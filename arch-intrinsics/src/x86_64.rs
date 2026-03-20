//! x86_64 CPU intrinsics: control registers, MSRs, port I/O, interrupts.

use core::arch::asm;

// ── Control Registers ───────────────────────────────────────

/// Read the CR0 register (machine status word).
#[inline]
pub unsafe fn read_cr0() -> u64 {
    let val: u64;
    // Safety: caller must be in ring 0.
    unsafe { asm!("mov {}, cr0", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Write the CR0 register.
#[inline]
pub unsafe fn write_cr0(val: u64) {
    // Safety: caller must be in ring 0; invalid values may triple-fault.
    unsafe { asm!("mov cr0, {}", in(reg) val, options(nomem, nostack, preserves_flags)) };
}

/// Read CR2 (page fault linear address).
#[inline]
pub unsafe fn read_cr2() -> u64 {
    let val: u64;
    unsafe { asm!("mov {}, cr2", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Read CR3 (page table base register).
#[inline]
pub unsafe fn read_cr3() -> u64 {
    let val: u64;
    unsafe { asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Write CR3 (switches page tables, flushes TLB).
#[inline]
pub unsafe fn write_cr3(val: u64) {
    unsafe { asm!("mov cr3, {}", in(reg) val, options(nomem, nostack, preserves_flags)) };
}

/// Read CR4 (extended machine control).
#[inline]
pub unsafe fn read_cr4() -> u64 {
    let val: u64;
    unsafe { asm!("mov {}, cr4", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Write CR4.
#[inline]
pub unsafe fn write_cr4(val: u64) {
    unsafe { asm!("mov cr4, {}", in(reg) val, options(nomem, nostack, preserves_flags)) };
}

// ── Model-Specific Registers ────────────────────────────────

/// Read a Model-Specific Register by index.
#[inline]
pub unsafe fn read_msr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack, preserves_flags),
        )
    };
    ((hi as u64) << 32) | (lo as u64)
}

/// Write a Model-Specific Register by index.
#[inline]
pub unsafe fn write_msr(msr: u32, val: u64) {
    let lo = val as u32;
    let hi = (val >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") lo,
            in("edx") hi,
            options(nomem, nostack, preserves_flags),
        )
    };
}

// ── Port I/O ────────────────────────────────────────────────

/// Read a byte from an I/O port.
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") val,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        )
    };
    val
}

/// Write a byte to an I/O port.
#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack, preserves_flags),
        )
    };
}

/// Read a 16-bit word from an I/O port.
#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    unsafe {
        asm!(
            "in ax, dx",
            out("ax") val,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        )
    };
    val
}

/// Write a 16-bit word to an I/O port.
#[inline]
pub unsafe fn outw(port: u16, val: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") val,
            options(nomem, nostack, preserves_flags),
        )
    };
}

/// Read a 32-bit dword from an I/O port.
#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    unsafe {
        asm!(
            "in eax, dx",
            out("eax") val,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        )
    };
    val
}

/// Write a 32-bit dword to an I/O port.
#[inline]
pub unsafe fn outl(port: u16, val: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") val,
            options(nomem, nostack, preserves_flags),
        )
    };
}

// ── Interrupt Control ───────────────────────────────────────

/// Disable interrupts (clear IF in RFLAGS).
#[inline]
pub unsafe fn cli() {
    unsafe { asm!("cli", options(nomem, nostack)) };
}

/// Enable interrupts (set IF in RFLAGS).
#[inline]
pub unsafe fn sti() {
    unsafe { asm!("sti", options(nomem, nostack)) };
}

/// Halt the CPU until the next interrupt.
#[inline]
pub unsafe fn hlt() {
    unsafe { asm!("hlt", options(nomem, nostack)) };
}

/// Read RFLAGS register.
#[inline]
pub unsafe fn read_rflags() -> u64 {
    let val: u64;
    unsafe {
        asm!(
            "pushfq",
            "pop {}",
            out(reg) val,
            options(nomem, preserves_flags),
        )
    };
    val
}

/// Check if interrupts are enabled (IF flag in RFLAGS).
#[inline]
pub unsafe fn interrupts_enabled() -> bool {
    // Safety: reads RFLAGS
    (unsafe { read_rflags() } & (1 << 9)) != 0
}

// ── TLB ─────────────────────────────────────────────────────

/// Invalidate a single TLB entry for the given virtual address.
#[inline]
pub unsafe fn invlpg(addr: u64) {
    unsafe { asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags)) };
}

// ── Misc ────────────────────────────────────────────────────

/// Read the Time Stamp Counter.
#[inline]
pub unsafe fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack, preserves_flags),
        )
    };
    ((hi as u64) << 32) | (lo as u64)
}

/// Serializing read of the Time Stamp Counter (RDTSCP).
/// Also returns the auxiliary MSR value (typically processor ID).
#[inline]
pub unsafe fn rdtscp() -> (u64, u32) {
    let lo: u32;
    let hi: u32;
    let aux: u32;
    unsafe {
        asm!(
            "rdtscp",
            out("eax") lo,
            out("edx") hi,
            out("ecx") aux,
            options(nomem, nostack, preserves_flags),
        )
    };
    (((hi as u64) << 32) | (lo as u64), aux)
}

/// CPUID instruction. Returns (eax, ebx, ecx, edx).
///
/// rbx is reserved by LLVM, so we save/restore it manually.
#[inline]
pub unsafe fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;
    unsafe {
        asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_out:e}, ebx",
            "pop rbx",
            inout("eax") leaf => eax,
            inout("ecx") subleaf => ecx,
            ebx_out = out(reg) ebx,
            out("edx") edx,
            options(nostack, preserves_flags),
        )
    };
    (eax, ebx, ecx, edx)
}
