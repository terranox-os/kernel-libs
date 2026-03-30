//! Freestanding ELF32/ELF64 parser.
//!
//! Parses ELF binaries from byte slices — no heap, no I/O.
//! Used by TerranoxOS (userspace loading) and HermeticaOS (module loading).

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod parser;
pub mod reloc;
pub mod symbols;

pub use parser::*;
