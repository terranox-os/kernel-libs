/*
 * sync_test.c -- Host-compiled tests for gen_sync C API.
 *
 * Returns 0 on success, non-zero on failure count.
 */

#include "gen_sync.h"
#include <stdio.h>

static int failures = 0;

#define ASSERT(cond, msg)                                            \
    do {                                                             \
        if (!(cond)) {                                               \
            fprintf(stderr, "FAIL: %s (%s:%d)\n",                    \
                    msg, __FILE__, __LINE__);                         \
            failures++;                                              \
        }                                                            \
    } while (0)

/* ── Spinlock tests ───────────────────────────────────────── */

static void test_spin_lock_unlock(void)
{
    GenSpinLock lock = GEN_SPINLOCK_INIT;
    ASSERT(!gen_spin_is_locked(&lock), "initial unlocked");

    gen_spin_lock(&lock);
    ASSERT(gen_spin_is_locked(&lock), "locked after lock");

    gen_spin_unlock(&lock);
    ASSERT(!gen_spin_is_locked(&lock), "unlocked after unlock");
}

static void test_spin_try_lock_success(void)
{
    GenSpinLock lock = GEN_SPINLOCK_INIT;

    int got = gen_spin_try_lock(&lock);
    ASSERT(got == 1, "try_lock on free lock");
    ASSERT(gen_spin_is_locked(&lock), "locked after try_lock");

    gen_spin_unlock(&lock);
    ASSERT(!gen_spin_is_locked(&lock), "unlocked after try_lock release");
}

static void test_spin_try_lock_fail(void)
{
    GenSpinLock lock = GEN_SPINLOCK_INIT;

    gen_spin_lock(&lock);
    int got = gen_spin_try_lock(&lock);
    ASSERT(got == 0, "try_lock on held lock fails");

    gen_spin_unlock(&lock);
}

static void test_spin_is_locked_state(void)
{
    GenSpinLock lock = GEN_SPINLOCK_INIT;
    ASSERT(!gen_spin_is_locked(&lock), "is_locked false initially");

    gen_spin_lock(&lock);
    ASSERT(gen_spin_is_locked(&lock), "is_locked true when held");

    gen_spin_unlock(&lock);
    ASSERT(!gen_spin_is_locked(&lock), "is_locked false after release");
}

/* ── Once tests ───────────────────────────────────────────── */

static int once_call_count = 0;

static void once_init_fn(void *ctx)
{
    int *counter = (int *)ctx;
    (*counter)++;
}

static void test_once_init_runs(void)
{
    GenOnce once = GEN_ONCE_INIT;
    once_call_count = 0;

    ASSERT(!gen_once_is_initialized(&once), "not initialized initially");

    gen_once_call(&once, once_init_fn, &once_call_count);
    ASSERT(once_call_count == 1, "init_fn ran once");
    ASSERT(gen_once_is_initialized(&once), "initialized after call");
}

static void test_once_second_call_noop(void)
{
    GenOnce once = GEN_ONCE_INIT;
    once_call_count = 0;

    gen_once_call(&once, once_init_fn, &once_call_count);
    gen_once_call(&once, once_init_fn, &once_call_count);
    ASSERT(once_call_count == 1, "second call is noop");
}

static void test_once_is_initialized(void)
{
    GenOnce once = GEN_ONCE_INIT;
    ASSERT(!gen_once_is_initialized(&once), "uninit returns false");

    gen_once_call(&once, once_init_fn, &once_call_count);
    ASSERT(gen_once_is_initialized(&once), "initialized returns true");
}

/* ── Atomic bitops tests ──────────────────────────────────── */

static void test_atomic_set_clear_test(void)
{
    _Atomic(uint32_t) bm[4] = {0};
    ASSERT(!gen_atomic_bit_test(bm, 5), "initial zero");

    gen_atomic_bit_set(bm, 5);
    ASSERT(gen_atomic_bit_test(bm, 5), "set bit 5");

    gen_atomic_bit_clear(bm, 5);
    ASSERT(!gen_atomic_bit_test(bm, 5), "clear bit 5");
}

static void test_atomic_test_and_set(void)
{
    _Atomic(uint32_t) bm[2] = {0};

    bool was_set = gen_atomic_bit_test_and_set(bm, 7);
    ASSERT(!was_set, "test_and_set on clear bit");
    ASSERT(gen_atomic_bit_test(bm, 7), "bit set after test_and_set");

    was_set = gen_atomic_bit_test_and_set(bm, 7);
    ASSERT(was_set, "test_and_set on set bit");
}

static void test_atomic_test_and_clear(void)
{
    _Atomic(uint32_t) bm[2] = {0};
    gen_atomic_bit_set(bm, 3);

    bool was_set = gen_atomic_bit_test_and_clear(bm, 3);
    ASSERT(was_set, "test_and_clear on set bit");
    ASSERT(!gen_atomic_bit_test(bm, 3), "bit clear after test_and_clear");

    was_set = gen_atomic_bit_test_and_clear(bm, 3);
    ASSERT(!was_set, "test_and_clear on clear bit");
}

static void test_atomic_toggle(void)
{
    _Atomic(uint32_t) bm[2] = {0};

    bool was_set = gen_atomic_bit_toggle(bm, 10);
    ASSERT(!was_set, "toggle from 0");
    ASSERT(gen_atomic_bit_test(bm, 10), "bit set after first toggle");

    was_set = gen_atomic_bit_toggle(bm, 10);
    ASSERT(was_set, "toggle from 1");
    ASSERT(!gen_atomic_bit_test(bm, 10), "bit clear after second toggle");
}

static void test_atomic_across_words(void)
{
    _Atomic(uint32_t) bm[4] = {0};

    gen_atomic_bit_set(bm, 31);
    gen_atomic_bit_set(bm, 32);
    gen_atomic_bit_set(bm, 33);
    gen_atomic_bit_set(bm, 95);

    ASSERT(gen_atomic_bit_test(bm, 31), "bit 31 set");
    ASSERT(gen_atomic_bit_test(bm, 32), "bit 32 set (word 1)");
    ASSERT(gen_atomic_bit_test(bm, 33), "bit 33 set (word 1)");
    ASSERT(gen_atomic_bit_test(bm, 95), "bit 95 set (word 2)");
    ASSERT(!gen_atomic_bit_test(bm, 30), "bit 30 unset");
    ASSERT(!gen_atomic_bit_test(bm, 34), "bit 34 unset");

    /* test_and_set across word boundary */
    bool was_set = gen_atomic_bit_test_and_set(bm, 33);
    ASSERT(was_set, "test_and_set bit 33 already set");

    was_set = gen_atomic_bit_test_and_clear(bm, 33);
    ASSERT(was_set, "test_and_clear bit 33 was set");
    ASSERT(!gen_atomic_bit_test(bm, 33), "bit 33 cleared");
}

/* ── Main ─────────────────────────────────────────────────── */

int main(void)
{
    /* Spinlock */
    test_spin_lock_unlock();
    test_spin_try_lock_success();
    test_spin_try_lock_fail();
    test_spin_is_locked_state();

    /* Once */
    test_once_init_runs();
    test_once_second_call_noop();
    test_once_is_initialized();

    /* Atomic bitops */
    test_atomic_set_clear_test();
    test_atomic_test_and_set();
    test_atomic_test_and_clear();
    test_atomic_toggle();
    test_atomic_across_words();

    if (failures == 0) {
        fprintf(stderr, "sync_test: all tests passed\n");
    } else {
        fprintf(stderr, "sync_test: %d test(s) FAILED\n", failures);
    }

    return failures;
}
