/*
 * gen_sync.h -- Synchronization primitives for freestanding kernels.
 *
 * C API for ticket spinlock, one-time initialization, and atomic
 * bitmap operations. The Rust crate provides equivalent operations
 * with safe abstractions (SpinLock<T>, Once<T>, atomic_bitops).
 *
 * Freestanding: requires only <stdint.h>, <stdbool.h>, and <stdatomic.h>.
 */

#ifndef GEN_SYNC_H
#define GEN_SYNC_H

#include <stdint.h>
#include <stdbool.h>
#include <stdatomic.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Ticket Spinlock ────────────────────────────────────── */

/*
 * Fair FIFO spinlock based on a ticket/turn mechanism.
 *
 * Waiters acquire a ticket via atomic increment and spin until
 * now_serving matches their ticket. Unlock advances now_serving.
 */
typedef struct GenSpinLock {
    _Atomic(uint32_t) next_ticket;
    _Atomic(uint32_t) now_serving;
} GenSpinLock;

#define GEN_SPINLOCK_INIT {0, 0}

/*
 * Acquire the lock, spinning until it is available.
 */
void gen_spin_lock(GenSpinLock *lock);

/*
 * Try to acquire the lock without spinning.
 * Returns 1 on success, 0 if the lock is already held.
 */
int gen_spin_try_lock(GenSpinLock *lock);

/*
 * Release the lock. Must be called by the holder.
 */
void gen_spin_unlock(GenSpinLock *lock);

/*
 * Returns true if the lock is currently held.
 */
bool gen_spin_is_locked(const GenSpinLock *lock);

/* ── One-Time Initialization ────────────────────────────── */

/*
 * State values:
 *   0 = UNINIT   -- not yet initialized
 *   1 = RUNNING  -- init_fn is executing
 *   2 = COMPLETE -- initialization finished
 */
typedef struct GenOnce {
    _Atomic(uint8_t) state;
} GenOnce;

#define GEN_ONCE_INIT {0}

/*
 * Call init_fn(ctx) exactly once. If another thread is already
 * running the initializer, spin until it completes.
 */
void gen_once_call(GenOnce *once, void (*init_fn)(void *ctx), void *ctx);

/*
 * Returns true if initialization is complete.
 */
bool gen_once_is_initialized(const GenOnce *once);

/* ── Atomic Bitmap Operations ───────────────────────────── */

/*
 * All functions operate on an array of _Atomic(uint32_t).
 * Bit numbering: bit N lives in word[N/32], mask 1u << (N%32).
 *
 * For non-atomic bitmap operations, use kernel-bitops (gen_bitops.h).
 */

void gen_atomic_bit_set(_Atomic(uint32_t) *bitmap, uint32_t bit);
void gen_atomic_bit_clear(_Atomic(uint32_t) *bitmap, uint32_t bit);
bool gen_atomic_bit_test(const _Atomic(uint32_t) *bitmap, uint32_t bit);

/*
 * Toggle bit. Returns true if the bit was previously set.
 */
bool gen_atomic_bit_toggle(_Atomic(uint32_t) *bitmap, uint32_t bit);

/*
 * Atomically set bit, returning true if it was already set.
 */
bool gen_atomic_bit_test_and_set(_Atomic(uint32_t) *bitmap, uint32_t bit);

/*
 * Atomically clear bit, returning true if it was previously set.
 */
bool gen_atomic_bit_test_and_clear(_Atomic(uint32_t) *bitmap, uint32_t bit);

#ifdef __cplusplus
}
#endif

#endif /* GEN_SYNC_H */
