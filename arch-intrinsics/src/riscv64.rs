//! RISC-V 64-bit intrinsics: CSR access, WFI, fence.

use core::arch::asm;

// ── CSR Read/Write ──────────────────────────────────────────

/// Read a CSR by name.
macro_rules! read_csr {
    ($name:ident, $csr:literal) => {
        #[inline]
        pub unsafe fn $name() -> u64 {
            let val: u64;
            unsafe { asm!(concat!("csrr {}, ", $csr), out(reg) val, options(nomem, nostack, preserves_flags)) };
            val
        }
    };
}

/// Write a CSR by name.
macro_rules! write_csr {
    ($name:ident, $csr:literal) => {
        #[inline]
        pub unsafe fn $name(val: u64) {
            unsafe { asm!(concat!("csrw ", $csr, ", {}"), in(reg) val, options(nomem, nostack, preserves_flags)) };
        }
    };
}

/// Set bits in a CSR.
macro_rules! set_csr {
    ($name:ident, $csr:literal) => {
        #[inline]
        pub unsafe fn $name(bits: u64) {
            unsafe { asm!(concat!("csrs ", $csr, ", {}"), in(reg) bits, options(nomem, nostack, preserves_flags)) };
        }
    };
}

/// Clear bits in a CSR.
macro_rules! clear_csr {
    ($name:ident, $csr:literal) => {
        #[inline]
        pub unsafe fn $name(bits: u64) {
            unsafe { asm!(concat!("csrc ", $csr, ", {}"), in(reg) bits, options(nomem, nostack, preserves_flags)) };
        }
    };
}

// Machine-level CSRs
read_csr!(read_mstatus, "mstatus");
write_csr!(write_mstatus, "mstatus");
set_csr!(set_mstatus, "mstatus");
clear_csr!(clear_mstatus, "mstatus");

read_csr!(read_mtvec, "mtvec");
write_csr!(write_mtvec, "mtvec");

read_csr!(read_mepc, "mepc");
write_csr!(write_mepc, "mepc");

read_csr!(read_mcause, "mcause");
read_csr!(read_mtval, "mtval");
read_csr!(read_mip, "mip");

read_csr!(read_mie, "mie");
write_csr!(write_mie, "mie");
set_csr!(set_mie, "mie");
clear_csr!(clear_mie, "mie");

read_csr!(read_mhartid, "mhartid");

// Supervisor-level CSRs
read_csr!(read_sstatus, "sstatus");
write_csr!(write_sstatus, "sstatus");

read_csr!(read_stvec, "stvec");
write_csr!(write_stvec, "stvec");

read_csr!(read_sepc, "sepc");
write_csr!(write_sepc, "sepc");

read_csr!(read_scause, "scause");
read_csr!(read_stval, "stval");

read_csr!(read_satp, "satp");
write_csr!(write_satp, "satp");

read_csr!(read_sie, "sie");
write_csr!(write_sie, "sie");
set_csr!(set_sie, "sie");
clear_csr!(clear_sie, "sie");

read_csr!(read_sip, "sip");

// Time CSR (read-only in U/S mode)
read_csr!(read_time, "time");
read_csr!(read_cycle, "cycle");

// ── Fence / Barriers ────────────────────────────────────────

/// Full memory fence (iorw, iorw).
#[inline]
pub unsafe fn fence() {
    unsafe { asm!("fence iorw, iorw", options(nostack, preserves_flags)) };
}

/// Instruction fence.
#[inline]
pub unsafe fn fence_i() {
    unsafe { asm!("fence.i", options(nostack, preserves_flags)) };
}

// ── Wait / Power ────────────────────────────────────────────

/// Wait for interrupt.
#[inline]
pub unsafe fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) };
}

// ── TLB ─────────────────────────────────────────────────────

/// Flush all TLB entries.
#[inline]
pub unsafe fn sfence_vma_all() {
    unsafe { asm!("sfence.vma zero, zero", options(nostack, preserves_flags)) };
}

/// Flush TLB entries for a specific virtual address.
#[inline]
pub unsafe fn sfence_vma(vaddr: u64) {
    unsafe { asm!("sfence.vma {}, zero", in(reg) vaddr, options(nostack, preserves_flags)) };
}
