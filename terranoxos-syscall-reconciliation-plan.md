# Reconciliation plan: TerranoxOS syscall reference vs. kernel-libs

> **Status: COMPLETED.** Implemented in PR #13 (merged 2026-03-29). See CHANGELOG.md v0.2.0.

*March 2026 — Generated from gap analysis of `terranoxos-syscall-ref.md` against `genesis-abi`*

---

## Context

The new `terranoxos-syscall-ref.md` document defines 82 TerranoxOS-specific syscalls across 12 subsystem blocks (plus 4 legacy sigil/sandbox), a hierarchical capability model, POSIX errno mappings, and 10 new data structures. The existing `genesis-abi` crate has 45 syscalls (23 shared + 8 TerranoxOS + 7 RT + 7 Hermetica), a flat 16-bit capability bitmask, and no errno translation. Final total after reconciliation: 119 syscalls (23 shared + 82 TRX + 7 RT + 7 Hermetica).

This plan implements the full reconciliation in kernel-libs as a working prototype, ahead of the planned shared infrastructure repo split (`trx-abi`, `trx-cap`, `trx-idl`, etc.).

### User decisions

| Decision | Choice |
|----------|--------|
| Syscall numbering | Rebase document's 82 TRX-specific syscalls into the existing 0x0100 TerranoxOS range |
| Capability model | Hierarchical DAG (128-bit domain-partitioned bitmask) |
| Missing syscalls (SIGIL_SIGN, SIGIL_VERIFY, SANDBOX_*) | Keep in repo, reconcile later |
| Error mapping | Add `gen_result_to_errno()` / `gen_result_from_errno()` translation functions |
| Scope | Full implementation in kernel-libs |

---

## Files to modify

| File | Changes |
|------|---------|
| `genesis-abi/include/genesis_syscall.h` | Add ~83 new TerranoxOS syscalls, subsystem grouping, renumber existing 8 with backward-compat aliases |
| `genesis-abi/include/genesis_result.h` | Add errno mapping functions, ~4 new error codes, update `gen_result_name()` |
| `genesis-abi/include/genesis_module.h` | Add `TrxCapSet` type, hierarchical capability constants, helpers |
| `genesis-abi/src/lib.rs` | Mirror all C changes: syscalls, capabilities, errno, data structures |
| `genesis-abi/tests/compile_test.c` | Add static asserts for all new types, values, sizes |
| `.github/workflows/ci.yml` | Update drift-check counts and grep patterns |
| `genesis-abi/BUILD.bazel` | Add new header to `hdrs` list |
| `CLAUDE.md` | Update test counts, capability docs, syscall counts |

## File to create

| File | Purpose |
|------|---------|
| `genesis-abi/include/genesis_trx_types.h` | 10 data structures from document Appendix C |

---

## Step 1: Add `TrxCapSet` type and hierarchical capability constants

**Rationale**: Capabilities are referenced by every syscall definition, so they must exist first. The existing `GenCapability` (flat 16-bit) stays untouched for GenesisOS-RT / HermeticaOS backward compatibility.

### Design: 128-bit domain-partitioned bitmask

```c
typedef struct TrxCapSet {
    uint64_t lo;  /* domains 0-7 */
    uint64_t hi;  /* domains 8-11 + reserved */
} TrxCapSet;
```

Each of the 12 capability domains gets a contiguous block of bits. Parent constants (e.g., `TRX_CAP_PROCESS`) are the bitwise OR of all their children. `TRX_CAP_ROOT` is the OR of all domains. Hierarchy is resolved at compile time — no runtime graph traversal, no allocation.

### Bit allocation

**`lo` word (bits 0-63):**

| Bits | Domain | Sub-capabilities |
|------|--------|-----------------|
| 0-3 | process | create, signal, inspect, manage |
| 4-7 | memory | alloc, map, share, dma |
| 8-10 | thread | create, join, affinity |
| 11-13 | ipc | channel, signal, event |
| 14-17 | fs | read, write, create, delete |
| 18-20 | io | port, irq, mmio |
| 21-24 | display | compositor, surface, buffer, mode |
| 25-27 | input | keyboard, pointer, touch |
| 28-63 | *(reserved)* | expansion within existing domains |

