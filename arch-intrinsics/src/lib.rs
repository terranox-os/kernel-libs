//! Freestanding CPU intrinsics for bare-metal kernel code.
//!
//! Each module is gated on `target_arch`. Only the module matching
//! the compilation target is included — no OS-gating.
//!
//! These wrap single CPU instructions via inline assembly. They are
//! the only crate in kernel-libs that performs direct hardware access.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "arm")]
pub mod arm_cm;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;
