<!--
SPDX-License-Identifier: CC-BY-4.0
-->

# Zig for trx-libc: Technical Reference

*March 2026 — Detailed reference for implementing TerranoxOS POSIX libc in Zig*

---

## 1. C ABI exports — how Zig produces libc symbols

Zig's `export fn` emits a symbol with the platform C calling convention, no decoration. The resulting `.a` archive is indistinguishable from one produced by a C compiler.

```zig
// src/unistd.zig
const syscall = @import("internal/syscall.zig");

export fn read(fd: c_int, buf: [*]u8, count: usize) isize {
    return syscall.ret(syscall.syscall3(.READ, @intCast(fd), @intFromPtr(buf), count));
}

export fn write(fd: c_int, buf: [*]const u8, count: usize) isize {
    return syscall.ret(syscall.syscall3(.WRITE, @intCast(fd), @intFromPtr(buf), count));
}

export fn close(fd: c_int) c_int {
    return @intCast(syscall.ret(syscall.syscall1(.CLOSE, @intCast(fd))));
}
```

This produces symbols `read`, `write`, `close` in the archive — identical to what musl or glibc would export. Rust links against them via `extern "C" { fn read(...) -> isize; }` or through its std library.

**Producing a static archive:**

```bash
# Command-line
zig build-lib src/libc.zig -target x86_64-freestanding-none -O ReleaseSafe

# Or via build.zig
const lib = b.addStaticLibrary(.{
    .name = "c",
    .root_source_file = b.path("src/libc.zig"),
    .target = target,
    .optimize = .ReleaseSafe,
});
b.installArtifact(lib);  // produces zig-out/lib/libc.a
```

