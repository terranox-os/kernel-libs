# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-29

### Added

- **genesis-abi**: TerranoxOS syscall reference reconciliation
  - 82 TerranoxOS syscalls organized across 12 subsystem blocks (process, thread, memory, IPC, filesystem, display/compositor, input, GPU/DRM, networking, timers, system/audit, sigil/sandbox)
  - `TrxCapSet`: 128-bit hierarchical capability type with 12 domains and 40 leaf sub-capabilities. Compile-time hierarchy via domain-partitioned bitmask — no runtime graph traversal
  - `genesis_trx_types.h`: 10 TerranoxOS syscall data structures (`GenTrxCapToken`, `GenTrxCapTokenSet`, `GenTrxProcessInfo`, `GenTrxDisplayInfo`, `GenTrxInputEvent`, `GenTrxTouchEvent`, `GenTrxWaitItem`, `GenTrxTimespec`, `GenTrxGpuInfo`, `GenTrxAuditEntry`)
  - POSIX errno mapping: `gen_result_to_errno()` / `gen_result_from_errno()` with all 33 error codes mapped
  - 4 new error codes: `GEN_ERR_CHANNEL_CLOSED` (-35), `GEN_ERR_DISPLAY_OFFLINE` (-36), `GEN_ERR_GPU_ERROR` (-37), `GEN_ERR_HANDLE_LIMIT` (-99)
  - `gen_syscall_trx_subsystem()` helper for subsystem classification
- **arch-intrinsics**: C inline asm headers for all 4 architectures (x86_64, AArch64, ARM Cortex-M, RISC-V 64) — header-only, static inline functions matching Rust API
- **crypto**: C implementations of CRC-32 (IEEE 802.3, table/bitwise), SHA-256 (FIPS 180-4), HMAC-SHA256 (RFC 2104) — verified with same standard test vectors as Rust
- **sync**: C implementations using C11 atomics — ticket spinlock, one-time initialization, atomic bitmap operations
- **collections**: C implementations of intrusive doubly-linked list and red-black tree (CLRS algorithm) — 17 C tests including 100-node stress test

All new C code is dual-language alongside existing Rust implementations. Enables direct consumption by GenesisOS-RT and HermeticaOS without FFI overhead.

Updated test totals: 178 Rust + 144 C = 322 tests

### Changed

- **genesis-abi**: Existing 8 TerranoxOS syscalls renumbered into subsystem-grouped positions
  - `GEN_SYS_CAP_GRANT` (0x0100) → `GEN_SYS_TRX_PROCESS_CAP_GRANT` (0x0105)
  - `GEN_SYS_CAP_REVOKE` (0x0101) → `GEN_SYS_TRX_PROCESS_CAP_REVOKE` (0x0106)
  - `GEN_SYS_CAP_CHECK` (0x0102) → `GEN_SYS_TRX_PROCESS_CAP_QUERY` (0x0107)
  - `GEN_SYS_SIGIL_SIGN` (0x0103) → `GEN_SYS_TRX_SIGIL_SIGN` (0x01B0)
  - `GEN_SYS_SIGIL_VERIFY` (0x0104) → `GEN_SYS_TRX_SIGIL_VERIFY` (0x01B1)
  - `GEN_SYS_AUDIT_LOG` (0x0105) → `GEN_SYS_TRX_AUDIT_READ` (0x01A3)
  - `GEN_SYS_SANDBOX_CREATE` (0x0106) → `GEN_SYS_TRX_SANDBOX_CREATE` (0x01B2)
  - `GEN_SYS_SANDBOX_ENTER` (0x0107) → `GEN_SYS_TRX_SANDBOX_ENTER` (0x01B3)
  - Old names preserved as deprecated `#define` aliases (to be removed in v0.3.0)

### Breaking

- TerranoxOS syscall numbers changed from contiguous 0x0100–0x0107 to subsystem-grouped positions. Source-level aliases preserve compilation compatibility, but binary ABI breaks for any code that hardcoded the old numeric values.

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
- **sync**: Ticket spinlock (fair FIFO), `Once<T>`, atomic bitmap operations — all Miri-verified
- **arch-intrinsics**: CPU intrinsics for x86_64, AArch64, ARM Cortex-M, RISC-V 64
- **alloc**: Bitmap PMM, pool allocator (O(1)), slab cache, bump allocator (Rust)
- **collections**: `StaticVec<T,N>`, `RingBuf<T,N>`, `StaticHashMap<K,V,N>`, `IntrusiveList`, `RbTree` — Miri-verified
- **crypto**: CRC-32 (IEEE 802.3), SHA-256 (FIPS 180-4), HMAC-SHA256 (RFC 2104)
- **elf**: ELF64 little-endian parser with symbol table and x86_64 RELA relocations
- **devicetree**: FDT parser (big-endian DTB) and ACPI parser (RSDP v1/v2, MADT)
