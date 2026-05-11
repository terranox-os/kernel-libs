<!--
SPDX-License-Identifier: CC-BY-4.0
-->

# TerranoxOS Ecosystem — Shared Infrastructure Architecture Reference

> **Status: SUPERSEDED.** This was the original v1.0 architecture specification. Multiple sections are stale (CapSet u32 → TrxCapSet 128-bit, sequential syscall numbers → hex subsystem blocks, gcc → clang, Makefile → Bazel). **Do not use for implementation decisions.**
>
> Current authoritative documents:
> - `../plans/TRX-DOC-1100-shared-infra-plan.md` — repo structure and shared crate design
> - `../plans/TRX-DOC-1101-libc-plan.md` — Zig POSIX libc design
> - `TRX-DOC-1000-syscall-abi-reference.md` — 119-syscall ABI reference
> - `genesis-abi/include/` — canonical ABI types (C headers = source of truth)
>
> Kept for historical context only.

Version 2.0 — March 2026

---

## 1. The Four Projects

```
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   TerranoxOS    │  │  GenesisOS-RT   │  │   HermeticaOS   │  │    SigilVM      │
│                 │  │                 │  │                 │  │                 │
│ Desktop OS      │  │ Robotics RTOS   │  │ Hot-swap VMM    │  │ Verified        │
│ Capability-based│  │ Real-time       │  │ Module kernel   │  │ Bytecode OS     │
│                 │  │                 │  │                 │  │                 │
│ Lang: C + Rust  │  │ Lang: C + Rust  │  │ Lang: C + Rust  │  │ Lang: Zig + Rust│
│ Target: x86-64  │  │ Target: ARM     │  │ Target: x86-64  │  │ Target: x86-64  │
│ Repo: terranox  │  │ Repo: genesis-rt│  │ Repo: hermetica │  │ Repo: sigilvm   │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │                    │
         │                    │                    │                    │
         └──────────┬─────────┴──────────┬─────────┘                   │
                    │                    │                              │
                    ▼                    ▼                              ▼
         ┌────────────────────────────────────────────────────────────────┐
         │                    SHARED INFRASTRUCTURE                      │
         │                                                               │
         │  kernel-libs (C)     — memory, strings, formatting, PMM       │
         │  trx-cap (Rust)      — capability system                      │
         │  trx-abi (Rust)      — syscall ABI definitions                │
         │  trx-idl (Rust)      — capability-safe IDL                    │
         │  trx-store (Rust)    — content-addressed storage              │
         │  trx-sigil (Rust)    — cryptographic signing                  │
         │  trx-test (Rust)     — shared test framework                  │
         │  Error code scheme   — gap-based -1..-98                      │
         └───────────────────────────────────────────────────────────────┘
```

---

## 2. Shared Layer: kernel-libs (C)

The foundational C library shared by ALL four kernels. Compiled as a static archive (`.a`) and linked into each kernel binary. No external dependencies. Freestanding C only.

### 2.1 Module Layout

```
kernel-libs/
├── include/
│   ├── gen_types.h          # uint8_t..uint64_t, bool, NULL, size_t
│   ├── gen_mem.h            # memcpy, memset, memmove, memcmp
│   ├── gen_str.h            # strnlen, strncmp, strncpy
│   ├── gen_kfmt.h           # gen_kprintf (callback-based, no FILE*)
│   ├── gen_pmm.h            # Physical memory manager (bitmap allocator)
│   ├── gen_err.h            # Error code scheme (gap-based)
│   ├── gen_cap.h            # Capability types + cap_check/cap_derive (C API)
│   ├── gen_hash.h           # BLAKE3 content-addressed hashing
│   └── gen_assert.h         # KASSERT macro (serial + halt)
│
├── src/
│   ├── gen_mem.c            # memcpy (word-aligned fast path), memset, memmove, memcmp
│   ├── gen_str.c            # strnlen, strncmp, strncpy
│   ├── gen_kfmt.c           # gen_kprintf (supports %d, %x, %s, %p, %lu, %lx)
│   ├── gen_pmm.c            # Bitmap PMM: init, alloc_frame, free_frame
│   ├── gen_cap.c            # cap_check, cap_derive, cap_revoke
│   └── gen_hash.c           # BLAKE3 (from reference implementation, trimmed)
│
├── frama-c/
│   ├── cap_proofs.c         # ACSL annotations for cap_check/cap_derive
│   └── pmm_proofs.c         # ACSL annotations for PMM invariants
│
├── tests/
│   ├── test_mem.c           # Host-native tests (clang -fsanitize=address)
│   ├── test_str.c
│   ├── test_kfmt.c
│   ├── test_pmm.c
│   └── test_cap.c
│
├── Makefile                 # Builds libkernel.a for each target
└── README.md
```