**`hi` word (bits 0-63):**

| Bits | Domain | Sub-capabilities |
|------|--------|-----------------|
| 0-2 | gpu | render, compute, alloc |
| 3-5 | net | socket, bind, raw |
| 6-8 | time | read, sleep, timer |
| 9-11 | system | reboot, module, audit |
| 12-63 | *(reserved)* | future domains |

### Key properties

- **No allocation**: `TrxCapSet` is a 16-byte value type, `#[repr(C)]` in Rust
- **Hierarchy via constants**: `TRX_CAP_PROCESS.contains(TRX_CAP_PROCESS_CREATE)` is true because the parent OR-includes the child
- **Backward compat**: `GenCapability` (flat `uint64_t`) is preserved. Mapping functions bridge the two systems
- **ACSL**: Pure functions, straightforward `requires`/`ensures`/`assigns \nothing` contracts

### Files touched

- `genesis_module.h` — add `TrxCapSet` typedef, ~40 `#define` constants, `trx_cap_contains()`, `trx_cap_union()`, `trx_cap_intersection()`, `trx_cap_difference()`, `trx_cap_implies_parent()`
- `lib.rs` — mirror as `#[repr(C)] pub struct TrxCapSet { pub lo: u64, pub hi: u64 }` with `const fn` methods
- `compile_test.c` — `_Static_assert(sizeof(TrxCapSet) == 16, ...)`, exercise helpers
- `ci.yml` — add grep pattern for `TRX_CAP_` constants

### New tests (~20)

- `trx_capset_size` — 16 bytes
- `trx_capset_domain_bits_unique` — no two sub-caps share a bit
- `trx_capset_parent_contains_children` — e.g., `TRX_CAP_PROCESS.contains(TRX_CAP_PROCESS_CREATE)`
- `trx_capset_root_contains_all`
- `trx_capset_none_contains_nothing`
- `trx_capset_union_intersection_difference`
- `trx_capset_each_domain_no_overlap` — process bits don't intersect memory bits, etc.
- `trx_capset_mapping_to_gen_capability` — round-trip for overlapping capabilities

---

## Step 2: Add data structures (`genesis_trx_types.h`)

Create new header with 10 structs from the document's Appendix C, using `GenTrx` prefix for TerranoxOS-specific types:

| Document name | C type name | Size (bytes) | Notes |
|---------------|------------|-------|-------|
| `cap_t` | `GenTrxCapToken` | 16 | `{ uint64_t id; uint64_t rights; }` |
| `cap_set_t` | `GenTrxCapTokenSet` | 8+ | Header with `count`; tokens follow in memory |
| `process_info_t` | `GenTrxProcessInfo` | 40 | pid, state, memory, cpu_time, thread/cap counts |
| `display_info_t` | `GenTrxDisplayInfo` | 56 | resolution, refresh, connector, name[32], _pad0 |
| `input_event_t` | `GenTrxInputEvent` | 24 | timestamp, type, code, value, device_id (8-byte aligned) |
| `touch_event_t` | `GenTrxTouchEvent` | 32 | timestamp, slot, type, x, y, pressure, _pad0 |
| `wait_item_t` | `GenTrxWaitItem` | 16 | handle, events, observed |
| `timespec_t` | `GenTrxTimespec` | 16 | `{ int64_t tv_sec; int64_t tv_nsec; }` |
| `gpu_info_t` | `GenTrxGpuInfo` | 88 | vendor, device, VRAM, formats, driver_name[64] |
| `audit_entry_t` | `GenTrxAuditEntry` | 48 | timestamp, pid, tid, capability, syscall_nr, result |

### Files touched

- New `genesis_trx_types.h` — all 10 structs with ACSL annotations
- `BUILD.bazel` — add to `hdrs` list
- `lib.rs` — mirror all structs as `#[repr(C)]` with derive `Debug, Clone, Copy`
- `compile_test.c` — `_Static_assert` for all sizes and key field offsets

### New tests (~20)

- Size assertions for all 10 structs (Rust + C)
- Alignment >= 4 for all structs

---

## Step 3: Add errno mapping

### New constants and functions in `genesis_result.h`

