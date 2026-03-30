//! Freestanding ELF64 little-endian only parser.
//!
//! Parses ELF64 little-endian binaries from byte slices — no heap, no I/O.
//! ELFCLASS32 and big-endian files are rejected at parse time.
//! Used by TerranoxOS (userspace loading) and HermeticaOS (module loading).

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod parser;
pub mod reloc;
pub mod symbols;

pub use parser::*;
