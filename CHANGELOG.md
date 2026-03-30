# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-29

### Added

- **arch-intrinsics**: C inline asm headers for all 4 architectures (x86_64, AArch64, ARM Cortex-M, RISC-V 64) — header-only, static inline functions matching Rust API
- **crypto**: C implementations of CRC-32 (IEEE 802.3, table/bitwise), SHA-256 (FIPS 180-4), HMAC-SHA256 (RFC 2104) — verified with same standard test vectors as Rust
- **sync**: C implementations using C11 atomics — ticket spinlock, one-time initialization, atomic bitmap operations
- **collections**: C implementations of intrusive doubly-linked list and red-black tree (CLRS algorithm) — 17 C tests including 100-node stress test

All new C code is dual-language alongside existing Rust implementations. Enables direct consumption by GenesisOS-RT and HermeticaOS without FFI overhead.

## [0.1.0] - 2026-03-22

### Added

- **genesis-abi**: Foundational ABI types (`GenResult`, `GenSyscallNr`, `GenCapability`, `GenModuleDescriptor`) with C headers as source of truth and Rust mirror
  - 29 error codes across 7 groups (general, security, I/O, format, module, RT, syscall)
  - 45 syscall definitions across 4 namespaced ranges (shared, TerranoxOS, GenesisOS-RT, HermeticaOS)
  - 16 capability bits with bitwise set operations
  - Module descriptor with kernel API function pointer table
  - `result-names` feature for human-readable error strings
- **primitives**: Freestanding C memory/string operations (`gen_memcpy`, `gen_memset`, `gen_memmove`, `gen_memcmp`, `gen_strlen`, `gen_strncmp`, `gen_strnlen`, `gen_secure_zero`) with compiler-required aliases
- **bitops**: Dual C + Rust bitmap operations (set/clear/test/find-first-zero, region ops, popcount, `BitIter`)
- **kfmt**: Callback-based kernel printf (C) and `fmt::Write` wrapper (Rust) with width/padding/length modifiers
- **sync**: Ticket spinlock (fair FIFO), `Once<T>` with fast-path Acquire load, atomic bitops on `&[AtomicU32]`
- **arch-intrinsics**: CPU intrinsics for x86_64, AArch64, ARM Cortex-M, and RISC-V 64 (control registers, barriers, TLB/cache, interrupts)
- **alloc**: Bitmap PMM, pool allocator (O(1) embedded free-list), bump allocator, slab cache with partial/full/empty lists and `alloc_grow`
- **collections**: `StaticVec<T,N>`, `RingBuf<T,N>` (SPSC), `StaticHashMap<K,V,N>` (FNV-1a), `IntrusiveList`, `RbTree` (intrusive red-black tree)
- **crypto**: CRC-32 IEEE 802.3 (table/bitwise), SHA-256 FIPS 180-4 (streaming + one-shot), HMAC-SHA256 RFC 2104
- **elf**: ELF64 little-endian parser with header/section/segment parsing, symbol table with `SymbolIter`, x86_64 RELA relocations
- **devicetree**: FDT parser (big-endian DTB, node traversal, property list) and ACPI parser (RSDP v1/v2, MADT entries)
- CI pipeline with 8 jobs (rust-tests, cross-compile, Miri, Clippy, fmt, C build/test, ABI drift check, Frama-C)
- Cross-compilation for 4 bare-metal targets (x86_64-unknown-none, aarch64-unknown-none, thumbv7em-none-eabi, riscv64gc-unknown-none-elf)
- Frama-C ACSL formal verification on all C sources (primitives, bitops, alloc)
- Bazel build system (bzlmod) alongside Cargo workspace
- 149 Rust tests + 104 C tests = 253 total, including Miri verification on unsafe code

[0.1.0]: https://github.com/terranox-os/kernel-libs/releases/tag/v0.1.0