```c
/* POSIX errno values used at syscall boundary */
#define GEN_POSIX_EPERM      1
#define GEN_POSIX_ENOENT     2
#define GEN_POSIX_ESRCH      3
#define GEN_POSIX_EBADF      9
#define GEN_POSIX_EAGAIN    11
#define GEN_POSIX_ENOMEM    12
#define GEN_POSIX_EACCES    13
#define GEN_POSIX_EFAULT    14
#define GEN_POSIX_EBUSY     16
#define GEN_POSIX_EEXIST    17
#define GEN_POSIX_EINVAL    22
#define GEN_POSIX_EPIPE     32
#define GEN_POSIX_ENOSYS    38
#define GEN_POSIX_ETIMEDOUT 110

static inline int gen_result_to_errno(GenResult r);
static inline GenResult gen_result_from_errno(int e);
```

### Mapping table (from document Appendix B)

| GenResult | POSIX errno |
|-----------|-------------|
| `GEN_ERR_PERMISSION_DENIED` (-16) | `EPERM` (1) |
| `GEN_ERR_NOT_FOUND` (-3) | `ENOENT` (2) |
| `GEN_ERR_INVALID_ARG` (-1) | `EINVAL` (22) |
| `GEN_ERR_OUT_OF_MEMORY` (-2) | `ENOMEM` (12) |
| `GEN_ERR_BAD_ADDRESS` (-34) | `EFAULT` (14) |
| `GEN_ERR_BUSY` (-7) | `EBUSY` (16) |
| `GEN_ERR_ALREADY_EXISTS` (-4) | `EEXIST` (17) |
| `GEN_ERR_TIMEOUT` (-8) | `ETIMEDOUT` (110) |
| `GEN_ERR_BAD_SYSCALL` (-96) | `ENOSYS` (38) |
| `GEN_ERR_BAD_HANDLE` (-97) | `EBADF` (9) |
| `GEN_ERR_IO` (-32) | `EPIPE` (32) |
| `GEN_OK` (0) | 0 |

### Potential new error codes

| Code | Name | Purpose |
|------|------|---------|
| -35 | `GEN_ERR_CHANNEL_CLOSED` | IPC channel peer closed |
| -36 | `GEN_ERR_DISPLAY_OFFLINE` | Display not connected |
| -37 | `GEN_ERR_GPU_ERROR` | GPU command submission failure |
| -99 | `GEN_ERR_HANDLE_LIMIT` | Too many open handles |

### Files touched

- `genesis_result.h` — add errno constants, mapping functions, new error codes, ACSL, update `gen_result_name()`
- `lib.rs` — mirror everything, update `name()` match under `result-names` feature
- `compile_test.c` — exercise mapping functions
- `ci.yml` — update error code count (~29 → ~33)

### New tests (~10)

- Round-trip all 14 mapped values
- `gen_result_to_errno(GEN_OK)` == 0
- Unmapped codes return a sentinel
- New error codes are unique and negative

---

## Step 4: Renumber existing 8 TerranoxOS syscalls with backward-compat aliases

### TerranoxOS 0x0100 range — subsystem block layout

Each subsystem gets a 16-slot (0x10) block, aligning with hex boundaries. Subsystem index is `(nr - 0x0100) >> 4`.

```
0x0100-0x010F  Subsystem 0: Process management
0x0110-0x011F  Subsystem 1: Thread management
0x0120-0x012F  Subsystem 2: Memory management
0x0130-0x013F  Subsystem 3: IPC channels
0x0140-0x014F  Subsystem 4: File system extensions
0x0150-0x015F  Subsystem 5: Display / compositor
0x0160-0x016F  Subsystem 6: Input devices
0x0170-0x017F  Subsystem 7: GPU / DRM
0x0180-0x018F  Subsystem 8: Networking
0x0190-0x019F  Subsystem 9: Time / timers
0x01A0-0x01AF  Subsystem 10: System / audit
0x01B0-0x01BF  Subsystem 11: Sigil / sandbox (legacy, reconcile later)
0x01C0-0x01FF  Reserved for future subsystems
```

### Renumbering map for existing 8 syscalls

