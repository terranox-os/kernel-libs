/*
 * spinlock.c -- Ticket spinlock using C11 atomics.
 *
 * Fair FIFO ordering: each caller gets a monotonically increasing
 * ticket and spins until now_serving matches. Unlock advances
 * now_serving to release the next waiter.
 */

#include "gen_sync.h"

void gen_spin_lock(GenSpinLock *lock)
{
    uint32_t ticket = atomic_fetch_add_explicit(
        &lock->next_ticket, 1, memory_order_relaxed);

    while (atomic_load_explicit(&lock->now_serving,
                                memory_order_acquire) != ticket) {
        /* spin */
    }
}

int gen_spin_try_lock(GenSpinLock *lock)
{
    uint32_t serving = atomic_load_explicit(
        &lock->now_serving, memory_order_relaxed);
    uint32_t expected = serving;

    if (atomic_compare_exchange_strong_explicit(
            &lock->next_ticket,
            &expected,
            serving + 1,
            memory_order_acquire,
            memory_order_relaxed)) {
        return 1;
    }
    return 0;
}

void gen_spin_unlock(GenSpinLock *lock)
{
    atomic_fetch_add_explicit(
        &lock->now_serving, 1, memory_order_release);
}

bool gen_spin_is_locked(const GenSpinLock *lock)
{
    uint32_t ticket = atomic_load_explicit(
        &lock->next_ticket, memory_order_relaxed);
    uint32_t serving = atomic_load_explicit(
        &lock->now_serving, memory_order_relaxed);
    return ticket != serving;
}
