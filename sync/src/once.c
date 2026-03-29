/*
 * once.c -- One-time initialization using C11 atomics.
 *
 * State machine:
 *   UNINIT (0) --CAS--> RUNNING (1) --store--> COMPLETE (2)
 *
 * Exactly one thread wins the CAS and runs init_fn. All other
 * threads (including late arrivals) spin until COMPLETE.
 */

#include "gen_sync.h"

#define GEN_ONCE_UNINIT   0
#define GEN_ONCE_RUNNING  1
#define GEN_ONCE_COMPLETE 2

void gen_once_call(GenOnce *once, void (*init_fn)(void *ctx), void *ctx)
{
    uint8_t state = atomic_load_explicit(&once->state,
                                         memory_order_acquire);
    if (state == GEN_ONCE_COMPLETE) {
        return;
    }

    uint8_t expected = GEN_ONCE_UNINIT;
    if (atomic_compare_exchange_strong_explicit(
            &once->state,
            &expected,
            GEN_ONCE_RUNNING,
            memory_order_acquire,
            memory_order_relaxed)) {
        /* We won the race -- run the initializer. */
        init_fn(ctx);
        atomic_store_explicit(&once->state, GEN_ONCE_COMPLETE,
                              memory_order_release);
    } else {
        /* Another thread is running init_fn -- spin until done. */
        while (atomic_load_explicit(&once->state,
                                    memory_order_acquire)
               != GEN_ONCE_COMPLETE) {
            /* spin */
        }
    }
}

bool gen_once_is_initialized(const GenOnce *once)
{
    return atomic_load_explicit(&once->state,
                                memory_order_acquire) == GEN_ONCE_COMPLETE;
}