| Old number | Old name | New number | New name | Block |
|-----------|----------|-----------|----------|-------|
| 0x0100 | `GEN_SYS_CAP_GRANT` | 0x0105 | `GEN_SYS_TRX_PROCESS_CAP_GRANT` | Process |
| 0x0101 | `GEN_SYS_CAP_REVOKE` | 0x0106 | `GEN_SYS_TRX_PROCESS_CAP_REVOKE` | Process |
| 0x0102 | `GEN_SYS_CAP_CHECK` | 0x0107 | `GEN_SYS_TRX_PROCESS_CAP_QUERY` | Process |
| 0x0103 | `GEN_SYS_SIGIL_SIGN` | 0x01B0 | `GEN_SYS_TRX_SIGIL_SIGN` | Sigil/sandbox |
| 0x0104 | `GEN_SYS_SIGIL_VERIFY` | 0x01B1 | `GEN_SYS_TRX_SIGIL_VERIFY` | Sigil/sandbox |
| 0x0105 | `GEN_SYS_AUDIT_LOG` | 0x01A3 | `GEN_SYS_TRX_AUDIT_READ` | System/audit |
| 0x0106 | `GEN_SYS_SANDBOX_CREATE` | 0x01B2 | `GEN_SYS_TRX_SANDBOX_CREATE` | Sigil/sandbox |
| 0x0107 | `GEN_SYS_SANDBOX_ENTER` | 0x01B3 | `GEN_SYS_TRX_SANDBOX_ENTER` | Sigil/sandbox |

Old names become `#define` aliases pointing to new names (deprecated, source-compatible):

```c
/* Deprecated — use GEN_SYS_TRX_PROCESS_CAP_GRANT instead */
#define GEN_SYS_CAP_GRANT  GEN_SYS_TRX_PROCESS_CAP_GRANT
```

**Note**: This is a **binary ABI break** (numeric values change). Acceptable at v0.1.0. Source-level aliases prevent recompilation breakage.

### Files touched

- `genesis_syscall.h` — renumber, add aliases, add subsystem base/limit macros, add `gen_syscall_trx_subsystem()` helper
- `lib.rs` — mirror renumbered constants, add deprecated aliases
- `compile_test.c` — update value assertions, add alias checks

---

## Step 5: Add all new TerranoxOS syscall constants

Add ~83 new `GEN_SYS_TRX_*` constants organized by subsystem. Naming: `GEN_SYS_TRX_<SUBSYSTEM>_<ACTION>`.

### Syscall mapping (document → 0x0100 range)

Syscalls that overlap with the shared range (exit, read, write, open, close, stat, fstat, lseek, yield, sleep, clock_gettime, exec, wait, mmap, munmap, brk, ioctl, dup2, pipe, fork, fcntl, poll, getpid) are **not duplicated** — they stay at 0x0000. A comment block documents this.

#### Subsystem 0: Process management (0x0100-0x010F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0100 | `GEN_SYS_TRX_PROCESS_CREATE` | `trx_process_create` |
| 0x0101 | *(reserved — exit is shared GEN_SYS_EXIT)* | |
| 0x0102 | *(reserved — wait is shared GEN_SYS_WAIT)* | |
| 0x0103 | `GEN_SYS_TRX_PROCESS_KILL` | `trx_process_kill` |
| 0x0104 | `GEN_SYS_TRX_PROCESS_INFO` | `trx_process_info` |
| 0x0105 | `GEN_SYS_TRX_PROCESS_CAP_GRANT` | `trx_process_cap_grant` (existing, renumbered) |
| 0x0106 | `GEN_SYS_TRX_PROCESS_CAP_REVOKE` | `trx_process_cap_revoke` (existing, renumbered) |
| 0x0107 | `GEN_SYS_TRX_PROCESS_CAP_QUERY` | `trx_process_cap_query` (existing, renumbered) |
| 0x0108 | *(reserved — exec is shared GEN_SYS_EXEC)* | |