**Reference**: [Zig Language Reference — export](https://ziglang.org/documentation/0.15.0/#export)

---

## 2. Consuming genesis-abi C headers with @cImport

`@cImport` translates C headers at compile time — no bindings generator, no FFI boilerplate. The Zig compiler invokes clang's parser internally.

```zig
// src/internal/syscall.zig
const c = @cImport({
    @cDefine("__STDC_HOSTED__", "0"); // required for freestanding headers
    @cInclude("genesis_syscall.h");
    @cInclude("genesis_result.h");
});

// Now use c.GEN_SYS_READ, c.GEN_SYS_WRITE, etc. directly
pub const SyscallNr = enum(u32) {
    EXIT          = c.GEN_SYS_EXIT,
    WRITE         = c.GEN_SYS_WRITE,
    READ          = c.GEN_SYS_READ,
    MMAP          = c.GEN_SYS_MMAP,
    MUNMAP        = c.GEN_SYS_MUNMAP,
    YIELD         = c.GEN_SYS_YIELD,
    GETPID        = c.GEN_SYS_GETPID,
    SLEEP         = c.GEN_SYS_SLEEP,
    CLOCK_GETTIME = c.GEN_SYS_CLOCK_GETTIME,
    OPEN          = c.GEN_SYS_OPEN,
    CLOSE         = c.GEN_SYS_CLOSE,
    STAT          = c.GEN_SYS_STAT,
    FSTAT         = c.GEN_SYS_FSTAT,
    LSEEK         = c.GEN_SYS_LSEEK,
    // ... all 119 syscalls from genesis_syscall.h (82 TRX + 23 shared + 7 RT + 7 Hermetica)
};
```

This eliminates manual constant duplication. The Zig libc reads syscall numbers directly from the C headers that are the ABI source of truth. If the kernel adds or renumbers syscalls, the Zig code picks it up automatically.

**In build.zig, specify include paths:**

```zig
lib.addIncludePath(b.path("include"));
lib.addIncludePath(b.path("../genesis-abi/include"));
lib.addIncludePath(b.path("../primitives/include"));
```

**Reference**: [Zig Language Reference — @cImport](https://ziglang.org/documentation/0.15.0/#cImport)

---

## 3. Freestanding compilation

Zig does not have a `#![no_std]` annotation. The target determines what's available. When targeting `freestanding`, most of `std` still works — only OS-dependent parts (std.os, std.fs, std.net, std.heap.page_allocator) are unavailable.

```zig
// build.zig
const target = b.resolveTargetQuery(.{
    .cpu_arch = .x86_64,
    .os_tag = .freestanding,  // no OS assumptions
    .abi = .none,             // no C runtime
});

const lib = b.addStaticLibrary(.{
    .name = "c",
    .root_source_file = b.path("src/libc.zig"),
    .target = target,
    .optimize = .ReleaseSafe,
});

// Kernel/freestanding settings
lib.root_module.red_zone = false;        // interrupts would clobber it
lib.root_module.stack_protector = false;  // no __stack_chk_fail available
```

**What works in freestanding:**

| Module | Available | Notes |
|--------|-----------|-------|
| `std.mem` | Yes | memcpy, memset, alignment, slices |
| `std.math` | Yes | all math functions |
| `std.fmt` | Yes | format strings (printf engine) |
| `std.hash` | Yes | CRC-32, wyhash, etc. |
| `std.sort` | Yes | insertion sort, block sort |
| `std.unicode` | Yes | UTF-8 encode/decode |
| `std.os` | **No** | requires OS syscalls |
| `std.fs` | **No** | requires OS filesystem |
| `std.net` | **No** | requires OS networking |
| `std.heap.page_allocator` | **No** | requires mmap-equivalent |

**Custom OS tag** (for when TerranoxOS has its own target):

Zig supports `.os_tag = .other` with a custom OS package provided at build time. This allows defining TerranoxOS-specific behaviors (signal handling, TLS model) without patching the Zig compiler.

**Reference**: [OSDev Wiki — Zig Bare Bones](https://wiki.osdev.org/Zig_Bare_Bones), [Zig GitHub Issue #3784 — Custom OS targets](https://github.com/ziglang/zig/issues/3784)

---

## 4. Inline assembly for SYSCALL stubs

The syscall dispatch layer is the most critical piece. Zig's inline assembly syntax differs from GCC but maps to the same LLVM backend.

### x86_64 syscall wrappers

Adapted from Zig's own `lib/std/os/linux/x86_64.zig`:

```zig
// src/arch/x86_64.zig

pub fn syscall0(number: SyscallNr) usize {
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> usize),
        : [number] "{rax}" (@intFromEnum(number)),
        : "rcx", "r11", "memory"
    );
}

pub fn syscall1(number: SyscallNr, arg1: usize) usize {
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> usize),
        : [number] "{rax}" (@intFromEnum(number)),
          [arg1] "{rdi}" (arg1),
        : "rcx", "r11", "memory"
    );
}

pub fn syscall2(number: SyscallNr, arg1: usize, arg2: usize) usize {
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> usize),
        : [number] "{rax}" (@intFromEnum(number)),
          [arg1] "{rdi}" (arg1),
          [arg2] "{rsi}" (arg2),
        : "rcx", "r11", "memory"
    );
}

pub fn syscall3(number: SyscallNr, arg1: usize, arg2: usize, arg3: usize) usize {
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> usize),
        : [number] "{rax}" (@intFromEnum(number)),
          [arg1] "{rdi}" (arg1),
          [arg2] "{rsi}" (arg2),
          [arg3] "{rdx}" (arg3),
        : "rcx", "r11", "memory"
    );
}

pub fn syscall4(number: SyscallNr, arg1: usize, arg2: usize, arg3: usize,
                arg4: usize) usize {
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> usize),
        : [number] "{rax}" (@intFromEnum(number)),
          [arg1] "{rdi}" (arg1),
          [arg2] "{rsi}" (arg2),
          [arg3] "{rdx}" (arg3),
          [arg4] "{r10}" (arg4),  // r10 NOT rcx — SYSCALL clobbers rcx
        : "rcx", "r11", "memory"
    );
}

pub fn syscall5(number: SyscallNr, arg1: usize, arg2: usize, arg3: usize,
                arg4: usize, arg5: usize) usize {
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> usize),
        : [number] "{rax}" (@intFromEnum(number)),
          [arg1] "{rdi}" (arg1),
          [arg2] "{rsi}" (arg2),
          [arg3] "{rdx}" (arg3),
          [arg4] "{r10}" (arg4),
          [arg5] "{r8}" (arg5),
        : "rcx", "r11", "memory"
    );
}

pub fn syscall6(number: SyscallNr, arg1: usize, arg2: usize, arg3: usize,
                arg4: usize, arg5: usize, arg6: usize) usize {
    return asm volatile ("syscall"
        : [ret] "={rax}" (-> usize),
        : [number] "{rax}" (@intFromEnum(number)),
          [arg1] "{rdi}" (arg1),
          [arg2] "{rsi}" (arg2),
          [arg3] "{rdx}" (arg3),
          [arg4] "{r10}" (arg4),
          [arg5] "{r8}" (arg5),
          [arg6] "{r9}" (arg6),
        : "rcx", "r11", "memory"
    );
}
```

### Register mapping (from TRX-DOC-1000-syscall-abi-reference.md Part II)

| Register | SYSCALL purpose | Notes |
|----------|----------------|-------|
| `rax` | Syscall number (in) / return value (out) | Kernel overwrites with result |
| `rdi` | 1st argument | Same as System V ABI |
| `rsi` | 2nd argument | Same as System V ABI |
| `rdx` | 3rd argument | Same as System V ABI |
| `r10` | 4th argument | **Differs from SysV** (SysV uses `rcx`) |
| `r8` | 5th argument | Same as System V ABI |
| `r9` | 6th argument | Same as System V ABI |
| `rcx` | **Clobbered** | Hardware saves RIP here |
| `r11` | **Clobbered** | Hardware saves RFLAGS here |

### Errno translation

The kernel returns -errno in rax (per TRX-DOC-1000-syscall-abi-reference.md line 158-162). The libc translation is trivial:

```zig
// src/internal/errno.zig
pub fn ret(raw: usize) isize {
    const signed: isize = @bitCast(raw);
    if (signed > -4096 and signed < 0) {
        getErrnoPtr().* = @intCast(-signed);
        return -1;
    }
    return signed;
}
```

**Reference**: [Zig Language Reference — Assembly](https://ziglang.org/documentation/0.15.0/#Assembly), [Zig std source — lib/std/os/linux/x86_64.zig](https://github.com/ziglang/zig/blob/master/lib/std/os/linux/x86_64.zig)

---

## 5. Bazel integration (rules_zig)

`rules_zig` 0.12.3 is on the Bazel Central Registry and supports bzlmod.

### MODULE.bazel addition

```starlark
# Add to existing kernel-libs MODULE.bazel
bazel_dep(name = "rules_zig", version = "0.12.3")

zig = use_extension("@rules_zig//zig:extensions.bzl", "zig")
zig.toolchain(zig_version = "0.15.2")
use_repo(zig, "zig_sdk")
register_toolchains("@zig_sdk//:toolchain")
```

### libc/BUILD.bazel

```starlark
load("@rules_zig//zig:defs.bzl", "zig_static_library", "zig_test")

package(default_visibility = ["//visibility:public"])

zig_static_library(
    name = "trx_libc",
    main = "src/libc.zig",
    c_include_dirs = [
        "include",
        "//genesis-abi:include",
        "//primitives:include",
    ],
    target = select({
        "@platforms//cpu:x86_64":  "x86_64-freestanding-none",
        "@platforms//cpu:aarch64": "aarch64-freestanding-none",
        "@platforms//cpu:riscv64": "riscv64-freestanding-none",
    }),
    extra_args = ["-fno-stack-protector", "-mno-red-zone"],
)

zig_test(
    name = "trx_libc_test",
    main = "src/libc.zig",
    c_include_dirs = [
        "include",
        "//genesis-abi:include",
        "//primitives:include",
    ],
)

# C headers for consumers
cc_library(
    name = "trx_libc_headers",
    hdrs = glob(["include/**/*.h"]),
    includes = ["include"],
)
```

**Compatibility**: rules_zig works alongside the existing rules_cc (0.2.17) and rules_rust (0.69.0) in the kernel-libs MODULE.bazel.

**Reference**: [rules_zig on BCR](https://registry.bazel.build/modules/rules_zig), [GitHub — aherrmann/rules_zig](https://github.com/aherrmann/rules_zig)

---

## 6. comptime lookup tables

Zig's `comptime` evaluates arbitrary code at compile time. Lookup tables are generated with zero runtime cost and guaranteed correctness.

### CRC-32 table

```zig
// src/crypto/crc32.zig
const crc32_table: [256]u32 = comptime blk: {
    @setEvalBranchQuota(10000);
    var table: [256]u32 = undefined;
    for (0..256) |i| {
        var crc: u32 = @intCast(i);
        for (0..8) |_| {
            crc = if (crc & 1 != 0) (crc >> 1) ^ 0xEDB88320 else crc >> 1;
        }
        table[i] = crc;
    }
    break :blk table;
};

export fn gen_crc32(data: [*]const u8, len: usize) u32 {
    var crc: u32 = 0xFFFFFFFF;
    for (data[0..len]) |byte| {
        const idx = (crc ^ byte) & 0xFF;
        crc = (crc >> 8) ^ crc32_table[idx];
    }
    return crc ^ 0xFFFFFFFF;
}
```

### ctype table

```zig
// src/ctype.zig
const UPPER: u8 = 0x01;
const LOWER: u8 = 0x02;
const ALPHA: u8 = 0x04;
const DIGIT: u8 = 0x08;
const SPACE: u8 = 0x10;
const PRINT: u8 = 0x20;
const PUNCT: u8 = 0x40;
const CNTRL: u8 = 0x80;

const ctype_table: [256]u8 = comptime blk: {
    var t = [_]u8{0} ** 256;
    for ('A'..('Z' + 1)) |c| { t[c] |= UPPER | ALPHA | PRINT; }
    for ('a'..('z' + 1)) |c| { t[c] |= LOWER | ALPHA | PRINT; }
    for ('0'..('9' + 1)) |c| { t[c] |= DIGIT | PRINT; }
    t[' '] |= SPACE | PRINT;
    t['\t'] |= SPACE; t['\n'] |= SPACE;
    t['\r'] |= SPACE; t['\x0b'] |= SPACE; t['\x0c'] |= SPACE;
    // ... punctuation, control characters
    break :blk t;
};

export fn isalpha(c: c_int) c_int {
    if (c < 0 or c > 255) return 0;
    return if (ctype_table[@intCast(c)] & ALPHA != 0) @as(c_int, 1) else 0;
}
```

**Reference**: [Zig Language Reference — comptime](https://ziglang.org/documentation/0.15.0/#comptime)

---

## 7. Cross-compilation (all targets from one binary)

Zig bundles LLVM backends for all targets. No separate toolchain installation needed.

```zig
// build.zig — multi-target build
const targets = [_]std.Target.Query{
    .{ .cpu_arch = .x86_64,  .os_tag = .freestanding, .abi = .none },
    .{ .cpu_arch = .aarch64, .os_tag = .freestanding, .abi = .none },
    .{ .cpu_arch = .riscv64, .os_tag = .freestanding, .abi = .none },
    .{ .cpu_arch = .thumb,   .os_tag = .freestanding, .abi = .eabi },
};

pub fn build(b: *std.Build) void {
    for (targets) |t| {
        const resolved = b.resolveTargetQuery(t);
        const lib = b.addStaticLibrary(.{
            .name = b.fmt("c-{s}", .{@tagName(t.cpu_arch)}),
            .root_source_file = b.path("src/libc.zig"),
            .target = resolved,
            .optimize = .ReleaseSafe,
        });
        lib.root_module.red_zone = false;
        lib.root_module.stack_protector = false;

        // Disable SSE/AVX for kernel-context code
        if (t.cpu_arch == .x86_64) {
            lib.root_module.code_model = .kernel;
        }

        b.installArtifact(lib);
    }
}
```

```bash
# Single command builds all targets
zig build

# Outputs:
# zig-out/lib/libc-x86_64.a
# zig-out/lib/libc-aarch64.a
# zig-out/lib/libc-riscv64.a
# zig-out/lib/libc-thumb.a
```

**Reference**: [Zig cross-compilation documentation](https://ziglang.org/documentation/0.15.0/#Cross-compilation)

---

## 8. Thread-local storage (errno)

### The problem

In freestanding mode, Zig's `threadlocal` keyword does **not work** because there is no OS to set up TLS segments. The libc must bootstrap TLS itself.

### The solution: manual TLS via architecture registers

On x86_64, the FS segment register points to a Thread Control Block (TCB) allocated by the libc:

```zig
// src/internal/tls.zig

pub const ThreadControlBlock = extern struct {
    self: *ThreadControlBlock,  // self-pointer (required by ABI)
    errno: c_int,
    stack_base: usize,
    stack_size: usize,
    tid: i64,
    // ... additional thread-local state
};

/// Get the current thread's errno pointer.
/// Called by __errno_location() (glibc/musl compat) and internally.
pub fn getErrnoPtr() *c_int {
    const tcb: *ThreadControlBlock = asm ("mov %%fs:0, %[ret]"
        : [ret] "=r" (-> *ThreadControlBlock),
    );
    return &tcb.errno;
}

/// Exported for C compatibility (programs that call __errno_location)
export fn __errno_location() *c_int {
    return getErrnoPtr();
}

/// Initialize TLS for the main thread (called from crt0)
pub fn initMainThreadTls() void {
    // 1. Allocate TCB via trx_mem_alloc
    // 2. Copy .tdata from ELF PT_TLS segment
    // 3. Zero .tbss section
    // 4. Set FS base via wrfsbase instruction
}

/// Initialize TLS for a new pthread (called from pthread_create wrapper)
pub fn initThreadTls(stack_base: usize, stack_size: usize) *ThreadControlBlock {
    // Same as above but for a new thread
}
```

### AArch64 equivalent

```zig
// TPIDR_EL0 register holds the TCB pointer on AArch64
fn getErrnoPtr() *c_int {
    const tcb: *ThreadControlBlock = asm ("mrs %[ret], tpidr_el0"
        : [ret] "=r" (-> *ThreadControlBlock),
    );
    return &tcb.errno;
}
```

### Alternative: `-fsingle-threaded`

For early bootstrap (Phase 0), before threading is implemented, compile with `-fsingle-threaded`. This converts `threadlocal` to plain globals. Replace with proper TLS in Phase 3 when pthreads are added.

**Reference**: [Zig Language Reference — threadlocal](https://ziglang.org/documentation/0.15.0/#threadlocal), System V ABI AMD64 supplement (TLS chapter)

---

## 9. Zig version and stability

| Version | Status | Release date |
|---------|--------|-------------|
| **0.15.2** | Latest stable | 2025-10-11 |
| 0.16.0-dev | Development | Expected stable ~Q1 2026 |
| 1.0 | Planned | No date announced |

**Recommendation**: Pin to **Zig 0.15.2** in MODULE.bazel via rules_zig. This matches the approach of pinning Rust to 1.84.0.

**Breaking changes between versions** (notable):
- 0.15 removed `usingnamespace` (use explicit imports instead)
- 0.15 removed `async`/`await` (redesigned I/O model)
- 0.15 changed inline asm clobber syntax to struct-based
- 0.14 → 0.15 changed `std.mem.Allocator` interface

Lock the version and update deliberately.

**Reference**: [Zig releases](https://ziglang.org/download/), [Zig 0.15.0 release notes](https://ziglang.org/download/0.15.0/release-notes.html)

---

## 10. Reference projects

| Project | URL | Relevance |
|---------|-----|-----------|
| **ziglibc** | [github.com/marler8997/ziglibc](https://github.com/marler8997/ziglibc) | Closest reference: C standard + POSIX in Zig, exports C ABI, can bootstrap GCC |
| **Pluto OS** | [github.com/andrewrk/pluto](https://github.com/andrewrk/pluto) | x86 microkernel in Zig, freestanding patterns |
| **Zig std.os.linux** | In Zig source tree: `lib/std/os/linux/` | Canonical syscall asm patterns for all architectures |
| **Zig std.c** | In Zig source tree: `lib/std/c.zig` | How Zig exports libc-compatible symbols |
| **mlibc** | [github.com/managarm/mlibc](https://github.com/managarm/mlibc) | C libc with pluggable sysdeps layer — architecture reference (even though it's C) |

---

## 11. Zig vs C for libc implementation

| Aspect | C | Zig |
|--------|---|-----|
| Syscall number source | Manual `#define` or `#include` | `@cImport("genesis_syscall.h")` — auto-synced |
| Lookup tables | Runtime init or manual pre-computed arrays | `comptime` — guaranteed compile-time, zero-cost |
| Buffer overflow | Manual bounds checks everywhere | Slices with bounds checking by default |
| String handling | Null-terminated `char*`, error-prone | Length-tracked slices, explicit sentinel `[:0]` |
| Cross-compile | Separate toolchain per target | Single `zig build` for all targets |
| printf engine | Rewrite from kfmt or port musl's | `std.fmt` available in freestanding, or custom |
| malloc | Manual implementation | Manual implementation (same effort) |
| Test framework | Custom ASSERT macro | Built-in `test` blocks with `zig test` |
| Formal verification | Frama-C ACSL annotations | Not supported (Frama-C is C-only) |
| Build system | Bazel rules_cc (existing) | Bazel rules_zig (0.12.3 on BCR) or zig build |
| Compile speed | Fast (clang) | Fast (LLVM, caches aggressively) |
| Debug experience | GDB, LLDB | GDB, LLDB (full DWARF support) |
| Ecosystem maturity | 50+ years | Pre-1.0, ~8 years |

### What Zig gains

- **Safety in implementation**: Zig's slice bounds checking, optional types, and explicit error handling catch bugs that C implementations historically get wrong (buffer overflows in printf, off-by-one in string functions, integer overflow in malloc bookkeeping)
- **@cImport eliminates drift**: No manual `#define` synchronization between kernel-libs C headers and libc constants
- **comptime replaces codegen**: CRC tables, ctype tables, and syscall number enums are compile-time computed
- **Single toolchain**: One `zig` binary cross-compiles to all TerranoxOS targets

### What Zig loses

- **No Frama-C**: Cannot formally verify the libc with ACSL annotations (Frama-C only works on C). WP proofs on critical functions (malloc, memcpy) would need a separate verification strategy
- **Pre-1.0 risk**: Breaking changes between Zig versions require active maintenance
- **Smaller ecosystem**: Fewer examples of production Zig libc implementations compared to C