### 2.2 Build Targets

```
# TerranoxOS, HermeticaOS, SigilVM (x86-64 freestanding)
make TARGET=x86_64 CC=clang CFLAGS="-ffreestanding -mno-red-zone -mcmodel=kernel -fno-pic"
→ libkernel_x86_64.a

# GenesisOS-RT (AArch64 freestanding — Raspberry Pi 5)
make TARGET=aarch64 CC=aarch64-none-elf-clang CFLAGS="-ffreestanding -mcpu=cortex-a76"
→ libkernel_aarch64.a

# GenesisOS-RT (ARM Cortex-M — STM32)
make TARGET=cortex_m CC=arm-none-eabi-clang CFLAGS="-ffreestanding -mcpu=cortex-m4 -mthumb"
→ libkernel_cortex_m.a

# Host-native tests (Linux, for development)
make test CC=clang CFLAGS="-g -fsanitize=address,undefined"
→ runs all tests with ASAN + UBSAN
```

### 2.3 Who Uses What

```
                          TerranoxOS  GenesisOS-RT  HermeticaOS  SigilVM
                          ──────────  ────────────  ───────────  ───────
gen_mem (memcpy/set/move)     ✓            ✓            ✓          ✓
gen_str (strnlen/strncmp)     ✓            ✓            ✓          ✓
gen_kprintf                   ✓            ✓            ✓          ✓
gen_pmm                       ✓            ✓            ✓          ✓
gen_cap                       ✓            ✓*           ✓          ✓**
gen_hash (BLAKE3)             ✓            ─            ✓          ✓
gen_assert (KASSERT)          ✓            ✓            ✓          ✓

* GenesisOS-RT uses simplified cap model (no transitive revocation)
** SigilVM checks caps at load time (verifier), not runtime
```

---

## 3. Shared Layer: Rust Crates

Six shared Rust crates consumed by TerranoxOS upper layers, HermeticaOS, and SigilVM (verifier). GenesisOS-RT does not use Rust crates (pure C + optional Rust for tooling only).

### 3.1 Crate Dependency Graph

```
                    ┌──────────┐
                    │ trx-test │  Test framework (dev-dependency only)
                    └────┬─────┘
                         │ (dev-depends on all below)
    ┌────────────────────┼────────────────────┐
    │                    │                    │
    ▼                    ▼                    ▼
┌─────────┐      ┌──────────┐        ┌───────────┐
│ trx-idl │      │trx-store │        │ trx-sigil │
│         │      │          │        │           │
│Cap-safe │      │Content-  │        │Crypto     │
│IDL for  │      │addressed │        │signing    │
│IPC      │      │storage   │        │(BLAKE3 +  │
│         │      │(BLAKE3)  │        │Ed25519)   │
└────┬────┘      └────┬─────┘        └─────┬─────┘
     │                │                    │
     │                ▼                    │
     │          ┌──────────┐               │
     └─────────►│ trx-abi  │◄──────────────┘
                │          │
                │Syscall # │
                │ABI defs  │
                │Error     │
                │codes     │
                └────┬─────┘
                     │
                     ▼
               ┌──────────┐
               │ trx-cap  │
               │          │
               │Capability│
               │types,    │
               │check,    │
               │derive,   │
               │revoke    │
               └──────────┘
```

### 3.2 Crate Details

#### trx-cap — Capability System

```
trx-cap/
├── Cargo.toml           # [no_std], no allocator required
├── src/
│   ├── lib.rs           # CapSet, CapError, CAP_* constants
│   ├── check.rs         # cap_check(pid, required) → Result<(), CapError>
│   ├── derive.rs        # cap_derive(parent, child, restricted) → enforces subset rule
│   ├── revoke.rs        # cap_revoke(pid, cap) → transitive revocation
│   ├── table.rs         # CapTable: per-process capability storage
│   └── ffi.rs           # extern "C" API for kernel-libs interop
└── tests/
    └── test_cap.rs      # Kani proof harnesses + unit tests
```