#### Subsystem 1: Thread management (0x0110-0x011F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0110 | `GEN_SYS_TRX_THREAD_CREATE` | `trx_thread_create` |
| 0x0111 | `GEN_SYS_TRX_THREAD_EXIT` | `trx_thread_exit` |
| 0x0112 | `GEN_SYS_TRX_THREAD_JOIN` | `trx_thread_join` |
| 0x0113 | *(reserved — yield is shared GEN_SYS_YIELD)* | |
| 0x0114 | `GEN_SYS_TRX_THREAD_SET_AFFINITY` | `trx_thread_set_affinity` |
| 0x0115 | `GEN_SYS_TRX_THREAD_GET_AFFINITY` | `trx_thread_get_affinity` |
| 0x0116 | `GEN_SYS_TRX_THREAD_SET_NAME` | `trx_thread_set_name` |
| 0x0117 | `GEN_SYS_TRX_FUTEX_WAIT` | `trx_futex_wait` |
| 0x0118 | `GEN_SYS_TRX_FUTEX_WAKE` | `trx_futex_wake` |

#### Subsystem 2: Memory management (0x0120-0x012F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0120 | *(reserved — mmap is shared GEN_SYS_MMAP)* | |
| 0x0121 | *(reserved — munmap is shared GEN_SYS_MUNMAP)* | |
| 0x0122 | `GEN_SYS_TRX_MEM_PROTECT` | `trx_mem_protect` |
| 0x0123 | `GEN_SYS_TRX_MEM_MAP` | `trx_mem_map` |
| 0x0124 | `GEN_SYS_TRX_MEM_UNMAP` | `trx_mem_unmap` |
| 0x0125 | `GEN_SYS_TRX_MEM_SHARE_CREATE` | `trx_mem_share_create` |
| 0x0126 | `GEN_SYS_TRX_MEM_SHARE_MAP` | `trx_mem_share_map` |
| 0x0127 | `GEN_SYS_TRX_MEM_SHARE_UNMAP` | `trx_mem_share_unmap` |
| 0x0128 | `GEN_SYS_TRX_MEM_DMA_ALLOC` | `trx_mem_dma_alloc` |
| 0x0129 | `GEN_SYS_TRX_MEM_DMA_FREE` | `trx_mem_dma_free` |

#### Subsystem 3: IPC channels (0x0130-0x013F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0130 | `GEN_SYS_TRX_CHANNEL_CREATE` | `trx_channel_create` |
| 0x0131 | `GEN_SYS_TRX_CHANNEL_SEND` | `trx_channel_send` |
| 0x0132 | `GEN_SYS_TRX_CHANNEL_RECV` | `trx_channel_recv` |
| 0x0133 | `GEN_SYS_TRX_CHANNEL_CLOSE` | `trx_channel_close` |
| 0x0134 | `GEN_SYS_TRX_CHANNEL_POLL` | `trx_channel_poll` |
| 0x0135 | `GEN_SYS_TRX_SIGNAL_CREATE` | `trx_signal_create` |
| 0x0136 | `GEN_SYS_TRX_SIGNAL_RAISE` | `trx_signal_raise` |
| 0x0137 | `GEN_SYS_TRX_SIGNAL_WAIT` | `trx_signal_wait` |
| 0x0138 | `GEN_SYS_TRX_SIGNAL_CLEAR` | `trx_signal_clear` |
| 0x0139 | `GEN_SYS_TRX_EVENT_WAIT_MANY` | `trx_event_wait_many` |

#### Subsystem 4: File system extensions (0x0140-0x014F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0140 | *(reserved — open is shared GEN_SYS_OPEN)* | |
| 0x0141 | *(reserved — close is shared GEN_SYS_CLOSE)* | |
| 0x0142 | *(reserved — read is shared GEN_SYS_READ)* | |
| 0x0143 | *(reserved — write is shared GEN_SYS_WRITE)* | |
| 0x0144 | *(reserved — seek is shared GEN_SYS_LSEEK)* | |
| 0x0145 | *(reserved — stat is shared GEN_SYS_STAT)* | |
| 0x0146 | *(reserved — fstat is shared GEN_SYS_FSTAT)* | |
| 0x0147 | `GEN_SYS_TRX_FS_MKDIR` | `trx_fs_mkdir` |
| 0x0148 | `GEN_SYS_TRX_FS_UNLINK` | `trx_fs_unlink` |
| 0x0149 | `GEN_SYS_TRX_FS_RENAME` | `trx_fs_rename` |

