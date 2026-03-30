//! Freestanding hardware description parsers.
//!
//! - `fdt`: Flattened Device Tree parser (ARM, RISC-V)
//! - `acpi`: RSDP/MADT parser (x86)

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod acpi;
pub mod fdt;
