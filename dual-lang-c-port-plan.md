# Plan: Port Rust-only crates to dual-language (C + Rust)

> **Status: COMPLETED.** Implemented in PR #26 (merged 2026-03-29). See CHANGELOG.md v0.2.0.

*March 2026 — Generated from language split analysis*

---

## Context

The language split report identified 4 Rust-only crates that should have C implementations for direct consumption by GenesisOS-RT and HermeticaOS without FFI overhead. This follows the existing dual-language pattern used by bitops, kfmt, and alloc. Rust implementations stay alongside — both languages tested, both in CI.

Current split: **28% C / 72% Rust**. Target after this work: **~60% C / 40% Rust**.

## Scope

| Crate | What to port | New C files | Est. effort |
|-------|-------------|-------------|-------------|
| arch-intrinsics | 4 arch headers (inline asm) | 4 headers, 0 .c files | Small |
| crypto | CRC-32, SHA-256, HMAC-SHA256 | 1 header, 3 .c files | Medium |
| sync | spinlock, once, atomic_bitops | 1 header, 3 .c files | Medium |
| collections (partial) | intrusive_list, rbtree | 1 header, 2 .c files | Medium |

**Not porting**: static_vec, ringbuf, static_hashmap (const generics are load-bearing Rust features with no C equivalent).

---

## Step 1: arch-intrinsics — C inline asm headers

Header-only library (no .c files). Each architecture gets its own header with `static inline` functions matching the Rust API 1:1.

### Files to create

- `arch-intrinsics/include/gen_arch_x86_64.h` — 25 functions (CR0-4, MSR, port I/O, interrupts, TLB, RDTSC, CPUID)
- `arch-intrinsics/include/gen_arch_aarch64.h` — ~32 functions (sysregs via macros, barriers, TLB, cache, WFI)
- `arch-intrinsics/include/gen_arch_arm_cm.h` — ~20 functions (PRIMASK, BASEPRI, MSP/PSP, PendSV, SysTick)
- `arch-intrinsics/include/gen_arch_riscv64.h` — ~40 functions (CSR macros M+S mode, fence, sfence.vma)

### Files to modify

- `arch-intrinsics/BUILD.bazel` — add `cc_library` target with all 4 headers, arch-gated via `select()`
- `.github/workflows/ci.yml` — add C freestanding compile check for each arch header

### Naming convention

```c
// gen_arch_x86_64.h
static inline uint64_t gen_read_cr0(void) {
    uint64_t val;
    __asm__ __volatile__("mov %%cr0, %0" : "=r"(val));
    return val;
}
```

Prefix: `gen_` (matches existing convention). No ACSL annotations (inline asm is opaque to Frama-C).

### Bazel target

```starlark
cc_library(
    name = "gen_arch_intrinsics",
    hdrs = select({
        "@platforms//cpu:x86_64": ["include/gen_arch_x86_64.h"],
        "@platforms//cpu:aarch64": ["include/gen_arch_aarch64.h"],
        "@platforms//cpu:armv7e-m": ["include/gen_arch_arm_cm.h"],
        "@platforms//cpu:riscv64": ["include/gen_arch_riscv64.h"],
    }),
    includes = ["include"],
    copts = ["-ffreestanding", "-nostdlib", "-std=c17", "-Wall", "-Wextra", "-Werror", "-Wpedantic"],
)
```

### Testing

No C test binary (same as Rust — inline asm cannot run on host). CI verifies compilation via cross-compile with clang in the Docker toolchain.

---

## Step 2: crypto — CRC-32, SHA-256, HMAC-SHA256

Pure algorithmic code. No generics, no unsafe, no traits.

### Files to create

- `crypto/include/gen_crypto.h` — declares all types and functions
- `crypto/src/crc32.c` — CRC-32 IEEE 802.3 (table-based + bitwise fallback via `#ifdef GEN_CRC32_NO_TABLE`)
- `crypto/src/sha256.c` — SHA-256 FIPS 180-4 (streaming + one-shot)
- `crypto/src/hmac.c` — HMAC-SHA256 RFC 2104 (calls sha256 internally)
- `crypto/tests/crypto_test.c` — test with same standard vectors as Rust