#### Subsystem 5: Display / compositor (0x0150-0x015F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0150 | `GEN_SYS_TRX_DISPLAY_ENUMERATE` | `trx_display_enumerate` |
| 0x0151 | `GEN_SYS_TRX_DISPLAY_SET_MODE` | `trx_display_set_mode` |
| 0x0152 | `GEN_SYS_TRX_COMPOSITOR_CREATE` | `trx_compositor_create` |
| 0x0153 | `GEN_SYS_TRX_COMPOSITOR_PRESENT` | `trx_compositor_present` |
| 0x0154 | `GEN_SYS_TRX_SURFACE_CREATE` | `trx_surface_create` |
| 0x0155 | `GEN_SYS_TRX_SURFACE_DESTROY` | `trx_surface_destroy` |
| 0x0156 | `GEN_SYS_TRX_SURFACE_RESIZE` | `trx_surface_resize` |
| 0x0157 | `GEN_SYS_TRX_BUFFER_CREATE` | `trx_buffer_create` |
| 0x0158 | `GEN_SYS_TRX_BUFFER_MAP` | `trx_buffer_map` |
| 0x0159 | `GEN_SYS_TRX_BUFFER_UNMAP` | `trx_buffer_unmap` |

#### Subsystem 6: Input devices (0x0160-0x016F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0160 | `GEN_SYS_TRX_INPUT_ENUMERATE` | `trx_input_enumerate` |
| 0x0161 | `GEN_SYS_TRX_INPUT_OPEN` | `trx_input_open` |
| 0x0162 | `GEN_SYS_TRX_INPUT_CLOSE` | `trx_input_close` |
| 0x0163 | `GEN_SYS_TRX_INPUT_READ_EVENTS` | `trx_input_read_events` |
| 0x0164 | `GEN_SYS_TRX_INPUT_GRAB` | `trx_input_grab` |
| 0x0165 | `GEN_SYS_TRX_INPUT_UNGRAB` | `trx_input_ungrab` |
| 0x0166 | `GEN_SYS_TRX_INPUT_SET_KEYMAP` | `trx_input_set_keymap` |
| 0x0167 | `GEN_SYS_TRX_TOUCH_READ_EVENTS` | `trx_touch_read_events` |
| 0x0168 | `GEN_SYS_TRX_INPUT_SET_ACCEL` | `trx_input_set_accel` (use fixed-point i32, not f64) |

#### Subsystem 7: GPU / DRM (0x0170-0x017F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0170 | `GEN_SYS_TRX_GPU_OPEN` | `trx_gpu_open` |
| 0x0171 | `GEN_SYS_TRX_GPU_CLOSE` | `trx_gpu_close` |
| 0x0172 | `GEN_SYS_TRX_GPU_ALLOC_BO` | `trx_gpu_alloc_bo` |
| 0x0173 | `GEN_SYS_TRX_GPU_FREE_BO` | `trx_gpu_free_bo` |
| 0x0174 | `GEN_SYS_TRX_GPU_MAP_BO` | `trx_gpu_map_bo` |
| 0x0175 | `GEN_SYS_TRX_GPU_SUBMIT` | `trx_gpu_submit` |
| 0x0176 | `GEN_SYS_TRX_GPU_WAIT_FENCE` | `trx_gpu_wait_fence` |
| 0x0177 | `GEN_SYS_TRX_GPU_EXPORT_DMABUF` | `trx_gpu_export_dmabuf` |
| 0x0178 | `GEN_SYS_TRX_GPU_IMPORT_DMABUF` | `trx_gpu_import_dmabuf` |
| 0x0179 | `GEN_SYS_TRX_GPU_GET_INFO` | `trx_gpu_get_info` |

#### Subsystem 8: Networking (0x0180-0x018F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0180 | `GEN_SYS_TRX_NET_SOCKET` | `trx_net_socket` |
| 0x0181 | `GEN_SYS_TRX_NET_BIND` | `trx_net_bind` |
| 0x0182 | `GEN_SYS_TRX_NET_LISTEN` | `trx_net_listen` |
| 0x0183 | `GEN_SYS_TRX_NET_ACCEPT` | `trx_net_accept` |
| 0x0184 | `GEN_SYS_TRX_NET_CONNECT` | `trx_net_connect` |
| 0x0185 | `GEN_SYS_TRX_NET_SENDMSG` | `trx_net_sendmsg` |
| 0x0186 | `GEN_SYS_TRX_NET_RECVMSG` | `trx_net_recvmsg` |

