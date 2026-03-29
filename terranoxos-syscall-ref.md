# TerranoxOS syscall reference

*Version 0.1.0 — March 2026 — Antonette Caldwell*

This document serves three purposes: (1) a complete reference for the System V AMD64 ABI calling convention, (2) the Linux kernel syscall convention that TerranoxOS's kernel-libs translate to, and (3) the authoritative catalog of TerranoxOS's 91 syscalls organized by subsystem with capability requirements, designed for the full compositor desktop ecosystem.

---

## Part I: System V AMD64 ABI calling convention

The System V AMD64 ABI governs how userspace functions call each other on x86-64 Linux, BSD, and TerranoxOS. Every Rust, C, and C++ function compiled by GCC, Clang, or rustc follows this convention unless explicitly overridden.

### Register assignment for function calls

| Register | Purpose | Callee-saved? |
|----------|---------|---------------|
| `rdi` | 1st integer/pointer argument | No |
| `rsi` | 2nd integer/pointer argument | No |
| `rdx` | 3rd integer/pointer argument | No |
| `rcx` | 4th integer/pointer argument | No |
| `r8` | 5th integer/pointer argument | No |
| `r9` | 6th integer/pointer argument | No |
| `rax` | Return value (1st) | No |
| `rdx` | Return value (2nd, for 128-bit returns) | No |
| `r10` | Temporary / static chain pointer | No |
| `r11` | Temporary | No |
| `rbx` | Callee-saved general purpose | **Yes** |
| `rbp` | Frame pointer (callee-saved) | **Yes** |
| `r12` | Callee-saved general purpose | **Yes** |
| `r13` | Callee-saved general purpose | **Yes** |
| `r14` | Callee-saved general purpose | **Yes** |
| `r15` | Callee-saved general purpose | **Yes** |
| `rsp` | Stack pointer | **Yes** |

### Floating-point / SSE registers

| Register | Purpose | Callee-saved? |
|----------|---------|---------------|
| `xmm0` – `xmm7` | 1st–8th floating-point arguments; `xmm0`–`xmm1` for return | No |
| `xmm8` – `xmm15` | Temporary | No |
| `al` (low byte of `rax`) | Number of vector registers used (for variadic calls) | No |

### Stack frame layout

```
High addresses
┌──────────────────────────┐
│ 7th+ arguments (pushed   │  ← caller pushes right-to-left
│ right-to-left on stack)  │
├──────────────────────────┤
│ return address           │  ← pushed by CALL instruction
├──────────────────────────┤
│ saved rbp (if frame ptr) │  ← optional, push rbp; mov rbp,rsp
├──────────────────────────┤
│ local variables          │
├──────────────────────────┤
│ callee-saved registers   │  ← rbx, r12–r15 if used
├──────────────────────────┤
│ red zone (128 bytes)     │  ← below rsp, usable by leaf functions
└──────────────────────────┘
Low addresses
```

### Key rules

