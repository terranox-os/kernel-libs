# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

kernel-libs is a shared library repository of freestanding, zero-dependency crates (Rust) and static libraries (C) consumed by three OS kernels: **TerranoxOS** (security-focused desktop), **GenesisOS-RT** (robotics RTOS), and **HermeticaOS** (experimental hot-swap modules). The planning documents (`shared_kernel_libs_v11.docx`, `kernel_libs_impl_plan_v11.docx`) are the authoritative specification.

## Build Systems

- **Bazel** (`WORKSPACE.bazel`, `BUILD.bazel` per crate) — primary build for GenesisOS-RT integration
- **Cargo** (`Cargo.toml` workspace at root) — Rust crates, used by TerranoxOS Rust layers
- **C static libraries** — compiled with `-ffreestanding -nostdlib`, linked by C kernel cores via Makefile/LDFLAGS

### Common Commands (once code exists)

```bash
# Rust crates
cargo build                        # build all Rust crates
cargo test                         # host-compiled unit tests (no QEMU needed)
cargo test -p <crate>              # test a single crate (e.g., cargo test -p kernel-crypto)
cargo miri test -p <crate>         # run under Miri for UB detection

# C libraries
# Build via Bazel:
bazel build //primitives:gen_primitives
bazel test //primitives:primitives_test

# Frama-C verification (C libraries only)
frama-c -wp -wp-prover alt-ergo,z3,cvc5 primitives/src/*.c
```

## Design Rules (All Code Must Follow)

1. **Zero upward dependencies** — libraries never call into any OS kernel or reference global state
2. **Caller provides resources** — no internal allocation; callers pass buffers, function pointers (C), or trait objects (Rust)
3. **`#![no_std]`, `no_alloc`** (Rust) / `-ffreestanding -nostdlib` (C) — no heap, no OS, no libc
4. **Architecture-gated, not OS-gated** — use `#[cfg(target_arch)]` or `#ifdef __x86_64__`, never `#ifdef __linux__`
5. **Dual-language where needed** — crates consumed by both C and Rust kernels must provide both implementations or export C-ABI from Rust. Cross-language build deps must never break single-language builds.
6. **Frama-C ACSL annotations** required on all C functions (requires/ensures/assigns/loop invariants)

## Architecture

### Crate Dependency Layers

```
Layer 0:  genesis-abi (C headers = source of truth, Rust mirror with CI drift check)
Layer 1:  primitives(C) | bitops(C+Rust) | kfmt(C+Rust) | sync(Rust) | arch-intrinsics(Rust)
Layer 2:  alloc(C+Rust) | collections(Rust) | crypto(Rust)
Layer 3:  elf(Rust) | devicetree(Rust)
```

Within a layer, crates have no inter-dependencies. Build in any order within a layer.

### Key Cross-Crate Dependency

`alloc` (C side: `bitmap_pmm.c`) calls `gen_bit_*`/`gen_bitmap_*` from `bitops` C API. This is a **C-to-C only** dependency — the Rust side of bitops is independent.

### Critical Implementation Details

- **genesis-abi**: C headers under `include/` are the ABI source of truth. The Rust crate (`src/lib.rs`) mirrors them. CI must check for drift between C and Rust definitions.
- **primitives**: Functions are namespaced as `gen_memcpy`, `gen_memset`, etc. Compiler-required symbols (`memcpy`, `memset`, `memmove`, `memcmp`) are aliased **only in `aliases.c`** via `__attribute__((alias))`. Never put alias definitions in headers (causes duplicate symbols at link time).
- **bitops**: Dual C + Rust implementations with identical semantics. C uses `uint32_t*` raw pointers; Rust uses `&[u32]` slices with bounds checking. C implementation uses `__builtin_ctz()` and `__builtin_popcount()`.
- **kfmt**: Callback signature uses `uint8_t` (not `char`) for unambiguous byte semantics across platforms. Rust side uses `core::fmt::Write` trait with `FnMut(u8)` closure.
- **sync**: `atomic_bitops` uses `AtomicU32` with `AcqRel`/`Acquire` ordering. Non-atomic bitops in `bitops/` are for interrupt-disabled critical sections only.
- **alloc**: Slab allocator deferred to v0.4+. Bump allocator (Rust) is for early init / per-frame scratch. Pool allocator (C) is O(1) fixed-block for RTOS.
- **collections**: Red-black tree deferred to v0.4+.

### "Extract from GenesisOS" Means Rewrite

GenesisOS is now C++. Extraction means **rewriting C++ to pure C17** — not copy-paste. Remove all C++ constructs.

## Versioning / Release Plan

| Version | Crates |
|---------|--------|
| v0.1.0 | genesis-abi |
| v0.2.0 | primitives, bitops, kfmt, sync, arch-intrinsics (x86-64) |
| v0.3.0 | alloc (PMM + pool + bump), collections, crypto |
| v0.4.0 | elf, devicetree, arch-intrinsics (AArch64 + ARM-CM), alloc (slab) |

## Testing Strategy

- **Rust crates**: `cargo test` (host-compiled, no QEMU). Miri for UB detection. `cargo-fuzz` for coverage-guided fuzzing.
- **C libraries**: Bazel test targets or standalone test harness. Frama-C WP plugin for formal verification (Alt-Ergo, Z3, CVC5 solvers). Failing proofs block merge in CI.
- All tests must run on the host — no hardware or emulator required.
