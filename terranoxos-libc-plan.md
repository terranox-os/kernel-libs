# TerranoxOS Custom POSIX C Library (trx-libc)

*March 2026 — Design plan for a from-scratch POSIX.1-2017 C library*

---

## Context

TerranoxOS is not Linux-compatible. It has 82 TerranoxOS-specific syscalls across 12 subsystem blocks (0x0100-0x01BF), plus 23 shared syscalls (0x0000-0x0016) = 105 usable by TerranoxOS. Capability-based security (no UIDs), custom calling convention. Existing C libraries (musl, glibc) target Linux syscalls and carry assumptions that don't apply. This plan designs a custom libc from scratch.

**Implementation language**: Zig (`export fn` produces C ABI symbols)
**Public interface**: C headers (stdio.h, pthread.h, etc.) for C and Rust consumers
**Toolchain**: clang/LLVM only — no gcc anywhere in the stack
**Build system**: Bazel (bzlmod, rules_zig 0.12.3 + rules_cc for C headers)
**Zig version**: 0.15.2 (pinned)
**Scope**: ~170 files, ~17,000 LOC, 7 phases over ~16 weeks
**Location**: Initially in kernel-libs monorepo as `libc/`, migrates to `terranox-os/trx-libc` during the shared infra repo split

**Technical reference**: See [terranoxos-zig-libc-reference.md](terranoxos-zig-libc-reference.md) for detailed Zig code examples, inline assembly patterns, Bazel integration, and comptime table generation.

---

## Toolchain stack (all LLVM-based)

```
Zig compiler (bundles LLVM 19) ──→ libc.a (C ABI static archive)
clang 21.1.8 (LLVM)           ──→ kernel C code, kernel-libs C side
rustc (LLVM backend)           ──→ kernel-libs Rust side, userspace Rust
                                    │
                   all produce ──→  ELF objects, linked by lld (LLVM linker)
```

No gcc, no GNU ld, no GNU as. The Docker toolchain image (`ghcr.io/terranox-os/terranox-toolchain-musl:21.1.8`) provides clang. Zig bundles its own LLVM. Rust uses LLVM backend.

---

## Architecture

```
Userspace (Rust programs, C programs)
  #include <stdio.h>  /  extern "C" { fn read() -> isize; }
        │
        │ C ABI function calls
        ▼
┌──────────────────────────────────────────────────────────┐
│                    trx-libc (Zig)                        │
│                                                          │
│  Public C headers ──→ include/stdio.h, pthread.h, ...   │
│  (consumed by C and Rust programs)                       │
│                                                          │
│  Implementation ──→ Zig source files                     │
│  ├── export fn read() → syscall dispatch                 │
│  ├── export fn malloc() → dlmalloc-style allocator       │
│  ├── export fn pthread_create() → trx_thread + futex     │
│  ├── @cImport("genesis_syscall.h") → auto-sync numbers   │
│  └── comptime tables (CRC-32, ctype) → zero-cost         │
│                                                          │
│  Produces: libc.a (static archive, standard C symbols)   │
└────────────────────────┬─────────────────────────────────┘
                         │ SYSCALL instruction
                         │ rax=nr, rdi/rsi/rdx/r10/r8/r9
                         ▼
┌──────────────────────────────────────────────────────────┐
│                  TerranoxOS Kernel                        │
│  119 syscalls (82 TRX + 23 shared + 7 RT + 7 Hermetica)  │
│  Links against: kernel-libs C side                       │
│  (gen_sync, gen_crypto, gen_collections, gen_alloc, ...) │
└──────────────────────────────────────────────────────────┘
```

### Key design property: @cImport eliminates ABI drift

The Zig libc imports syscall numbers directly from the genesis-abi C headers:

```zig
const c = @cImport({
    @cInclude("genesis_syscall.h");
    @cInclude("genesis_result.h");
});
// c.GEN_SYS_READ, c.GEN_SYS_WRITE, etc. — auto-synced with kernel
```