### Header API

```c
/* CRC-32 */
uint32_t gen_crc32(const uint8_t *data, size_t len);
uint32_t gen_crc32_update(uint32_t crc, const uint8_t *data, size_t len);

/* SHA-256 */
typedef struct GenSha256 {
    uint32_t state[8];
    uint8_t buf[64];
    size_t buf_len;
    uint64_t total_len;
} GenSha256;

void gen_sha256_init(GenSha256 *ctx);
void gen_sha256_update(GenSha256 *ctx, const uint8_t *data, size_t len);
void gen_sha256_finalize(GenSha256 *ctx, uint8_t digest[32]);
void gen_sha256_digest(const uint8_t *data, size_t len, uint8_t digest[32]);

/* HMAC-SHA256 */
void gen_hmac_sha256(const uint8_t *key, size_t key_len,
                     const uint8_t *data, size_t data_len,
                     uint8_t mac[32]);
```

### Files to modify

- `crypto/BUILD.bazel` — add `cc_library "gen_crypto"` (srcs: 3 .c files, hdrs: gen_crypto.h, deps: genesis_abi) + `cc_test "crypto_test"`
- `.github/workflows/ci.yml` — add C compile + test execution

### Testing

Same FIPS 180-4 / RFC 4231 / IEEE 802.3 test vectors as the Rust tests:
- CRC-32: empty, "123456789" (0xCBF43926), single byte, incremental
- SHA-256: empty, "abc", two-block, incremental
- HMAC: RFC 4231 cases 1, 2, long key

### ACSL

Annotate `gen_sha256_init`, `gen_sha256_update`, `gen_sha256_finalize` with requires/ensures/assigns. CRC-32 and HMAC are simpler — annotate with basic contracts.

---

## Step 3: sync — spinlock, once, atomic_bitops

Translate to C11 atomics. The RAII `SpinLockGuard` becomes explicit lock/unlock calls.

### Files to create

- `sync/include/gen_sync.h` — types + function declarations
- `sync/src/spinlock.c` — ticket spinlock (C11 atomics)
- `sync/src/once.c` — one-time initialization
- `sync/src/atomic_bitops.c` — atomic bitmap operations
- `sync/tests/sync_test.c` — single-threaded correctness tests

### Header API

```c
#include <stdatomic.h>

/* ── Spinlock ───────────────────────────────────────────── */

typedef struct GenSpinLock {
    _Atomic(uint32_t) next_ticket;
    _Atomic(uint32_t) now_serving;
} GenSpinLock;

#define GEN_SPINLOCK_INIT {0, 0}
void gen_spin_lock(GenSpinLock *lock);
int  gen_spin_try_lock(GenSpinLock *lock);  /* returns 1 on success */
void gen_spin_unlock(GenSpinLock *lock);
int  gen_spin_is_locked(const GenSpinLock *lock);

/* ── Once ───────────────────────────────────────────────── */

typedef struct GenOnce {
    _Atomic(uint8_t) state;  /* 0=UNINIT, 1=RUNNING, 2=COMPLETE */
} GenOnce;

#define GEN_ONCE_INIT {0}
void gen_once_call(GenOnce *once, void (*init_fn)(void *ctx), void *ctx);
int  gen_once_is_initialized(const GenOnce *once);

/* ── Atomic bitops ──────────────────────────────────────── */

void gen_atomic_bit_set(_Atomic(uint32_t) *bitmap, uint32_t bit);
void gen_atomic_bit_clear(_Atomic(uint32_t) *bitmap, uint32_t bit);
int  gen_atomic_bit_test(const _Atomic(uint32_t) *bitmap, uint32_t bit);
int  gen_atomic_bit_toggle(_Atomic(uint32_t) *bitmap, uint32_t bit);
int  gen_atomic_bit_test_and_set(_Atomic(uint32_t) *bitmap, uint32_t bit);
int  gen_atomic_bit_test_and_clear(_Atomic(uint32_t) *bitmap, uint32_t bit);
```