- **16-byte stack alignment**: `rsp` must be 16-byte aligned *before* the `CALL` instruction. After `CALL` pushes the 8-byte return address, `rsp` is 8-byte aligned inside the callee. Callees typically `push rbp` to restore 16-byte alignment.
- **Red zone**: the 128 bytes below `rsp` are reserved for leaf functions (functions that don't call other functions). They can use this space without adjusting `rsp`. **The red zone must NOT be used in kernel code** — interrupts will clobber it.
- **Struct return**: structs ≤16 bytes are returned in `rax`/`rdx`. Larger structs are returned via a hidden pointer passed in `rdi` (the caller allocates space and passes the address as an implicit first argument).
- **Variadic functions**: `al` must contain the number of SSE registers used for arguments (0–8).

### Example: `ssize_t read(int fd, void *buf, size_t count)`

```nasm
; Caller side (SysV ABI function call)
mov   edi, 3            ; fd = 3 (1st arg in rdi)
lea   rsi, [rbp-256]    ; buf = stack buffer (2nd arg in rsi)
mov   rdx, 256          ; count = 256 (3rd arg in rdx)
call  read              ; SysV ABI call — returns in rax
```

---

## Part II: Linux syscall convention (x86-64)

The Linux syscall convention is **not** the SysV ABI. It governs the `SYSCALL` instruction boundary between userspace and kernel. TerranoxOS's kernel-libs implement this convention for Linux compatibility.

### Register assignment for SYSCALL

| Register | Purpose | Notes |
|----------|---------|-------|
| `rax` | Syscall number | Caller sets; overwritten with return value |
| `rdi` | 1st argument | Same as SysV |
| `rsi` | 2nd argument | Same as SysV |
| `rdx` | 3rd argument | Same as SysV |
| `r10` | 4th argument | **Differs from SysV** (`rcx` in SysV) |
| `r8` | 5th argument | Same as SysV |
| `r9` | 6th argument | Same as SysV |
| `rax` | Return value | Negative = `-errno` |

### Why `r10` instead of `rcx`

The `SYSCALL` instruction itself clobbers two registers:
- `rcx` ← saved `rip` (return address)
- `r11` ← saved `rflags`

Since the hardware destroys `rcx` before the kernel sees it, the 4th argument must use `r10` instead. This is the single most important difference between SysV function calls and Linux syscalls.

### Registers preserved across SYSCALL

| Preserved | Clobbered |
|-----------|-----------|
| `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` (arguments) | `rax` (return value) |
| `rbx`, `rbp`, `r12`–`r15`, `rsp` (callee-saved) | `rcx` (saved rip by hardware) |
| | `r11` (saved rflags by hardware) |

### Libc wrapper: SysV → SYSCALL translation

The userspace libc (or vDSO) bridges the two conventions:

```nasm
; glibc wrapper for read(2)
; Entry: SysV ABI — fd in edi, buf in rsi, count in rdx
read:
    mov    eax, 0          ; SYS_read = 0
    syscall                ; rdi, rsi, rdx already correct
                           ; (no 4th arg, so rcx→r10 not needed)
    cmp    rax, -4096
    ja     .error          ; negative > -4096 means error
    ret
.error:
    neg    eax
    mov    [errno], eax    ; set errno
    mov    rax, -1
    ret
```

For a 4-argument syscall like `ptrace(request, pid, addr, data)`:

```nasm
ptrace:
    mov    r10, rcx        ; 4th arg: rcx (SysV) → r10 (syscall)
    mov    eax, 101        ; SYS_ptrace = 101
    syscall
    ret
```

### SYSRET behavior

`SYSRET` returns to userspace by:
- `rip` ← `rcx` (restored from where SYSCALL saved it)
- `rflags` ← `r11` (restored)
- Privilege drops from ring 0 to ring 3

**SYSRET vulnerability note**: Intel CPUs have a known issue where `SYSRET` with a non-canonical `rcx` faults in ring 0 with the user's `rsp`. The kernel must validate `rcx` before executing `SYSRET` or use `IRETQ` as a fallback for suspect return addresses.

### Error convention

Linux syscalls return a single `rax` value:
- Success: `rax` ≥ 0 (or the positive result)
- Error: `rax` = negative errno value (e.g., -`EBADF` = -9, -`EFAULT` = -14)

TerranoxOS kernel-libs follow this convention but extend it with the gap-based error scheme for kernel-internal errors. The translation layer in kernel-libs maps TerranoxOS error codes to POSIX errno values at the syscall boundary.

---

## Part III: TerranoxOS syscall catalog (91 syscalls)

### Design principles

Every TerranoxOS syscall:
- Requires an explicit capability. No syscall is ungated.
- Returns a result through `rax` following the Linux negative-errno convention.
- Is assigned a stable syscall number. Numbers are grouped by subsystem with gaps for future expansion.
- Has a corresponding `.tidl` declaration (to be generated from this catalog).

### Capability hierarchy (abbreviated)

```
cap::root
├── cap::process
│   ├── cap::process::create
│   ├── cap::process::signal
│   ├── cap::process::inspect
│   └── cap::process::manage
├── cap::memory
│   ├── cap::memory::alloc
│   ├── cap::memory::map
│   ├── cap::memory::share
│   └── cap::memory::dma
├── cap::thread
│   ├── cap::thread::create
│   ├── cap::thread::join
│   └── cap::thread::affinity
├── cap::ipc
│   ├── cap::ipc::channel
│   ├── cap::ipc::signal
│   └── cap::ipc::event
├── cap::fs
│   ├── cap::fs::read
│   ├── cap::fs::write
│   ├── cap::fs::create
│   └── cap::fs::delete
├── cap::io
│   ├── cap::io::port
│   ├── cap::io::irq
│   └── cap::io::mmio
├── cap::display
│   ├── cap::display::compositor
│   ├── cap::display::surface
│   ├── cap::display::buffer
│   └── cap::display::mode
├── cap::input
│   ├── cap::input::keyboard
│   ├── cap::input::pointer
│   └── cap::input::touch
├── cap::gpu
│   ├── cap::gpu::render
│   ├── cap::gpu::compute
│   └── cap::gpu::alloc
├── cap::net
│   ├── cap::net::socket
│   ├── cap::net::bind
│   └── cap::net::raw
├── cap::time
│   ├── cap::time::read
│   ├── cap::time::sleep
│   └── cap::time::timer
└── cap::system
    ├── cap::system::reboot
    ├── cap::system::module
    └── cap::system::audit
```

---

### Subsystem 0: Process management (0–9)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 0 | `trx_process_create` | `cap::process::create` | `path: *const u8, argv: *const *const u8, caps: cap_set_t` | `pid: i64` | Create a new process from an ELF binary. The new process inherits only the capabilities in `caps` (no ambient authority). |
| 1 | `trx_process_exit` | *(implicit)* | `status: i32` | *(does not return)* | Terminate the calling process. Cleanup handlers run in reverse order. |
| 2 | `trx_process_wait` | `cap::process::manage` | `pid: i64, status: *mut i32, flags: u32` | `pid: i64` | Wait for a child process to change state. |
| 3 | `trx_process_kill` | `cap::process::signal` | `pid: i64, signal: i32` | `0 / -errno` | Send a signal to a process. Requires capability over the target. |
| 4 | `trx_process_info` | `cap::process::inspect` | `pid: i64, info: *mut process_info_t` | `0 / -errno` | Query process metadata (state, memory usage, capabilities). |
| 5 | `trx_process_cap_grant` | `cap::process::manage` | `pid: i64, cap: cap_t` | `0 / -errno` | Grant a capability to a child process. Must hold the capability yourself (transitive grant). |
| 6 | `trx_process_cap_revoke` | `cap::process::manage` | `pid: i64, cap: cap_t` | `0 / -errno` | Revoke a capability from a child process. Triggers transitive revocation in the capability DAG. |
| 7 | `trx_process_cap_query` | `cap::process::inspect` | `pid: i64, caps: *mut cap_set_t` | `0 / -errno` | Query the capability set of a process. |
| 8 | `trx_process_exec` | `cap::process::create` | `path: *const u8, argv: *const *const u8` | *(does not return)* | Replace the current process image (exec). |
| 9 | *(reserved)* | | | | Future expansion. |

---

### Subsystem 1: Thread management (10–19)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 10 | `trx_thread_create` | `cap::thread::create` | `entry: fn_ptr, stack: *mut u8, stack_size: usize, arg: *mut u8` | `tid: i64` | Create a new thread in the calling process. |
| 11 | `trx_thread_exit` | *(implicit)* | `retval: *mut u8` | *(does not return)* | Terminate the calling thread. |
| 12 | `trx_thread_join` | `cap::thread::join` | `tid: i64, retval: *mut *mut u8` | `0 / -errno` | Wait for a thread to terminate and collect its return value. |
| 13 | `trx_thread_yield` | *(implicit)* | *(none)* | `0` | Voluntarily yield the CPU to another runnable thread. |
| 14 | `trx_thread_set_affinity` | `cap::thread::affinity` | `tid: i64, cpumask: *const u8, len: usize` | `0 / -errno` | Pin a thread to a set of CPUs. |
| 15 | `trx_thread_get_affinity` | `cap::thread::affinity` | `tid: i64, cpumask: *mut u8, len: usize` | `0 / -errno` | Query a thread's CPU affinity mask. |
| 16 | `trx_thread_set_name` | *(implicit)* | `tid: i64, name: *const u8, len: usize` | `0 / -errno` | Set a thread's debug name (max 32 bytes). |
| 17 | `trx_futex_wait` | *(implicit)* | `addr: *const u32, expected: u32, timeout_ns: i64` | `0 / -errno` | Block if `*addr == expected`. Foundation for userspace mutex/condvar. |
| 18 | `trx_futex_wake` | *(implicit)* | `addr: *const u32, count: u32` | `woken: i64` | Wake up to `count` threads blocked on `addr`. |
| 19 | *(reserved)* | | | | Future expansion. |

---

### Subsystem 2: Memory management (20–29)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 20 | `trx_mem_alloc` | `cap::memory::alloc` | `size: usize, flags: u32` | `addr: *mut u8` | Allocate anonymous memory pages. Flags: `MEM_READ`, `MEM_WRITE`, `MEM_EXEC`. |
| 21 | `trx_mem_free` | `cap::memory::alloc` | `addr: *mut u8, size: usize` | `0 / -errno` | Release previously allocated pages. |
| 22 | `trx_mem_protect` | `cap::memory::alloc` | `addr: *mut u8, size: usize, prot: u32` | `0 / -errno` | Change page protection flags. Cannot add `MEM_EXEC` without `cap::memory::alloc`. |
| 23 | `trx_mem_map` | `cap::memory::map` | `fd: i64, offset: u64, size: usize, prot: u32` | `addr: *mut u8` | Map a file or device into the process address space. |
| 24 | `trx_mem_unmap` | `cap::memory::map` | `addr: *mut u8, size: usize` | `0 / -errno` | Unmap a previously mapped region. |
| 25 | `trx_mem_share_create` | `cap::memory::share` | `size: usize, flags: u32` | `handle: i64` | Create a shared memory object. Returns a transferable handle. |
| 26 | `trx_mem_share_map` | `cap::memory::share` | `handle: i64, prot: u32` | `addr: *mut u8` | Map a shared memory object into the calling process. |
| 27 | `trx_mem_share_unmap` | `cap::memory::share` | `addr: *mut u8, size: usize` | `0 / -errno` | Unmap a shared memory region. |
| 28 | `trx_mem_dma_alloc` | `cap::memory::dma` | `size: usize, align: usize` | `phys: u64, virt: *mut u8` | Allocate physically contiguous DMA-capable memory. Returns both physical and virtual addresses. |
| 29 | `trx_mem_dma_free` | `cap::memory::dma` | `virt: *mut u8, size: usize` | `0 / -errno` | Free DMA memory. |

---

### Subsystem 3: IPC channels (30–39)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 30 | `trx_channel_create` | `cap::ipc::channel` | `flags: u32, ep0: *mut i64, ep1: *mut i64` | `0 / -errno` | Create a bidirectional channel pair. Returns two endpoint handles. |
| 31 | `trx_channel_send` | `cap::ipc::channel` | `ep: i64, data: *const u8, len: usize, handles: *const i64, handle_count: u32` | `0 / -errno` | Send a message (data + handles) on a channel endpoint. Handles are moved, not copied. |
| 32 | `trx_channel_recv` | `cap::ipc::channel` | `ep: i64, buf: *mut u8, buf_len: usize, handles: *mut i64, handle_count: *mut u32` | `bytes_read: i64` | Receive a message from a channel endpoint. Blocks if no message available. |
| 33 | `trx_channel_close` | *(implicit)* | `ep: i64` | `0 / -errno` | Close a channel endpoint. Peer sees `EPIPE` on next send/recv. |
| 34 | `trx_channel_poll` | `cap::ipc::channel` | `eps: *const i64, count: u32, events: *mut u32, timeout_ns: i64` | `ready: i64` | Poll multiple channel endpoints for readability/writability. |
| 35 | `trx_signal_create` | `cap::ipc::signal` | `flags: u32` | `handle: i64` | Create a signal object (edge-triggered event). |
| 36 | `trx_signal_raise` | `cap::ipc::signal` | `handle: i64, bits: u32` | `0 / -errno` | Set signal bits on a signal object. Wakes waiters. |
| 37 | `trx_signal_wait` | `cap::ipc::signal` | `handle: i64, mask: u32, timeout_ns: i64` | `observed: u32` | Wait for any of the masked signal bits to be set. |
| 38 | `trx_signal_clear` | `cap::ipc::signal` | `handle: i64, bits: u32` | `0 / -errno` | Clear signal bits. |
| 39 | `trx_event_wait_many` | `cap::ipc::event` | `items: *mut wait_item_t, count: u32, timeout_ns: i64` | `ready: i64` | Wait on a heterogeneous set of objects (channels, signals, timers). Unified event multiplexer. |

---

### Subsystem 4: File system (40–49)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 40 | `trx_fs_open` | `cap::fs::read` or `cap::fs::write` | `path: *const u8, flags: u32, mode: u32` | `fd: i64` | Open a file. Capability required depends on flags (read vs write vs create). |
| 41 | `trx_fs_close` | *(implicit)* | `fd: i64` | `0 / -errno` | Close a file descriptor. |
| 42 | `trx_fs_read` | `cap::fs::read` | `fd: i64, buf: *mut u8, count: usize` | `bytes: i64` | Read from a file descriptor. |
| 43 | `trx_fs_write` | `cap::fs::write` | `fd: i64, data: *const u8, count: usize` | `bytes: i64` | Write to a file descriptor. |
| 44 | `trx_fs_seek` | `cap::fs::read` | `fd: i64, offset: i64, whence: u32` | `pos: i64` | Reposition file offset. |
| 45 | `trx_fs_stat` | `cap::fs::read` | `path: *const u8, stat: *mut stat_t` | `0 / -errno` | Query file metadata by path. |
| 46 | `trx_fs_fstat` | `cap::fs::read` | `fd: i64, stat: *mut stat_t` | `0 / -errno` | Query file metadata by descriptor. |
| 47 | `trx_fs_mkdir` | `cap::fs::create` | `path: *const u8, mode: u32` | `0 / -errno` | Create a directory. |
| 48 | `trx_fs_unlink` | `cap::fs::delete` | `path: *const u8` | `0 / -errno` | Remove a file or empty directory. |
| 49 | `trx_fs_rename` | `cap::fs::write` | `old: *const u8, new: *const u8` | `0 / -errno` | Rename/move a file. Requires write capability on both source and destination directories. |

---

### Subsystem 5: Display / compositor (50–59)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 50 | `trx_display_enumerate` | `cap::display::mode` | `displays: *mut display_info_t, count: *mut u32` | `0 / -errno` | List connected displays with resolution, refresh rate, and DRM connector info. |
| 51 | `trx_display_set_mode` | `cap::display::mode` | `display_id: u32, mode: *const mode_t` | `0 / -errno` | Set display resolution and refresh rate. Requires exclusive mode-setting capability. |
| 52 | `trx_compositor_create` | `cap::display::compositor` | `flags: u32` | `handle: i64` | Create a compositor instance. Only one per seat. The compositor owns the display pipeline. |
| 53 | `trx_compositor_present` | `cap::display::compositor` | `handle: i64, layers: *const layer_t, count: u32` | `0 / -errno` | Submit a frame to the display. Layers are composited back-to-front. Blocks until vsync. |
| 54 | `trx_surface_create` | `cap::display::surface` | `width: u32, height: u32, format: u32, flags: u32` | `handle: i64` | Create a renderable surface (backing memory allocated by the kernel). |
| 55 | `trx_surface_destroy` | `cap::display::surface` | `handle: i64` | `0 / -errno` | Destroy a surface and release its backing memory. |
| 56 | `trx_surface_resize` | `cap::display::surface` | `handle: i64, width: u32, height: u32` | `0 / -errno` | Resize a surface. Invalidates existing buffer contents. |
| 57 | `trx_buffer_create` | `cap::display::buffer` | `width: u32, height: u32, format: u32, usage: u32` | `handle: i64` | Create a GPU-accessible buffer object (for wlroots-style buffer management). |
| 58 | `trx_buffer_map` | `cap::display::buffer` | `handle: i64, prot: u32` | `addr: *mut u8, stride: u32` | Map a buffer into userspace for CPU access. Returns base address and row stride. |
| 59 | `trx_buffer_unmap` | `cap::display::buffer` | `handle: i64` | `0 / -errno` | Unmap a buffer from userspace. |

---

### Subsystem 6: Input devices (60–69)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 60 | `trx_input_enumerate` | `cap::input::keyboard` or `cap::input::pointer` | `devices: *mut input_dev_t, count: *mut u32` | `0 / -errno` | List available input devices (keyboards, mice, touchpads, touchscreens). |
| 61 | `trx_input_open` | `cap::input::keyboard` or `cap::input::pointer` | `dev_id: u32, flags: u32` | `handle: i64` | Open an input device for event reading. Only one process may hold an input device (exclusive). |
| 62 | `trx_input_close` | *(implicit)* | `handle: i64` | `0 / -errno` | Release an input device. |
| 63 | `trx_input_read_events` | `cap::input::keyboard` or `cap::input::pointer` | `handle: i64, events: *mut input_event_t, max: u32` | `count: i64` | Read pending input events. Non-blocking if no events queued. |
| 64 | `trx_input_grab` | `cap::input::keyboard` | `handle: i64` | `0 / -errno` | Grab exclusive input (for lock screens, fullscreen games). Other processes stop receiving events. |
| 65 | `trx_input_ungrab` | `cap::input::keyboard` | `handle: i64` | `0 / -errno` | Release exclusive input grab. |
| 66 | `trx_input_set_keymap` | `cap::input::keyboard` | `handle: i64, keymap: *const u8, len: usize` | `0 / -errno` | Load an XKB keymap for a keyboard device. |
| 67 | `trx_touch_read_events` | `cap::input::touch` | `handle: i64, events: *mut touch_event_t, max: u32` | `count: i64` | Read multi-touch events (slots, x, y, pressure). |
| 68 | `trx_input_set_accel` | `cap::input::pointer` | `handle: i64, profile: u32, speed: f64` | `0 / -errno` | Set pointer acceleration profile and speed. |
| 69 | *(reserved)* | | | | Future expansion (stylus, gesture). |

---

### Subsystem 7: GPU / DRM (70–79)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 70 | `trx_gpu_open` | `cap::gpu::render` | `dev_id: u32` | `handle: i64` | Open a GPU render node. |
| 71 | `trx_gpu_close` | *(implicit)* | `handle: i64` | `0 / -errno` | Close a GPU handle. |
| 72 | `trx_gpu_alloc_bo` | `cap::gpu::alloc` | `handle: i64, size: u64, flags: u32` | `bo_handle: u32` | Allocate a GPU buffer object (GEM/dumb buffer). |
| 73 | `trx_gpu_free_bo` | `cap::gpu::alloc` | `handle: i64, bo_handle: u32` | `0 / -errno` | Free a GPU buffer object. |
| 74 | `trx_gpu_map_bo` | `cap::gpu::alloc` | `handle: i64, bo_handle: u32` | `addr: *mut u8` | Map a GPU buffer object into userspace. |
| 75 | `trx_gpu_submit` | `cap::gpu::render` | `handle: i64, cmdbuf: *const u8, len: usize` | `fence: i64` | Submit a command buffer to the GPU. Returns a fence handle for synchronization. |
| 76 | `trx_gpu_wait_fence` | `cap::gpu::render` | `fence: i64, timeout_ns: i64` | `0 / -errno` | Wait for a GPU fence to signal (command buffer completion). |
| 77 | `trx_gpu_export_dmabuf` | `cap::gpu::alloc` | `handle: i64, bo_handle: u32` | `dmabuf_fd: i64` | Export a GPU buffer as a DMA-BUF file descriptor for cross-process sharing. |
| 78 | `trx_gpu_import_dmabuf` | `cap::gpu::alloc` | `handle: i64, dmabuf_fd: i64` | `bo_handle: u32` | Import a DMA-BUF into the GPU's address space. |
| 79 | `trx_gpu_get_info` | `cap::gpu::render` | `handle: i64, info: *mut gpu_info_t` | `0 / -errno` | Query GPU capabilities (vendor, VRAM size, supported formats, Vulkan features). |

---

### Subsystem 8: Networking (80–86)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 80 | `trx_net_socket` | `cap::net::socket` | `domain: u32, type: u32, protocol: u32` | `fd: i64` | Create a network socket. |
| 81 | `trx_net_bind` | `cap::net::bind` | `fd: i64, addr: *const sockaddr_t, len: u32` | `0 / -errno` | Bind a socket to an address. Binding to privileged ports requires `cap::net::bind`. |
| 82 | `trx_net_listen` | `cap::net::socket` | `fd: i64, backlog: u32` | `0 / -errno` | Mark socket as listening. |
| 83 | `trx_net_accept` | `cap::net::socket` | `fd: i64, addr: *mut sockaddr_t, len: *mut u32` | `new_fd: i64` | Accept an incoming connection. |
| 84 | `trx_net_connect` | `cap::net::socket` | `fd: i64, addr: *const sockaddr_t, len: u32` | `0 / -errno` | Connect to a remote address. |
| 85 | `trx_net_sendmsg` | `cap::net::socket` | `fd: i64, msg: *const msghdr_t, flags: u32` | `bytes: i64` | Send a message on a connected or unconnected socket (supports scatter-gather). |
| 86 | `trx_net_recvmsg` | `cap::net::socket` | `fd: i64, msg: *mut msghdr_t, flags: u32` | `bytes: i64` | Receive a message from a socket (supports scatter-gather). |

---

### Subsystem 9: Time / timers (87–90)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 87 | `trx_clock_gettime` | `cap::time::read` | `clock_id: u32, ts: *mut timespec_t` | `0 / -errno` | Read a clock. `CLOCK_MONOTONIC`, `CLOCK_REALTIME`, `CLOCK_BOOTTIME`. Suitable for vDSO fast-path. |
| 88 | `trx_sleep` | `cap::time::sleep` | `duration_ns: u64` | `remaining_ns: i64` | Sleep for a duration. Returns remaining nanoseconds if interrupted by a signal. |
| 89 | `trx_timer_create` | `cap::time::timer` | `clock_id: u32, flags: u32` | `handle: i64` | Create a timer object. Can be one-shot or periodic. |
| 90 | `trx_timer_set` | `cap::time::timer` | `handle: i64, interval_ns: u64, initial_ns: u64` | `0 / -errno` | Arm a timer. `initial_ns` is the first expiration; `interval_ns` is the repeat period (0 for one-shot). Timer expiration raises a signal on the associated signal object. |

---

### Subsystem 10: System / audit (91–95, reserved)

| Nr | Name | Capability | Arguments | Return | Description |
|----|------|-----------|-----------|--------|-------------|
| 91 | `trx_system_reboot` | `cap::system::reboot` | `reason: u32` | *(does not return)* | Reboot or power off the system. |
| 92 | `trx_module_load` | `cap::system::module` | `bytecode: *const u8, len: usize, sig: *const u8, sig_len: usize` | `handle: i64` | Load a SigilVM kernel module. Bytecode is verified before execution. Signature checked against Sigil trust store. |
| 93 | `trx_module_unload` | `cap::system::module` | `handle: i64` | `0 / -errno` | Unload a kernel module. Fails if any process holds a reference. Used by HermeticaOS hot-swap. |
| 94 | `trx_audit_read` | `cap::system::audit` | `buf: *mut audit_entry_t, max: u32` | `count: i64` | Read capability audit log entries. Each entry records which capability was checked, by whom, and whether it was granted or denied. |
| 95 | `trx_audit_set_policy` | `cap::system::audit` | `policy: *const audit_policy_t` | `0 / -errno` | Configure audit policy (which capabilities to log, verbosity level). |

*Syscall numbers 96–127 are reserved for future subsystems.*

---

## Appendix A: Syscall number map (compact)

```
 0– 9   Process management      trx_process_*
10–19   Thread management       trx_thread_*, trx_futex_*
20–29   Memory management       trx_mem_*
30–39   IPC channels            trx_channel_*, trx_signal_*, trx_event_*
40–49   File system             trx_fs_*
50–59   Display / compositor    trx_display_*, trx_compositor_*, trx_surface_*, trx_buffer_*
60–69   Input devices           trx_input_*, trx_touch_*
70–79   GPU / DRM               trx_gpu_*
80–86   Networking              trx_net_*
87–90   Time / timers           trx_clock_*, trx_sleep, trx_timer_*
91–95   System / audit          trx_system_*, trx_module_*, trx_audit_*
96–127  Reserved for expansion
```

## Appendix B: Error code mapping (kernel-libs)

TerranoxOS kernel-libs translate between TerranoxOS gap-based error codes and POSIX errno values at the syscall boundary:

| TerranoxOS code | POSIX errno | Meaning |
|-----------------|-------------|---------|
| -1 | `EPERM` (1) | Operation not permitted (capability denied) |
| -2 | `ENOENT` (2) | No such file, process, or object |
| -3 | `ESRCH` (3) | No such process |
| -9 | `EBADF` (9) | Bad file descriptor or handle |
| -11 | `EAGAIN` (11) | Resource temporarily unavailable (try again) |
| -12 | `ENOMEM` (12) | Out of memory |
| -13 | `EACCES` (13) | Permission denied (distinct from capability denial) |
| -14 | `EFAULT` (14) | Bad address (invalid pointer from userspace) |
| -16 | `EBUSY` (16) | Resource busy (exclusive lock held) |
| -17 | `EEXIST` (17) | Object already exists |
| -22 | `EINVAL` (22) | Invalid argument |
| -32 | `EPIPE` (32) | Channel peer closed |
| -38 | `ENOSYS` (38) | Syscall not implemented |
| -110 | `ETIMEDOUT` (110) | Operation timed out |

Kernel-internal errors (the gap-based scheme: -1..-10 general, -16..-24 constraint, etc.) are **not** exposed through the syscall interface. They are translated to the closest POSIX errno before returning to userspace.

## Appendix C: Key data structures

```c
// Capability token (128-bit)
typedef struct {
    uint64_t id;          // unique capability ID in the kernel DAG
    uint64_t rights;      // bitmask of rights (read, write, grant, revoke)
} cap_t;

// Capability set (variable-length bitmap)
typedef struct {
    uint32_t count;
    cap_t    caps[];      // flexible array member
} cap_set_t;

// Process info
typedef struct {
    int64_t  pid;
    int32_t  state;       // RUNNING, SLEEPING, STOPPED, ZOMBIE
    uint64_t memory_bytes;
    uint64_t cpu_time_ns;
    uint32_t thread_count;
    uint32_t cap_count;
} process_info_t;

// Display info
typedef struct {
    uint32_t display_id;
    uint32_t width_px;
    uint32_t height_px;
    uint32_t refresh_mhz; // millihertz (60000 = 60 Hz)
    uint32_t connector;   // DRM connector type
    char     name[32];
} display_info_t;

// Input event (libinput-compatible layout)
typedef struct {
    uint64_t timestamp_ns;
    uint32_t type;        // EV_KEY, EV_REL, EV_ABS
    uint32_t code;        // KEY_A, REL_X, ABS_MT_POSITION_X
    int32_t  value;
    uint32_t device_id;
} input_event_t;

// Touch event (multi-touch)
typedef struct {
    uint64_t timestamp_ns;
    uint32_t slot;        // finger tracking ID
    uint32_t type;        // TOUCH_DOWN, TOUCH_MOVE, TOUCH_UP
    int32_t  x;           // surface-relative
    int32_t  y;
    int32_t  pressure;    // 0–65535
} touch_event_t;

// Wait item (for trx_event_wait_many)
typedef struct {
    int64_t  handle;      // channel, signal, or timer handle
    uint32_t events;      // WAIT_READABLE, WAIT_WRITABLE, WAIT_SIGNAL
    uint32_t observed;    // filled by kernel with triggered events
} wait_item_t;

// Timespec (POSIX-compatible)
typedef struct {
    int64_t  tv_sec;
    int64_t  tv_nsec;
} timespec_t;

// GPU info
typedef struct {
    uint32_t vendor_id;
    uint32_t device_id;
    uint64_t vram_bytes;
    uint32_t max_texture_size;
    uint32_t supported_formats; // bitmask of pixel formats
    char     driver_name[64];
} gpu_info_t;

// Audit entry
typedef struct {
    uint64_t timestamp_ns;
    int64_t  pid;
    int64_t  tid;
    cap_t    capability;
    uint32_t syscall_nr;
    uint32_t result;      // GRANTED or DENIED
} audit_entry_t;
```