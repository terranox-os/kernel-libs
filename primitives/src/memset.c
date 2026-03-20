/*
 * memset.c — Fill memory with a constant byte.
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"
#include <stdint.h>

/*@
  requires n == 0 || \valid((unsigned char *)dst + (0 .. n - 1));
  assigns ((unsigned char *)dst)[0 .. n - 1];
  ensures \forall integer i; 0 <= i < n ==>
          ((unsigned char *)dst)[i] == (unsigned char)c;
  ensures \result == dst;
*/
void *gen_memset(void *dst, int c, size_t n)
{
    unsigned char *d = (unsigned char *)dst;
    unsigned char  val = (unsigned char)c;

    /* Build a word-sized pattern for bulk fill */
    if (n >= sizeof(unsigned long) &&
        (uintptr_t)d % sizeof(unsigned long) == 0)
    {
        unsigned long pattern = 0;
        for (size_t i = 0; i < sizeof(unsigned long); i++) {
            pattern |= (unsigned long)val << (i * 8);
        }

        unsigned long *dw = (unsigned long *)d;
        size_t words = n / sizeof(unsigned long);

        /*@
          loop invariant 0 <= words;
          loop assigns words, dw[0 ..], dw;
          loop variant words;
        */
        while (words > 0) {
            *dw++ = pattern;
            words--;
        }

        d = (unsigned char *)dw;
        n %= sizeof(unsigned long);
    }

    /*@
      loop invariant 0 <= n;
      loop assigns n, d[0 ..], d;
      loop variant n;
    */
    while (n > 0) {
        *d++ = val;
        n--;
    }

    return dst;
}