### Design decisions

- **No generic data protection**: The Rust `SpinLock<T>` wraps data with the lock. The C version is a standalone lock — caller is responsible for protecting their own data. This matches Linux kernel `spinlock_t` pattern.
- **Once stores no data**: The Rust `Once<T>` stores the computed value. The C version just tracks initialized state — caller stores the value externally. The init callback receives a `void *ctx` for caller context.
- **C11 atomics**: Use `_Atomic(uint32_t)` with `memory_order_acquire`, `memory_order_release`, `memory_order_relaxed` matching the Rust orderings exactly.

### Files to modify

- `sync/BUILD.bazel` — add `cc_library "gen_sync"` + `cc_test "sync_test"`
- `.github/workflows/ci.yml` — add C compile + test

### Testing

Single-threaded tests (same as Rust — no multi-threaded test infra in C tests):
- Spinlock: lock/unlock, try_lock succeeds/fails, is_locked state
- Once: call_once runs, second call is noop, is_initialized
- Atomic bitops: set/clear/test, test_and_set, test_and_clear, across word boundaries

---

## Step 4: collections (partial) — intrusive_list, rbtree

These are pure pointer-based data structures already doing C-style work in Rust's unsafe blocks.

### Files to create

- `collections/include/gen_collections.h` — types + function declarations for list and rbtree
- `collections/src/intrusive_list.c` — doubly-linked circular list
- `collections/src/rbtree.c` — red-black tree (Cormen et al. algorithm)
- `collections/tests/collections_test.c` — test both data structures

### Header API

```c
/* ── Intrusive doubly-linked list ───────────────────────── */

typedef struct GenListNode {
    struct GenListNode *next;
    struct GenListNode *prev;
} GenListNode;

typedef struct GenList {
    GenListNode head;
} GenList;

#define GEN_LIST_INIT(name) { .head = { &(name).head, &(name).head } }
void gen_list_init(GenList *list);
int  gen_list_is_empty(const GenList *list);
void gen_list_push_front(GenList *list, GenListNode *node);
void gen_list_push_back(GenList *list, GenListNode *node);
GenListNode *gen_list_pop_front(GenList *list);
GenListNode *gen_list_pop_back(GenList *list);
void gen_list_remove(GenListNode *node);

/* ── Red-black tree (u64 keys, intrusive nodes) ─────────── */

typedef struct GenRbNode {
    uint64_t key;
    struct GenRbNode *left;
    struct GenRbNode *right;
    struct GenRbNode *parent;
    uint8_t color;  /* 0=red, 1=black */
} GenRbNode;

typedef struct GenRbTree {
    GenRbNode *root;
    size_t len;
} GenRbTree;

void gen_rb_tree_init(GenRbTree *tree);
size_t gen_rb_tree_len(const GenRbTree *tree);
int  gen_rb_tree_is_empty(const GenRbTree *tree);
void gen_rb_tree_insert(GenRbTree *tree, GenRbNode *node);
void gen_rb_tree_remove(GenRbTree *tree, GenRbNode *node);
GenRbNode *gen_rb_tree_find(const GenRbTree *tree, uint64_t key);
GenRbNode *gen_rb_tree_min(const GenRbTree *tree);
GenRbNode *gen_rb_tree_max(const GenRbTree *tree);

/* Callback-based iteration (replaces Rust Iterator trait) */
typedef int (*gen_rb_visit_fn)(GenRbNode *node, void *ctx);
void gen_rb_tree_inorder(const GenRbTree *tree, gen_rb_visit_fn visitor, void *ctx);
```

### Files to modify

- `collections/BUILD.bazel` — add `cc_library "gen_collections"` + `cc_test "collections_test"`
- `.github/workflows/ci.yml` — add C compile + test

### Testing

- List: empty, push_front/back, pop_front/back, remove middle, pop from empty returns NULL
- RbTree: empty, insert single, insert ascending/descending (10 nodes), find hit/miss, remove leaf/root/all, inorder traversal produces sorted order, large tree (100 nodes)

### ACSL

