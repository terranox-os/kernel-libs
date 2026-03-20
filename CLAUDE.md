# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

kernel-libs is a shared library repository of freestanding, zero-dependency crates (Rust) and static libraries (C) consumed by three OS kernels: **TerranoxOS** (security-focused desktop), **GenesisOS-RT** (robotics RTOS), and **HermeticaOS** (experimental hot-swap modules). The planning documents (`shared_kernel_libs_v11.docx`, `kernel_libs_impl_plan_v11.docx`) are the authoritative specification.

## Build & Test Commands

```bash
# Cargo (Rust crates)
cargo build                              # build workspace
cargo test                               # run all tests (host-compiled, no QEMU)
cargo test -p <crate>                    # single crate (e.g. kernel-crypto, kernel-elf)
cargo +nightly miri test -p <crate>      # UB detection (use for sync, collections)

# Bazel (all C + Rust targets)
bazel build //...                        # build everything
bazel test //...                         # run all 14 test targets
bazel build //genesis-abi:genesis_abi    # single C target
bazel build //crypto:kernel_crypto       # single Rust target
bazel test //sync:kernel_sync_test       # single Rust test

# Frama-C verification (C libraries with ACSL annotations)
frama-c -wp -wp-prover alt-ergo,z3,cvc5 primitives/src/*.c
frama-c -wp -wp-prover alt-ergo,z3,cvc5 bitops/src/bitmap.c
frama-c -wp -wp-prover alt-ergo,z3,cvc5 alloc/src/bitmap_pmm.c alloc/src/pool.c
```

### Crate / Target Names

| Directory | Cargo crate | Bazel C target | Bazel Rust target |
|-----------|------------|----------------|-------------------|
| genesis-abi | `genesis-abi` | `//genesis-abi:genesis_abi` | `//genesis-abi:genesis_abi_rs` |
| primitives | — | `//primitives:gen_primitives` | — |
| bitops | `kernel-bitops` | `//bitops:gen_bitops` | `//bitops:kernel_bitops` |
| kfmt | `kernel-kfmt` | `//kfmt:gen_kfmt` | `//kfmt:kernel_kfmt` |
| sync | `kernel-sync` | — | `//sync:kernel_sync` |
| arch-intrinsics | `kernel-arch-intrinsics` | — | `//arch-intrinsics:kernel_arch_intrinsics` |
| alloc | `kernel-alloc` | `//alloc:gen_alloc` | `//alloc:kernel_alloc` |
| collections | `kernel-collections` | — | `//collections:kernel_collections` |
| crypto | `kernel-crypto` | — | `//crypto:kernel_crypto` |
| elf | `kernel-elf` | — | `//elf:kernel_elf` |
| devicetree | `kernel-devicetree` | — | `//devicetree:kernel_devicetree` |

## Design Rules (All Code Must Follow)

1. **Zero upward dependencies** — libraries never call into any OS kernel or reference global state
2. **Caller provides resources** — no internal allocation; callers pass buffers, function pointers (C), or trait objects (Rust)
3. **`#![no_std]`, `no_alloc`** (Rust) / `-ffreestanding -nostdlib` (C) — no heap, no OS, no libc
4. **Architecture-gated, not OS-gated** — use `#[cfg(target_arch)]` or `#ifdef __x86_64__`, never `#ifdef __linux__`
5. **Dual-language where needed** — crates consumed by both C and Rust kernels provide both implementations. Cross-language build deps must never break single-language builds.
6. **Frama-C ACSL annotations** required on all C functions (requires/ensures/assigns/loop invariants)

## Architecture

### Crate Dependency Layers

```
Layer 0:  genesis-abi (C headers = source of truth, Rust mirror)
Layer 1:  primitives(C) | bitops(C+Rust) | kfmt(C+Rust) | sync(Rust) | arch-intrinsics(Rust)
Layer 2:  alloc(C+Rust) | collections(Rust) | crypto(Rust)
Layer 3:  elf(Rust) | devicetree(Rust)
```

Within a layer, crates have no inter-dependencies. Build in any order within a layer.

### Key Cross-Crate Dependency

`alloc` (C side: `bitmap_pmm.c`) calls `gen_bit_*`/`gen_bitmap_*` from `bitops` C API. This is a **C-to-C only** dependency — the Rust side of bitops is independent.

### Critical Implementation Details

