<!--
SPDX-License-Identifier: CC-BY-4.0
-->

# TerranoxOS Shared Infrastructure — Migration Plan

*March 2026 — Repo structure, crate scaffolding, and migration strategy*

---

## Context

The TerranoxOS ecosystem has 4 OS projects sharing code through kernel-libs (currently a monorepo). This plan defines the hybrid multi-repo structure (Option C from ../spec/TRX-DOC-1001-shared-infra-reference.md) and the initial scaffolding for the shared Rust crates.

**Approach**: Gradual migration. kernel-libs stays as-is. New shared Rust crates live in a new `terranox-crates` repo. Each OS project includes shared repos via git submodules.

---

## 1. Repository map (9 repos)

```
terranox-os/ (GitHub org)
│
├── kernel-libs              EXISTING — freestanding C+Rust libs
│   (genesis-abi, primitives, bitops, kfmt, sync, alloc,
│    arch-intrinsics, collections, crypto, elf, devicetree)
│
├── terranox-crates          NEW — shared Rust crates
│   (trx-cap, trx-abi, trx-idl, trx-store, trx-sigil, trx-test)
│
├── terranox-tools           EXISTING — Lattice shell + World package manager
│   ├── Lattice (5 crates): SAT/SMT constraint shell
│   │   (lattice-core, lattice-solver, lattice-parse, lattice-exec, lattice-shell)
│   ├── World (9 crates): Nickel-based package manager
│   │   (world-core, world-build, world-cli, world-resolve, world-store,
│   │    world-vm, world-nickel, world-sbom, world-cache)
│   ├── System tools (4 crates): terranox-strata, initramfs, installer, iso
│   └── Derivations: 300+ Nickel package definitions
│
├── trx-libc                 PLANNED — native POSIX libc in Zig
│   (from TRX-DOC-1101-libc-plan.md)
│
├── trx-musl                 PLANNED — musl libc fork (Linux compat layer)
│
├── terranox-toolchain       EXISTING — clang/LLVM cross-compilation toolchain
│   (stage1 compiler WIP, TerranoxOS triple support,
│    Docker image: ghcr.io/terranox-os/terranox-toolchain-musl)
│
├── terranox                 EXISTING — desktop OS (kernel + userspace)
│
├── genesis-rt               EXISTING — robotics RTOS
│
├── hermetica                PLANNED — hot-swap module kernel
│
└── sigilvm                  EXISTING — verified bytecode OS
```

---

## 2. terranox-crates — new shared Rust repo

### Repository structure

```
terranox-crates/
├── Cargo.toml               # workspace root
├── MODULE.bazel              # Bazel bzlmod (rules_rust)
├── CLAUDE.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── .github/
│   └── workflows/
│       └── ci.yml            # cargo test, clippy, fmt, miri
├── crates/
│   ├── trx-cap/
│   │   ├── Cargo.toml        # no_std, no alloc
│   │   └── src/
│   │       ├── lib.rs         # CapSet, Cap enum, CapError
│   │       ├── check.rs       # cap_check(pid, required)
│   │       ├── derive.rs      # cap_derive(parent, child, restricted)
│   │       ├── revoke.rs      # cap_revoke(pid, cap) — transitive
│   │       ├── table.rs       # CapTable: per-process storage
│   │       └── ffi.rs         # extern "C" for kernel-libs interop
│   │
│   ├── trx-abi/
│   │   ├── Cargo.toml        # no_std
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── syscall.rs     # SYS_* constants (all 91)
│   │       ├── errno.rs       # POSIX errno constants
│   │       ├── types.rs       # pid_t, off_t, mode_t, etc.
│   │       ├── stat.rs        # struct stat layout
│   │       └── signal.rs      # signal numbers
│   │
│   ├── trx-idl/
│   │   ├── Cargo.toml        # std (build-time tool)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs      # parse .tidl files
│   │       ├── codegen.rs     # generate Rust client/server stubs
│   │       └── types.rs       # IDL type system (maps to trx-cap)
│   │
│   ├── trx-store/
│   │   ├── Cargo.toml        # no_std optional, alloc required
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── hash.rs        # BLAKE3 hashing
│   │       ├── store.rs       # content-addressed blob store
│   │       └── verify.rs      # integrity verification on read
│   │
│   ├── trx-sigil/
│   │   ├── Cargo.toml        # no_std
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sign.rs        # Ed25519 signing
│   │       ├── verify.rs      # signature verification
│   │       └── keyring.rs     # key management
│   │
│   └── trx-test/
│       ├── Cargo.toml        # no_std + std feature
│       └── src/
│           ├── lib.rs
│           ├── harness.rs     # test runner (serial output in no_std)
│           ├── mock.rs        # mock FDs, mock capabilities
│           └── assert.rs      # custom assertions
```

### Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/trx-cap",
    "crates/trx-abi",
    "crates/trx-idl",
    "crates/trx-store",
    "crates/trx-sigil",
    "crates/trx-test",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/terranox-os/terranox-crates"

[workspace.dependencies]
trx-cap   = { path = "crates/trx-cap" }
trx-abi   = { path = "crates/trx-abi" }
trx-sigil = { path = "crates/trx-sigil" }
trx-store = { path = "crates/trx-store" }
trx-test  = { path = "crates/trx-test" }
```

### Crate dependency graph

```
trx-test (dev-dependency only)
    │
    ├──► trx-idl ──► trx-abi ──► trx-cap
    ├──► trx-store ──► trx-abi
    └──► trx-sigil ──► trx-abi
```

trx-cap is the foundation (no dependencies). trx-abi depends on trx-cap. Everything else depends on trx-abi.

---

## 3. Submodule wiring

### TerranoxOS (desktop — uses everything)

```
terranox/
├── .gitmodules
│     kernel-libs = https://github.com/terranox-os/kernel-libs.git
│     crates      = https://github.com/terranox-os/terranox-crates.git
├── kernel-libs/          (submodule)
├── crates/               (submodule)
├── kernel/               (C + Rust, links kernel-libs + crates)
├── userspace/            (Rust, depends on crates)
│   └── trx-shell/
└── Cargo.toml            (workspace includes crates/* and kernel/*)
```

### GenesisOS-RT (robotics — C only, no Rust crates)

```
genesis-rt/
├── .gitmodules
│     kernel-libs = https://github.com/terranox-os/kernel-libs.git
├── kernel-libs/          (submodule)
├── kernel/               (pure C, links kernel-libs C side only)
└── Makefile
```

### HermeticaOS (VMM — uses kernel-libs + crates)

```
hermetica/
├── .gitmodules
│     kernel-libs = https://github.com/terranox-os/kernel-libs.git
│     crates      = https://github.com/terranox-os/terranox-crates.git
├── kernel-libs/          (submodule)
├── crates/               (submodule)
├── kernel/               (C + Rust VMM layer)
└── Cargo.toml
```

### SigilVM (bytecode — uses kernel-libs + crates for verifier)

```
sigilvm/
├── .gitmodules
│     kernel-libs = https://github.com/terranox-os/kernel-libs.git
│     crates      = https://github.com/terranox-os/terranox-crates.git
├── kernel-libs/          (submodule)
├── crates/               (submodule)
├── kernel/
│   ├── zig/              (Zig kernel: boot, HAL, JIT, scheduler)
│   └── rust/             (Rust verifier, uses trx-cap + trx-sigil)
└── build.zig
```

---

## 4. API verification against current decisions

### trx-cap — must use TrxCapSet (128-bit), not CapSet (u32)

The ../spec/TRX-DOC-1001-shared-infra-reference.md defines `CapSet = u32` with 10 capabilities. This is **outdated**. PR #13 introduced `TrxCapSet` (128-bit, 12 domains, 40 leaf capabilities). trx-cap must mirror this:

```rust
// trx-cap uses the 128-bit hierarchical model from genesis_module.h
#[repr(C)]
pub struct TrxCapSet { pub lo: u64, pub hi: u64 }

// 12 domains, 40 leaves — matches kernel-libs TrxCapSet
pub const PROCESS_CREATE: TrxCapSet = TrxCapSet { lo: 1 << 0, hi: 0 };
pub const PROCESS_SIGNAL: TrxCapSet = TrxCapSet { lo: 1 << 1, hi: 0 };
// ... (all 40 leaves from genesis_module.h)
pub const GPU_RENDER: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 0 };
pub const GPU_COMPUTE: TrxCapSet = TrxCapSet { lo: 0, hi: 1 << 1 };
// ... etc
```

NOT: `pub type CapSet = u32;` (../spec/TRX-DOC-1001-shared-infra-reference.md is stale here)

### trx-abi — 119 syscalls, not 91

The reconciliation added 82 TerranoxOS-specific syscalls + 23 shared + 7 RT + 7 Hermetica = **119 total** across 4 ranges. trx-abi must define all of them, organized by the subsystem blocks:

```
Shared (0x0000): 23 syscalls (EXIT, READ, WRITE, MMAP, ...)
TerranoxOS (0x0100): 82 syscalls across 12 subsystem blocks
GenesisOS-RT (0x0200): 7 syscalls
HermeticaOS (0x0300): 7 syscalls
```

### trx-idl — generates stubs for C, Rust, and Zig

The IDL compiler must generate:
- **Rust** client/server stubs (for TerranoxOS userspace + HermeticaOS + SigilVM verifier)
- **C** stubs (for GenesisOS-RT kernel, kernel-libs C consumers)
- **Zig** stubs (for trx-libc, SigilVM kernel)

Each stub includes capability requirements from the .tidl declaration.

### trx-store — extracted from world-core, uses BLAKE3

Content-addressed storage extracted from terranox-tools' `world-core::store` + `world-core::hash`. Uses BLAKE3 (not SHA-256). kernel-libs crypto has SHA-256/HMAC; trx-store brings its own BLAKE3 or depends on the `blake3` crate.

### trx-sigil — extracted from world-core + world-cache, uses Ed25519

Signing extracted from terranox-tools' `world-core::signing` + `world-cache::sign`. Ed25519 via `ed25519-dalek` crate.

---

## 4a. Language interface diagrams per kernel

### TerranoxOS (desktop) — C kernel + Rust upper layers + Rust userspace

```
┌─────────────────────────────────────────────────────┐
│  USERSPACE (Rust)                                   │
│  trx-shell, Lattice, apps                           │
│  Links: trx-libc (Zig → libc.a, C ABI)             │
│         terranox-crates (Rust, via Cargo)            │
└──────────────────────┬──────────────────────────────┘
                       │ SYSCALL (rax=nr, rdi/rsi/rdx/r10/r8/r9)
┌──────────────────────▼──────────────────────────────┐
│  KERNEL                                             │
│                                                     │
│  ┌──────────────┐  extern "C"  ┌─────────────────┐  │
│  │ kernel-libs  │◄────────────►│ Rust upper      │  │
│  │ (C)          │  FFI bridge  │ layers          │  │
│  │              │              │                 │  │
│  │ gen_sync     │  Rust calls  │ trx-cap (Rust)  │  │
│  │ gen_crypto   │  C functions │ VFS, scheduler  │  │
│  │ gen_alloc    │              │ syscall dispatch│  │
│  │ gen_kfmt     │              │ sentinel        │  │
│  └──────────────┘              └─────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### GenesisOS-RT (robotics) — pure C kernel

```
┌─────────────────────────────────────────────────────┐
│  HOST TOOLS (Rust) — runs on laptop, not on robot   │
│  trx-roboview telemetry dashboard                   │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  KERNEL (pure C — no Rust in kernel)                │
│                                                     │
│  ┌──────────────┐                                   │
│  │ kernel-libs  │  Linked directly (C → C)          │
│  │ (C side only)│                                   │
│  │              │  gen_sync, gen_crypto,             │
│  │              │  gen_collections, gen_alloc,       │
│  │              │  gen_arch_arm_cm (inline asm)      │
│  └──────────────┘                                   │
│                                                     │
│  RT scheduler, CAN bus, IMU, PWM motor control      │
│  All C, compiled with clang --target=arm-none-eabi  │
└─────────────────────────────────────────────────────┘
```

### HermeticaOS (VMM) — C kernel + Rust VMM layer

```
┌─────────────────────────────────────────────────────┐
│  KERNEL                                             │
│                                                     │
│  ┌──────────────┐  extern "C"  ┌─────────────────┐  │
│  │ kernel-libs  │◄────────────►│ Rust VMM layer  │  │
│  │ (C)          │  FFI bridge  │                 │  │
│  │              │              │ trx-cap (Rust)  │  │
│  │ gen_sync     │              │ trx-sigil (Rust)│  │
│  │ gen_alloc    │              │ VM lifecycle    │  │
│  │ gen_crypto   │              │ module hot-swap │  │
│  └──────────────┘              └─────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### SigilVM (bytecode) — Zig kernel (~54%) + Rust verifier (~39%) + C tests

```
┌──────────────────────────────────────────────────────────┐
│  TOOLS (host-side)                                       │
│  svmcheck (Rust CLI, calls verifier crate)               │
│  inspect.zig, addr.zig (Zig dev tools)                   │
│  sigil-cc (compile+verify driver, shell script)          │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│  KERNEL                                                  │
│                                                          │
│  ┌──────────────┐  @cImport    ┌──────────────────────┐  │
│  │ kernel-libs  │◄────────────►│ Zig kernel (~5K LOC) │  │
│  │ (C, submod)  │  Zig reads   │                      │  │
│  │              │  C headers   │ boot/  (Limine entry, │  │
│  │ gen_sync     │              │        GDT, IDT, PIC) │  │
│  │ gen_crypto   │              │ hal/   (serial, timer, │  │
│  │ gen_alloc    │              │        framebuf, net)  │  │
│  │ gen_kfmt     │              │ mem/   (PMM, heaps)   │  │
│  │ gen_primitives              │ sched/ (context, queue)│  │
│  │              │              │ jit/   (eBPF→x86-64,  │  │
│  │ (TODO: link  │              │        regalloc, emit) │  │
│  │  static .a)  │              │ loader/(ELF, verify,  │  │
│  └──────────────┘              │        interpret, sign)│  │
│                                │ helpers/ (21 helpers:  │  │
│                                │  fs, ipc, map, mem,   │  │
│                                │  misc, net)           │  │
│                                └───────────┬───────────┘  │
│                                 extern "C" │ (FFI)        │
│                                ┌───────────▼───────────┐  │
│                                │ Rust (~3.3K LOC)      │  │
│                                │                       │  │
│                                │ verifier/ (2.4K LOC)  │  │
│                                │   analyze, diagnostics│  │
│                                │   loops, types, heap  │  │
│                                │   ownership, state    │  │
│                                │                       │  │
│                                │ caps/ (500 LOC)       │  │
│                                │   table, check, derive│  │
│                                │   revoke, profiles    │  │
│                                │   (LOCAL — not yet    │  │
│                                │    extracted to       │  │
│                                │    trx-cap crate)     │  │
│                                │                       │  │
│                                │ sigil/ (330 LOC)      │  │
│                                │   HMAC-SHA256 signing │  │
│                                │   (LOCAL — not yet    │  │
│                                │    extracted to       │  │
│                                │    trx-sigil crate)   │  │
│                                │                       │  │
│                                │ → libverifier.a       │  │
│                                └───────────────────────┘  │
└──────────────────────────────────────────────────────────┘

Build: zig build → cargo rustc (→ libverifier.a) → link all + kernel-libs
Branch: develop-0.3.0, TCB ~6-7K LOC (below 15K target)
```

**Current state vs plan:**
- caps/ and sigil/ are **internal to sigil-vm** (not yet extracted to shared trx-cap/trx-sigil crates)
- kernel-libs submodule exists but static archive linking is **TODO** in build.zig
- svmcheck is **Rust CLI** (30K LOC) calling the Rust verifier crate, not the legacy Zig version
- Interpreter fallback exists alongside JIT (S1 milestone artifact)

---

## 4b. Relationship between kernel-libs and terranox-crates

kernel-libs stays as the **freestanding C+Rust foundation**. terranox-crates is a **higher-level Rust layer** that builds on top.

```
                    terranox-crates (Rust, higher-level)
                    ├── trx-cap (capability logic)
                    ├── trx-abi (syscall ABI, mirrors genesis-abi)
                    ├── trx-idl (IDL compiler)
                    ├── trx-store (BLAKE3 content addressing)
                    ├── trx-sigil (Ed25519 signing)
                    └── trx-test (test framework)
                              │
                              │ depends on (via path or submodule)
                              ▼
                    kernel-libs (C+Rust, freestanding)
                    ├── genesis-abi (ABI types, source of truth)
                    ├── primitives (memcpy, memset, etc.)
                    ├── bitops, kfmt, sync, alloc
                    ├── collections, crypto, elf, devicetree
                    └── arch-intrinsics
```

### What migrates eventually

| From kernel-libs | To terranox-crates | When |
|------------------|-------------------|------|
| genesis-abi (Rust mirror) | trx-abi | When trx-abi stabilizes |
| TrxCapSet (128-bit caps) | trx-cap | When trx-cap implements full DAG |
| crypto (Rust: CRC-32, SHA-256, HMAC) | trx-sigil (for signing) + trx-store (for hashing) | When those crates need crypto |

The C side of kernel-libs **never migrates** — it stays as the freestanding kernel library for all 4 projects.

---

## 5. terranox-tools → shared infra mapping

terranox-tools has 18 crates. Some map to shared infra crates, others stay TerranoxOS-specific.

### Components that map to shared crates

| terranox-tools component | Shared infra crate | Mapping |
|-------------------------|-------------------|---------|
| `world-core::store` (BLAKE3 content-addressing) | **trx-store** | world-core's `StorePath`, `hash.rs`, `store_verified.rs` become trx-store's core |
| `world-core::signing` (Ed25519 cosign) | **trx-sigil** | world-core's `signing.rs` + world-cache's `sign.rs` become trx-sigil |
| `world-cache` (NAR + zstd + R2/S3) | **trx-store** (extended) | Binary cache is the distribution layer of content-addressed storage |
| `lattice-core::constraint` | **trx-cap** (informed by) | Constraint types inform capability modeling but don't merge |
| `world-sbom` (SPDX/VEX) | Stays in terranox-tools | SBOM is package-management-specific |
| `world-vm` (Forge bytecode) | Stays in terranox-tools | Build script VM is World-specific |

### Components that stay in terranox-tools

| Component | Reason |
|-----------|--------|
| lattice-shell, lattice-parse, lattice-exec | Shell + parser + executor are TerranoxOS userspace tools |
| lattice-solver | SAT/SMT engine — shareable in theory but no other consumer currently |
| world-build, world-cli, world-resolve, world-nickel | Package manager core — TerranoxOS-specific workflows |
| terranox-strata, terranox-initramfs, terranox-installer, terranox-iso | System bootstrapping — entirely TerranoxOS-specific |

### Reorganization recommendation

```
terranox-tools/ (STAYS — TerranoxOS userspace monorepo)
  ├── crates/lattice-*/       # Shell stack (5 crates)
  ├── crates/world-*/         # Package manager (9 crates, minus extracted store/signing)
  ├── crates/terranox-*/      # System tools (4 crates)
  └── derivations/            # 300+ Nickel package defs

terranox-crates/ (NEW — shared infra)
  ├── crates/trx-cap/         # Capability system (from kernel-libs TrxCapSet + new DAG logic)
  ├── crates/trx-abi/         # Syscall ABI (mirrors genesis-abi)
  ├── crates/trx-idl/         # Capability-safe IDL compiler
  ├── crates/trx-store/       # Content-addressed storage (extracted from world-core)
  ├── crates/trx-sigil/       # Ed25519 signing (extracted from world-core + world-cache)
  └── crates/trx-test/        # Test framework

terranox-tools depends on terranox-crates (world-core uses trx-store, trx-sigil)
```

### Migration steps for extraction

1. **Extract trx-store**: Move `world-core::store`, `world-core::hash`, `world-core::store_verified` into `trx-store` crate. world-core becomes a thin wrapper that re-exports trx-store types.
2. **Extract trx-sigil**: Move `world-core::signing` + `world-cache::sign` into `trx-sigil`. world-core/world-cache depend on trx-sigil.
3. **Wire dependency**: terranox-tools adds terranox-crates as a git submodule. world-core's Cargo.toml gains `trx-store = { path = "../../crates/trx-store" }`.

---

## 6. Differences from ../spec/TRX-DOC-1001-shared-infra-reference.md

The reference document was written before several decisions. Key corrections:

| Reference doc says | Current plan |
|-------------------|-------------|
| `gen_cap.h` in kernel-libs (C) | kernel-libs has `TrxCapSet` (128-bit C struct); trx-cap (Rust) implements the DAG logic |
| `gen_hash.h` BLAKE3 in kernel-libs | kernel-libs has SHA-256/HMAC (C+Rust); BLAKE3 goes in trx-store/trx-sigil |
| `trx-musl` as only libc | trx-libc (Zig, native) + trx-musl (Linux compat) — both kept |
| gcc build commands | clang/LLVM only — no gcc in the stack |
| Makefile for kernel-libs | Bazel (bzlmod) is primary; Makefile removed |
| `CapSet = u32` (32-bit) | `TrxCapSet` (128-bit, 12 domains, 40 leaves) from PR #13 |
| Linux-compatible syscall numbers | TerranoxOS-specific 91-syscall ABI (../spec/TRX-DOC-1000-syscall-abi-reference.md) |

---

## 6. Implementation phases

### Phase 1: Create terranox-crates repo (Week 1)

- Create `terranox-os/terranox-crates` on GitHub
- Scaffold: Cargo.toml workspace, MODULE.bazel, CLAUDE.md, CI pipeline
- Empty crate directories with Cargo.toml stubs
- CI: cargo build, cargo test, clippy, fmt, miri

### Phase 2: trx-cap — capability system (~800 LOC, Week 2-3)

Foundation crate. `#![no_std]`, no allocator required.

- `CapSet`: 128-bit bitmask (mirrors TrxCapSet from kernel-libs)
- `Cap` enum: all 40 leaf capabilities + 12 domain parents
- `cap_check(pid, required)`: check if process holds capabilities
- `cap_derive(parent, child, restricted)`: create restricted subset
- `cap_revoke(pid, cap)`: transitive revocation through DAG
- `CapTable`: per-process capability storage (fixed-size array)
- `ffi.rs`: `extern "C"` wrappers for kernel-libs C interop
- Tests: Kani proof harnesses + unit tests

### Phase 3: trx-abi — syscall ABI definitions (~500 LOC, Week 3-4)

Depends on trx-cap. `#![no_std]`.

- 119 syscall numbers across all 4 ranges (23 shared + 82 TRX + 7 RT + 7 Hermetica) (mirrors genesis_syscall.h)
- POSIX errno constants (mirrors genesis_result.h errno mapping)
- Type aliases: `pid_t = i64`, `off_t = i64`, `mode_t = u32`, etc.
- `struct stat` layout matching TerranoxOS kernel
- Signal number definitions

### Phase 4: trx-sigil — cryptographic signing (~600 LOC, Week 4-5)

Depends on trx-abi. `#![no_std]`.

- Ed25519 signing (for SigilVM program signatures)
- Signature verification
- Key management (keyring)
- Uses kernel-libs crypto (SHA-256) or self-contained Ed25519 implementation

### Phase 5: trx-store — content-addressed storage (~400 LOC, Week 5-6)

Depends on trx-abi. `#![no_std]` optional, `alloc` required.

- BLAKE3 hashing (self-contained or wrapping an existing implementation)
- Content-addressed blob store
- Integrity verification on read

### Phase 6: trx-idl — capability-safe IDL (~2,000 LOC, Week 6-8)

Depends on trx-abi + trx-cap. Uses `std` (build-time tool, not no_std).

- `.tidl` file parser
- Rust client/server stub generation
- IDL type system that maps to trx-cap capabilities
- Each IPC interface method declares required capabilities

### Phase 7: trx-test — shared test framework (~500 LOC, Week 8-9)

Depends on all other crates (dev-dependency).

- Test runner for `no_std` environments (serial output)
- Mock objects: file descriptors, capabilities, syscall responses
- Custom assertions with detailed output
- Works with `cargo test` on host and serial output on kernel

### Phase 8: Wire submodules (Week 9-10)

- Add `terranox-crates` as git submodule in terranox, hermetica, sigilvm
- Update each project's Cargo.toml to include crates/* as workspace members
- Verify CI passes in all projects with the new submodule

---

## 7. CI for terranox-crates

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --check

  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with: { components: miri }
      - run: cargo +nightly miri test -p trx-cap

  cross:
    strategy:
      matrix:
        target: [x86_64-unknown-none, aarch64-unknown-none, riscv64gc-unknown-none-elf]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: "${{ matrix.target }}" }
      - run: cargo build --workspace --target ${{ matrix.target }}
```

---

## 8. Verification tools per crate

| Crate | Verification | What it proves |
|-------|-------------|---------------|
| trx-cap | Kani proofs | cap_derive never escalates privileges for ALL inputs |
| trx-abi | Size assertions | Struct sizes match C ABI (compile-time _Static_assert equivalent) |
| trx-sigil | Test vectors | Ed25519 signatures match RFC 8032 vectors |
| trx-store | Test vectors | BLAKE3 hashes match reference implementation |
| trx-idl | Round-trip tests | Parse → generate → re-parse produces identical AST |
| trx-test | Self-test | Framework tests itself (meta-testing) |
| All | Miri | No undefined behavior in unsafe code |
| All | svmcheck | Bytecode + contract verification (when integrated with SigilVM) |