Annotate list operations with requires/ensures (pointer validity, circular invariant). RbTree ACSL is complex — start with basic contracts, defer full WP proofs.

---

## Step 5: CI and documentation updates

### Files to modify

- `.github/workflows/ci.yml` — add compilation + test execution for all 4 new C libraries
- `CLAUDE.md` — update crate table (add C targets for sync, arch-intrinsics, crypto, collections), update test counts
- `CHANGELOG.md` — add v0.3.0 entry

### New CI steps (c-build-test job)

```bash
# arch-intrinsics (header-only, compile check per arch)
clang --target=x86_64 -ffreestanding -nostdlib -std=c17 -fsyntax-only \
  -I arch-intrinsics/include ...

# crypto
clang -ffreestanding -nostdlib -std=c17 -c crypto/src/crc32.c -I crypto/include
clang -ffreestanding -nostdlib -std=c17 -c crypto/src/sha256.c -I crypto/include
clang -ffreestanding -nostdlib -std=c17 -c crypto/src/hmac.c -I crypto/include
clang -std=c17 -o /tmp/crypto_test crypto/tests/crypto_test.c \
  /tmp/crc32.o /tmp/sha256.o /tmp/hmac.o -I crypto/include
/tmp/crypto_test

# sync
clang -ffreestanding -nostdlib -std=c17 -c sync/src/spinlock.c -I sync/include
clang -ffreestanding -nostdlib -std=c17 -c sync/src/once.c -I sync/include
clang -ffreestanding -nostdlib -std=c17 -c sync/src/atomic_bitops.c -I sync/include
clang -std=c17 -o /tmp/sync_test sync/tests/sync_test.c \
  /tmp/spinlock.o /tmp/once.o /tmp/atomic_bitops.o -I sync/include
/tmp/sync_test

# collections
clang -ffreestanding -nostdlib -std=c17 -c collections/src/intrusive_list.c \
  -I collections/include
clang -ffreestanding -nostdlib -std=c17 -c collections/src/rbtree.c \
  -I collections/include
clang -std=c17 -o /tmp/collections_test collections/tests/collections_test.c \
  /tmp/intrusive_list.o /tmp/rbtree.o -I collections/include
/tmp/collections_test
```

---

## Commit strategy (git flow)

New branch: `feature/dual-lang-c-ports` (branched from `main` for v0.2.0; future work should branch from `develop`)

| Commit | Content |
|--------|---------|
| 1 | arch-intrinsics: 4 C headers + BUILD.bazel |
| 2 | crypto: gen_crypto.h + crc32.c + sha256.c + hmac.c + tests |
| 3 | sync: gen_sync.h + spinlock.c + once.c + atomic_bitops.c + tests |
| 4 | collections: gen_collections.h + intrusive_list.c + rbtree.c + tests |
| 5 | CI + docs: ci.yml updates, CLAUDE.md, CHANGELOG.md |

---

## Verification

```bash
# Rust tests unchanged (regression)
cargo test --workspace

# C builds (Docker clang toolchain)
docker run --rm --entrypoint "" -v "$(pwd):/src:ro" -w /src \
  ghcr.io/terranox-os/terranox-toolchain-musl:latest \
  clang -ffreestanding -nostdlib -std=c17 -Wall -Wextra -Werror -Wpedantic \
  -I crypto/include -c crypto/src/sha256.c -o /dev/null

# C tests (link + run on host)
# Each test binary returns 0 on success

# Cross-compile (arch-intrinsics headers)
# Verified via cargo build --workspace --target <arch> (unchanged)
```

## Risks

| Risk | Mitigation |
|------|------------|
| C11 `<stdatomic.h>` not available in all freestanding environments | Use GCC/Clang `__atomic_*` builtins as fallback with `#ifdef` |
| RbTree C port correctness | Use identical test vectors as Rust; test with 100+ node trees |
| CPUID `rbx` clobbering | Save/restore `rbx` manually in C asm (same as Rust version) |
| Frama-C on sync code | Skip ACSL for atomic operations (Frama-C doesn't model C11 atomics) |