**Key types (updated v0.2.0 — 128-bit hierarchical model from PR #13):**
```rust
/// 128-bit domain-partitioned capability bitmask.
/// 12 domains, 40 leaf capabilities. Hierarchy resolved at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct TrxCapSet {
    pub lo: u64,  // domains 0-7: process, memory, thread, ipc, fs, io, display, input
    pub hi: u64,  // domains 8-11: gpu, net, time, system
}

// 40 leaf capabilities across 12 domains:
// lo[0..3]   process: create, signal, inspect, manage
// lo[4..7]   memory:  alloc, map, share, dma
// lo[8..10]  thread:  create, join, affinity
// lo[11..13] ipc:     channel, signal, event
// lo[14..17] fs:      read, write, create, delete
// lo[18..20] io:      port, irq, mmio
// lo[21..24] display: compositor, surface, buffer, mode
// lo[25..27] input:   keyboard, pointer, touch
// hi[0..2]   gpu:     render, compute, alloc
// hi[3..5]   net:     socket, bind, raw
// hi[6..8]   time:    read, sleep, timer
// hi[9..11]  system:  reboot, module, audit

pub fn cap_check(pid: u32, required: TrxCapSet) -> Result<(), CapError>;
pub fn cap_derive(parent: u32, child: u32, restricted: TrxCapSet) -> Result<(), CapError>;
```

> **Note:** This replaces the original `CapSet = u32` (10 capabilities) from v0.1.0.
> The C mirror is `TrxCapSet` in `genesis_module.h`. See `kernel-libs/genesis-abi/`.

**Consumed by:**
- TerranoxOS: every syscall dispatch checks caps (runtime)
- HermeticaOS: VM creation checks caps (runtime)
- SigilVM: verifier checks caps at program load time (static)
- trx-shell: launcher shows required caps before launching apps (UI)

#### trx-abi — Syscall ABI Definitions

```
trx-abi/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Re-exports
│   ├── syscall.rs       # Syscall numbers: SYS_READ=0, SYS_WRITE=1, ...
│   ├── errno.rs         # Error codes: EPERM=1, ENOENT=2, ...
│   ├── types.rs         # pid_t, uid_t, gid_t, off_t, size_t
│   ├── stat.rs          # struct stat layout (matches Linux for compat)
│   ├── ioctl.rs         # DRM ioctl numbers
│   └── signal.rs        # Signal numbers and structures
└── tests/
    └── test_abi.rs      # Verify struct sizes match Linux ABI
```

**Key constants:**
```rust
// Syscall numbers (x86-64, Linux-compatible)
pub const SYS_READ: usize = 0;
pub const SYS_WRITE: usize = 1;
pub const SYS_OPEN: usize = 2;
pub const SYS_CLOSE: usize = 3;
// ... all 91 desktop syscalls defined here

// Error codes
pub const EPERM: i32 = 1;
pub const ENOENT: i32 = 2;
pub const ENOSYS: i32 = 38;
// ...
```

**Consumed by:**
- TerranoxOS: syscall table references these numbers
- trx-shell: userspace uses these for raw syscall wrappers
- trx-compat: gap analysis tool compares against this list

#### trx-idl — Capability-Safe IDL

```
trx-idl/
├── Cargo.toml
├── src/
│   ├── lib.rs           # IDL parser + code generator
│   ├── parser.rs        # Parse .trx-idl interface definitions
│   ├── codegen.rs       # Generate Rust client/server stubs
│   └── types.rs         # IDL type system (maps to trx-cap)
└── examples/
    └── vfs.trx-idl      # Example: VFS interface definition
```

**Purpose:** Define IPC interfaces where capability requirements are part of the interface definition. When you define a VFS service, the IDL specifies which caps a client needs to call each method.

#### trx-store — Content-Addressed Storage

```
trx-store/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── hash.rs          # BLAKE3 hashing (wraps kernel-libs gen_hash)
│   ├── store.rs         # Content-addressed blob store
│   └── verify.rs        # Verify blob integrity on read
└── tests/
```

**Purpose:** Every binary, config file, and module is stored by its BLAKE3 hash. Detects corruption and tampering. Used by TerranoxOS package management and HermeticaOS module loading.

#### trx-sigil — Cryptographic Signing

```
trx-sigil/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── sign.rs          # Ed25519 signing (for Sigil program signatures)
│   ├── verify.rs        # Signature verification
│   └── keyring.rs       # Key management
└── tests/
```

**Purpose:** Programs on SigilVM are signed with Ed25519. TerranoxOS uses this for package verification. HermeticaOS uses it for module authentication.

#### trx-test — Shared Test Framework

```
trx-test/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── harness.rs       # Test runner for no_std environments
│   ├── mock.rs          # Mock file descriptors, mock capabilities
│   └── assert.rs        # Custom assertions with detailed output
└── tests/
```

**Purpose:** Shared test infrastructure. Runs on host (cargo test) and on kernel (serial output). Provides mock objects for testing kernel code without hardware.

---

## 4. Error Code Scheme

Gap-based error codes shared across ALL projects. Each range is reserved for a subsystem. No collisions between projects.

```
Range         Subsystem          Constants              Used By
──────────    ─────────────      ───────────────────    ──────────────────
-1  .. -10    General            GEN_ERR_UNKNOWN  -1    All four projects
                                 GEN_ERR_NOMEM    -2
                                 GEN_ERR_INVAL    -3
                                 GEN_ERR_BUSY     -4
                                 GEN_ERR_EXIST    -5
                                 GEN_ERR_NOENT    -6
                                 GEN_ERR_RANGE    -7
                                 GEN_ERR_NOSPC    -8
                                 GEN_ERR_IO       -9
                                 GEN_ERR_TIMEOUT  -10

-16 .. -18    Security           GEN_ERR_PERM     -16   All (cap_check)
                                 GEN_ERR_AUTH     -17   TerranoxOS, Hermetica
                                 GEN_ERR_REVOKED  -18   TerranoxOS, Hermetica

-32 .. -34    I/O                GEN_ERR_BADF     -32   All
                                 GEN_ERR_PIPE     -33   TerranoxOS
                                 GEN_ERR_AGAIN    -34   TerranoxOS, GenesisOS-RT

-48 .. -50    Format             GEN_ERR_FMT      -48   All
                                 GEN_ERR_UTF8     -49   TerranoxOS, SigilVM
                                 GEN_ERR_PARSE    -50   All

-64 .. -67    Module             GEN_ERR_MOD_LOAD -64   HermeticaOS
                                 GEN_ERR_MOD_VER  -65   HermeticaOS
                                 GEN_ERR_MOD_DEP  -66   HermeticaOS
                                 GEN_ERR_MOD_SIG  -67   HermeticaOS, SigilVM

-80 .. -82    Real-time          GEN_ERR_DEADLINE -80   GenesisOS-RT
                                 GEN_ERR_PRIORITY -81   GenesisOS-RT
                                 GEN_ERR_OVERRUN  -82   GenesisOS-RT

-96 .. -98    Syscall            GEN_ERR_NOSYS    -96   TerranoxOS
                                 GEN_ERR_FAULT    -97   TerranoxOS
                                 GEN_ERR_INTR     -98   TerranoxOS
```

### 4.1 How Error Codes Are Defined

In kernel-libs (C):
```c
// gen_err.h — authoritative source
#define GEN_ERR_UNKNOWN   (-1)
#define GEN_ERR_NOMEM     (-2)
#define GEN_ERR_PERM      (-16)
#define GEN_ERR_BADF      (-32)
// ...
```

In trx-abi (Rust):
```rust
// Generated from gen_err.h via bindgen, or manually mirrored
pub const GEN_ERR_UNKNOWN: i32 = -1;
pub const GEN_ERR_NOMEM: i32 = -2;
pub const GEN_ERR_PERM: i32 = -16;
// ...
```

In SigilVM (Zig):
```zig
// @cImport("gen_err.h") or manual mirror
pub const GEN_ERR_UNKNOWN: i32 = -1;
pub const GEN_ERR_NOMEM: i32 = -2;
pub const GEN_ERR_PERM: i32 = -16;
// ...
```

---

## 5. Language Boundaries

### 5.1 How Languages Interface

```
┌─────────────────────────────────────────────────────────────────────┐
│                        TerranoxOS                                   │
│                                                                     │
│  ┌──────────────────┐    extern "C"    ┌──────────────────────┐    │
│  │   kernel-libs    │◄────────────────►│  Rust upper layers    │    │
│  │   (C)            │    FFI bridge    │  (trx-cap, trx-abi,  │    │
│  │                  │                  │   VFS, scheduler,    │    │
│  │  gen_pmm.c       │  ← Rust calls → │   syscall dispatch)  │    │
│  │  gen_cap.c       │    C functions   │                      │    │
│  │  gen_kprintf.c   │                  │  #[no_std]           │    │
│  └──────────────────┘                  └──────────────────────┘    │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    trx-shell (Rust userspace)                 │   │
│  │  trx-bar · trx-launcher · trx-notify · trx-lock · trx-settings│  │
│  │                                                               │   │
│  │  wayland-client · smithay-client-toolkit · cairo-rs · zbus    │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                        SigilVM                                      │
│                                                                     │
│  ┌──────────────────┐    @cImport     ┌──────────────────────┐     │
│  │   kernel-libs    │◄───────────────►│  Zig kernel          │     │
│  │   (C)            │   Zig calls C   │  (boot, HAL, sched,  │     │
│  │                  │                 │   JIT, loader,       │     │
│  │                  │                 │   helpers)           │     │
│  └──────────────────┘                 └──────────┬───────────┘     │
│                                                   │                 │
│                                       extern "C"  │                 │
│                                                   ▼                 │
│                                       ┌──────────────────────┐     │
│                                       │  Rust verifier       │     │
│                                       │  (trx-cap, trx-sigil │     │
│                                       │   diagnostics.rs)    │     │
│                                       │                      │     │
│                                       │  #[no_std]           │     │
│                                       │  → libverifier.a     │     │
│                                       └──────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                       GenesisOS-RT                                   │
│                                                                     │
│  ┌──────────────────┐                                               │
│  │   kernel-libs    │  (C only — no Rust in the kernel)             │
│  │   (C)            │                                               │
│  │                  │  All kernel code is C:                        │
│  │  gen_pmm.c       │  scheduler, serial, CAN bus, IMU driver,     │
│  │  gen_cap.c       │  control loops, RT task management            │
│  │  gen_kprintf.c   │                                               │
│  └──────────────────┘                                               │
│                                                                     │
│  Rust is used only for HOST-SIDE tooling:                           │
│  trx-roboview (telemetry dashboard on laptop, not on robot)         │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                       HermeticaOS                                    │
│                                                                     │
│  ┌──────────────────┐    extern "C"    ┌──────────────────────┐    │
│  │   kernel-libs    │◄────────────────►│  Rust VMM layer      │    │
│  │   (C)            │    FFI bridge    │  (trx-cap,           │    │
│  │                  │                  │   VM lifecycle,      │    │
│  │                  │                  │   module hot-swap,   │    │
│  │                  │                  │   trx-sigil verify)  │    │
│  └──────────────────┘                  └──────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 FFI Bridge Pattern

Every language boundary uses the same pattern:

```c
// kernel-libs: gen_cap.h (C side)
int cap_check(uint32_t pid, uint32_t required_cap);
int cap_derive(uint32_t parent, uint32_t child, uint32_t restricted);
```

```rust
// trx-cap: ffi.rs (Rust side, calling C)
extern "C" {
    fn cap_check(pid: u32, required_cap: u32) -> i32;
    fn cap_derive(parent: u32, child: u32, restricted: u32) -> i32;
}

// Safe Rust wrapper
pub fn check(pid: u32, cap: CapSet) -> Result<(), CapError> {
    let ret = unsafe { cap_check(pid, cap) };
    if ret == 0 { Ok(()) } else { Err(CapError::from_raw(ret)) }
}
```

```zig
// SigilVM: caps.zig (Zig side, calling C)
const gen_cap = @cImport(@cInclude("gen_cap.h"));

pub fn capCheck(pid: u32, required: u32) !void {
    const ret = gen_cap.cap_check(pid, required);
    if (ret != 0) return error.PermissionDenied;
}
```

---

## 6. Unified Workspace Structure

### 6.1 Monorepo Layout

```
terranox/                            # Workspace root
├── kernel-libs/                     # SHARED: C library (all projects)
│   ├── include/                     # Headers
│   ├── src/                         # Implementation
│   ├── frama-c/                     # Formal verification proofs
│   └── tests/                       # Host-native tests (ASAN/UBSAN)
│
├── crates/                          # SHARED: Rust crates
│   ├── trx-cap/                     # Capability system
│   ├── trx-abi/                     # Syscall ABI definitions
│   ├── trx-idl/                     # Capability-safe IDL
│   ├── trx-store/                   # Content-addressed storage
│   ├── trx-sigil/                   # Cryptographic signing
│   └── trx-test/                    # Test framework
│
├── terranox-os/                     # PROJECT: Desktop OS
│   ├── kernel/
│   │   ├── boot/                    # Limine entry, GDT, IDT
│   │   ├── mm/                      # VMM, page tables
│   │   ├── syscall/                 # Syscall dispatch (91 handlers)
│   │   ├── vfs/                     # Virtual filesystem
│   │   ├── drm/                     # Display (DRM/KMS)
│   │   ├── net/                     # Networking stack
│   │   ├── sched/                   # Preemptive scheduler
│   │   └── sentinel/                # Security monitor (Rust)
│   ├── userspace/
│   │   ├── trx-shell/               # Desktop shell (Rust)
│   │   │   ├── trx-bar/             # Status bar
│   │   │   ├── trx-launcher/        # App launcher
│   │   │   ├── trx-notify/          # Notification daemon
│   │   │   ├── trx-lock/            # Screen locker
│   │   │   └── trx-settings/        # System settings
│   │   ├── trx-strata/              # Init system
│   │   └── trx-term/                # Terminal emulator (C)
│   ├── CLAUDE.md
│   ├── Cargo.toml                   # Workspace members
│   └── linker.ld
│
├── genesis-rt/                      # PROJECT: Robotics RTOS
│   ├── kernel/                      # Pure C kernel
│   │   ├── boot/                    # Limine (Pi 5) or STM32 startup
│   │   ├── sched/                   # Rate-monotonic RT scheduler
│   │   ├── drivers/
│   │   │   ├── serial/              # UART
│   │   │   ├── can/                 # CAN bus (bxCAN / MCP2515)
│   │   │   ├── imu/                 # MPU-6050 IMU
│   │   │   └── motor/               # PWM motor control
│   │   └── rt/                      # RT task management, WCET
│   ├── tools/
│   │   └── trx-roboview/            # Telemetry dashboard (Rust, runs on laptop)
│   ├── CLAUDE.md
│   └── Makefile                     # C-only build (no Cargo)
│
├── hermetica-os/                    # PROJECT: Hot-swap module kernel
│   ├── kernel/
│   │   ├── boot/                    # Limine entry
│   │   ├── vmm/                     # VT-x VMM (VMXON, VMCS, EPT)
│   │   ├── modules/                 # Hot-swap module loader
│   │   ├── mm/                      # Memory management
│   │   └── sentinel/                # VM security monitor
│   ├── CLAUDE.md
│   ├── Cargo.toml
│   └── linker.ld
│
├── sigilvm/                         # PROJECT: Verified bytecode OS
│   ├── kernel/
│   │   ├── zig/                     # Zig kernel (~60%)
│   │   │   ├── boot/                # Zig boot entry
│   │   │   ├── hal/                 # Hardware abstraction
│   │   │   ├── sched/               # Cooperative scheduler
│   │   │   ├── jit/                 # eBPF → x86-64 JIT compiler
│   │   │   ├── loader/              # ELF loader for bytecode
│   │   │   ├── helpers/             # 21 helper functions
│   │   │   └── maps/                # Ring, Hash, Array maps
│   │   └── rust/                    # Rust verifier (~40%)
│   │       ├── verifier/            # eBPF bytecode verifier
│   │       │   ├── analyze.rs       # Instruction-by-instruction analysis
│   │       │   ├── diagnostics.rs   # Rich error reporting
│   │       │   └── types.rs         # 10 register types
│   │       └── caps/                # Capability checking (trx-cap)
│   ├── programs/
│   │   └── sentinel/                # Bytecode security monitor
│   ├── CLAUDE.md
│   ├── build.zig                    # Zig orchestrates, calls cargo for Rust
│   └── Cargo.toml                   # Rust verifier crate
│
├── tools/                           # SHARED: Development tools
│   ├── addr.zig                     # Address debugger
│   ├── elf_inspect.zig              # ELF inspector
│   ├── svmcheck.zig                 # SigilVM verifier (host-native)
│   ├── trx-compat.py               # Library compatibility analyzer
│   ├── build.zig                    # Builds all Zig tools
│   └── README.md
│
├── docs/                            # SHARED: Documentation
│   ├── deep-dives/                  # 8 deep dive documents
│   │   ├── filesystem.md
│   │   ├── gpu_driver.md
│   │   ├── container_isolation.md
│   │   ├── networking.md
│   │   ├── hypervisor.md
│   │   ├── ebpf.md
│   │   ├── kernel_tracing.md
│   │   └── verified_bytecode_os.md
│   ├── cheat-sheets/
│   │   └── kernel_address_cheat_sheet.md
│   ├── plans/
│   │   ├── TERRANOX_MILESTONE_PLAN.md
│   │   ├── TERRANOX_COMPOSITOR_PLAN.md
│   │   ├── TERRANOX_FULL_DESKTOP_PLAN.md
│   │   ├── SIGILVM_MILESTONE_PLAN.md
│   │   └── TERRANOX_ISSUES.md
│   └── architecture/
│       ├── THIS_FILE.md             # You are here
│       ├── unified_abi_spec.md
│       └── color_system.md          # Purple (#8b5cf6) + Green (#5ce0b8)
│
├── forks/                           # Forked upstream projects
│   ├── trx-wlroots/                # wlroots + capability hooks
│   ├── trx-libinput/                # libinput + CAP_INPUT_RAW
│   ├── trx-foot/                    # foot + sandbox-aware spawning
│   ├── trx-pipewire/                # PipeWire + CAP_AUDIO
│   ├── trx-dbus/                    # dbus-broker + Sentinel logging
│   └── trx-musl/                    # musl libc for TerranoxOS sysroot
│
└── Cargo.toml                       # Root workspace (all Rust crates)
```

### 6.2 Root Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    # Shared crates
    "crates/trx-cap",
    "crates/trx-abi",
    "crates/trx-idl",
    "crates/trx-store",
    "crates/trx-sigil",
    "crates/trx-test",

    # TerranoxOS kernel + userspace
    "terranox-os/kernel",
    "terranox-os/userspace/trx-shell/trx-bar",
    "terranox-os/userspace/trx-shell/trx-launcher",
    "terranox-os/userspace/trx-shell/trx-notify",
    "terranox-os/userspace/trx-shell/trx-lock",
    "terranox-os/userspace/trx-shell/trx-settings",
    "terranox-os/userspace/trx-strata",

    # HermeticaOS kernel
    "hermetica-os/kernel",

    # SigilVM Rust verifier
    "sigilvm/kernel/rust/verifier",
    "sigilvm/kernel/rust/caps",

    # Host-side tools
    "genesis-rt/tools/trx-roboview",
]

[workspace.dependencies]
trx-cap   = { path = "crates/trx-cap" }
trx-abi   = { path = "crates/trx-abi" }
trx-idl   = { path = "crates/trx-idl" }
trx-store = { path = "crates/trx-store" }
trx-sigil = { path = "crates/trx-sigil" }
trx-test  = { path = "crates/trx-test" }
```

---

## 7. Build Flow

### 7.1 Per-Project Build Commands

```bash
# ─── Shared kernel-libs (C) ───
cd kernel-libs && make TARGET=x86_64
# → libkernel_x86_64.a

# ─── TerranoxOS (C kernel + Rust upper layers + Rust userspace) ───
cd terranox-os
cargo build --release --target x86_64-unknown-none    # kernel
cargo build --release                                  # userspace (trx-shell)
# Links against ../kernel-libs/libkernel_x86_64.a

# ─── GenesisOS-RT (C only) ───
cd genesis-rt
make TARGET=aarch64    # or TARGET=cortex_m for STM32
# Links against ../kernel-libs/libkernel_aarch64.a

# ─── HermeticaOS (C + Rust) ───
cd hermetica-os
cargo build --release --target x86_64-unknown-none
# Links against ../kernel-libs/libkernel_x86_64.a

# ─── SigilVM (Zig + Rust + C) ───
cd sigilvm
zig build -Doptimize=ReleaseSafe
# Zig calls: cargo build -p verifier --target x86_64-unknown-none
# Zig links: ../kernel-libs/libkernel_x86_64.a + libverifier.a

# ─── Shared tools (Zig) ───
cd tools
zig build
# → addr, elf, svmcheck
```

### 7.2 Dependency Flow at Build Time

```
                    ┌──────────────┐
                    │  kernel-libs │ ← Built first (C, make)
                    │  libkernel.a │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┬────────────┐
              │            │            │            │
              ▼            ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
        │TerranoxOS│ │GenesisRT │ │Hermetica │ │ SigilVM  │
        │ (cargo)  │ │ (make)   │ │ (cargo)  │ │(zig build│
        │          │ │          │ │          │ │ + cargo) │
        └──────────┘ └──────────┘ └──────────┘ └──────────┘
              │                         │            │
              │        ┌────────────────┘            │
              ▼        ▼                             ▼
        ┌────────────────┐                    ┌──────────┐
        │  Rust crates   │                    │  Rust    │
        │  (trx-cap,     │                    │ verifier │
        │   trx-abi,     │                    │ crate    │
        │   trx-sigil)   │                    └──────────┘
        └────────────────┘
```

---

## 8. Color System

Defined here as the authoritative reference for all UI components across the ecosystem.

### 8.1 Primary Palette

```
PURPOSE              HEX        USAGE
──────────────────   ────────   ─────────────────────────────────────
OS Identity          #8b5cf6    Shell prompt, workspace pills, active
(Purple)                        borders, brand logo, tab selectors,
                                dock indicators, file headers, code
                                highlights, window glow

Security / Trust     #5ce0b8    Sentinel status, capability badges,
(Green)                         build success (✓), test passed (ok),
                                cap_check() highlights, verified
                                notifications, granted permissions,
                                MEM usage bars

Danger / Denied      #e05050    Capability denied, errors, #PF,
(Red)                           connection refused, process killed

Warning              #f0a050    Warnings, approaching limits,
(Amber)                         unverified content, degraded state

Informational        #50a0f0    CPU usage, network activity, info
(Blue)                          messages, documentation links

Background           #0a0c10    Desktop background, deepest layer
Surface 1            #13161d    Window backgrounds
Surface 2            #1a1e28    Title bars, panel backgrounds
Surface 3            #222836    Hover states, elevated elements
Border               #2a3040    Default borders
Text Bright          #f0f2f5    Primary text (high contrast)
Text Normal          #d0d4dc    Body text
Text Dim             #606878    Secondary text, labels
Text Muted           #404858    Disabled, placeholder
```

### 8.2 Per-Project Accent Overrides

Each project uses the shared palette but may emphasize different secondary colors in its own UI:

```
TerranoxOS:     Purple primary + Green security     (desktop OS)
GenesisOS-RT:   Purple primary + Amber RT warnings  (real-time deadlines)
HermeticaOS:    Purple primary + Blue VM indicators  (virtualization)
SigilVM:        Purple primary + Green verified      (bytecode verification)
```

---

## 9. Shared Tool Chain

### 9.1 Host-Side Development Tools

```
Tool           Language   Purpose                           Used By
────────────   ────────   ─────────────────────────────     ──────────
addr           Zig        Address decomposer, PTE decoder   All
elf            Zig        ELF inspector, crash analysis     All
svmcheck       Zig        SigilVM bytecode verifier         SigilVM
trx-compat     Python     Library gap analysis              TerranoxOS
trx-roboview   Rust       Telemetry dashboard               GenesisOS-RT
```

### 9.2 Verification Tools

```
Tool           Target Code    What It Proves
────────────   ───────────    ─────────────────────────────────────
Frama-C/WP     kernel-libs    cap_derive subset rule, PMM invariants
                (C)           Buffer bounds, no UB

Kani           trx-cap        cap_derive no escalation for ALL inputs
               trx-abi        Struct size matches Linux ABI
                (Rust)

svmcheck       SigilVM        No uninit reads, no use-after-free,
               programs       no null deref, bounded loops,
                (eBPF)        ownership transfer correctness

ASAN/UBSAN     Host tests     Runtime memory safety, undefined behavior
                (C, Rust)

cargo miri     Rust crates    UB detection in unsafe Rust code
```

---

## 10. What Gets Shared vs What's Per-Project

```
                          SHARED                    PER-PROJECT
                          ──────                    ───────────
Memory functions          gen_mem (C)               Allocator strategy
String functions          gen_str (C)               String encoding choices
Formatted printing        gen_kprintf (C)           Output device (serial/VGA/net)
PMM                       gen_pmm (C)               VMM / page table format
Capability types          trx-cap (Rust)            Where checks are enforced
Error codes               gen_err.h (C)             Error handling policy
ABI definitions           trx-abi (Rust)            Syscall implementation
Content addressing        trx-store (Rust)          Storage backend
Signing                   trx-sigil (Rust)          Trust model
Color palette             #8b5cf6 / #5ce0b8         Per-project secondary accent
Dev tools                 addr, elf, svmcheck       Project-specific scripts
Deep dive docs            All 8 documents           Project-specific CLAUDE.md
Test framework            trx-test (Rust)           Test harnesses
```

---

## 11. Migration Path

### 11.1 Current State → Unified Workspace

```
TODAY (separate repos):
  github.com/terranox-os/terranox-os     ← exists
  github.com/terranox-os/genesis-os      ← exists (needs rename)
  github.com/terranox-os/hermetica-os    ← needs creation
  github.com/terranox-os/sigilvm         ← needs creation

OPTION A: Keep separate repos, share kernel-libs as git submodule
  Each repo has: git submodule add ../kernel-libs kernel-libs
  PRO: Independent release cycles, clean git history
  CON: Submodule management, version drift

OPTION B: Monorepo (terranox/ workspace above)
  Single repo, all four projects + shared code
  PRO: Atomic changes across shared code, one CI pipeline
  CON: Large repo, all contributors see all code

OPTION C: Hybrid (recommended)
  kernel-libs     → separate repo (git submodule in all four)
  crates/         → separate repo (Cargo workspace, git submodule)
  tools/          → separate repo
  Each OS project → own repo, includes shared repos as submodules
  PRO: Clean separation, shared code has its own versioning
  CON: More repos to manage (7 total)
```

### 11.2 Recommended: Option C (Hybrid)

```
Repos:
  terranox-kernel-libs    (shared C)
  terranox-crates         (shared Rust: trx-cap, trx-abi, ...)
  terranox-tools          (addr, elf, svmcheck, trx-compat)
  terranox-os             (desktop OS, submodules kernel-libs + crates)
  genesis-rt              (robotics, submodule kernel-libs only)
  hermetica-os            (VMM, submodules kernel-libs + crates)
  sigilvm                 (verified OS, submodules kernel-libs + crates)
```

Each project's git structure:
```
terranox-os/
├── .gitmodules
│   kernel-libs = https://github.com/terranox-os/terranox-kernel-libs
│   crates      = https://github.com/terranox-os/terranox-crates
├── kernel-libs/          ← submodule
├── crates/               ← submodule
├── kernel/               ← project-specific
├── userspace/            ← project-specific
└── Cargo.toml
```