#### Subsystem 9: Time / timers (0x0190-0x019F)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x0190 | *(reserved — clock_gettime is shared GEN_SYS_CLOCK_GETTIME)* | |
| 0x0191 | *(reserved — sleep is shared GEN_SYS_SLEEP)* | |
| 0x0192 | `GEN_SYS_TRX_TIMER_CREATE` | `trx_timer_create` |
| 0x0193 | `GEN_SYS_TRX_TIMER_SET` | `trx_timer_set` |

#### Subsystem 10: System / audit (0x01A0-0x01AF)

| Nr | Name | Document equivalent |
|----|------|-------------------|
| 0x01A0 | `GEN_SYS_TRX_SYSTEM_REBOOT` | `trx_system_reboot` |
| 0x01A1 | `GEN_SYS_TRX_MODULE_LOAD` | `trx_module_load` |
| 0x01A2 | `GEN_SYS_TRX_MODULE_UNLOAD` | `trx_module_unload` |
| 0x01A3 | `GEN_SYS_TRX_AUDIT_READ` | `trx_audit_read` (existing AUDIT_LOG, renumbered) |
| 0x01A4 | `GEN_SYS_TRX_AUDIT_SET_POLICY` | `trx_audit_set_policy` |

#### Subsystem 11: Sigil / sandbox — legacy (0x01B0-0x01BF)

| Nr | Name | Notes |
|----|------|-------|
| 0x01B0 | `GEN_SYS_TRX_SIGIL_SIGN` | Existing, renumbered from 0x0103 |
| 0x01B1 | `GEN_SYS_TRX_SIGIL_VERIFY` | Existing, renumbered from 0x0104 |
| 0x01B2 | `GEN_SYS_TRX_SANDBOX_CREATE` | Existing, renumbered from 0x0106 |
| 0x01B3 | `GEN_SYS_TRX_SANDBOX_ENTER` | Existing, renumbered from 0x0107 |

### Total TerranoxOS syscall count: 82 (TRX-specific) + 4 (legacy sigil/sandbox) = 86 in 0x0100 range

### Files touched

- `genesis_syscall.h` — add all constants, subsystem base/limit macros, `gen_syscall_trx_subsystem()` helper
- `lib.rs` — mirror all constants
- `compile_test.c` — representative value assertions per subsystem
- `ci.yml` — update syscall count (23 shared + ~95 TerranoxOS + 7 RT + 7 Hermetica = ~132)

### New tests (~25)

- All TerranoxOS syscalls in 0x0100-0x01FF
- No duplicate numbers across all ranges
- Subsystem classification works
- Deprecated aliases resolve to new names
- No collision with shared, RT, or Hermetica ranges

---

## Step 6: Tests, CI, and documentation finalization

- Finalize `ci.yml` drift-check counts and patterns
- Update `CLAUDE.md`: test count (~350+), syscall count (~132), capability model description
- Update `CHANGELOG.md` with breaking change notice for renumbered syscalls
- Run full validation:

```bash
cargo test -p genesis-abi                          # all Rust tests
cargo test -p genesis-abi --features result-names  # feature-gated
cargo clippy -- -D warnings                        # lint
clang -ffreestanding -nostdlib -std=c17 -Wall -Wextra -Werror -Wpedantic \
    -Igenesis-abi/include genesis-abi/tests/compile_test.c -o /dev/null
./scripts/frama-c-verify.sh --check-only           # ACSL parse
```

---

## Risks and mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Binary ABI break from renumbering | Existing binaries using old numbers break | Pre-1.0 software; source aliases preserve recompilation compat |
| `f64` in syscall 68 (`trx_input_set_accel`) | FPU unavailable in kernel context | Encode as fixed-point `i32` (speed * 1000) |
| `cap_set_t` flexible array member | No VLA in Rust | Use header struct + documented pointer-follows convention |
| Frama-C annotations on new functions | May fail WP proofs | Start with `--check-only` (parse), defer full proofs |
| CI regex patterns for drift check | New `TRX_CAP_` / `GEN_SYS_TRX_` patterns needed | Add separate grep steps or extend existing patterns |