- **genesis-abi**: C headers under `include/` are the ABI source of truth. The Rust crate (`src/lib.rs`) mirrors them. CI must check for drift between C and Rust definitions. Error codes use gaps between groups (general -1..-10, security -16..-18, I/O -32..-34, format -48..-50, module -64..-67) to allow future additions. Syscall ranges are 256 entries each: shared 0x0000, TerranoxOS 0x0100, GenesisOS-RT 0x0200, HermeticaOS 0x0300.
- **primitives**: Functions namespaced as `gen_memcpy`, `gen_memset`, etc. Compiler-required symbols (`memcpy`, `memset`, `memmove`, `memcmp`) are thin wrappers in `aliases.c` — link exactly once. Word-aligned fast paths in memcpy/memset. `gen_secure_zero` uses volatile writes to prevent dead-store elimination.
- **bitops**: Dual C + Rust with identical semantics. C uses `uint32_t*` raw pointers + `__builtin_ctz()`/`__builtin_popcount()`; Rust uses `&[u32]` slices with bounds checking + `BitIter`. Bitmap convention: 0 = free, 1 = allocated.
- **kfmt**: C callback signature uses `uint8_t` (not `char`) for unambiguous byte semantics. Supports `%d/%i/%u/%x/%X/%p/%s/%c/%%`, width, zero-padding, `l`/`ll` length modifiers. Rust side provides `KernelWriter<F: FnMut(u8)>` implementing `core::fmt::Write`, `CountingWriter`, and `kwrite!` macro.
- **sync**: Ticket spinlock (fair FIFO), `Once<T>` with fast-path Acquire load, `atomic_bitops` on `&[AtomicU32]` with `AcqRel` ordering. All verified under Miri.
- **arch-intrinsics**: `#[cfg(target_arch)]` gated. x86_64 (CR0-4, MSR, port I/O, CLI/STI/HLT, RDTSC, CPUID), AArch64 (sysreg macros, DMB/DSB/ISB, TLB/cache, WFI), ARM Cortex-M (PRIMASK/BASEPRI, MSP/PSP, PendSV, SysTick), RISC-V 64 (CSR macros M+S mode, fence, sfence.vma).
- **alloc**: Bitmap PMM calls bitops C API; Pool uses embedded free-list (O(1)); Bump allocator (Rust) for early-init scratch. Slab cache (`GenSlabCache`) manages fixed-size objects across partial/full/empty slab lists with O(1) alloc from partial, page allocator callbacks for growth, and shrink to return empty slabs. `gen_slab_cache_alloc_grow` accepts a caller-provided `GenSlab` struct for dynamic expansion (no hidden allocation).
- **collections**: `StaticVec<T,N>` (inline `MaybeUninit` array), `RingBuf<T,N>` (SPSC, `UnsafeCell`+atomics), `StaticHashMap<K,V,N>` (open addressing, FNV-1a, tombstone reuse), `IntrusiveList` (doubly-linked, raw pointers), `RbTree` (intrusive red-black tree, `u64` keys, insert/remove/find/min/max, `RbInorderIter`, Miri-verified).
- **crypto**: CRC-32 IEEE 802.3 (feature-gated lookup table vs bitwise), SHA-256 FIPS 180-4 (streaming + one-shot), HMAC-SHA256 RFC 2104. All verified against standard test vectors.
- **elf**: ELF64 little-endian parser. Header/section/segment parsing, symbol table with `SymbolIter` and `find_symbol_by_name`, RELA relocations with `apply_x86_64_rela` (R_X86_64_64, PC32, 32, 32S, RELATIVE).
- **devicetree**: FDT parser (big-endian DTB blobs, node traversal by path, `FdtPropertyList` fixed-capacity). ACPI parser (RSDP v1/v2 with checksum, 16-byte aligned scan, MADT with Local APIC / I/O APIC / Interrupt Override / NMI entries).

## Testing

- **149 Rust tests + 94 C tests = 243 total**, all passing
- Rust tests run via `cargo test` on host (no QEMU)
- C tests compile with GCC and link against object files — see `*/tests/` directories
- Miri verified: `genesis-abi`, `sync`, `rbtree` (critical for unsafe + atomics correctness)
- Crypto tests use FIPS 180-4, RFC 4231, IEEE 802.3 standard test vectors
- `#[cfg(test)] extern crate alloc` pattern used in no_std crates that need `Vec` in tests

## CI Pipeline (`.github/workflows/ci.yml`)

Seven jobs run on push/PR to `main`:

| Job | What it checks |
|-----|---------------|
| `rust-tests` | `cargo build` + `cargo test --workspace` + feature-gated tests |
| `rust-miri` | Miri on all crates with unsafe (sync, alloc, collections, crypto, elf, devicetree) |
| `rust-clippy` | `cargo clippy -- -D warnings` |
| `rust-fmt` | `cargo fmt --check` |
| `c-build-test` | GCC compile all C sources with `-Werror -Wpedantic -ffreestanding`, run all C test binaries |
| `abi-drift-check` | Verify error code counts, syscall counts, and struct sizes match between C headers and Rust mirror |
| `frama-c` | Frama-C WP verification of ACSL annotations on all C sources (primitives, bitops, alloc) |

Run Frama-C locally: `./scripts/frama-c-verify.sh` (full proofs) or `./scripts/frama-c-verify.sh --check-only` (parse annotations only).
