/*
 * secure_zero.c — Compiler-resistant memory zeroing.
 *
 * Uses volatile writes to prevent dead-store elimination.
 * Critical for clearing sensitive data (keys, tokens, passwords).
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"

/*@
  requires n == 0 || \valid((volatile unsigned char *)dst + (0 .. n - 1));
  assigns ((volatile unsigned char *)dst)[0 .. n - 1];
  ensures \forall integer i; 0 <= i < n ==>
          ((volatile unsigned char *)dst)[i] == 0;
*/
void gen_secure_zero(volatile void *dst, size_t n)
{
    volatile unsigned char *d = (volatile unsigned char *)dst;

    /*@
      loop invariant 0 <= n;
      loop assigns n, *d, d;
      loop variant n;
    */
    while (n > 0) {
        *d++ = 0;
        n--;
    }
}