No manual constant duplication. If the kernel adds or renumbers syscalls (like the PR #13 reconciliation), the Zig code picks it up automatically on next build.

---

## POSIX header reference

### Legend

- **Scope**: U = userspace only, K = kernel-relevant (used by kernel modules or drivers), B = both
- **Backed by**: which TerranoxOS syscalls implement the functionality
- **POSIX ref**: IEEE Std 1003.1-2017 section

### Standard C headers (ISO C11 / C17)

| Header | Purpose | Scope | Backed by | POSIX ref |
|--------|---------|-------|-----------|-----------|
| `assert.h` | Runtime assertion macro (`assert()`) | U | None (abort() on failure) | C11 7.2 |
| `ctype.h` | Character classification (`isalpha`, `toupper`) | U | None (Zig comptime table lookup) | C11 7.4 |
| `errno.h` | Error number definitions (`EINVAL`, `ENOMEM`, ...) | B | Kernel returns -errno via SYSCALL; libc maps to thread-local `errno` | POSIX errno.h |
| `float.h` | Floating-point limits (`FLT_MAX`, `DBL_EPSILON`) | U | None (compiler-provided) | C11 7.7 |
| `inttypes.h` | Fixed-width integer format macros (`PRId64`) | U | None (format strings only) | C11 7.8 |
| `limits.h` | Implementation limits (`PATH_MAX`, `INT_MAX`) | B | Defines ABI-visible limits for kernel and userspace | POSIX limits.h |
| `locale.h` | Locale settings (`setlocale`, `LC_*`) | U | None (C/POSIX locale only, no kernel involvement) | C11 7.11 |
| `setjmp.h` | Non-local jumps (`setjmp`/`longjmp`) | U | None (Zig inline asm register save/restore) | C11 7.13 |
| `stdarg.h` | Variadic function macros (`va_start`, `va_arg`) | B | None (compiler-provided builtin) | C11 7.16 |
| `stdbool.h` | Boolean type (`bool`, `true`, `false`) | B | None (compiler-provided) | C11 7.18 |
| `stddef.h` | Common definitions (`size_t`, `NULL`, `offsetof`) | B | None (compiler-provided) | C11 7.19 |
| `stdint.h` | Fixed-width integer types (`uint32_t`, `int64_t`) | B | None (compiler-provided) | C11 7.20 |
| `stdio.h` | Buffered I/O (`FILE`, `printf`, `fopen`) | U | `GEN_SYS_OPEN` (0x0009), `GEN_SYS_READ` (0x0002), `GEN_SYS_WRITE` (0x0001), `GEN_SYS_CLOSE` (0x000A), `GEN_SYS_LSEEK` (0x000D) | POSIX stdio.h |
| `stdlib.h` | General utilities (`malloc`, `exit`, `atoi`, `qsort`) | U | `GEN_SYS_MMAP` (0x0003) for malloc backing; `GEN_SYS_EXIT` (0x0000) for exit | POSIX stdlib.h |
| `string.h` | String/memory operations (`memcpy`, `strlen`, `strcmp`) | B | None (delegates to kernel-libs primitives via @cImport or Zig std.mem) | C11 7.24 |
| `strings.h` | BSD string functions (`strcasecmp`, `bzero`) | U | None (pure computation) | POSIX strings.h |
| `time.h` | Time types and functions (`time_t`, `clock_gettime`) | U | `GEN_SYS_CLOCK_GETTIME` (0x0008), `GEN_SYS_SLEEP` (0x0007) | POSIX time.h |
| `wchar.h` | Wide character support (stub — ASCII only) | U | None | C11 7.29 |

### POSIX system headers

| Header | Purpose | Scope | Backed by | POSIX ref |
|--------|---------|-------|-----------|-----------|
| `unistd.h` | POSIX system calls (`read`, `write`, `close`, `fork`, `exec`) | U | `GEN_SYS_READ` (0x0002), `GEN_SYS_WRITE` (0x0001), `GEN_SYS_CLOSE` (0x000A), `GEN_SYS_LSEEK` (0x000D), `GEN_SYS_TRX_PROCESS_CREATE` (0x0100), `GEN_SYS_EXEC` (0x0013) | POSIX unistd.h |
| `fcntl.h` | File control (`open`, `fcntl`, `O_RDONLY`) | U | `trx_fs_open` (40) | POSIX fcntl.h |
| `pthread.h` | POSIX threads (`pthread_create`, `pthread_mutex_*`) | U | `GEN_SYS_TRX_THREAD_CREATE` (0x0110), `GEN_SYS_TRX_THREAD_EXIT` (0x0111), `GEN_SYS_TRX_THREAD_JOIN` (0x0112), `GEN_SYS_TRX_FUTEX_WAIT` (0x0117), `GEN_SYS_TRX_FUTEX_WAKE` (0x0118) | POSIX pthread.h |
| `sched.h` | Scheduling (`sched_yield`) | U | `GEN_SYS_YIELD` (0x0005), `GEN_SYS_TRX_THREAD_SET_AFFINITY` (0x0114) | POSIX sched.h |
| `semaphore.h` | POSIX semaphores (`sem_wait`, `sem_post`) | U | Built on `trx_futex_wait`/`trx_futex_wake` (userspace atomics + kernel fallback) | POSIX semaphore.h |
| `signal.h` | Signal handling (`sigaction`, `kill`, `raise`) | U | `GEN_SYS_TRX_PROCESS_KILL` (0x0103) requires `cap::process::signal`; dispatch via `GEN_SYS_TRX_SIGNAL_WAIT` (0x0137) | POSIX signal.h |
| `poll.h` | I/O multiplexing (`poll`) | U | `GEN_SYS_TRX_CHANNEL_POLL` (0x0134) or `GEN_SYS_TRX_EVENT_WAIT_MANY` (0x0139) | POSIX poll.h |

### System type and stat headers

| Header | Purpose | Scope | Backed by | POSIX ref |
|--------|---------|-------|-----------|-----------|
| `sys/types.h` | Primitive system types (`pid_t`, `off_t`, `ssize_t`, `mode_t`) | B | Defines ABI types matching kernel expectations (pid_t = int64_t per TerranoxOS) | POSIX sys/types.h |
| `sys/stat.h` | File status (`stat`, `fstat`, `mkdir`, `mode_t`) | U | `GEN_SYS_STAT` (0x000B), `GEN_SYS_FSTAT` (0x000C), `GEN_SYS_TRX_FS_MKDIR` (0x0147) | POSIX sys/stat.h |
| `sys/mman.h` | Memory mapping (`mmap`, `munmap`, `mprotect`) | U | `GEN_SYS_MMAP` (0x0003), `GEN_SYS_MUNMAP` (0x0004), `GEN_SYS_TRX_MEM_PROTECT` (0x0122), `GEN_SYS_TRX_MEM_MAP` (0x0123) | POSIX sys/mman.h |
| `sys/wait.h` | Process wait (`waitpid`, `WEXITSTATUS`) | U | `GEN_SYS_WAIT` (0x0014) | POSIX sys/wait.h |
| `sys/select.h` | I/O multiplexing (`select`, `FD_SET`) | U | Wrapper around `GEN_SYS_TRX_EVENT_WAIT_MANY` (0x0139) | POSIX sys/select.h |
| `sys/socket.h` | Socket API (`socket`, `bind`, `connect`, `sockaddr`) | U | `GEN_SYS_TRX_NET_SOCKET` (0x0180), `GEN_SYS_TRX_NET_BIND` (0x0181), `GEN_SYS_TRX_NET_CONNECT` (0x0184) | POSIX sys/socket.h |
| `sys/un.h` | Unix domain socket address (`sockaddr_un`) | U | `trx_net_socket` with local domain | POSIX sys/un.h |
| `sys/uio.h` | Scatter/gather I/O (`readv`, `writev`, `iovec`) | U | Composed from `trx_fs_read`/`trx_fs_write` in a loop | POSIX sys/uio.h |
| `sys/ioctl.h` | Device I/O control (stub — returns -ENOTTY) | U | NOT backed — TerranoxOS uses per-subsystem syscalls instead | POSIX sys/ioctl.h |
| `sys/time.h` | Time types (`timeval`, `gettimeofday`) | U | `GEN_SYS_CLOCK_GETTIME` (0x0008) | POSIX sys/time.h |

### Network headers

| Header | Purpose | Scope | Backed by | POSIX ref |
|--------|---------|-------|-----------|-----------|
| `netinet/in.h` | IPv4/IPv6 address structs (`sockaddr_in`, `INADDR_ANY`, `htons`) | U | Address structs for `trx_net_bind`/`trx_net_connect` | POSIX netinet/in.h |
| `netinet/tcp.h` | TCP options (`TCP_NODELAY`) | U | Socket options for `setsockopt` on trx_net sockets | POSIX netinet/tcp.h |
| `netdb.h` | Network database (`getaddrinfo`, `gethostbyname`) | U | Pure userspace DNS resolution (numeric-only initially) | POSIX netdb.h |

### TerranoxOS extension headers (non-POSIX)

| Header | Purpose | Scope | Backed by |
|--------|---------|-------|-----------|
| `terranox/syscall.h` | Raw syscall wrappers (`trx_syscall0..6`) | B | Direct SYSCALL instruction |
| `terranox/types.h` | Re-export of genesis_trx_types.h structs | B | ABI struct definitions |
| `terranox/capability.h` | Capability grant/revoke/query API | U | `GEN_SYS_TRX_PROCESS_CAP_GRANT` (0x0105), `_REVOKE` (0x0106), `_QUERY` (0x0107) |
| `terranox/display.h` | Display/compositor/surface/buffer API | U | `GEN_SYS_TRX_DISPLAY_*` / `_COMPOSITOR_*` / `_SURFACE_*` / `_BUFFER_*` (0x0150-0x0159) |
| `terranox/input.h` | Input device enumeration and event reading | U | `GEN_SYS_TRX_INPUT_*` / `_TOUCH_*` (0x0160-0x0168) |
| `terranox/gpu.h` | GPU render nodes, buffer objects, command submission | U | `GEN_SYS_TRX_GPU_*` (0x0170-0x0179) |
| `terranox/ipc.h` | Channel-based IPC, signals, event multiplexer | U | `GEN_SYS_TRX_CHANNEL_*` / `_SIGNAL_*` / `_EVENT_*` (0x0130-0x0139) |

### Dependency diagram

```
                        ┌─────────────────────────────────┐
                        │     User application code       │
                        │  C:    #include <stdio.h>       │
                        │  Rust: extern "C" { fn read(); }│
                        └──────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
   ┌──────────▼──────────┐  ┌─────────▼─────────┐  ┌──────────▼──────────┐
   │   POSIX headers (C) │  │  System hdrs (C)  │  │ TerranoxOS hdrs (C) │
   │ stdio.h stdlib.h    │  │ sys/socket.h      │  │ terranox/display.h  │
   │ string.h pthread.h  │  │ sys/mman.h        │  │ terranox/gpu.h      │
   │ errno.h time.h      │  │ poll.h signal.h   │  │ terranox/ipc.h      │
   └──────────┬──────────┘  └─────────┬─────────┘  └──────────┬──────────┘
              │                        │                        │
              └────────────────────────┼────────────────────────┘
                                       │
                        ┌──────────────▼──────────────────┐
                        │   trx-libc implementation (Zig) │
                        │  export fn read/write/malloc/   │
                        │  pthread_create/socket/...      │
                        │                                 │
                        │  @cImport("genesis_syscall.h")  │
                        │  comptime CRC-32 + ctype tables │
                        │  std.fmt for printf engine      │
                        └──────────────┬──────────────────┘
                                       │
                        ┌──────────────▼──────────────────┐
                        │   Syscall dispatch (Zig asm)    │
                        │  asm volatile ("syscall")       │
                        │  rax=nr  rdi/rsi/rdx/r10/r8/r9 │
                        └──────────────┬──────────────────┘
                                       │
                        ┌──────────────▼──────────────────┐
                        │     TerranoxOS kernel           │
                        │  119 syscalls, capability-gated │
                        │  Links: kernel-libs (C side)    │
                        └──────────────┬──────────────────┘
                                       │
                        ┌──────────────▼──────────────────┐
                        │     SigilVM (Rust + Zig)        │
                        │  svmcheck: bytecode verify,     │
                        │  contract test, sandbox exec    │
                        └─────────────────────────────────┘
```

---

## Repository structure

```
libc/
  BUILD.bazel              # rules_zig for Zig sources, rules_cc for C headers
  build.zig                # standalone Zig build (also usable outside Bazel)
  include/                 # C headers (public API — consumed by C and Rust)
    assert.h, ctype.h, errno.h, fcntl.h, inttypes.h, limits.h,
    locale.h, poll.h, pthread.h, sched.h, semaphore.h, setjmp.h,
    signal.h, stdio.h, stdlib.h, string.h, strings.h, time.h,
    unistd.h, wchar.h
    sys/
      ioctl.h, mman.h, select.h, socket.h, stat.h, time.h,
      types.h, uio.h, un.h, wait.h
    netinet/
      in.h, tcp.h
    netdb.h
    terranox/
      capability.h, display.h, gpu.h, input.h, ipc.h,
      syscall.h, types.h
  src/                     # Zig implementation
    libc.zig               # root module (imports all submodules)
    arch/
      x86_64.zig           # SYSCALL asm, crt0, setjmp, TLS via FS register
      aarch64.zig          # SVC asm, crt0, setjmp, TLS via TPIDR_EL0
      riscv64.zig          # ECALL asm, crt0, setjmp, TLS via TP register
    internal/
      syscall.zig          # @cImport genesis_syscall.h, SyscallNr enum, ret()
      errno.zig            # __errno_location(), ThreadControlBlock
      lock.zig             # futex-based lightweight lock
    crt.zig                # __libc_start_main, atexit, TLS init
    ctype.zig              # comptime 256-byte lookup table
    errno.zig              # POSIX errno constants
    fcntl.zig              # open, fcntl
    locale.zig             # C/POSIX locale stubs
    malloc.zig             # dlmalloc-style allocator (small bins + boundary tag)
    net.zig                # socket, bind, listen, accept, connect, send, recv
    process.zig            # exec, wait, kill, getpid (fork = -ENOSYS stub)
    pthread.zig            # create, join, mutex, cond, rwlock, key, once
    signal.zig             # sigaction, kill, raise (dispatch thread model)
    stdio.zig              # FILE, printf (std.fmt or custom), fopen/fclose/fread/fwrite
    stdlib.zig             # atoi, strtol, qsort, bsearch, rand (xorshift64)
    string.zig             # delegates to std.mem or kernel-libs gen_memcpy
    sys.zig                # mmap, stat, ioctl stub, select
    time.zig               # clock_gettime, nanosleep
    unistd.zig             # read, write, close, lseek, unlink, pipe, dup2
    terranox/
      capability.zig       # cap_grant/revoke/query wrappers
      display.zig          # display/compositor/surface/buffer wrappers
      gpu.zig              # GPU render node wrappers
      input.zig            # input device wrappers
      ipc.zig              # channel/signal/event wrappers
  tests/                   # Zig test blocks + C test binaries
    test_string.zig
    test_stdlib.zig
    test_stdio.zig
    test_malloc.zig
    test_pthread.zig
    test_ctype.zig
```

---

## Phase 0: Scaffolding + syscall layer (Week 1-2, ~1,500 LOC)

**Goal**: `write(1, "hello\n", 6); _exit(0);` works.

### Syscall dispatch (Zig inline asm, x86_64)

```zig
// src/arch/x86_64.zig
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
```

### Errno translation

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

### CRT startup (Zig inline asm)

```zig
// src/arch/x86_64.zig — _start entry
export fn _start() callconv(.naked) noreturn {
    asm volatile (
        \\xorl %%ebp, %%ebp
        \\movq %%rsp, %%rdi
        \\andq $-16, %%rsp
        \\call __libc_start_main
        \\ud2
    );
    unreachable;
}
```

### POSIX wrapper example

```zig
// src/unistd.zig
export fn read(fd: c_int, buf: [*]u8, count: usize) isize {
    return syscall.ret(syscall.syscall3(.READ, @intCast(fd), @intFromPtr(buf), count));
}
```

**Milestone**: hello world binary links and runs.

---

## Phase 1: Core POSIX (Week 3-5, ~4,500 LOC)

### string (~300 LOC)
Delegates to Zig's `std.mem` (memcpy, memset, memcmp all available in freestanding) or kernel-libs `gen_memcpy` via @cImport. Add: strcpy, strcat, strchr, strstr, strtok_r, strerror, strdup.

### stdlib (~500 LOC)
atoi, strtol/strtoul, qsort (std.sort available in freestanding), bsearch, rand/srand (xorshift64), atexit.

### ctype (~100 LOC)
comptime 256-byte lookup table (see zig-libc-reference.md section 6).

### unistd (~400 LOC)
read, write, close, lseek, unlink, pipe, dup2, getpid, getcwd, sysconf.

### fcntl + sys/stat (~300 LOC)
open, fcntl, stat, fstat, mkdir.

### stdio (~1,800 LOC)
FILE struct with buffered I/O. printf family can use Zig's `std.fmt` (available in freestanding) or a custom engine. stdin/stdout/stderr statically allocated.

---

## Phase 2: malloc (Week 4-5, ~1,200 LOC)

Dlmalloc-style boundary-tag allocator:
- **Small (8-256B)**: size-class bins with singly-linked free lists
- **Medium (257B-128KB)**: boundary-tag coalescing, best-fit search
- **Large (>128KB)**: direct trx_mem_alloc, freed back to kernel

Thread-safe via single global futex mutex. Zig's explicit allocator interface (`std.mem.Allocator`) can wrap this for Zig-native callers.

---

## Phase 3: pthreads (Week 6-8, ~2,500 LOC)

| POSIX | TerranoxOS syscall |
|-------|-------------------|
| pthread_create | trx_thread_create (0x0110) — libc allocates stack+TLS |
| pthread_exit | trx_thread_exit (0x0111) |
| pthread_join | trx_thread_join (0x0112) |
| sched_yield | GEN_SYS_YIELD (0x0005, shared) |
| pthread_mutex_lock | CAS fast-path + trx_futex_wait (0x0117) fallback |
| pthread_mutex_unlock | trx_futex_wake (0x0118) |
| pthread_cond_wait | trx_futex_wait |
| pthread_cond_signal | trx_futex_wake(count=1) |

Mutex: three-state futex (Drepper algorithm). TLS: manual via FS register (x86_64), TPIDR_EL0 (AArch64), TP (RISC-V).

---

## Phase 4: Networking (Week 9-10, ~1,500 LOC)

Direct mapping to `GEN_SYS_TRX_NET_*` syscalls (0x0180-0x0186). Plus poll/select via `GEN_SYS_TRX_EVENT_WAIT_MANY` (0x0139). getaddrinfo numeric-only initially.

---

## Phase 5: Signals (Week 11-12, ~1,000 LOC)

Fuchsia-inspired: hidden per-process signal dispatch thread. `sigaction()` registers handlers. `kill()` maps to `trx_process_kill` (requires `cap::process::signal`). Dispatch thread blocks on `trx_signal_wait`.

---

## Phase 6: TerranoxOS extensions (Week 13-14, ~800 LOC)

Zig `export fn` wrappers with errno handling for all TerranoxOS-specific subsystems (capability, display, input, GPU, IPC).

---

## Phase 7: Secondary architectures (Week 15-16, ~600 LOC/arch)

Port syscall asm, crt0, setjmp, TLS to AArch64 and RISC-V 64. All non-arch Zig code is already architecture-independent. Zig cross-compiles all targets from a single binary — no separate toolchain.

---

## POSIX features intentionally NOT implemented

| Feature | Reason |
|---------|--------|
| fork() | TerranoxOS uses trx_process_create. Stub returns -ENOSYS. |
| dlopen/dlsym | Static linking only. |
| setuid/getuid | Capability-based, no UIDs. Stub returns 0. |
| ioctl | Per-subsystem syscalls instead. Returns -ENOTTY. |
| SysV IPC | Channel-based IPC. Deprecated on Linux too. |
| Real-time signals | Not needed for desktop. |
| Full locale/wchar | C/POSIX locale only. |
| math.h transcendentals | Link compiler-rt or use Zig std.math (available freestanding). |

---

## Dependencies on kernel-libs

| Phase | Requires |
|-------|----------|
| 0 | GEN_SYS_* constants from genesis_syscall.h (via @cImport, PR #13) |
| 1 | gen_result_to_errno() from genesis_result.h; gen_memcpy/strlen from primitives (optional — Zig std.mem is alternative) |
| 5 | TrxCapSet from genesis_module.h |
| 6 | GenTrx* structs from genesis_trx_types.h |

---

## Testing strategy

### Level 1: Zig test blocks (~150 tests)
Pure-function tests using Zig's built-in `test` keyword. Run on host via `zig test`. Covers string, stdlib, ctype, malloc internals.

```zig
test "strlen of hello" {
    const s = "hello";
    try std.testing.expectEqual(@as(usize, 5), strlen(s));
}
```

### Level 2: Syscall-mock tests (~50 tests)
Mock the syscall dispatch layer to test errno translation, FILE buffering, mutex state machines without a kernel.

### Level 3: svmcheck verification (all phases)
- **Bytecode verification**: `svmcheck --verify` on compiled `.o` files
- **Syscall contract testing**: `svmcheck --contracts` validates wrappers match TIDL declarations
- **Sandbox execution**: `svmcheck --sandbox` runs test programs with capability enforcement

### Level 4: Integration tests (~30 tests)
Full programs under TerranoxOS kernel in QEMU.

### Level 5: Conformance
Open POSIX Test Suite (~1,700 tests) + musl libc-test after maturity.

### CI pipeline

```yaml
libc-test:
  steps:
    - zig build (freestanding, all targets)
    - zig test (host, pure-function tests)
    - svmcheck --verify on all .o files
    - svmcheck --contracts against TIDL definitions
    - svmcheck --sandbox on test binaries
```

---

## Effort summary

| Component | LOC | Phase |
|-----------|-----|-------|
| Syscall dispatch + CRT (Zig asm) | 1,200 | 0 |
| string + stdlib + ctype | 900 | 1 |
| unistd + fcntl + stat | 700 | 1 |
| stdio (FILE, printf, I/O) | 1,800 | 1 |
| malloc | 1,200 | 2 |
| pthreads | 2,500 | 3 |
| Networking | 1,500 | 4 |
| Signals | 1,000 | 5 |
| TerranoxOS extensions | 800 | 6 |
| Arch ports (AArch64 + RV64) | 1,200 | 7 |
| POSIX C headers | 2,000 | 0-1 |
| Tests | 2,500 | 0-7 |
| **Total** | **~17,300** | **16 weeks** |

*LOC estimates are ~10% lower than the C version due to Zig's more concise syntax, comptime tables, and std.fmt/std.mem reuse in freestanding mode.*
